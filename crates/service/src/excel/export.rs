//! Excel plan export implementation using rust_xlsxwriter.

use crate::dto::{AppSettings, PlanDetails};
use exam_panel_core::domain::{Campus, Exam, Grade, Role, RuleSetting, SchoolYear, Teacher};
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
    sheet_by_teacher.set_column_width(3, 14)?; // Kỳ thi
    sheet_by_teacher.set_column_width(4, 12)?; // Khối
    sheet_by_teacher.set_column_width(5, 16)?; // Vai trò
    sheet_by_teacher.set_column_width(6, 35)?; // Cùng ban với

    let teacher_headers = [
        "Mã GV",
        "Họ và tên",
        "Phân hiệu",
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

        let code_str = t.and_then(|t| t.code.as_deref()).unwrap_or("-");
        let name_str = t.map(|t| t.full_name.as_str()).unwrap_or("-");
        let campus_str = campus.map(|c| c.name.as_str()).unwrap_or("-");
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
        sheet_by_teacher.write_with_format(r, 3, exam_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 4, grade_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 5, role_str, &cell_center)?;
        sheet_by_teacher.write_with_format(r, 6, &co_str, &cell_left)?;
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
        H1 => "H1 - Một vai trò mỗi ban",
        H2 => "H2 - Không vượt quá một vai trò mỗi kỳ thi",
        H3 => "H3 - Đúng khối chuyên môn",
        H4 => "H4 - Đa dạng phân hiệu (cùng ban khác cơ sở)",
        H5 => "H5 - Không phân công vào kỳ thi bận",
        H6 => "H6 - Tuân thủ khóa cứng (Pin/Forbid)",
        H7 => "H7 - Cân bằng tải tuyệt đối",
        S1 => "S1 - Phân bổ tải công việc đều",
        S2 => "S2 - Cân bằng lượt ra đề / phản biện",
        S3 => "S3 - Hạn chế làm việc liên tiếp",
        S4 => "S4 - Đa dạng phân hiệu mở rộng",
        S5 => "S5 - Luân chuyển đối tác làm việc",
        S6 => "S6 - Phân bổ khối dạy hợp lý",
        S7 => "S7 - Cân bằng tỷ lệ cơ sở",
        S8 => "S8 - Ưu tiên phân hiệu giáo viên",
    }
}
