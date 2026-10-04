//! Template generation for Excel data import (`mau-nhap-du-lieu.xlsx`).

use exam_panel_core::domain::Campus;
use rust_xlsxwriter::{Color, DataValidation, Format, FormatBorder, Workbook, XlsxError};
use std::path::Path;

/// Generates the standard data import Excel workbook template v2 (`mau-nhap-du-lieu.xlsx`).
///
/// Pre-populates the "Phân hiệu", "Môn", and "Môn đảm nhiệm" sheets with defaults or existing data,
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

    guide_sheet.write_with_format(
        1,
        1,
        "HƯỚNG DẪN NHẬP DỮ LIỆU EXAMPANEL (TEMPLATE V2)",
        &title_format,
    )?;

    let mut r = 3;
    guide_sheet.write_with_format(r, 1, "1. Sheet 'Phân hiệu'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã phân hiệu (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã định danh duy nhất của phân hiệu (VD: PH1, PH2). Bắt buộc.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Tên phân hiệu (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Tên đầy đủ của phân hiệu (VD: Phân hiệu 1 - Ba Đình). Bắt buộc.",
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
    guide_sheet.write_with_format(r, 1, "Cách gọi", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Tên thường gọi/hiển thị trong bảng phân công (VD: T An, C Bình). Tùy chọn.",
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
    r += 1;
    guide_sheet.write_with_format(r, 1, "Chỉ tiêu riêng", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Chỉ định số lượt nhiệm vụ cả năm cho giáo viên (VD: 12). Tùy chọn, ghi đè công thức q.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Số việc tối đa mỗi kỳ", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Giới hạn số nhiệm vụ tối đa trong một kỳ thi cho giáo viên này (VD: 3). Tùy chọn.",
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

    r += 2;
    guide_sheet.write_with_format(r, 1, "4. Sheet 'Môn'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã môn (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã môn học viết tắt (VD: VL, CN). Bắt buộc và duy nhất.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Tên môn (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Tên đầy đủ của môn học (VD: Vật lí, Công nghệ). Bắt buộc.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Số người ra đề", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Số giáo viên ra đề mỗi ban đề môn này (mặc định 2 cho Vật lí, 1 cho Công nghệ).",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Số người phản biện", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Số giáo viên phản biện mỗi ban đề (mặc định 1).",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Số phân hiệu tối thiểu", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Số phân hiệu tối thiểu đại diện trong mỗi ban đề (mặc định 2 theo H3).",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Màu", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã màu định dạng hex (VD: #2563EB, #10B981) dùng để hiển thị trên bảng tổ.",
        &instruction_format,
    )?;

    r += 2;
    guide_sheet.write_with_format(r, 1, "5. Sheet 'Môn đảm nhiệm'", &section_header_format)?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã GV hoặc Cách gọi (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã giáo viên hoặc cách gọi hoặc họ tên giáo viên được phân công chuyên môn.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Mã môn (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Mã môn tương ứng tại sheet Môn (VD: VL, CN). Bắt buộc.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Vai trò (*)", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "Vai trò được phân công: 'Cả hai', 'Ra đề', hoặc 'Phản biện'. Mặc định 'Cả hai'.",
        &instruction_format,
    )?;
    r += 1;
    guide_sheet.write_with_format(r, 1, "Phạm vi", &header_format)?;
    guide_sheet.write_with_format(
        r,
        2,
        "'Theo khối dạy' (chỉ các khối GV đang trực tiếp giảng dạy) hoặc 'Mọi khối' (được phân công mọi khối). Mặc định 'Theo khối dạy'.",
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
            ("PH1", "Phân hiệu 1"),
            ("PH2", "Phân hiệu 2"),
            ("PH3", "Phân hiệu 3"),
            ("PH4", "Phân hiệu 4"),
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
    teacher_sheet.set_column_width(2, 16)?; // Cách gọi
    teacher_sheet.set_column_width(3, 18)?; // Mã phân hiệu
    teacher_sheet.set_column_width(4, 20)?; // Khối dạy
    teacher_sheet.set_column_width(5, 14)?; // Hệ số tải
    teacher_sheet.set_column_width(6, 14)?; // Đang dạy
    teacher_sheet.set_column_width(7, 25)?; // Ghi chú
    teacher_sheet.set_column_width(8, 16)?; // Chỉ tiêu riêng
    teacher_sheet.set_column_width(9, 22)?; // Số việc tối đa mỗi kỳ

    teacher_sheet.write_with_format(0, 0, "Mã GV", &header_format)?;
    teacher_sheet.write_with_format(0, 1, "Họ và tên (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 2, "Cách gọi", &header_format)?;
    teacher_sheet.write_with_format(0, 3, "Mã phân hiệu (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 4, "Khối dạy (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 5, "Hệ số tải", &header_format)?;
    teacher_sheet.write_with_format(0, 6, "Đang dạy", &header_format)?;
    teacher_sheet.write_with_format(0, 7, "Ghi chú", &header_format)?;
    teacher_sheet.write_with_format(0, 8, "Chỉ tiêu riêng", &header_format)?;
    teacher_sheet.write_with_format(0, 9, "Số việc tối đa mỗi kỳ", &header_format)?;

    // Sample data rows (18 teachers across campuses for realistic and feasible scheduling)
    let sample_teachers = [
        (
            "GV001",
            "Nguyễn Văn An",
            "T An",
            "PH1",
            "10, 11",
            "1",
            "Có",
            "Tổ trưởng chuyên môn",
            "",
            "",
        ),
        (
            "GV002",
            "Trần Thị Bình",
            "C Bình",
            "PH1",
            "10, 12",
            "1",
            "Có",
            "Tổ phó chuyên môn",
            "",
            "",
        ),
        (
            "GV003",
            "Lê Văn Cường",
            "T Cường",
            "PH2",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV004",
            "Phạm Thị Dung",
            "C Dung",
            "PH2",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV005",
            "Hoàng Văn Em",
            "T Em",
            "PH1",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV006",
            "Đỗ Thị Giang",
            "C Giang",
            "PH2",
            "10, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV007",
            "Vũ Hải Hà",
            "T Hà",
            "PH1",
            "10, 11, 12",
            "0.5",
            "Có",
            "Giảm tải 50%",
            "",
            "",
        ),
        (
            "GV008",
            "Bùi Văn Hùng",
            "T Hùng",
            "PH2",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV009",
            "Ngô Thị Mai",
            "C Mai",
            "PH1",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV010",
            "Đinh Văn Nam",
            "T Nam",
            "PH2",
            "10, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV011",
            "Lý Thị Nga",
            "C Nga",
            "PH1",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV012",
            "Trương Văn Phúc",
            "T Phúc",
            "PH2",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV013",
            "Võ Thị Quỳnh",
            "C Quỳnh",
            "PH1",
            "10, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV014",
            "Dương Văn Sơn",
            "T Sơn",
            "PH2",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV015",
            "Tạ Thị Thảo",
            "C Thảo",
            "PH1",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV016",
            "Lương Văn Tuấn",
            "T Tuấn",
            "PH2",
            "10, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV017",
            "Hồ Thị Uyên",
            "C Uyên",
            "PH1",
            "10, 11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV018",
            "Phan Văn Vinh",
            "T Vinh",
            "PH2",
            "10, 11, 12",
            "1",
            "Có",
            "",
            "12",
            "3",
        ),
    ];

    for (i, (code, name, display_name, campus, grades, load, active, note, quota, max_tasks)) in
        sample_teachers.iter().enumerate()
    {
        let row = (i + 1) as u32;
        teacher_sheet.write_with_format(row, 0, *code, &cell_format)?;
        teacher_sheet.write_with_format(row, 1, *name, &cell_format)?;
        teacher_sheet.write_with_format(row, 2, *display_name, &cell_format)?;
        teacher_sheet.write_with_format(row, 3, *campus, &cell_format)?;
        teacher_sheet.write_with_format(row, 4, *grades, &cell_format)?;
        teacher_sheet.write_with_format(row, 5, *load, &cell_format)?;
        teacher_sheet.write_with_format(row, 6, *active, &cell_format)?;
        teacher_sheet.write_with_format(row, 7, *note, &cell_format)?;
        if !quota.is_empty() {
            teacher_sheet.write_with_format(row, 8, *quota, &cell_format)?;
        }
        if !max_tasks.is_empty() {
            teacher_sheet.write_with_format(row, 9, *max_tasks, &cell_format)?;
        }
    }

    // Data validations for Teachers sheet (rows 1 to 500)
    if !campus_codes.is_empty() {
        let val_campus = DataValidation::new().allow_list_strings(&campus_codes)?;
        teacher_sheet.add_data_validation(1, 3, 500, 3, &val_campus)?;
    }

    let val_load = DataValidation::new().allow_list_strings(&["1", "0.75", "0.5", "0.25", "0"])?;
    teacher_sheet.add_data_validation(1, 5, 500, 5, &val_load)?;

    let val_active = DataValidation::new().allow_list_strings(&["Có", "Không"])?;
    teacher_sheet.add_data_validation(1, 6, 500, 6, &val_active)?;

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

    unavail_sheet.write_with_format(1, 0, "GV001", &cell_format)?;
    unavail_sheet.write_with_format(1, 1, "GK1", &cell_format)?;
    unavail_sheet.write_with_format(1, 2, "Tập huấn chuyên môn Sở GD&ĐT", &cell_format)?;

    let val_exam = DataValidation::new().allow_list_strings(&["GK1", "CK1", "GK2", "CK2"])?;
    unavail_sheet.add_data_validation(1, 1, 500, 1, &val_exam)?;

    // -------------------------------------------------------------------------
    // 5. Sheet "Môn"
    // -------------------------------------------------------------------------
    let subject_sheet = workbook.add_worksheet();
    subject_sheet.set_name("Môn")?;
    subject_sheet.set_freeze_panes(1, 0)?;
    subject_sheet.set_column_width(0, 16)?;
    subject_sheet.set_column_width(1, 25)?;
    subject_sheet.set_column_width(2, 20)?;
    subject_sheet.set_column_width(3, 20)?;
    subject_sheet.set_column_width(4, 25)?;
    subject_sheet.set_column_width(5, 18)?;

    subject_sheet.write_with_format(0, 0, "Mã môn (*)", &required_header_format)?;
    subject_sheet.write_with_format(0, 1, "Tên môn (*)", &required_header_format)?;
    subject_sheet.write_with_format(0, 2, "Số người ra đề", &header_format)?;
    subject_sheet.write_with_format(0, 3, "Số người phản biện", &header_format)?;
    subject_sheet.write_with_format(0, 4, "Số phân hiệu tối thiểu", &header_format)?;
    subject_sheet.write_with_format(0, 5, "Màu", &header_format)?;

    // Sample subjects
    let default_subjects = [
        ("VL", "Vật lí", 2, 1, 2, "#2563EB"),
        ("CN", "Công nghệ", 1, 1, 2, "#10B981"),
    ];
    for (i, (code, name, setters, reviewers, min_camp, color)) in
        default_subjects.iter().enumerate()
    {
        let row = (i + 1) as u32;
        subject_sheet.write_with_format(row, 0, *code, &cell_format)?;
        subject_sheet.write_with_format(row, 1, *name, &cell_format)?;
        subject_sheet.write_with_format(row, 2, *setters, &cell_format)?;
        subject_sheet.write_with_format(row, 3, *reviewers, &cell_format)?;
        subject_sheet.write_with_format(row, 4, *min_camp, &cell_format)?;
        subject_sheet.write_with_format(row, 5, *color, &cell_format)?;
    }

    // -------------------------------------------------------------------------
    // 6. Sheet "Môn đảm nhiệm"
    // -------------------------------------------------------------------------
    let comp_sheet = workbook.add_worksheet();
    comp_sheet.set_name("Môn đảm nhiệm")?;
    comp_sheet.set_freeze_panes(1, 0)?;
    comp_sheet.set_column_width(0, 26)?;
    comp_sheet.set_column_width(1, 18)?;
    comp_sheet.set_column_width(2, 20)?;
    comp_sheet.set_column_width(3, 24)?;

    comp_sheet.write_with_format(0, 0, "Mã GV hoặc Cách gọi (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 1, "Mã môn (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 2, "Vai trò (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 3, "Phạm vi", &header_format)?;

    // Sample competencies covering all 18 teachers
    for i in 1..=15 {
        let code = format!("GV{i:03}");
        comp_sheet.write_with_format(i as u32, 0, &code, &cell_format)?;
        comp_sheet.write_with_format(i as u32, 1, "VL", &cell_format)?;
        comp_sheet.write_with_format(i as u32, 2, "Cả hai", &cell_format)?;
        comp_sheet.write_with_format(i as u32, 3, "Theo khối dạy", &cell_format)?;
    }
    for i in 16..=18 {
        let code = format!("GV{i:03}");
        comp_sheet.write_with_format(i as u32, 0, &code, &cell_format)?;
        comp_sheet.write_with_format(i as u32, 1, "CN", &cell_format)?;
        comp_sheet.write_with_format(i as u32, 2, "Cả hai", &cell_format)?;
        comp_sheet.write_with_format(i as u32, 3, "Mọi khối", &cell_format)?;
    }

    let val_role = DataValidation::new().allow_list_strings(&["Cả hai", "Ra đề", "Phản biện"])?;
    comp_sheet.add_data_validation(1, 2, 500, 2, &val_role)?;

    let val_scope = DataValidation::new().allow_list_strings(&["Theo khối dạy", "Mọi khối"])?;
    comp_sheet.add_data_validation(1, 3, 500, 3, &val_scope)?;

    let val_subj = DataValidation::new().allow_list_strings(&["VL", "CN"])?;
    comp_sheet.add_data_validation(1, 1, 500, 1, &val_subj)?;

    // Save workbook to file
    workbook.save(target_path)?;

    Ok(())
}

/// Generates a filled Excel workbook for the Q-shaped demo (synthetic data) using template v2.
///
/// Contains all 12 teachers, 4 campuses, 2 subjects (VL, CN), unavailabilities,
/// and full competencies matching the canonical Q problem.
pub fn generate_q_sample_template_v2(target_path: &Path) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_background_color(Color::RGB(0xDBEAFE))
        .set_border(FormatBorder::Thin);

    let required_header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_font_color(Color::RGB(0x991B1B))
        .set_background_color(Color::RGB(0xFEE2E2))
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_font_size(10)
        .set_border(FormatBorder::Thin);

    // 1. Phân hiệu
    let campus_sheet = workbook.add_worksheet();
    campus_sheet.set_name("Phân hiệu")?;
    campus_sheet.set_freeze_panes(1, 0)?;
    campus_sheet.set_column_width(0, 18)?;
    campus_sheet.set_column_width(1, 35)?;

    campus_sheet.write_with_format(0, 0, "Mã phân hiệu (*)", &required_header_format)?;
    campus_sheet.write_with_format(0, 1, "Tên phân hiệu (*)", &required_header_format)?;

    let campuses = [
        ("PH1", "Phân hiệu 1 - Cơ sở chính"),
        ("PH2", "Phân hiệu 2 - Cơ sở Nam"),
        ("PH3", "Phân hiệu 3 - Cơ sở Đông"),
        ("PH4", "Phân hiệu 4 - Cơ sở Bắc"),
    ];
    for (i, (code, name)) in campuses.iter().enumerate() {
        let row = (i + 1) as u32;
        campus_sheet.write_with_format(row, 0, *code, &cell_format)?;
        campus_sheet.write_with_format(row, 1, *name, &cell_format)?;
    }

    // 2. Giáo viên
    let teacher_sheet = workbook.add_worksheet();
    teacher_sheet.set_name("Giáo viên")?;
    teacher_sheet.set_freeze_panes(1, 0)?;
    teacher_sheet.set_column_width(0, 14)?;
    teacher_sheet.set_column_width(1, 28)?;
    teacher_sheet.set_column_width(2, 16)?;
    teacher_sheet.set_column_width(3, 18)?;
    teacher_sheet.set_column_width(4, 20)?;
    teacher_sheet.set_column_width(5, 14)?;
    teacher_sheet.set_column_width(6, 14)?;
    teacher_sheet.set_column_width(7, 25)?;
    teacher_sheet.set_column_width(8, 16)?;
    teacher_sheet.set_column_width(9, 22)?;

    teacher_sheet.write_with_format(0, 0, "Mã GV", &header_format)?;
    teacher_sheet.write_with_format(0, 1, "Họ và tên (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 2, "Cách gọi", &header_format)?;
    teacher_sheet.write_with_format(0, 3, "Mã phân hiệu (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 4, "Khối dạy (*)", &required_header_format)?;
    teacher_sheet.write_with_format(0, 5, "Hệ số tải", &header_format)?;
    teacher_sheet.write_with_format(0, 6, "Đang dạy", &header_format)?;
    teacher_sheet.write_with_format(0, 7, "Ghi chú", &header_format)?;
    teacher_sheet.write_with_format(0, 8, "Chỉ tiêu riêng", &header_format)?;
    teacher_sheet.write_with_format(0, 9, "Số việc tối đa mỗi kỳ", &header_format)?;

    let teachers = [
        (
            "GV001",
            "Hoàng Thị Hiền",
            "C Hiền",
            "PH1",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV002",
            "Nguyễn Thị Lài",
            "C Lài",
            "PH1",
            "12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV003",
            "Trần Văn Phúc",
            "T Phúc",
            "PH2",
            "10, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV004",
            "Phạm Văn Lộc",
            "T Lộc",
            "PH2",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV005",
            "Lê Thị Thư",
            "C Thư",
            "PH3",
            "11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV006",
            "Vũ Thị Na",
            "C Na",
            "PH3",
            "10, 11",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV007",
            "Đỗ Thị Bình",
            "C Bình",
            "PH4",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV008",
            "Bùi Thị Quí",
            "C Quí",
            "PH4",
            "11, 12",
            "1",
            "Có",
            "",
            "",
            "3",
        ),
        (
            "GV009",
            "Ngô Thị Tú",
            "C Tú",
            "PH2",
            "10",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV010",
            "Đinh Thị Như",
            "C Như",
            "PH3",
            "10",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV011",
            "Lý Thị Lan",
            "C Lan",
            "PH4",
            "12",
            "1",
            "Có",
            "",
            "",
            "",
        ),
        (
            "GV012",
            "Trương Văn Nghĩa",
            "T Nghĩa",
            "PH1",
            "10, 11, 12",
            "1",
            "Có",
            "Ra đề độc quyền CN",
            "12",
            "3",
        ),
    ];

    for (i, (code, name, display_name, campus, grades, load, active, note, quota, max_tasks)) in
        teachers.iter().enumerate()
    {
        let row = (i + 1) as u32;
        teacher_sheet.write_with_format(row, 0, *code, &cell_format)?;
        teacher_sheet.write_with_format(row, 1, *name, &cell_format)?;
        teacher_sheet.write_with_format(row, 2, *display_name, &cell_format)?;
        teacher_sheet.write_with_format(row, 3, *campus, &cell_format)?;
        teacher_sheet.write_with_format(row, 4, *grades, &cell_format)?;
        teacher_sheet.write_with_format(row, 5, *load, &cell_format)?;
        teacher_sheet.write_with_format(row, 6, *active, &cell_format)?;
        teacher_sheet.write_with_format(row, 7, *note, &cell_format)?;
        if !quota.is_empty() {
            teacher_sheet.write_with_format(row, 8, *quota, &cell_format)?;
        }
        if !max_tasks.is_empty() {
            teacher_sheet.write_with_format(row, 9, *max_tasks, &cell_format)?;
        }
    }

    // 3. Lịch vắng
    let unavail_sheet = workbook.add_worksheet();
    unavail_sheet.set_name("Lịch vắng")?;
    unavail_sheet.set_freeze_panes(1, 0)?;
    unavail_sheet.set_column_width(0, 28)?;
    unavail_sheet.set_column_width(1, 18)?;
    unavail_sheet.set_column_width(2, 40)?;

    unavail_sheet.write_with_format(0, 0, "Mã GV hoặc Họ tên (*)", &required_header_format)?;
    unavail_sheet.write_with_format(0, 1, "Mã kỳ thi (*)", &required_header_format)?;
    unavail_sheet.write_with_format(0, 2, "Lý do", &header_format)?;

    // 4. Môn
    let subject_sheet = workbook.add_worksheet();
    subject_sheet.set_name("Môn")?;
    subject_sheet.set_freeze_panes(1, 0)?;
    subject_sheet.set_column_width(0, 16)?;
    subject_sheet.set_column_width(1, 25)?;
    subject_sheet.set_column_width(2, 20)?;
    subject_sheet.set_column_width(3, 20)?;
    subject_sheet.set_column_width(4, 25)?;
    subject_sheet.set_column_width(5, 18)?;

    subject_sheet.write_with_format(0, 0, "Mã môn (*)", &required_header_format)?;
    subject_sheet.write_with_format(0, 1, "Tên môn (*)", &required_header_format)?;
    subject_sheet.write_with_format(0, 2, "Số người ra đề", &header_format)?;
    subject_sheet.write_with_format(0, 3, "Số người phản biện", &header_format)?;
    subject_sheet.write_with_format(0, 4, "Số phân hiệu tối thiểu", &header_format)?;
    subject_sheet.write_with_format(0, 5, "Màu", &header_format)?;

    let q_subjects = [
        ("VL", "Vật lí", 2, 1, 2, "#2563EB"),
        ("CN", "Công nghệ", 1, 1, 2, "#10B981"),
    ];
    for (i, (code, name, setters, reviewers, min_camp, color)) in q_subjects.iter().enumerate() {
        let row = (i + 1) as u32;
        subject_sheet.write_with_format(row, 0, *code, &cell_format)?;
        subject_sheet.write_with_format(row, 1, *name, &cell_format)?;
        subject_sheet.write_with_format(row, 2, *setters, &cell_format)?;
        subject_sheet.write_with_format(row, 3, *reviewers, &cell_format)?;
        subject_sheet.write_with_format(row, 4, *min_camp, &cell_format)?;
        subject_sheet.write_with_format(row, 5, *color, &cell_format)?;
    }

    // 5. Môn đảm nhiệm
    let comp_sheet = workbook.add_worksheet();
    comp_sheet.set_name("Môn đảm nhiệm")?;
    comp_sheet.set_freeze_panes(1, 0)?;
    comp_sheet.set_column_width(0, 26)?;
    comp_sheet.set_column_width(1, 18)?;
    comp_sheet.set_column_width(2, 20)?;
    comp_sheet.set_column_width(3, 24)?;

    comp_sheet.write_with_format(0, 0, "Mã GV hoặc Cách gọi (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 1, "Mã môn (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 2, "Vai trò (*)", &required_header_format)?;
    comp_sheet.write_with_format(0, 3, "Phạm vi", &header_format)?;

    let mut comp_row = 1;
    // T Nghĩa: CN, Ra đề, Mọi khối
    comp_sheet.write_with_format(comp_row, 0, "T Nghĩa", &cell_format)?;
    comp_sheet.write_with_format(comp_row, 1, "CN", &cell_format)?;
    comp_sheet.write_with_format(comp_row, 2, "Ra đề", &cell_format)?;
    comp_sheet.write_with_format(comp_row, 3, "Mọi khối", &cell_format)?;
    comp_row += 1;

    // All teachers except T Nghĩa have VL (Cả hai, Theo khối dạy) and CN (Phản biện, Mọi khối)
    for (_, _, display_name, _, _, _, _, _, _, _) in &teachers {
        if *display_name != "T Nghĩa" {
            // VL: Cả hai, Theo khối dạy
            comp_sheet.write_with_format(comp_row, 0, *display_name, &cell_format)?;
            comp_sheet.write_with_format(comp_row, 1, "VL", &cell_format)?;
            comp_sheet.write_with_format(comp_row, 2, "Cả hai", &cell_format)?;
            comp_sheet.write_with_format(comp_row, 3, "Theo khối dạy", &cell_format)?;
            comp_row += 1;

            // CN: Phản biện, Mọi khối
            comp_sheet.write_with_format(comp_row, 0, *display_name, &cell_format)?;
            comp_sheet.write_with_format(comp_row, 1, "CN", &cell_format)?;
            comp_sheet.write_with_format(comp_row, 2, "Phản biện", &cell_format)?;
            comp_sheet.write_with_format(comp_row, 3, "Mọi khối", &cell_format)?;
            comp_row += 1;
        }
    }

    workbook.save(target_path)?;
    Ok(())
}
