//! Excel data import implementation (parsing, validation, preview simulation, and atomic apply).

use crate::dto::{
    CampusImportRow, DeactivatedTeacherPreview, FeasibilityReportWithQuotas, ImportApplyResult,
    ImportCellError, ImportPreviewResult, ImportRowStatus, ImportSummaryCounts, TeacherImportRow,
    UnavailabilityImportRow,
};
use crate::error::AppError;
use crate::excel::normalize::{
    normalize_code, normalize_text, parse_boolean, parse_grade_codes, parse_load_weight,
};
use calamine::{open_workbook_auto, Data, Reader, Sheets};
use exam_panel_core::domain::{
    quota::calculate_quotas, Campus, CampusId, GradeId, SchoolYearId, Teacher, TeacherGrade,
    TeacherId,
};
use exam_panel_storage::Store;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Helper to convert a Calamine cell value to normalized string.
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

/// Finds the best matching sheet name case-insensitively using keyword aliases.
fn find_sheet_name(names: &[String], keywords: &[&str]) -> Option<String> {
    for name in names {
        let lower = normalize_text(name).to_lowercase();
        for kw in keywords {
            if lower.contains(kw) {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Generates a preview of the Excel import file without modifying the database.
pub fn preview_import(
    store: &Store,
    school_year_id: SchoolYearId,
    file_path: &Path,
    mode: &str,
) -> Result<ImportPreviewResult, AppError> {
    if !file_path.exists() {
        return Err(AppError::not_found(format!(
            "Tập tin Excel không tồn tại: {}",
            file_path.display()
        )));
    }

    let mut workbook: Sheets<BufReader<File>> = open_workbook_auto(file_path)
        .map_err(|e| AppError::internal(format!("Lỗi đọc file Excel: {e}")))?;

    let sheet_names = workbook.sheet_names().to_vec();

    // Load existing DB master data
    let existing_campuses = store.get_campuses()?;
    let existing_teachers = store.get_teachers()?;
    let existing_exams = store.get_exams(school_year_id)?;
    let existing_grades = store.get_grades()?;
    let existing_unavailabilities = store.get_unavailabilities(school_year_id)?;

    // Map existing grade code (10, 11, 12) -> GradeId
    let grade_code_to_id: HashMap<i32, GradeId> =
        existing_grades.iter().map(|g| (g.code, g.id)).collect();

    // Map existing teacher id -> set of grade ids in current school year
    let mut existing_teacher_grades: HashMap<i64, HashSet<GradeId>> = HashMap::new();
    for t in &existing_teachers {
        let g_ids = store.get_teacher_grades(t.id, school_year_id)?;
        existing_teacher_grades.insert(t.id.value(), g_ids.into_iter().collect());
    }

    // -------------------------------------------------------------------------
    // 1. Parse Sheet "Phân hiệu"
    // -------------------------------------------------------------------------
    let mut campus_rows = Vec::new();
    let mut campus_code_map: HashMap<String, CampusId> = existing_campuses
        .iter()
        .map(|c| (normalize_code(&c.code), c.id))
        .collect();

    let campus_sheet_name = find_sheet_name(&sheet_names, &["phân hiệu", "phan hieu", "campus"]);
    let mut seen_campus_codes = HashSet::new();

    if let Some(sheet_name) = campus_sheet_name {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut is_header = true;
            for (idx, row) in range.rows().enumerate() {
                if is_header {
                    is_header = false;
                    continue;
                }

                let row_number = idx + 1; // 1-indexed Excel row
                let raw_code = row.first().map(cell_to_string).unwrap_or_default();
                let raw_name = row.get(1).map(cell_to_string).unwrap_or_default();

                if raw_code.trim().is_empty() && raw_name.trim().is_empty() {
                    continue; // Skip empty row
                }

                let code = normalize_code(&raw_code);
                let name = normalize_text(&raw_name);

                let mut errors = Vec::new();
                if code.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "code".to_string(),
                        code: "import.error.required_campus_code".to_string(),
                        message: "Mã phân hiệu không được để trống".to_string(),
                    });
                } else if !seen_campus_codes.insert(code.clone()) {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "code".to_string(),
                        code: "import.error.duplicate_campus_code".to_string(),
                        message: format!("Mã phân hiệu '{code}' bị trùng lặp trong tập tin"),
                    });
                }

                if name.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "name".to_string(),
                        code: "import.error.required_campus_name".to_string(),
                        message: "Tên phân hiệu không được để trống".to_string(),
                    });
                }

                let status = if !errors.is_empty() {
                    ImportRowStatus::Error
                } else if let Some(existing) = existing_campuses
                    .iter()
                    .find(|c| normalize_code(&c.code) == code)
                {
                    if normalize_text(&existing.name) == name {
                        ImportRowStatus::Unchanged
                    } else {
                        ImportRowStatus::Update
                    }
                } else {
                    ImportRowStatus::New
                };

                // Add to temporary code map for validating teachers sheet
                if !code.is_empty() && !campus_code_map.contains_key(&code) {
                    campus_code_map.insert(code.clone(), CampusId(10_000 + idx as i64));
                }

                campus_rows.push(CampusImportRow {
                    row_index: row_number,
                    status,
                    code,
                    name,
                    errors,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 2. Parse Sheet "Giáo viên"
    // -------------------------------------------------------------------------
    let mut teacher_rows = Vec::new();
    let mut imported_teacher_refs = HashSet::new(); // codes and full names for unavail sheet
    let mut seen_teacher_codes = HashSet::new();

    let teacher_sheet_name = find_sheet_name(&sheet_names, &["giáo viên", "giao vien", "teacher"]);

    if let Some(sheet_name) = teacher_sheet_name {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut is_header = true;
            for (idx, row) in range.rows().enumerate() {
                if is_header {
                    is_header = false;
                    continue;
                }

                let row_number = idx + 1;
                let raw_code = row.first().map(cell_to_string).unwrap_or_default();
                let raw_name = row.get(1).map(cell_to_string).unwrap_or_default();
                let raw_campus = row.get(2).map(cell_to_string).unwrap_or_default();
                let raw_grades = row.get(3).map(cell_to_string).unwrap_or_default();
                let raw_load = row.get(4).map(cell_to_string).unwrap_or_default();
                let raw_active = row.get(5).map(cell_to_string).unwrap_or_default();
                let raw_note = row.get(6).map(cell_to_string).unwrap_or_default();

                if raw_code.trim().is_empty()
                    && raw_name.trim().is_empty()
                    && raw_campus.trim().is_empty()
                    && raw_grades.trim().is_empty()
                {
                    continue;
                }

                let code_opt = if raw_code.trim().is_empty() {
                    None
                } else {
                    Some(normalize_code(&raw_code))
                };
                let full_name = normalize_text(&raw_name);
                let campus_code = normalize_code(&raw_campus);
                let note = if raw_note.trim().is_empty() {
                    None
                } else {
                    Some(normalize_text(&raw_note))
                };

                let mut errors = Vec::new();

                // Validate code uniqueness if present
                if let Some(ref c) = code_opt {
                    if !seen_teacher_codes.insert(c.clone()) {
                        errors.push(ImportCellError {
                            sheet: sheet_name.clone(),
                            row: row_number,
                            column: "code".to_string(),
                            code: "import.error.duplicate_teacher_code".to_string(),
                            message: format!("Mã giáo viên '{c}' bị trùng lặp trong tập tin"),
                        });
                    }
                }

                // Validate full_name
                if full_name.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "full_name".to_string(),
                        code: "import.error.required_teacher_name".to_string(),
                        message: "Họ và tên giáo viên không được để trống".to_string(),
                    });
                }

                // Validate campus code
                let matched_campus_id = if campus_code.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "campus_code".to_string(),
                        code: "import.error.required_campus_code".to_string(),
                        message: "Mã phân hiệu không được để trống".to_string(),
                    });
                    None
                } else if let Some(&cid) = campus_code_map.get(&campus_code) {
                    Some(cid)
                } else {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "campus_code".to_string(),
                        code: "import.error.campus_not_found".to_string(),
                        message: format!("Không tìm thấy phân hiệu có mã '{campus_code}'"),
                    });
                    None
                };

                // Validate grades
                let grade_codes = match parse_grade_codes(&raw_grades) {
                    Ok(gc) => gc,
                    Err(e) => {
                        errors.push(ImportCellError {
                            sheet: sheet_name.clone(),
                            row: row_number,
                            column: "grades".to_string(),
                            code: "import.error.invalid_grades".to_string(),
                            message: e,
                        });
                        Vec::new()
                    }
                };

                // Validate load weight
                let load_weight = match parse_load_weight(&raw_load, 1.0) {
                    Ok(lw) => lw,
                    Err(e) => {
                        errors.push(ImportCellError {
                            sheet: sheet_name.clone(),
                            row: row_number,
                            column: "load_weight".to_string(),
                            code: "import.error.invalid_load_weight".to_string(),
                            message: e,
                        });
                        1.0
                    }
                };

                // Validate active
                let active = match parse_boolean(&raw_active, true) {
                    Ok(a) => a,
                    Err(e) => {
                        errors.push(ImportCellError {
                            sheet: sheet_name.clone(),
                            row: row_number,
                            column: "active".to_string(),
                            code: "import.error.invalid_active".to_string(),
                            message: e,
                        });
                        true
                    }
                };

                // Teacher matching order:
                // 1. Mã GV (if present)
                // 2. (Họ tên, Mã phân hiệu)
                // 3. Họ tên alone only if unique; ambiguous matches are row errors.
                let mut matched_teacher: Option<&Teacher> = None;

                if let Some(ref c) = code_opt {
                    matched_teacher = existing_teachers
                        .iter()
                        .find(|t| t.code.as_ref().map(|tc| normalize_code(tc)) == Some(c.clone()));
                }

                if matched_teacher.is_none() {
                    if let Some(cid) = matched_campus_id {
                        matched_teacher = existing_teachers.iter().find(|t| {
                            normalize_text(&t.full_name) == full_name && t.campus_id == cid
                        });
                    }
                }

                if matched_teacher.is_none() && !full_name.is_empty() {
                    let matching_by_name: Vec<&Teacher> = existing_teachers
                        .iter()
                        .filter(|t| normalize_text(&t.full_name) == full_name)
                        .collect();

                    if matching_by_name.len() == 1 {
                        matched_teacher = Some(matching_by_name[0]);
                    } else if matching_by_name.len() > 1 {
                        errors.push(ImportCellError {
                            sheet: sheet_name.clone(),
                            row: row_number,
                            column: "full_name".to_string(),
                            code: "import.error.ambiguous_teacher_match".to_string(),
                            message: format!(
                                "Có nhiều giáo viên cùng tên '{full_name}'. Vui lòng nhập Mã GV hoặc phân hiệu chính xác"
                            ),
                        });
                    }
                }

                let status = if !errors.is_empty() {
                    ImportRowStatus::Error
                } else if let Some(existing) = matched_teacher {
                    // Check if unchanged
                    let current_gids = existing_teacher_grades
                        .get(&existing.id.value())
                        .cloned()
                        .unwrap_or_default();
                    let imported_gids: HashSet<GradeId> = grade_codes
                        .iter()
                        .filter_map(|gc| grade_code_to_id.get(gc).copied())
                        .collect();

                    let same_code = match (&code_opt, &existing.code) {
                        (Some(a), Some(b)) => a == &normalize_code(b),
                        (None, None) => true,
                        (Some(_), None) => false, // adding a code is an update
                        (None, Some(_)) => true,  // leaving blank preserves existing code
                    };

                    let same_data = same_code
                        && normalize_text(&existing.full_name) == full_name
                        && matched_campus_id == Some(existing.campus_id)
                        && (existing.load_weight - load_weight).abs() < 1e-6
                        && existing.active == active
                        && existing.note.as_deref().map(normalize_text) == note
                        && current_gids == imported_gids;

                    if same_data {
                        ImportRowStatus::Unchanged
                    } else {
                        ImportRowStatus::Update
                    }
                } else {
                    ImportRowStatus::New
                };

                let matched_teacher_id = matched_teacher.map(|t| t.id.value());

                if let Some(ref c) = code_opt {
                    imported_teacher_refs.insert(c.clone());
                }
                imported_teacher_refs.insert(full_name.clone());

                teacher_rows.push(TeacherImportRow {
                    row_index: row_number,
                    status,
                    code: code_opt,
                    full_name,
                    campus_code,
                    grades_str: raw_grades,
                    grade_codes,
                    load_weight,
                    active,
                    note,
                    matched_teacher_id,
                    errors,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 3. Parse Sheet "Lịch vắng"
    // -------------------------------------------------------------------------
    let mut unavail_rows = Vec::new();
    let unavail_sheet_name =
        find_sheet_name(&sheet_names, &["lịch vắng", "lich vang", "unavailability"]);

    if let Some(sheet_name) = unavail_sheet_name {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut is_header = true;
            for (idx, row) in range.rows().enumerate() {
                if is_header {
                    is_header = false;
                    continue;
                }

                let row_number = idx + 1;
                let raw_ref = row.first().map(cell_to_string).unwrap_or_default();
                let raw_exam = row.get(1).map(cell_to_string).unwrap_or_default();
                let raw_reason = row.get(2).map(cell_to_string).unwrap_or_default();

                if raw_ref.trim().is_empty() && raw_exam.trim().is_empty() {
                    continue;
                }

                let teacher_ref = normalize_text(&raw_ref);
                let exam_code = normalize_code(&raw_exam);
                let reason = if raw_reason.trim().is_empty() {
                    None
                } else {
                    Some(normalize_text(&raw_reason))
                };

                let mut errors = Vec::new();

                if teacher_ref.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "teacher_ref".to_string(),
                        code: "import.error.required_teacher_ref".to_string(),
                        message: "Mã GV hoặc Họ tên giáo viên không được để trống".to_string(),
                    });
                }

                let matched_exam = if exam_code.is_empty() {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "exam_code".to_string(),
                        code: "import.error.required_exam_code".to_string(),
                        message: "Mã kỳ thi không được để trống".to_string(),
                    });
                    None
                } else if let Some(e) = existing_exams
                    .iter()
                    .find(|e| normalize_code(&e.code) == exam_code)
                {
                    Some(e)
                } else {
                    errors.push(ImportCellError {
                        sheet: sheet_name.clone(),
                        row: row_number,
                        column: "exam_code".to_string(),
                        code: "import.error.exam_not_found".to_string(),
                        message: format!("Không tìm thấy kỳ thi có mã '{exam_code}'"),
                    });
                    None
                };

                // Match teacher from imported teachers or existing teachers
                let matched_tid = if !teacher_ref.is_empty() {
                    // 1. Try code
                    let from_imported = teacher_rows.iter().find(|t| {
                        t.code.as_ref().map(|c| normalize_code(c))
                            == Some(normalize_code(&teacher_ref))
                            || normalize_text(&t.full_name) == teacher_ref
                    });

                    if let Some(t) = from_imported {
                        t.matched_teacher_id
                    } else {
                        let from_existing: Vec<&Teacher> = existing_teachers
                            .iter()
                            .filter(|t| {
                                t.code.as_ref().map(|c| normalize_code(c))
                                    == Some(normalize_code(&teacher_ref))
                                    || normalize_text(&t.full_name) == teacher_ref
                            })
                            .collect();

                        if from_existing.len() == 1 {
                            Some(from_existing[0].id.value())
                        } else if from_existing.len() > 1 {
                            errors.push(ImportCellError {
                                sheet: sheet_name.clone(),
                                row: row_number,
                                column: "teacher_ref".to_string(),
                                code: "import.error.ambiguous_teacher_match".to_string(),
                                message: format!(
                                    "Có nhiều giáo viên trùng khớp với '{teacher_ref}'"
                                ),
                            });
                            None
                        } else {
                            errors.push(ImportCellError {
                                sheet: sheet_name.clone(),
                                row: row_number,
                                column: "teacher_ref".to_string(),
                                code: "import.error.teacher_not_found".to_string(),
                                message: format!("Không tìm thấy giáo viên '{teacher_ref}'"),
                            });
                            None
                        }
                    }
                } else {
                    None
                };

                let status = if !errors.is_empty() {
                    ImportRowStatus::Error
                } else if let (Some(tid), Some(exam)) = (matched_tid, matched_exam) {
                    let existing_entry = existing_unavailabilities
                        .iter()
                        .find(|u| u.teacher_id.value() == tid && u.exam_id == exam.id);

                    if let Some(existing) = existing_entry {
                        if existing.reason.as_deref().map(normalize_text) == reason {
                            ImportRowStatus::Unchanged
                        } else {
                            ImportRowStatus::Update
                        }
                    } else {
                        ImportRowStatus::New
                    }
                } else {
                    ImportRowStatus::New
                };

                unavail_rows.push(UnavailabilityImportRow {
                    row_index: row_number,
                    status,
                    teacher_ref,
                    exam_code,
                    reason,
                    matched_teacher_id: matched_tid,
                    errors,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. Mode "sync": Find active teachers in DB not present in imported file
    // -------------------------------------------------------------------------
    let mut deactivated_teachers = Vec::new();
    if mode == "sync" {
        let imported_ids: HashSet<i64> = teacher_rows
            .iter()
            .filter_map(|t| t.matched_teacher_id)
            .collect();

        for t in &existing_teachers {
            if t.active && !imported_ids.contains(&t.id.value()) {
                let campus_name = existing_campuses
                    .iter()
                    .find(|c| c.id == t.campus_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();

                deactivated_teachers.push(DeactivatedTeacherPreview {
                    id: t.id.value(),
                    code: t.code.clone(),
                    full_name: t.full_name.clone(),
                    campus_name,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 5. Compute Summaries
    // -------------------------------------------------------------------------
    let count_statuses = |rows: &[ImportRowStatus]| {
        let mut counts = ImportSummaryCounts {
            new_count: 0,
            update_count: 0,
            unchanged_count: 0,
            error_count: 0,
        };
        for s in rows {
            match s {
                ImportRowStatus::New => counts.new_count += 1,
                ImportRowStatus::Update => counts.update_count += 1,
                ImportRowStatus::Unchanged => counts.unchanged_count += 1,
                ImportRowStatus::Error => counts.error_count += 1,
                ImportRowStatus::Skipped => {}
            }
        }
        counts
    };

    let campuses_summary = count_statuses(
        &campus_rows
            .iter()
            .map(|r| r.status.clone())
            .collect::<Vec<_>>(),
    );
    let teachers_summary = count_statuses(
        &teacher_rows
            .iter()
            .map(|r| r.status.clone())
            .collect::<Vec<_>>(),
    );
    let unavailabilities_summary = count_statuses(
        &unavail_rows
            .iter()
            .map(|r| r.status.clone())
            .collect::<Vec<_>>(),
    );

    let can_apply = campuses_summary.error_count == 0
        && teachers_summary.error_count == 0
        && unavailabilities_summary.error_count == 0;

    // -------------------------------------------------------------------------
    // 6. In-memory Feasibility Simulation
    // -------------------------------------------------------------------------
    let mut feasibility_report = None;
    if can_apply {
        if let Ok(mut sim_problem) = store.load_problem(school_year_id) {
            // Apply campus additions/updates
            for c in &campus_rows {
                if c.status == ImportRowStatus::New {
                    sim_problem.campuses.push(Campus {
                        id: CampusId(10_000 + c.row_index as i64),
                        code: c.code.clone(),
                        name: c.name.clone(),
                        color: "#94a3b8".to_string(),
                    });
                }
            }

            // Apply deactivations in sync mode
            let deact_ids: HashSet<i64> = deactivated_teachers.iter().map(|d| d.id).collect();
            for t in &mut sim_problem.teachers {
                if deact_ids.contains(&t.id.value()) {
                    t.active = false;
                }
            }

            // Apply teacher additions/updates
            for (i, t_row) in teacher_rows.iter().enumerate() {
                let tid = t_row
                    .matched_teacher_id
                    .map(TeacherId)
                    .unwrap_or(TeacherId(20_000 + i as i64));

                let cid = campus_code_map
                    .get(&t_row.campus_code)
                    .copied()
                    .unwrap_or(CampusId(1));

                if let Some(existing) = sim_problem.teachers.iter_mut().find(|t| t.id == tid) {
                    existing.code = t_row.code.clone();
                    existing.campus_id = cid;
                    existing.load_weight = t_row.load_weight;
                    existing.active = t_row.active;
                } else if t_row.status == ImportRowStatus::New {
                    sim_problem.teachers.push(Teacher {
                        id: tid,
                        code: t_row.code.clone(),
                        full_name: t_row.full_name.clone(),
                        campus_id: cid,
                        load_weight: t_row.load_weight,
                        active: t_row.active,
                        note: t_row.note.clone(),
                    });
                }

                // Update teacher grades
                sim_problem.teacher_grades.retain(|tg| tg.teacher_id != tid);
                for gc in &t_row.grade_codes {
                    if let Some(&gid) = grade_code_to_id.get(gc) {
                        sim_problem.teacher_grades.push(TeacherGrade {
                            teacher_id: tid,
                            school_year_id,
                            grade_id: gid,
                        });
                    }
                }
            }

            // Run feasibility check on simulated problem
            let report = exam_panel_core::feasibility::check_feasibility(&sim_problem);
            let quotas = calculate_quotas(&sim_problem);
            feasibility_report = Some(FeasibilityReportWithQuotas { report, quotas });
        }
    }

    Ok(ImportPreviewResult {
        mode: mode.to_string(),
        can_apply,
        campuses: campus_rows,
        teachers: teacher_rows,
        unavailabilities: unavail_rows,
        campuses_summary,
        teachers_summary,
        unavailabilities_summary,
        deactivated_teachers,
        feasibility_report,
    })
}

/// Applies imported data to the database inside a single atomic transaction.
///
/// An automatic backup is created before writing.
pub fn apply_import(
    store: &mut Store,
    school_year_id: SchoolYearId,
    preview: &ImportPreviewResult,
) -> Result<ImportApplyResult, AppError> {
    if !preview.can_apply {
        return Err(AppError::validation(
            "Không thể áp dụng dữ liệu vì tập tin còn chứa dòng lỗi".to_string(),
        ));
    }

    // 1. Automatic backup before applying
    let backup_path_buf = store
        .auto_backup("pre-import")
        .map_err(|e| AppError::internal(format!("Không thể tạo bản sao lưu tự động: {e}")))?;
    let backup_path = backup_path_buf.to_string_lossy().to_string();

    let existing_campuses = store.get_campuses()?;
    let existing_exams = store.get_exams(school_year_id)?;
    let existing_grades = store.get_grades()?;
    let grade_code_to_id: HashMap<i32, GradeId> =
        existing_grades.iter().map(|g| (g.code, g.id)).collect();

    let mut campuses_created = 0;
    let mut campuses_updated = 0;
    let mut teachers_created = 0;
    let mut teachers_updated = 0;
    let mut teachers_deactivated = 0;
    let mut unavailabilities_created = 0;

    let mut campus_code_to_id: HashMap<String, CampusId> = existing_campuses
        .iter()
        .map(|c| (normalize_code(&c.code), c.id))
        .collect();

    let tx = store
        .conn_mut()
        .transaction()
        .map_err(|e| AppError::internal(format!("Không thể mở transaction: {e}")))?;

    // 2. Apply Campuses
    for c in &preview.campuses {
        match c.status {
            ImportRowStatus::New => {
                tx.execute(
                    "INSERT INTO campuses (code, name, color) VALUES (?1, ?2, ?3)",
                    rusqlite::params![c.code, c.name, "#64748b"],
                )
                .map_err(|e| AppError::internal(format!("Lỗi thêm phân hiệu: {e}")))?;
                let new_id = CampusId(tx.last_insert_rowid());
                campus_code_to_id.insert(c.code.clone(), new_id);
                campuses_created += 1;
            }
            ImportRowStatus::Update => {
                if let Some(&cid) = campus_code_to_id.get(&c.code) {
                    tx.execute(
                        "UPDATE campuses SET name = ?1 WHERE id = ?2",
                        rusqlite::params![c.name, cid.value()],
                    )
                    .map_err(|e| AppError::internal(format!("Lỗi cập nhật phân hiệu: {e}")))?;
                    campuses_updated += 1;
                }
            }
            _ => {}
        }
    }

    // 3. Apply Teachers & Per-year Grades
    let mut teacher_code_or_name_to_id: HashMap<String, TeacherId> = HashMap::new();

    for t in &preview.teachers {
        let cid = campus_code_to_id
            .get(&t.campus_code)
            .copied()
            .ok_or_else(|| {
                AppError::validation(format!("Mã phân hiệu '{}' không hợp lệ", t.campus_code))
            })?;

        let tid = match t.status {
            ImportRowStatus::New => {
                tx.execute(
                    "INSERT INTO teachers (code, full_name, campus_id, load_weight, active, note)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        t.code,
                        t.full_name,
                        cid.value(),
                        t.load_weight,
                        if t.active { 1 } else { 0 },
                        t.note,
                    ],
                )
                .map_err(|e| AppError::internal(format!("Lỗi thêm giáo viên: {e}")))?;
                let new_id = TeacherId(tx.last_insert_rowid());
                teachers_created += 1;
                new_id
            }
            ImportRowStatus::Update => {
                let existing_id = t.matched_teacher_id.ok_or_else(|| {
                    AppError::internal("Thiếu teacher_id cho dòng cập nhật".to_string())
                })?;
                tx.execute(
                    "UPDATE teachers SET code = ?1, full_name = ?2, campus_id = ?3,
                     load_weight = ?4, active = ?5, note = ?6, updated_at = datetime('now')
                     WHERE id = ?7",
                    rusqlite::params![
                        t.code,
                        t.full_name,
                        cid.value(),
                        t.load_weight,
                        if t.active { 1 } else { 0 },
                        t.note,
                        existing_id,
                    ],
                )
                .map_err(|e| AppError::internal(format!("Lỗi cập nhật giáo viên: {e}")))?;
                teachers_updated += 1;
                TeacherId(existing_id)
            }
            _ => TeacherId(t.matched_teacher_id.unwrap_or(0)),
        };

        if tid.value() > 0 {
            if let Some(ref c) = t.code {
                teacher_code_or_name_to_id.insert(c.clone(), tid);
            }
            teacher_code_or_name_to_id.insert(t.full_name.clone(), tid);

            // Update per-year teacher grades for this school year
            tx.execute(
                "DELETE FROM teacher_grades WHERE teacher_id = ?1 AND school_year_id = ?2",
                rusqlite::params![tid.value(), school_year_id.value()],
            )
            .map_err(|e| AppError::internal(format!("Lỗi xóa khối dạy cũ: {e}")))?;

            for gc in &t.grade_codes {
                if let Some(&gid) = grade_code_to_id.get(gc) {
                    tx.execute(
                        "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![tid.value(), school_year_id.value(), gid.value()],
                    )
                    .map_err(|e| AppError::internal(format!("Lỗi gán khối dạy: {e}")))?;
                }
            }
        }
    }

    // 4. In "sync" mode: Deactivate teachers not in file
    if preview.mode == "sync" {
        for deact in &preview.deactivated_teachers {
            tx.execute(
                "UPDATE teachers SET active = 0, updated_at = datetime('now') WHERE id = ?1",
                rusqlite::params![deact.id],
            )
            .map_err(|e| AppError::internal(format!("Lỗi hủy kích hoạt giáo viên: {e}")))?;
            teachers_deactivated += 1;
        }
    }

    // 5. Apply Unavailabilities
    for u in &preview.unavailabilities {
        let exam_id = existing_exams
            .iter()
            .find(|e| normalize_code(&e.code) == u.exam_code)
            .map(|e| e.id);

        let teacher_id = u
            .matched_teacher_id
            .map(TeacherId)
            .or_else(|| teacher_code_or_name_to_id.get(&u.teacher_ref).copied());

        if let (Some(eid), Some(tid)) = (exam_id, teacher_id) {
            tx.execute(
                "INSERT OR REPLACE INTO unavailability (teacher_id, exam_id, reason)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![tid.value(), eid.value(), u.reason],
            )
            .map_err(|e| AppError::internal(format!("Lỗi lưu lịch bận: {e}")))?;
            unavailabilities_created += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::internal(format!("Lỗi commit transaction: {e}")))?;

    Ok(ImportApplyResult {
        backup_path,
        campuses_created,
        campuses_updated,
        teachers_created,
        teachers_updated,
        teachers_deactivated,
        unavailabilities_created,
    })
}
