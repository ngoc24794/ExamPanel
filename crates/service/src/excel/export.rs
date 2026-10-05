//! Excel plan export implementation using rust_xlsxwriter.

use crate::dto::{AppSettings, PlanDetails};
use exam_panel_core::domain::{
    Campus, Exam, Grade, Role, RuleSetting, SchoolYear, Subject, Teacher,
};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::collections::HashMap;
use std::path::Path;

/// Maps a campus color name or hex into a soft pastel RGB suitable for print and grayscale.
fn campus_pastel_color(color_name: &str) -> Color {
    match color_name.to_lowercase().as_str() {
        "blue" | "#3b82f6" | "#60a5fa" => Color::RGB(0xDBEAFE), // blue-100
        "emerald" | "green" | "#10b981" => Color::RGB(0xD1FAE5), // emerald-100
        "amber" | "yellow" | "#f59e0b" => Color::RGB(0xFEF3C7), // amber-100
        "purple" | "violet" | "#8b5cf6" => Color::RGB(0xEDE9FE), // purple-100
        "rose" | "red" | "#ef4444" => Color::RGB(0xFEE2E2),     // red-100
        "cyan" | "#06b6d4" => Color::RGB(0xCFFAFE),             // cyan-100
        _ => Color::RGB(0xF1F5F9),                              // slate-100
    }
}

/// Exports a complete Plan workbook with 4 sheets:
/// 1. "Phân công" (Matrix with school info, signature block, A4 landscape setup)
/// 2. "Theo giáo viên" (One row per assignment with co-panelists)
/// 3. "Thống kê" (Workloads, quotas, deviations)
/// 4. "Tiêu chí" (Score report, rule weights, bounds)
#[allow(clippy::too_many_arguments)]
pub fn export_plan_workbook(
    target_path: &Path,
    plan_details: &PlanDetails,
    school_year: &SchoolYear,
    campuses: &[Campus],
    grades: &[Grade],
    exams: &[Exam],
    subjects: &[Subject],
    teachers: &[Teacher],
    settings: &AppSettings,
    rule_settings: &[RuleSetting],
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let is_draft = !plan_details.plan.is_final;

    // Formatting styles
    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center);

    let subtitle_format = Format::new()
        .set_italic()
        .set_font_size(11)
        .set_align(FormatAlign::Center);

    let draft_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::RGB(0xDC2626)) // red-600
        .set_align(FormatAlign::Center);

    let table_header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0xE2E8F0)) // slate-200
        .set_border(FormatBorder::Thin);

    let cell_center = Format::new()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let cell_left = Format::new()
        .set_font_size(10)
        .set_align(FormatAlign::Left)
        .set_border(FormatBorder::Thin);

    let header_bold_left = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Left);

    let header_center = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center);

    // Mappings for lookup
    let teacher_map: HashMap<i64, &Teacher> = teachers.iter().map(|t| (t.id.value(), t)).collect();
    let campus_map: HashMap<i64, &Campus> = campuses.iter().map(|c| (c.id.value(), c)).collect();

    // -------------------------------------------------------------------------
    // Sheet 0: "Bảng phân công (mẫu tổ)" (Q-Style Grid A4 Landscape)
    // -------------------------------------------------------------------------
    write_q_style_sheet(
        &mut workbook,
        plan_details,
        school_year,
        grades,
        exams,
        subjects,
        teachers,
        settings,
        is_draft,
    )?;

    // -------------------------------------------------------------------------
    // Sheet 1: "Phân công" (Matrix A4 Landscape)
    // -------------------------------------------------------------------------
    let sheet_matrix = workbook.add_worksheet();
    sheet_matrix.set_name("Phân công")?;

    // Page Setup
    sheet_matrix.set_paper_size(9); // A4
    sheet_matrix.set_landscape();
    sheet_matrix.set_print_fit_to_pages(1, 0); // 1 page wide
    sheet_matrix.set_repeat_rows(7, 7)?;
    sheet_matrix.set_footer("&RTrang &P / &N");

    sheet_matrix.set_column_width(0, 16)?; // Kỳ thi
    sheet_matrix.set_column_width(1, 14)?; // Khối
    sheet_matrix.set_column_width(2, 28)?; // Người ra đề 1
    sheet_matrix.set_column_width(3, 28)?; // Người ra đề 2
    sheet_matrix.set_column_width(4, 28)?; // Người phản biện

    // Header block: school info (left) and motto (right)
    let school_name = settings
        .school_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("TRƯỜNG THPT CHUYÊN")
        .to_uppercase();
    let dept_name = settings
        .department_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("TỔ CHUYÊN MÔN TOÁN")
        .to_uppercase();

    sheet_matrix.write_with_format(0, 0, &school_name, &header_bold_left)?;
    sheet_matrix.write_with_format(1, 0, &dept_name, &header_bold_left)?;

    sheet_matrix.write_with_format(0, 3, "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM", &header_center)?;
    let italic_center = Format::new()
        .set_italic()
        .set_font_size(10)
        .set_align(FormatAlign::Center);
    sheet_matrix.write_with_format(1, 3, "Độc lập - Tự do - Hạnh phúc", &italic_center)?;

    // Title
    let title_text = if is_draft {
        "[BẢN NHÁP] BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA".to_string()
    } else {
        "BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA".to_string()
    };
    sheet_matrix.merge_range(
        3,
        0,
        3,
        4,
        &title_text,
        if is_draft {
            &draft_format
        } else {
            &title_format
        },
    )?;

    let subtitle_text = format!(
        "Năm học: {} — Phương án: {}",
        school_year.name, plan_details.plan.name
    );
    sheet_matrix.merge_range(4, 0, 4, 4, &subtitle_text, &subtitle_format)?;

    // Table Headers
    let header_row = 7;
    sheet_matrix.write_with_format(header_row, 0, "Kỳ thi", &table_header_format)?;
    sheet_matrix.write_with_format(header_row, 1, "Khối", &table_header_format)?;
    sheet_matrix.write_with_format(header_row, 2, "Người ra đề 1", &table_header_format)?;
    sheet_matrix.write_with_format(header_row, 3, "Người ra đề 2", &table_header_format)?;
    sheet_matrix.write_with_format(header_row, 4, "Người phản biện", &table_header_format)?;

    // Populate rows
    let mut current_row = header_row + 1;
    for exam in exams {
        let exam_start_row = current_row;
        for grade in grades {
            let panel_assignments: Vec<_> = plan_details
                .assignments
                .iter()
                .filter(|a| a.exam_id == exam.id && a.grade_id == grade.id)
                .collect();

            let mut setters = Vec::new();
            let mut reviewer = None;

            for a in &panel_assignments {
                match a.role {
                    Role::Setter => setters.push(a.teacher_id),
                    Role::Reviewer => reviewer = Some(a.teacher_id),
                }
            }

            sheet_matrix.write_with_format(current_row, 1, &grade.name, &cell_center)?;

            // Setter 1
            if let Some(&tid) = setters.first() {
                let (text, bg_color) = format_teacher_cell(tid.value(), &teacher_map, &campus_map);
                let fmt = Format::new()
                    .set_font_size(10)
                    .set_align(FormatAlign::Left)
                    .set_border(FormatBorder::Thin)
                    .set_background_color(bg_color);
                sheet_matrix.write_with_format(current_row, 2, &text, &fmt)?;
            } else {
                sheet_matrix.write_with_format(current_row, 2, "-", &cell_center)?;
            }

            // Setter 2
            if let Some(&tid) = setters.get(1) {
                let (text, bg_color) = format_teacher_cell(tid.value(), &teacher_map, &campus_map);
                let fmt = Format::new()
                    .set_font_size(10)
                    .set_align(FormatAlign::Left)
                    .set_border(FormatBorder::Thin)
                    .set_background_color(bg_color);
                sheet_matrix.write_with_format(current_row, 3, &text, &fmt)?;
            } else {
                sheet_matrix.write_with_format(current_row, 3, "-", &cell_center)?;
            }

            // Reviewer
            if let Some(tid) = reviewer {
                let (text, bg_color) = format_teacher_cell(tid.value(), &teacher_map, &campus_map);
                let fmt = Format::new()
                    .set_font_size(10)
                    .set_align(FormatAlign::Left)
                    .set_border(FormatBorder::Thin)
                    .set_background_color(bg_color);
                sheet_matrix.write_with_format(current_row, 4, &text, &fmt)?;
            } else {
                sheet_matrix.write_with_format(current_row, 4, "-", &cell_center)?;
            }

            current_row += 1;
        }

        // Merge exam name across its grade rows
        if current_row > exam_start_row {
            sheet_matrix.merge_range(
                exam_start_row,
                0,
                current_row - 1,
                0,
                &format!("{} ({})", exam.name, exam.code),
                &cell_center,
            )?;
        }
    }

    // Signature Block
    current_row += 2;
    let place = settings
        .place_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Hà Nội");
    let date_str = format!("{}, ngày ..... tháng ..... năm 20.....", place);
    sheet_matrix.write_with_format(current_row, 3, &date_str, &italic_center)?;

    current_row += 1;
    let signer_title = settings
        .signer_title
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("TỔ TRƯỞNG CHUYÊN MÔN")
        .to_uppercase();
    sheet_matrix.write_with_format(current_row, 3, &signer_title, &header_center)?;

    current_row += 4;
    let signer_name = settings
        .signer_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("(Ký và ghi rõ họ tên)");
    sheet_matrix.write_with_format(current_row, 3, signer_name, &header_center)?;

    // -------------------------------------------------------------------------
    // Sheet 2: "Theo giáo viên" (One row per assignment)
    // -------------------------------------------------------------------------
    let sheet_by_teacher = workbook.add_worksheet();
    sheet_by_teacher.set_name("Theo giáo viên")?;
    sheet_by_teacher.set_freeze_panes(1, 0)?;

    sheet_by_teacher.set_column_width(0, 12)?; // Mã GV
    sheet_by_teacher.set_column_width(1, 26)?; // Họ tên
    sheet_by_teacher.set_column_width(2, 22)?; // Phân hiệu
    sheet_by_teacher.set_column_width(3, 16)?; // Môn
    sheet_by_teacher.set_column_width(4, 14)?; // Kỳ thi
    sheet_by_teacher.set_column_width(5, 12)?; // Khối
    sheet_by_teacher.set_column_width(6, 16)?; // Vai trò
    sheet_by_teacher.set_column_width(7, 35)?; // Cùng ban với

    let teacher_headers = [
        "Mã GV",
        "Họ và tên",
        "Phân hiệu",
        "Môn",
        "Kỳ thi",
        "Khối",
        "Vai trò",
        "Cùng ban với",
    ];
    for (c_idx, h) in teacher_headers.iter().enumerate() {
        sheet_by_teacher.write_with_format(0, c_idx as u16, *h, &table_header_format)?;
    }

    // Sort assignments by teacher name, then exam sort_order
    let mut sorted_assignments = plan_details.assignments.clone();
    sorted_assignments.sort_by(|a, b| {
        let t_a = teacher_map.get(&a.teacher_id.value());
        let t_b = teacher_map.get(&b.teacher_id.value());
        let name_a = t_a.map(|t| t.full_name.as_str()).unwrap_or("");
        let name_b = t_b.map(|t| t.full_name.as_str()).unwrap_or("");
        match name_a.cmp(name_b) {
            std::cmp::Ordering::Equal => a.exam_id.value().cmp(&b.exam_id.value()),
            other => other,
        }
    });

    for (idx, asg) in sorted_assignments.iter().enumerate() {
        let r = (idx + 1) as u32;
        let t = teacher_map.get(&asg.teacher_id.value());
        let campus = t.and_then(|t| campus_map.get(&t.campus_id.value()));
        let exam = exams.iter().find(|e| e.id == asg.exam_id);
        let grade = grades.iter().find(|g| g.id == asg.grade_id);
        let subject = subjects.iter().find(|s| s.id == asg.subject_id);

        let code_str = t.and_then(|t| t.code.as_deref()).unwrap_or("-");
        let name_str = t.map(|t| t.full_name.as_str()).unwrap_or("-");
        let campus_str = campus.map(|c| c.name.as_str()).unwrap_or("-");
        let subject_str = subject
            .map(|s| s.name.as_str())
            .unwrap_or(if subjects.is_empty() {
                "Toán"
            } else {
                &subjects[0].name
            });
        let exam_str = exam.map(|e| e.code.as_str()).unwrap_or("-");
        let grade_str = grade.map(|g| g.name.as_str()).unwrap_or("-");
        let role_str = match asg.role {
            Role::Setter => "Ra đề",
            Role::Reviewer => "Phản biện",
        };

        // Find co-panelists
        let co_panelists: Vec<String> = plan_details
            .assignments
            .iter()
            .filter(|o| {
                o.exam_id == asg.exam_id
                    && o.grade_id == asg.grade_id
                    && o.teacher_id != asg.teacher_id
            })
            .map(|o| {
                let ot = teacher_map.get(&o.teacher_id.value());
                let oname = ot.map(|t| t.full_name.as_str()).unwrap_or("?");
                let orole = match o.role {
                    Role::Setter => "Ra đề",
                    Role::Reviewer => "Phản biện",
                };
                format!("{oname} ({orole})")
            })
            .collect();

        let co_str = co_panelists.join("; ");

        sheet_by_teacher.write_with_format(r, 0, code_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 1, name_str, &cell_left)?;
        sheet_by_teacher.write_with_format(r, 2, campus_str, &cell_left)?;
        sheet_by_teacher.write_with_format(r, 3, subject_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 4, exam_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 5, grade_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 6, role_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 7, &co_str, &cell_left)?;
    }

    // -------------------------------------------------------------------------
    // Sheet 3: "Thống kê" (Workloads, Quotas, Deviations)
    // -------------------------------------------------------------------------
    let sheet_stats = workbook.add_worksheet();
    sheet_stats.set_name("Thống kê")?;
    sheet_stats.set_freeze_panes(1, 0)?;

    sheet_stats.set_column_width(0, 12)?; // Mã GV
    sheet_stats.set_column_width(1, 26)?; // Họ tên
    sheet_stats.set_column_width(2, 22)?; // Phân hiệu
    sheet_stats.set_column_width(3, 14)?; // Số đề ra
    sheet_stats.set_column_width(4, 16)?; // Số đề phản biện
    sheet_stats.set_column_width(5, 16)?; // Tổng nhiệm vụ
    sheet_stats.set_column_width(6, 14)?; // Hạn ngạch
    sheet_stats.set_column_width(7, 14)?; // Chênh lệch

    let stat_headers = [
        "Mã GV",
        "Họ và tên",
        "Phân hiệu",
        "Số đề ra",
        "Số phản biện",
        "Tổng nhiệm vụ",
        "Hạn ngạch",
        "Chênh lệch",
    ];
    for (c_idx, h) in stat_headers.iter().enumerate() {
        sheet_stats.write_with_format(0, c_idx as u16, *h, &table_header_format)?;
    }

    // Collect stats from score_report if available
    let teacher_stats = plan_details.score_report.as_ref().map(|sr| &sr.per_teacher);

    for (idx, t) in teachers.iter().enumerate() {
        let r = (idx + 1) as u32;
        let campus = campus_map.get(&t.campus_id.value());
        let campus_str = campus.map(|c| c.name.as_str()).unwrap_or("-");

        let stats = teacher_stats.and_then(|ts| ts.iter().find(|s| s.teacher_id == t.id));

        let setters_count = stats.map(|s| s.setter).unwrap_or_else(|| {
            plan_details
                .assignments
                .iter()
                .filter(|a| a.teacher_id == t.id && a.role == Role::Setter)
                .count()
        });

        let reviewer_count = stats.map(|s| s.reviewer).unwrap_or_else(|| {
            plan_details
                .assignments
                .iter()
                .filter(|a| a.teacher_id == t.id && a.role == Role::Reviewer)
                .count()
        });

        let total = setters_count + reviewer_count;
        let quota = stats.map(|s| s.quota).unwrap_or(0.0);
        let diff = (total as f64) - quota;

        sheet_stats.write_with_format(r, 0, t.code.as_deref().unwrap_or("-"), &cell_center)?;
        sheet_stats.write_with_format(r, 1, &t.full_name, &cell_left)?;
        sheet_stats.write_with_format(r, 2, campus_str, &cell_left)?;
        sheet_stats.write_with_format(r, 3, setters_count as i64, &cell_center)?;
        sheet_stats.write_with_format(r, 4, reviewer_count as i64, &cell_center)?;
        sheet_stats.write_with_format(r, 5, total as i64, &cell_center)?;
        sheet_stats.write_with_format(r, 6, quota, &cell_center)?;
        sheet_stats.write_with_format(r, 7, (diff * 100.0).round() / 100.0, &cell_center)?;
    }

    // -------------------------------------------------------------------------
    // Sheet 4: "Tiêu chí" (Score Report & Rules)
    // -------------------------------------------------------------------------
    let sheet_rules = workbook.add_worksheet();
    sheet_rules.set_name("Tiêu chí")?;
    sheet_rules.set_freeze_panes(1, 0)?;

    sheet_rules.set_column_width(0, 14)?; // Tiêu chí
    sheet_rules.set_column_width(1, 35)?; // Tên tiêu chí
    sheet_rules.set_column_width(2, 14)?; // Bật/Tắt
    sheet_rules.set_column_width(3, 14)?; // Trọng số
    sheet_rules.set_column_width(4, 18)?; // Số vi phạm
    sheet_rules.set_column_width(5, 18)?; // Đóng góp điểm

    let rule_headers = [
        "Mã tiêu chí",
        "Tên tiêu chí",
        "Trạng thái",
        "Trọng số",
        "Số vi phạm",
        "Đóng góp điểm",
    ];
    for (c_idx, h) in rule_headers.iter().enumerate() {
        sheet_rules.write_with_format(0, c_idx as u16, *h, &table_header_format)?;
    }

    let rule_scores = plan_details.score_report.as_ref().map(|sr| &sr.by_rule);

    for (idx, r_setting) in rule_settings.iter().enumerate() {
        let r = (idx + 1) as u32;
        let key_str = format!("{:?}", r_setting.key);
        let rule_name = rule_key_vietnamese_name(&r_setting.key);
        let status_str = if r_setting.enabled { "Bật" } else { "Tắt" };

        let (violations, contribution) = if let Some(scores) = rule_scores {
            if let Some(rs) = scores.iter().find(|s| s.rule == r_setting.key) {
                (rs.units as i64, rs.penalty)
            } else {
                (0, 0.0)
            }
        } else {
            (0, 0.0)
        };

        sheet_rules.write_with_format(r, 0, &key_str, &cell_center)?;
        sheet_rules.write_with_format(r, 1, rule_name, &cell_left)?;
        sheet_rules.write_with_format(r, 2, status_str, &cell_center)?;
        sheet_rules.write_with_format(r, 3, r_setting.weight, &cell_center)?;
        sheet_rules.write_with_format(r, 4, violations, &cell_center)?;
        sheet_rules.write_with_format(
            r,
            5,
            (contribution * 100.0).round() / 100.0,
            &cell_center,
        )?;
    }

    // Save to disk
    workbook.save(target_path)?;

    Ok(())
}

/// Helper to format teacher cell text: "Họ Tên (Mã PH)" and background pastel color.
fn format_teacher_cell(
    teacher_id: i64,
    teacher_map: &HashMap<i64, &Teacher>,
    campus_map: &HashMap<i64, &Campus>,
) -> (String, Color) {
    if let Some(t) = teacher_map.get(&teacher_id) {
        if let Some(c) = campus_map.get(&t.campus_id.value()) {
            let bg = campus_pastel_color(&c.color);
            (format!("{} ({})", t.full_name, c.code), bg)
        } else {
            (t.full_name.clone(), Color::RGB(0xF8FAFC))
        }
    } else {
        ("-".to_string(), Color::RGB(0xF8FAFC))
    }
}

/// Friendly Vietnamese names for rule keys.
fn rule_key_vietnamese_name(key: &exam_panel_core::domain::RuleKey) -> &'static str {
    use exam_panel_core::domain::RuleKey::*;
    match key {
        H1 => "H1 - Đủ số lượng và không trùng giáo viên",
        H2 => "H2 - Đúng chuyên môn môn học và khối dạy",
        H3 => "H3 - Đa dạng phân hiệu trong ban thi",
        H4 => "H4 - Giới hạn số nhiệm vụ trong mỗi kỳ thi",
        H5 => "H5 - Không phân công vào kỳ thi bận",
        H6 => "H6 - Tuân thủ khóa cố định (Pin / Forbid)",
        H7 => "H7 - Cân bằng định mức công việc",
        S1 => "S1 - Phân bổ số lượt phản biện",
        S2 => "S2 - Cân bằng vai trò ra đề / phản biện",
        S3 => "S3 - Tránh liên tiếp 2 kỳ làm cùng vai trò",
        S4 => "S4 - Tránh liên tiếp 2 kỳ làm cùng khối",
        S5 => "S5 - Tránh cặp đôi lặp lại",
        S6 => "S6 - Tránh ban thi toàn giáo viên dạy khối đó",
        S7 => "S7 - Phân bổ đều giữa các khối",
        S8 => "S8 - Phân hiệu chính cho môn học",
        S9 => "S9 - Hạn chế dồn nhiều nhiệm vụ trong một kỳ thi",
        S10 => "S10 - Phản biện đủ các môn phụ trách",
    }
}

/// Writes the Q-style assignment grid and attached totals panel (Sheet 0: "Bảng phân công (mẫu tổ)").
#[allow(clippy::too_many_arguments)]
fn write_q_style_sheet(
    workbook: &mut Workbook,
    plan_details: &PlanDetails,
    school_year: &SchoolYear,
    grades: &[Grade],
    exams: &[Exam],
    subjects: &[Subject],
    teachers: &[Teacher],
    settings: &AppSettings,
    is_draft: bool,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Bảng phân công (mẫu tổ)")?;

    sheet.set_paper_size(9); // A4
    sheet.set_landscape();
    sheet.set_print_fit_to_pages(1, 0); // 1 page wide
    sheet.set_repeat_rows(6, 7)?;
    sheet.set_footer("&RTrang &P / &N");

    // Formatting styles
    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);

    let draft_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::RGB(0xDC2626))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);

    let subtitle_format = Format::new()
        .set_italic()
        .set_font_size(11)
        .set_align(FormatAlign::Center);

    let table_header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xE2E8F0)) // slate-200
        .set_border(FormatBorder::Thin);

    let subject_header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xF1F5F9)) // slate-100
        .set_border(FormatBorder::Thin);

    let cell_center = Format::new()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let cell_left = Format::new()
        .set_font_size(10)
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let bold_center = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let header_bold_left = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Left);

    let header_center = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Center);

    let italic_center = Format::new()
        .set_italic()
        .set_font_size(10)
        .set_align(FormatAlign::Center);

    // School info
    let school_name = settings
        .school_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("TRƯỜNG THPT CHUYÊN")
        .to_uppercase();
    let dept_name = settings
        .department_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("TỔ CHUYÊN MÔN TOÁN")
        .to_uppercase();

    sheet.write_with_format(0, 0, &school_name, &header_bold_left)?;
    sheet.write_with_format(1, 0, &dept_name, &header_bold_left)?;

    let num_sub = subjects.len().max(1);
    let grid_cols = 2 + (grades.len() * num_sub) as u16;
    let spacer_col = grid_cols;
    let gv_col = spacer_col + 1;
    let tot_col = gv_col + 1;
    let de_col = gv_col + 2;
    let pb_col = gv_col + 3;
    let last_col = pb_col + exams.len() as u16;

    let motto_col = (last_col.saturating_sub(4)).max(gv_col);
    sheet.merge_range(
        0,
        motto_col,
        0,
        last_col,
        "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
        &header_center,
    )?;
    sheet.merge_range(
        1,
        motto_col,
        1,
        last_col,
        "Độc lập - Tự do - Hạnh phúc",
        &italic_center,
    )?;

    // Title
    let title_text = if is_draft {
        "[BẢN NHÁP] BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA".to_string()
    } else {
        "BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA".to_string()
    };
    sheet.merge_range(
        3,
        0,
        3,
        last_col,
        &title_text,
        if is_draft {
            &draft_format
        } else {
            &title_format
        },
    )?;

    let subtitle_text = format!(
        "Năm học: {} — Phương án: {}",
        school_year.name, plan_details.plan.name
    );
    sheet.merge_range(4, 0, 4, last_col, &subtitle_text, &subtitle_format)?;

    // Column widths
    sheet.set_column_width(0, 10)?; // Kỳ thi
    sheet.set_column_width(1, 10)?; // Vai trò
    for c in 2..spacer_col {
        sheet.set_column_width(c, 14)?;
    }
    sheet.set_column_width(spacer_col, 3)?;
    sheet.set_column_width(gv_col, 15)?;
    sheet.set_column_width(tot_col, 12)?;
    sheet.set_column_width(de_col, 8)?;
    sheet.set_column_width(pb_col, 8)?;
    for i in 0..exams.len() {
        sheet.set_column_width(pb_col + 1 + i as u16, 8)?;
    }

    // Header rows 6 & 7
    sheet.merge_range(6, 0, 7, 1, "Kì thi/khối", &table_header_format)?;

    for (g_idx, grade) in grades.iter().enumerate() {
        let start_col = 2 + (g_idx * num_sub) as u16;
        let end_col = start_col + num_sub as u16 - 1;
        if num_sub > 1 {
            sheet.merge_range(6, start_col, 6, end_col, &grade.name, &table_header_format)?;
            for (s_idx, s) in subjects.iter().enumerate() {
                let scol = start_col + s_idx as u16;
                sheet.write_with_format(7, scol, &s.code, &subject_header_format)?;
            }
        } else {
            sheet.merge_range(
                6,
                start_col,
                7,
                start_col,
                &grade.name,
                &table_header_format,
            )?;
        }
    }

    // Totals headers
    sheet.merge_range(6, gv_col, 7, gv_col, "GV", &table_header_format)?;
    sheet.merge_range(
        6,
        tot_col,
        7,
        tot_col,
        "Tổng lượt n.vụ",
        &table_header_format,
    )?;
    sheet.merge_range(6, de_col, 7, de_col, "Đề", &table_header_format)?;
    sheet.merge_range(6, pb_col, 7, pb_col, "PB", &table_header_format)?;
    for (e_idx, e) in exams.iter().enumerate() {
        let ecol = pb_col + 1 + e_idx as u16;
        sheet.merge_range(6, ecol, 7, ecol, &e.code, &table_header_format)?;
    }

    // Data rows
    let max_setters = subjects.iter().map(|s| s.setters).max().unwrap_or(1);
    let max_reviewers = subjects.iter().map(|s| s.reviewers).max().unwrap_or(1);
    let exam_block_size = max_setters + max_reviewers;

    let mut current_grid_row = 8u32;
    for (e_idx, exam) in exams.iter().enumerate() {
        let exam_start = current_grid_row;
        let exam_end = exam_start + exam_block_size as u32 - 1;

        let bg_color = if e_idx % 2 == 0 {
            Color::RGB(0xFFFFFF)
        } else {
            Color::RGB(0xF8FAFC)
        };
        let exam_cell_center = Format::new()
            .set_font_size(10)
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_background_color(bg_color);

        let grid_cell_fmt = Format::new()
            .set_font_size(10)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_background_color(bg_color);

        sheet.merge_range(exam_start, 0, exam_end, 0, &exam.code, &exam_cell_center)?;

        for k in 0..max_setters {
            let r = exam_start + k as u32;
            sheet.write_with_format(r, 1, "Đề", &exam_cell_center)?;
            for (g_idx, grade) in grades.iter().enumerate() {
                for (s_idx, subject) in subjects.iter().enumerate() {
                    let col = 2 + (g_idx * num_sub + s_idx) as u16;
                    if k < subject.setters {
                        let teacher_name = plan_details
                            .assignments
                            .iter()
                            .find(|a| {
                                a.exam_id == exam.id
                                    && a.grade_id == grade.id
                                    && a.subject_id == subject.id
                                    && a.role == Role::Setter
                                    && a.position == k as usize
                            })
                            .and_then(|a| teachers.iter().find(|t| t.id == a.teacher_id))
                            .map(|t| {
                                t.display_name
                                    .as_deref()
                                    .filter(|s| !s.trim().is_empty())
                                    .unwrap_or(&t.full_name)
                            })
                            .unwrap_or("-");
                        sheet.write_with_format(r, col, teacher_name, &grid_cell_fmt)?;
                    } else {
                        sheet.write_with_format(r, col, "", &grid_cell_fmt)?;
                    }
                }
            }
        }

        for m in 0..max_reviewers {
            let r = exam_start + max_setters as u32 + m as u32;
            sheet.write_with_format(r, 1, "P.Biện", &exam_cell_center)?;
            for (g_idx, grade) in grades.iter().enumerate() {
                for (s_idx, subject) in subjects.iter().enumerate() {
                    let col = 2 + (g_idx * num_sub + s_idx) as u16;
                    if m < subject.reviewers {
                        let teacher_name = plan_details
                            .assignments
                            .iter()
                            .find(|a| {
                                a.exam_id == exam.id
                                    && a.grade_id == grade.id
                                    && a.subject_id == subject.id
                                    && a.role == Role::Reviewer
                                    && a.position == m as usize
                            })
                            .and_then(|a| teachers.iter().find(|t| t.id == a.teacher_id))
                            .map(|t| {
                                t.display_name
                                    .as_deref()
                                    .filter(|s| !s.trim().is_empty())
                                    .unwrap_or(&t.full_name)
                            })
                            .unwrap_or("-");
                        sheet.write_with_format(r, col, teacher_name, &grid_cell_fmt)?;
                    } else {
                        sheet.write_with_format(r, col, "", &grid_cell_fmt)?;
                    }
                }
            }
        }

        current_grid_row += exam_block_size as u32;
    }

    // Totals rows
    let mut current_tot_row = 8u32;
    let mut grand_total = 0;
    let mut grand_de = 0;
    let mut grand_pb = 0;
    let mut grand_exam_totals = vec![0; exams.len()];

    for teacher in teachers {
        let name = teacher
            .display_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&teacher.full_name);
        let teacher_assignments: Vec<_> = plan_details
            .assignments
            .iter()
            .filter(|a| a.teacher_id == teacher.id)
            .collect();
        let total_count = teacher_assignments.len();
        let de_count = teacher_assignments
            .iter()
            .filter(|a| a.role == Role::Setter)
            .count();
        let pb_count = teacher_assignments
            .iter()
            .filter(|a| a.role == Role::Reviewer)
            .count();

        sheet.write_with_format(current_tot_row, gv_col, name, &cell_left)?;
        sheet.write_with_format(current_tot_row, tot_col, total_count as i64, &cell_center)?;
        sheet.write_with_format(current_tot_row, de_col, de_count as i64, &cell_center)?;
        sheet.write_with_format(current_tot_row, pb_col, pb_count as i64, &cell_center)?;

        for (e_idx, exam) in exams.iter().enumerate() {
            let e_count = teacher_assignments
                .iter()
                .filter(|a| a.exam_id == exam.id)
                .count();
            sheet.write_with_format(
                current_tot_row,
                pb_col + 1 + e_idx as u16,
                e_count as i64,
                &cell_center,
            )?;
            grand_exam_totals[e_idx] += e_count;
        }

        grand_total += total_count;
        grand_de += de_count;
        grand_pb += pb_count;
        current_tot_row += 1;
    }

    // Totals bottom summary row
    sheet.write_with_format(current_tot_row, gv_col, "Tổng cộng", &bold_center)?;
    sheet.write_with_format(current_tot_row, tot_col, grand_total as i64, &bold_center)?;
    sheet.write_with_format(current_tot_row, de_col, grand_de as i64, &bold_center)?;
    sheet.write_with_format(current_tot_row, pb_col, grand_pb as i64, &bold_center)?;
    for (e_idx, _) in exams.iter().enumerate() {
        sheet.write_with_format(
            current_tot_row,
            pb_col + 1 + e_idx as u16,
            grand_exam_totals[e_idx] as i64,
            &bold_center,
        )?;
    }
    current_tot_row += 1;

    // Signature block
    let bottom_row = current_grid_row.max(current_tot_row);
    let sig_row = bottom_row + 2;

    let place = settings
        .place_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Hà Nội");
    let title = settings
        .signer_title
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Tổ trưởng chuyên môn");
    let name = settings
        .signer_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("");

    let date_str = format!("{}, ngày ... tháng ... năm 20...", place);
    sheet.merge_range(
        sig_row,
        gv_col,
        sig_row,
        last_col,
        &date_str,
        &italic_center,
    )?;
    sheet.merge_range(
        sig_row + 1,
        gv_col,
        sig_row + 1,
        last_col,
        title,
        &bold_center,
    )?;
    sheet.merge_range(
        sig_row + 5,
        gv_col,
        sig_row + 5,
        last_col,
        name,
        &bold_center,
    )?;

    Ok(())
}
