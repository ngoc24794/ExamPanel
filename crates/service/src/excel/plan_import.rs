//! Plan import module from Q-style Excel workbook or TSV clipboard text.

use crate::dto::{ImportCellError, PlanImportPreview, PlanImportTeacherTotal};
use crate::error::AppError;
use calamine::{open_workbook_auto, Data, Reader, Sheets};
use exam_panel_core::domain::{Assignment, Exam, Grade, PlanId, Problem, Role, Teacher};
use exam_panel_core::score::evaluate;
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

/// Converts a 0-indexed (row, col) coordinate into an Excel cell address (e.g. (0,0) -> "A1", (7,1) -> "B8").
pub fn cell_address(row: usize, col: usize) -> String {
    let mut c = col;
    let mut col_str = String::new();
    loop {
        let rem = (c % 26) as u8;
        col_str.insert(0, (b'A' + rem) as char);
        if c < 26 {
            break;
        }
        c = (c / 26) - 1;
    }
    format!("{}{}", col_str, row + 1)
}

/// Converts a Calamine cell value to normalized string.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 && *f >= -1_000_000.0 && *f <= 1_000_000.0 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => {
            if *b {
                "Có".to_string()
            } else {
                "Không".to_string()
            }
        }
        Data::DateTime(dt) => format!("{dt}"),
        _ => String::new(),
    }
}

/// Normalizes text using Unicode NFC, trimmed and lowercased.
fn normalize_name(s: &str) -> String {
    s.nfc().collect::<String>().trim().to_lowercase()
}

/// Matches a raw text name to a Teacher:
/// 1. Exact match on display_name (cách gọi)
/// 2. Exact match on full_name
/// 3. NFC-normalized case-insensitive match
/// 4. Accent-insensitive match (strip diacritics)
pub fn find_teacher<'a>(
    raw: &str,
    teachers: &'a [Teacher],
) -> Result<Option<&'a Teacher>, &'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return Ok(None);
    }

    // 1. Exact match on display_name
    let exact_dn: Vec<_> = teachers
        .iter()
        .filter(|t| {
            t.display_name
                .as_deref()
                .map(|d| d.trim() == trimmed)
                .unwrap_or(false)
        })
        .collect();
    if exact_dn.len() == 1 {
        return Ok(Some(exact_dn[0]));
    } else if exact_dn.len() > 1 {
        return Err("ambiguous_teacher");
    }

    // 2. Exact match on full_name
    let exact_fn: Vec<_> = teachers
        .iter()
        .filter(|t| t.full_name.trim() == trimmed)
        .collect();
    if exact_fn.len() == 1 {
        return Ok(Some(exact_fn[0]));
    } else if exact_fn.len() > 1 {
        return Err("ambiguous_teacher");
    }

    // 3. NFC normalized match
    let q_norm = normalize_name(trimmed);
    let norm_matches: Vec<_> = teachers
        .iter()
        .filter(|t| {
            let dn = t
                .display_name
                .as_deref()
                .map(normalize_name)
                .unwrap_or_default();
            let fn_norm = normalize_name(&t.full_name);
            q_norm == dn || q_norm == fn_norm
        })
        .collect();

    if norm_matches.len() == 1 {
        return Ok(Some(norm_matches[0]));
    } else if norm_matches.len() > 1 {
        return Err("ambiguous_teacher");
    }

    // 4. Accent-insensitive match
    let q_stripped = crate::excel::normalize::strip_diacritics(trimmed);
    let stripped_matches: Vec<_> = teachers
        .iter()
        .filter(|t| {
            let dn = t
                .display_name
                .as_deref()
                .map(crate::excel::normalize::strip_diacritics)
                .unwrap_or_default();
            let fn_stripped = crate::excel::normalize::strip_diacritics(&t.full_name);
            q_stripped == dn || q_stripped == fn_stripped
        })
        .collect();

    if stripped_matches.len() == 1 {
        Ok(Some(stripped_matches[0]))
    } else if stripped_matches.len() > 1 {
        Err("ambiguous_teacher")
    } else {
        Err("unknown_teacher")
    }
}

/// Parses a 2D grid of strings into a `PlanImportPreview` validating against the problem.
pub fn parse_plan_grid(
    grid: &[Vec<String>],
    problem: &Problem,
) -> Result<PlanImportPreview, AppError> {
    let mut errors: Vec<ImportCellError> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut assignments: Vec<Assignment> = Vec::new();

    if grid.is_empty() {
        return Err(AppError::validation("Dữ liệu bảng phân công trống"));
    }

    // Locate header row 1 (look for "Kì thi/khối", "Kỳ thi", or Grade names)
    let mut header_row_1 = 0;
    for (r_idx, row) in grid.iter().enumerate().take(15) {
        if row.iter().any(|cell| {
            let lower = cell.to_lowercase();
            lower.contains("kì thi") || lower.contains("kỳ thi") || lower.contains("khối")
        }) {
            header_row_1 = r_idx;
            break;
        }
    }
    let header_row_2 = header_row_1 + 1;

    // Detect column mappings: col -> (GradeId, SubjectId)
    // Also detect totals columns: gv_col, total_col
    let mut col_to_target: HashMap<
        usize,
        (
            exam_panel_core::domain::GradeId,
            exam_panel_core::domain::SubjectId,
        ),
    > = HashMap::new();
    let mut col_gv: Option<usize> = None;
    let mut col_total: Option<usize> = None;

    let h1 = &grid[header_row_1];
    let h2 = if header_row_2 < grid.len() {
        &grid[header_row_2]
    } else {
        h1
    };

    let max_cols = h1.len().max(h2.len());
    let mut current_grade: Option<&Grade> = None;

    for c in 0..max_cols {
        let cell1 = h1.get(c).map(String::as_str).unwrap_or("");
        let cell2 = h2.get(c).map(String::as_str).unwrap_or("");

        // Check if cell1 introduces a new grade
        let norm1 = normalize_name(cell1);
        if let Some(g) = problem.grades.iter().find(|g| {
            let gname = normalize_name(&g.name);
            let gcode = format!("k{}", g.code);
            let gnum = format!("{}", g.code);
            norm1.contains(&gname) || norm1 == gcode || norm1 == gnum
        }) {
            current_grade = Some(g);
        }

        // Check totals headers
        if norm1 == "gv" || norm1 == "giáo viên" || normalize_name(cell2) == "gv" {
            col_gv = Some(c);
            current_grade = None;
            continue;
        }
        if norm1.contains("tổng") || normalize_name(cell2).contains("tổng") {
            col_total = Some(c);
            current_grade = None;
            continue;
        }

        // If we have a current_grade, check if cell2 (or cell1) matches a subject
        if let Some(g) = current_grade {
            let norm2 = normalize_name(cell2);
            let target_sub = problem.subjects.iter().find(|s| {
                let scode = normalize_name(&s.code);
                let sname = normalize_name(&s.name);
                norm2 == scode || norm2 == sname || norm1 == scode || norm1 == sname
            });

            if let Some(sub) = target_sub {
                col_to_target.insert(c, (g.id, sub.id));
            } else if problem.subjects.len() == 1 {
                col_to_target.insert(c, (g.id, problem.subjects[0].id));
            }
        }
    }

    if col_to_target.is_empty() {
        return Err(AppError::validation(
            "Không nhận diện được các cột môn và khối trong bảng phân công",
        ));
    }

    // Scan data rows starting after header_row_2
    let mut current_exam: Option<&Exam> = None;
    let mut exam_setters_seen = 0;
    let mut exam_reviewers_seen = 0;
    let mut file_totals: HashMap<exam_panel_core::domain::TeacherId, i64> = HashMap::new();

    let start_row = header_row_2 + 1;
    for (r, row) in grid.iter().enumerate().skip(start_row) {
        // Check for totals in totals table
        if let Some(gv_c) = col_gv {
            if let Some(gv_val) = row.get(gv_c).map(String::as_str) {
                if !gv_val.trim().is_empty() && !gv_val.contains("Tổng") {
                    if let Ok(Some(t)) = find_teacher(gv_val, &problem.teachers) {
                        if let Some(tot_c) = col_total {
                            if let Some(tot_val) = row.get(tot_c).map(String::as_str) {
                                if let Ok(val) = tot_val.trim().parse::<i64>() {
                                    file_totals.insert(t.id, val);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check if Col 0 introduces an Exam
        let col0_val = row.first().map(String::as_str).unwrap_or("");
        if !col0_val.trim().is_empty() {
            let norm0 = normalize_name(col0_val);
            if let Some(exam) = problem.exams.iter().find(|e| {
                let ecode = normalize_name(&e.code);
                let ename = normalize_name(&e.name);
                norm0.contains(&ecode) || norm0.contains(&ename)
            }) {
                current_exam = Some(exam);
                exam_setters_seen = 0;
                exam_reviewers_seen = 0;
            }
        }

        let exam = match current_exam {
            Some(e) => e,
            None => continue,
        };

        // Determine Role
        let col1_val = row.get(1).map(String::as_str).unwrap_or("");
        let norm1 = normalize_name(col1_val);

        let (role, pos) = if norm1.contains("đề") || norm1 == "đ" {
            let p = exam_setters_seen;
            exam_setters_seen += 1;
            (Role::Setter, p)
        } else if norm1.contains("biện") || norm1.contains("pb") {
            let p = exam_reviewers_seen;
            exam_reviewers_seen += 1;
            (Role::Reviewer, p)
        } else {
            // Not a role row
            continue;
        };

        // Process each target (Grade, Subject) column
        for (&col, &(g_id, s_id)) in &col_to_target {
            let cell_val = row.get(col).map(String::as_str).unwrap_or("");
            let addr = cell_address(r, col);
            let subject = match problem.subjects.iter().find(|s| s.id == s_id) {
                Some(s) => s,
                None => continue,
            };

            // Blank check vs subject composition
            if role == Role::Setter && pos >= subject.setters as usize {
                continue; // expected blank
            }
            if role == Role::Reviewer && pos >= subject.reviewers as usize {
                continue; // expected blank
            }

            if cell_val.trim().is_empty() || cell_val.trim() == "-" {
                errors.push(ImportCellError {
                    sheet: "Plan".to_string(),
                    row: r + 1,
                    column: addr.clone(),
                    code: "missing_seat".to_string(),
                    message: format!("Thiếu giáo viên tại ô {addr}"),
                });
                continue;
            }

            match find_teacher(cell_val, &problem.teachers) {
                Ok(Some(t)) => {
                    assignments.push(Assignment {
                        plan_id: PlanId(0),
                        exam_id: exam.id,
                        grade_id: g_id,
                        subject_id: s_id,
                        teacher_id: t.id,
                        role,
                        position: pos,
                    });
                }
                Ok(None) => {
                    errors.push(ImportCellError {
                        sheet: "Plan".to_string(),
                        row: r + 1,
                        column: addr.clone(),
                        code: "missing_seat".to_string(),
                        message: format!("Thiếu giáo viên tại ô {addr}"),
                    });
                }
                Err(code) => {
                    errors.push(ImportCellError {
                        sheet: "Plan".to_string(),
                        row: r + 1,
                        column: addr.clone(),
                        code: code.to_string(),
                        message: format!(
                            "Không nhận diện được giáo viên '{cell_val}' tại ô {addr}"
                        ),
                    });
                }
            }
        }
    }

    // Build teacher totals and compare with file_totals
    let mut teacher_totals: Vec<PlanImportTeacherTotal> = Vec::new();
    for t in &problem.teachers {
        let computed = assignments.iter().filter(|a| a.teacher_id == t.id).count() as i64;
        let de = assignments
            .iter()
            .filter(|a| a.teacher_id == t.id && a.role == Role::Setter)
            .count() as i64;
        let pb = assignments
            .iter()
            .filter(|a| a.teacher_id == t.id && a.role == Role::Reviewer)
            .count() as i64;
        let file_tot = file_totals.get(&t.id).copied();

        if let Some(ft) = file_tot {
            if ft != computed {
                warnings.push(format!(
                    "Giáo viên {}: trong bảng ghi {ft} nhiệm vụ, đếm được {computed} nhiệm vụ",
                    t.display_name.as_deref().unwrap_or(&t.full_name)
                ));
            }
        }

        teacher_totals.push(PlanImportTeacherTotal {
            teacher_id: t.id,
            teacher_name: t.full_name.clone(),
            display_name: t
                .display_name
                .clone()
                .unwrap_or_else(|| t.full_name.clone()),
            file_total: file_tot,
            computed_total: computed,
            setter_count: de,
            reviewer_count: pb,
        });
    }

    let can_apply = errors.is_empty();

    let (hard_violations, score_report) = if can_apply {
        let violations = validate_assignments(
            problem,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        let score = evaluate(problem, &assignments);
        (violations, Some(score))
    } else {
        (Vec::new(), None)
    };

    Ok(PlanImportPreview {
        assignments,
        teacher_totals,
        errors,
        warnings,
        can_apply,
        hard_violations,
        score_report,
    })
}

/// Reads an Excel file or parses TSV text and produces a `PlanImportPreview`.
pub fn preview_plan_import(
    problem: &Problem,
    file_path: Option<&Path>,
    tsv_content: Option<&str>,
) -> Result<PlanImportPreview, AppError> {
    let grid = if let Some(path) = file_path {
        let mut workbook: Sheets<BufReader<File>> = open_workbook_auto(path)
            .map_err(|e| AppError::validation(format!("Không thể mở file Excel: {e}")))?;

        // Find sheet: "Bảng phân công (mẫu tổ)" or "Phân công" or sheet 0
        let sheet_names = workbook.sheet_names().to_vec();
        let target_sheet = sheet_names
            .iter()
            .find(|s| {
                let lower = s.to_lowercase();
                lower.contains("mẫu tổ") || lower.contains("phân công")
            })
            .cloned()
            .unwrap_or_else(|| sheet_names.first().cloned().unwrap_or_default());

        let range = workbook
            .worksheet_range(&target_sheet)
            .map_err(|e| AppError::validation(format!("Lỗi đọc sheet Excel: {e}")))?;

        let mut rows = Vec::new();
        for r in range.rows() {
            let row_cells: Vec<String> = r.iter().map(cell_to_string).collect();
            rows.push(row_cells);
        }
        rows
    } else if let Some(tsv) = tsv_content {
        let mut rows = Vec::new();
        for line in tsv.lines() {
            let row_cells: Vec<String> = line.split('\t').map(|s| s.trim().to_string()).collect();
            rows.push(row_cells);
        }
        rows
    } else {
        return Err(AppError::validation(
            "Cần cung cấp đường dẫn file Excel hoặc nội dung văn bản TSV",
        ));
    };

    parse_plan_grid(&grid, problem)
}
