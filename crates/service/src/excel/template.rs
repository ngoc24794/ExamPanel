//! Template generation for Excel data import (`mau-nhap-du-lieu.xlsx`).

use exam_panel_core::domain::Campus;
use rust_xlsxwriter::{Color, DataValidation, Format, FormatBorder, Workbook, XlsxError};
use std::path::Path;

/// Generates the standard data import Excel workbook template (`mau-nhap-du-lieu.xlsx`).
///
/// Pre-populates the "Phân hiệu" sheet with existing campuses (if any) or defaults,
/// configures data validation dropdowns, column widths, and freezes header rows.
pub fn generate_import_template(target_path: &Path, campuses: &[Campus]) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    // Standard styling formats
    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::RGB(0x1E3A8A)); // navy

    let section_header_format = Format::new()
        .set_bold()
        .set_font_size(11)
        .set_font_color(Color::RGB(0x1E3A8A));

    let header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_background_color(Color::RGB(0xDBEAFE)) // sky-100
        .set_border(FormatBorder::Thin);

    let required_header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_font_color(Color::RGB(0x991B1B)) // red-800
        .set_background_color(Color::RGB(0xFEE2E2)) // red-100
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_font_size(10)
        .set_border(FormatBorder::Thin);

    let instruction_format = Format::new().set_font_size(10).set_text_wrap();

    // -------------------------------------------------------------------------
    // 1. Sheet "Hướng dẫn"
    // -------------------------------------------------------------------------
    let guide_sheet = workbook.add_worksheet();
    guide_sheet.set_name("Hướng dẫn")?;
    guide_sheet.set_column_width(0, 5)?;
    guide_sheet.set_column_width(1, 28)?;
    guide_sheet.set_column_width(2, 60)?;

    guide_sheet.write_with_format(1, 1, "HƯỚNG DẪN NHẬP DỮ LIỆU EXAMPANEL", &title_format)?;

    let mut r = 3;
    guide_sheet.write_with_format(r, 1, "1. Sheet 'Phân hiệu'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã phân hiệu (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã định danh duy nhất của phân hiệu/cơ sở (VD: CS1, CS2).",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Tên phân hiệu (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Tên đầy đủ của phân hiệu (VD: Cơ sở 1 - Ba Đình).",
        &instruction_format,
    )?;

    r += 2;
    guide_sheet.write_with_format(r, 1, "2. Sheet 'Giáo viên'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã GV", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã giáo viên (tùy chọn, VD: GV001). Giúp nhận diện chính xác khi trùng tên.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Họ và tên (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Họ tên đầy đủ của giáo viên (VD: Nguyễn Văn An). Bắt buộc.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã phân hiệu (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Chọn mã phân hiệu từ danh sách thả xuống hoặc nhập đúng mã tại sheet Phân hiệu.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Khối dạy (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Các khối giáo viên có thể dạy. Hỗ trợ: '10, 11', '10; 12', 'Khối 10, 11, 12', 'K10, K11'.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Hệ số tải", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Hệ số phân công: 1 (đầy đủ), 0.75, 0.5 (nửa tải), 0.25, 0 (không phân công). Mặc định 1.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Đang dạy", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "'Có' (đang công tác) hoặc 'Không' (tạm nghỉ/nghỉ thai sản/đi học). Mặc định 'Có'.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Ghi chú", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Ghi chú bổ sung (tổ trưởng, kiêm nhiệm, v.v.). Tùy chọn.",
        &instruction_format,
    )?;

    r += 2;
    guide_sheet.write_with_format(r, 1, "3. Sheet 'Lịch vắng'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã GV hoặc Họ tên (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã giáo viên hoặc họ tên giáo viên báo bận lịch coi/chấm thi.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã kỳ thi (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã kỳ thi không thể tham gia: GK1 (Giữa K1), CK1 (Cuối K1), GK2 (Giữa K2), CK2 (Cuối K2).",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Lý do", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Lý do báo bận (VD: Tập huấn chuyên môn, công tác). Tùy chọn.",
        &instruction_format,
    )?;

    // -------------------------------------------------------------------------
    // 2. Sheet "Phân hiệu"
    // -------------------------------------------------------------------------
    let campus_sheet = workbook.add_worksheet();
    campus_sheet.set_name("Phân hiệu")?;
    campus_sheet.set_freeze_panes(1, 0)?;
    campus_sheet.set_column_width(0, 18)?;
    campus_sheet.set_column_width(1, 35)?;

    campus_sheet.write_with_format(0, 0, "Mã phân hiệu (*)", &required_header_format)?;
    campus_sheet.write_with_format(0, 1, "Tên phân hiệu (*)", &required_header_format)?;

    let mut campus_codes = Vec::new();
    if !campuses.is_empty() {
        for (i, c) in campuses.iter().enumerate() {
            let row = (i + 1) as u32;
            campus_sheet.write_with_format(row, 0, &c.code, &cell_format)?;
            campus_sheet.write_with_format(row, 1, &c.name, &cell_format)?;
            campus_codes.push(c.code.clone());
        }
    } else {
        let default_campuses = [
            ("CS1", "Phân hiệu 1 - Ba Đình"),
            ("CS2", "Phân hiệu 2 - Cầu Giấy"),
            ("CS3", "Phân hiệu 3 - Hà Đông"),
            ("CS4", "Phân hiệu 4 - Hoàn Kiếm"),
        ];
        for (i, (code, name)) in default_campuses.iter().enumerate() {
            let row = (i + 1) as u32;
            campus_sheet.write_with_format(row, 0, *code, &cell_format)?;
            campus_sheet.write_with_format(row, 1, *name, &cell_format)?;
            campus_codes.push((*code).to_string());
        }
    }

    // -------------------------------------------------------------------------
    // 3. Sheet "Giáo viên"
    // -------------------------------------------------------------------------
    let teacher_sheet = workbook.add_worksheet();
    teacher_sheet.set_name("Giáo viên")?;
    teacher_sheet.set_freeze_panes(1, 0)?;
    teacher_sheet.set_column_width(0, 14)?;
    teacher_sheet.set_column_width(1, 28)?;
    teacher_sheet.set_column_width(2, 18)?;
    teacher_sheet.set_column_width(3, 20)?;
    teacher_sheet.set_column_width(4, 14)?;
    teacher_sheet.set_column_width(5, 14)?;
    teacher_sheet.set_column_width(6, 25)?;

    teacher_sheet.write_with_format(0, 0, "Mã GV", &header_format)?;
    teacher_sheet.write_with_format(0, 1, "Họ và tên (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 2, "Mã phân hiệu (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 3, "Khối dạy (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 4, "Hệ số tải", &header_format)?;
    teacher_sheet.write_with_format(0, 5, "Đang dạy", &header_format)?;
    teacher_sheet.write_with_format(0, 6, "Ghi chú", &header_format)?;

    // Sample data rows (18 teachers across campuses for realistic and feasible scheduling)
    let sample_teachers = [
        (
            "GV001",
            "Nguyễn Văn An",
            "CS1",
            "10, 11",
            "1",
            "Có",
            "Tổ trưởng chuyên môn",
        ),
        (
            "GV002",
            "Trần Thị Bình",
            "CS1",
            "10, 12",
            "1",
            "Có",
            "Tổ phó chuyên môn",
        ),
        ("GV003", "Lê Văn Cường", "CS2", "11, 12", "1", "Có", ""),
        ("GV004", "Phạm Thị Dung", "CS2", "10, 11", "1", "Có", ""),
        ("GV005", "Hoàng Văn Em", "CS1", "11, 12", "1", "Có", ""),
        ("GV006", "Đỗ Thị Giang", "CS2", "10, 12", "1", "Có", ""),
        (
            "GV007",
            "Vũ Hải Hà",
            "CS1",
            "10, 11, 12",
            "0.5",
            "Có",
            "Giảm tải 50%",
        ),
        ("GV008", "Bùi Văn Hùng", "CS2", "10, 11", "1", "Có", ""),
        ("GV009", "Ngô Thị Mai", "CS1", "11, 12", "1", "Có", ""),
        ("GV010", "Đinh Văn Nam", "CS2", "10, 12", "1", "Có", ""),
        ("GV011", "Lý Thị Nga", "CS1", "10, 11", "1", "Có", ""),
        ("GV012", "Trương Văn Phúc", "CS2", "11, 12", "1", "Có", ""),
        ("GV013", "Võ Thị Quỳnh", "CS1", "10, 12", "1", "Có", ""),
        ("GV014", "Dương Văn Sơn", "CS2", "10, 11", "1", "Có", ""),
        ("GV015", "Tạ Thị Thảo", "CS1", "11, 12", "1", "Có", ""),
        ("GV016", "Lương Văn Tuấn", "CS2", "10, 12", "1", "Có", ""),
        ("GV017", "Hồ Thị Uyên", "CS1", "10, 11, 12", "1", "Có", ""),
        ("GV018", "Phan Văn Vinh", "CS2", "10, 11, 12", "1", "Có", ""),
    ];

    for (i, (code, name, campus, grades, load, active, note)) in sample_teachers.iter().enumerate()
    {
        let row = (i + 1) as u32;
        teacher_sheet.write_with_format(row, 0, *code, &cell_format)?;
        teacher_sheet.write_with_format(row, 1, *name, &cell_format)?;
        teacher_sheet.write_with_format(row, 2, *campus, &cell_format)?;
        teacher_sheet.write_with_format(row, 3, *grades, &cell_format)?;
        teacher_sheet.write_with_format(row, 4, *load, &cell_format)?;
        teacher_sheet.write_with_format(row, 5, *active, &cell_format)?;
        teacher_sheet.write_with_format(row, 6, *note, &cell_format)?;
    }

    // Data validations for Teachers sheet (rows 1 to 500)
    // Campus code dropdown
    if !campus_codes.is_empty() {
        let val_campus = DataValidation::new().allow_list_strings(&campus_codes)?;
        teacher_sheet.add_data_validation(1, 2, 500, 2, &val_campus)?;
    }

    // Load weight dropdown: 1, 0.75, 0.5, 0.25, 0
    let val_load = DataValidation::new().allow_list_strings(&["1", "0.75", "0.5", "0.25", "0"])?;
    teacher_sheet.add_data_validation(1, 4, 500, 4, &val_load)?;

    // Active dropdown: Có, Không
    let val_active = DataValidation::new().allow_list_strings(&["Có", "Không"])?;
    teacher_sheet.add_data_validation(1, 5, 500, 5, &val_active)?;

    // -------------------------------------------------------------------------
    // 4. Sheet "Lịch vắng"
    // -------------------------------------------------------------------------
    let unavail_sheet = workbook.add_worksheet();
    unavail_sheet.set_name("Lịch vắng")?;
    unavail_sheet.set_freeze_panes(1, 0)?;
    unavail_sheet.set_column_width(0, 28)?;
    unavail_sheet.set_column_width(1, 18)?;
    unavail_sheet.set_column_width(2, 40)?;

    unavail_sheet.write_with_format(0, 0, "Mã GV hoặc Họ tên (*)", &required_header_format)?;
    unavail_sheet.write_with_format(0, 1, "Mã kỳ thi (*)", &required_header_format)?;
    unavail_sheet.write_with_format(0, 2, "Lý do", &header_format)?;

    // Sample row
    unavail_sheet.write_with_format(1, 0, "GV001", &cell_format)?;
    unavail_sheet.write_with_format(1, 1, "GK1", &cell_format)?;
    unavail_sheet.write_with_format(1, 2, "Tập huấn chuyên môn Sở GD&ĐT", &cell_format)?;

    // Exam code validation dropdown: GK1, CK1, GK2, CK2
    let val_exam = DataValidation::new().allow_list_strings(&["GK1", "CK1", "GK2", "CK2"])?;
    unavail_sheet.add_data_validation(1, 1, 500, 1, &val_exam)?;

    // Save workbook to file
    workbook.save(target_path)?;

    Ok(())
}
