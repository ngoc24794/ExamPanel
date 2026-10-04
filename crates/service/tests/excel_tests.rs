//! Integration tests for Excel template generation, import, and export.

use calamine::{open_workbook_auto, Reader, Sheets};
use exam_panel_service::dto::{CreateCampusInput, CreateSchoolYearInput, CreateTeacherInput};
use exam_panel_service::AppService;
use rust_xlsxwriter::Workbook;
use std::fs::File;
use std::io::BufReader;

#[test]
fn test_template_generation_and_calamine_sheets() {
    let temp_dir = tempfile::tempdir().unwrap();
    let template_path = temp_dir.path().join("mau-nhap-du-lieu.xlsx");

    let service = AppService::open_in_memory().unwrap();
    service
        .generate_import_template(&template_path)
        .expect("template generation should succeed");

    assert!(template_path.exists());

    let mut workbook: Sheets<BufReader<File>> =
        open_workbook_auto(&template_path).expect("open template workbook");
    let names = workbook.sheet_names().to_vec();

    assert_eq!(names.len(), 4);
    assert_eq!(names[0], "Hướng dẫn");
    assert_eq!(names[1], "Phân hiệu");
    assert_eq!(names[2], "Giáo viên");
    assert_eq!(names[3], "Lịch vắng");

    // Verify headers on "Phân hiệu"
    let range_campuses = workbook.worksheet_range("Phân hiệu").unwrap();
    let header_c0 = range_campuses.get_value((0, 0)).unwrap().to_string();
    assert!(header_c0.contains("Mã phân hiệu"));

    // Verify headers on "Giáo viên"
    let range_teachers = workbook.worksheet_range("Giáo viên").unwrap();
    let header_t1 = range_teachers.get_value((0, 1)).unwrap().to_string();
    assert!(header_t1.contains("Họ và tên"));
}

#[test]
fn test_template_roundtrip_preview_and_apply() {
    let temp_dir = tempfile::tempdir().unwrap();
    let excel_path = temp_dir.path().join("import_roundtrip.xlsx");

    let service = AppService::open_in_memory().unwrap();
    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".to_string(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap();

    // Create a custom Excel file with 4 sheets programmatically
    let mut wb = Workbook::new();

    let guide = wb.add_worksheet();
    guide.set_name("Hướng dẫn").unwrap();
    guide.write(0, 0, "Hướng dẫn").unwrap();

    let campuses = wb.add_worksheet();
    campuses.set_name("Phân hiệu").unwrap();
    campuses.write(0, 0, "Mã phân hiệu (*)").unwrap();
    campuses.write(0, 1, "Tên phân hiệu (*)").unwrap();
    campuses.write(1, 0, "PH1").unwrap();
    campuses.write(1, 1, "Phân hiệu 1 - Ba Đình").unwrap();
    campuses.write(2, 0, "PH2").unwrap();
    campuses.write(2, 1, "Phân hiệu 2 - Cầu Giấy").unwrap();

    let teachers = wb.add_worksheet();
    teachers.set_name("Giáo viên").unwrap();
    teachers.write(0, 0, "Mã GV").unwrap();
    teachers.write(0, 1, "Họ và tên (*)").unwrap();
    teachers.write(0, 2, "Mã phân hiệu (*)").unwrap();
    teachers.write(0, 3, "Khối dạy (*)").unwrap();
    teachers.write(0, 4, "Hệ số tải").unwrap();
    teachers.write(0, 5, "Đang dạy").unwrap();
    teachers.write(0, 6, "Ghi chú").unwrap();

    // Row 1: Teacher An
    teachers.write(1, 0, "GV001").unwrap();
    teachers.write(1, 1, "Nguyễn Văn An").unwrap();
    teachers.write(1, 2, "PH1").unwrap();
    teachers.write(1, 3, "10, 11").unwrap();
    teachers.write(1, 4, "1").unwrap();
    teachers.write(1, 5, "Có").unwrap();
    teachers.write(1, 6, "Tổ trưởng").unwrap();

    // Row 2: Teacher Binh
    teachers.write(2, 0, "GV002").unwrap();
    teachers.write(2, 1, "Trần Thị Bình").unwrap();
    teachers.write(2, 2, "PH2").unwrap();
    teachers.write(2, 3, "Khối 11, 12").unwrap();
    teachers.write(2, 4, "0.5").unwrap();
    teachers.write(2, 5, "Có").unwrap();

    let unavail = wb.add_worksheet();
    unavail.set_name("Lịch vắng").unwrap();
    unavail.write(0, 0, "Mã GV hoặc Họ tên (*)").unwrap();
    unavail.write(0, 1, "Mã kỳ thi (*)").unwrap();
    unavail.write(0, 2, "Lý do").unwrap();
    unavail.write(1, 0, "GV001").unwrap();
    unavail.write(1, 1, "GK1").unwrap();
    unavail.write(1, 2, "Họp chuyên môn").unwrap();

    wb.save(&excel_path).unwrap();

    // 1. Preview
    let preview = service
        .preview_import(sy.id, &excel_path, "upsert")
        .expect("preview should succeed");

    assert!(
        preview.can_apply,
        "campuses: {:?}\nteachers: {:?}\nunavails: {:?}",
        preview
            .campuses
            .iter()
            .map(|c| &c.errors)
            .collect::<Vec<_>>(),
        preview
            .teachers
            .iter()
            .map(|t| &t.errors)
            .collect::<Vec<_>>(),
        preview
            .unavailabilities
            .iter()
            .map(|u| &u.errors)
            .collect::<Vec<_>>(),
    );
    assert_eq!(preview.campuses_summary.new_count, 2);
    assert_eq!(preview.teachers_summary.new_count, 2);
    assert_eq!(preview.unavailabilities_summary.new_count, 1);

    // 2. Apply
    let apply_res = service
        .apply_import(sy.id, &preview)
        .expect("apply should succeed");

    assert_eq!(apply_res.campuses_created, 2);
    assert_eq!(apply_res.teachers_created, 2);
    assert_eq!(apply_res.unavailabilities_created, 1);

    // 3. Verify in DB
    let campuses_db = service.list_campuses().unwrap();
    assert_eq!(campuses_db.len(), 2);

    let teachers_db = service.list_teachers().unwrap();
    assert_eq!(teachers_db.len(), 2);
    let an = teachers_db
        .iter()
        .find(|t| t.full_name == "Nguyễn Văn An")
        .unwrap();
    assert_eq!(an.code.as_deref(), Some("GV001"));
    assert_eq!(an.load_weight, 1.0);

    let binh = teachers_db
        .iter()
        .find(|t| t.full_name == "Trần Thị Bình")
        .unwrap();
    assert_eq!(binh.load_weight, 0.5);

    let unavails = service.list_unavailabilities(sy.id).unwrap();
    assert_eq!(unavails.len(), 1);
    assert_eq!(unavails[0].reason.as_deref(), Some("Họp chuyên môn"));
}

#[test]
fn test_teacher_matching_and_ambiguity_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let excel_path = temp_dir.path().join("ambiguous_test.xlsx");

    let service = AppService::open_in_memory().unwrap();
    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".to_string(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap();

    let c1 = service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "blue".to_string(),
        })
        .unwrap();
    let c2 = service
        .create_campus(CreateCampusInput {
            code: "PH2".to_string(),
            name: "Phân hiệu 2".to_string(),
            color: "emerald".to_string(),
        })
        .unwrap();

    // Create 2 teachers with IDENTICAL full_name "Nguyễn Văn An" on different campuses
    let t1 = service
        .create_teacher(CreateTeacherInput {
            code: Some("GV001".to_string()),
            full_name: "Nguyễn Văn An".to_string(),
            campus_id: c1.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .unwrap();
    service
        .set_teacher_grades(
            t1.id,
            sy.id,
            vec![
                exam_panel_core::domain::GradeId(1),
                exam_panel_core::domain::GradeId(2),
            ],
        )
        .unwrap();

    let t2 = service
        .create_teacher(CreateTeacherInput {
            code: Some("GV002".to_string()),
            full_name: "Nguyễn Văn An".to_string(),
            campus_id: c2.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .unwrap();
    service
        .set_teacher_grades(
            t2.id,
            sy.id,
            vec![
                exam_panel_core::domain::GradeId(2),
                exam_panel_core::domain::GradeId(3),
            ],
        )
        .unwrap();

    // Case 1: Import row with "Nguyễn Văn An" but no campus and no code -> Ambiguous error!
    let mut wb1 = Workbook::new();
    let teachers1 = wb1.add_worksheet();
    teachers1.set_name("Giáo viên").unwrap();
    teachers1.write(0, 0, "Mã GV").unwrap();
    teachers1.write(0, 1, "Họ và tên (*)").unwrap();
    teachers1.write(0, 2, "Mã phân hiệu (*)").unwrap();
    teachers1.write(0, 3, "Khối dạy (*)").unwrap();
    teachers1.write(1, 0, "").unwrap(); // No code
    teachers1.write(1, 1, "Nguyễn Văn An").unwrap();
    teachers1.write(1, 2, "PH3").unwrap(); // Unknown campus
    teachers1.write(1, 3, "10, 11").unwrap();
    wb1.save(&excel_path).unwrap();

    let preview1 = service
        .preview_import(sy.id, &excel_path, "upsert")
        .unwrap();
    assert!(!preview1.can_apply);
    assert_eq!(preview1.teachers_summary.error_count, 1);

    // Case 2: Import row with code "GV001" -> matches specific teacher without ambiguity!
    let mut wb2 = Workbook::new();
    let teachers2 = wb2.add_worksheet();
    teachers2.set_name("Giáo viên").unwrap();
    teachers2.write(0, 0, "Mã GV").unwrap();
    teachers2.write(0, 1, "Họ và tên (*)").unwrap();
    teachers2.write(0, 2, "Mã phân hiệu (*)").unwrap();
    teachers2.write(0, 3, "Khối dạy (*)").unwrap();
    teachers2.write(1, 0, "GV001").unwrap();
    teachers2.write(1, 1, "Nguyễn Văn An").unwrap();
    teachers2.write(1, 2, "PH1").unwrap();
    teachers2.write(1, 3, "10, 11").unwrap();
    wb2.save(&excel_path).unwrap();

    let preview2 = service
        .preview_import(sy.id, &excel_path, "upsert")
        .unwrap();
    assert!(preview2.can_apply);
    assert_eq!(
        preview2.teachers[0].status,
        exam_panel_service::dto::ImportRowStatus::Unchanged
    );
    assert_eq!(preview2.teachers[0].matched_teacher_id.is_some(), true);
}

#[test]
fn test_sync_mode_deactivates_omitted_teachers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let excel_path = temp_dir.path().join("sync_test.xlsx");

    let service = AppService::open_in_memory().unwrap();
    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".to_string(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap();

    let c1 = service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "blue".to_string(),
        })
        .unwrap();

    // Create 3 teachers in DB: T1, T2, T3
    let _t1 = service
        .create_teacher(CreateTeacherInput {
            code: Some("GV01".to_string()),
            full_name: "Teacher 1".to_string(),
            campus_id: c1.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .unwrap();

    let _t2 = service
        .create_teacher(CreateTeacherInput {
            code: Some("GV02".to_string()),
            full_name: "Teacher 2".to_string(),
            campus_id: c1.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .unwrap();

    let t3 = service
        .create_teacher(CreateTeacherInput {
            code: Some("GV03".to_string()),
            full_name: "Teacher 3 (To Be Deactivated)".to_string(),
            campus_id: c1.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .unwrap();

    // Excel file contains only T1 and T2
    let mut wb = Workbook::new();
    let teachers = wb.add_worksheet();
    teachers.set_name("Giáo viên").unwrap();
    teachers.write(0, 0, "Mã GV").unwrap();
    teachers.write(0, 1, "Họ và tên (*)").unwrap();
    teachers.write(0, 2, "Mã phân hiệu (*)").unwrap();
    teachers.write(0, 3, "Khối dạy (*)").unwrap();

    teachers.write(1, 0, "GV01").unwrap();
    teachers.write(1, 1, "Teacher 1").unwrap();
    teachers.write(1, 2, "PH1").unwrap();
    teachers.write(1, 3, "10").unwrap();

    teachers.write(2, 0, "GV02").unwrap();
    teachers.write(2, 1, "Teacher 2").unwrap();
    teachers.write(2, 2, "PH1").unwrap();
    teachers.write(2, 3, "11").unwrap();

    wb.save(&excel_path).unwrap();

    // Preview in sync mode
    let preview = service
        .preview_import(sy.id, &excel_path, "sync")
        .expect("preview sync");

    assert!(preview.can_apply);
    assert_eq!(preview.deactivated_teachers.len(), 1);
    assert_eq!(preview.deactivated_teachers[0].id, t3.id.value());

    // Apply in sync mode
    let res = service.apply_import(sy.id, &preview).expect("apply sync");
    assert_eq!(res.teachers_deactivated, 1);

    // Verify T3 is now active = false in DB
    let teachers_after = service.list_teachers().unwrap();
    let t3_after = teachers_after.iter().find(|t| t.id == t3.id).unwrap();
    assert_eq!(t3_after.active, false);
}

#[test]
fn test_export_workbook_calamine_verification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let export_path = temp_dir.path().join("phan-cong-demo.xlsx");

    let service = AppService::open_in_memory().unwrap();
    service.seed_demo().expect("seed demo");

    let sy = service
        .list_school_years()
        .unwrap()
        .into_iter()
        .find(|y| y.is_current)
        .unwrap();

    // Generate a plan using service.run_optimize
    let outcome = service
        .run_optimize(
            sy.id,
            exam_panel_service::dto::OptimizeRequest {
                base_seed: Some(42),
                runs: 1,
                budget: exam_panel_service::dto::OptimizeBudget::Iterations(10),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .unwrap();

    let plan_ids = service.save_optimize_result(sy.id, outcome).unwrap();
    let saved_plan_id = plan_ids[0];

    // Export to Excel
    service
        .export_plan_excel(saved_plan_id, &export_path)
        .expect("export plan excel");

    assert!(export_path.exists());

    // Read back using Calamine and assert sheet structure
    let mut workbook: Sheets<BufReader<File>> =
        open_workbook_auto(&export_path).expect("open exported excel");

    let sheet_names = workbook.sheet_names().to_vec();
    assert_eq!(sheet_names.len(), 4);
    assert_eq!(sheet_names[0], "Phân công");
    assert_eq!(sheet_names[1], "Theo giáo viên");
    assert_eq!(sheet_names[2], "Thống kê");
    assert_eq!(sheet_names[3], "Tiêu chí");

    // Check Sheet "Phân công" contains [BẢN NHÁP] because is_final is false
    let range_matrix = workbook.worksheet_range("Phân công").unwrap();
    let title_cell = range_matrix.get_value((3, 0)).unwrap().to_string();
    assert!(title_cell.contains("[BẢN NHÁP]"));

    // Check Sheet "Theo giáo viên" has headers
    let range_teachers = workbook.worksheet_range("Theo giáo viên").unwrap();
    assert_eq!(
        range_teachers.get_value((0, 0)).unwrap().to_string(),
        "Mã GV"
    );
    assert_eq!(
        range_teachers.get_value((0, 1)).unwrap().to_string(),
        "Họ và tên"
    );

    // Check Sheet "Thống kê" has teacher rows
    let range_stats = workbook.worksheet_range("Thống kê").unwrap();
    assert!(range_stats.get_value((1, 1)).is_some());

    // Check Sheet "Tiêu chí" has rule rows
    let range_rules = workbook.worksheet_range("Tiêu chí").unwrap();
    assert_eq!(
        range_rules.get_value((0, 0)).unwrap().to_string(),
        "Mã tiêu chí"
    );
}

#[test]
fn test_transactional_apply_rollback_on_error() {
    let service = AppService::open_in_memory().unwrap();
    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".to_string(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap();

    let initial_campuses = service.list_campuses().unwrap().len();
    let initial_teachers = service.list_teachers().unwrap().len();

    // Construct a preview result with a valid campus row followed by an invalid teacher row (referencing unknown campus code)
    let preview = exam_panel_service::dto::ImportPreviewResult {
        mode: "upsert".to_string(),
        can_apply: true, // artificially set to true to force apply execution
        campuses: vec![exam_panel_service::dto::CampusImportRow {
            row_index: 2,
            status: exam_panel_service::dto::ImportRowStatus::New,
            code: "PH_NEW".to_string(),
            name: "Phân hiệu mới".to_string(),
            errors: Vec::new(),
        }],
        teachers: vec![exam_panel_service::dto::TeacherImportRow {
            row_index: 2,
            status: exam_panel_service::dto::ImportRowStatus::New,
            code: Some("GV_ERR".to_string()),
            full_name: "Teacher Error".to_string(),
            campus_code: "NON_EXISTENT_CAMPUS".to_string(),
            grades_str: "10".to_string(),
            grade_codes: vec![10],
            load_weight: 1.0,
            active: true,
            note: None,
            matched_teacher_id: None,
            errors: Vec::new(),
        }],
        unavailabilities: Vec::new(),
        campuses_summary: exam_panel_service::dto::ImportSummaryCounts {
            new_count: 1,
            update_count: 0,
            unchanged_count: 0,
            error_count: 0,
        },
        teachers_summary: exam_panel_service::dto::ImportSummaryCounts {
            new_count: 1,
            update_count: 0,
            unchanged_count: 0,
            error_count: 0,
        },
        unavailabilities_summary: exam_panel_service::dto::ImportSummaryCounts {
            new_count: 0,
            update_count: 0,
            unchanged_count: 0,
            error_count: 0,
        },
        deactivated_teachers: Vec::new(),
        feasibility_report: None,
    };

    let apply_res = service.apply_import(sy.id, &preview);
    assert!(
        apply_res.is_err(),
        "apply should fail due to invalid campus code"
    );

    // Verify atomic rollback: CS_NEW was NOT created!
    let campuses_after = service.list_campuses().unwrap();
    assert_eq!(campuses_after.len(), initial_campuses);
    assert!(!campuses_after.iter().any(|c| c.code == "CS_NEW"));

    let teachers_after = service.list_teachers().unwrap();
    assert_eq!(teachers_after.len(), initial_teachers);
}

#[test]
fn test_standard_template_feasibility_and_generate_downloads() {
    let target_path = std::path::PathBuf::from(
        r"C:\Users\ngocnv.HUONGVIETGROUP\Downloads\mau-nhap-du-lieu-chuan.xlsx",
    );
    let service = AppService::open_in_memory().unwrap();
    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".to_string(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap();

    service
        .generate_import_template(&target_path)
        .expect("generate template");

    let preview = service
        .preview_import(sy.id, &target_path, "merge")
        .expect("preview template");

    assert!(preview.can_apply);
    assert_eq!(preview.teachers.len(), 18);
    if let Some(ref feas) = preview.feasibility_report {
        assert!(
            feas.report.is_feasible,
            "Standard template must be 100% feasible, errors: {:?}",
            feas.report.errors
        );
        assert_eq!(feas.report.errors.len(), 0);
    }
}
