//! Integration tests for Plan Import (Roundtrip, TSV, Golden Q Table).

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, make_q_assignments, QVariant};
use exam_panel_core::domain::{Plan, PlanId, SchoolYearId};
use exam_panel_service::dto::{AppSettings, ApplyPlanImportInput, PlanDetails};
use exam_panel_service::excel::export::export_plan_workbook;
use exam_panel_service::excel::plan_import::preview_plan_import;
use exam_panel_service::service::AppService;
use std::fs;
use std::path::Path;

#[test]
fn test_plan_import_export_roundtrip() {
    let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);
    let original_assignments = make_q_assignments();

    let plan = Plan {
        id: PlanId(1),
        school_year_id: SchoolYearId(1),
        name: "Phương án test roundtrip".to_string(),
        source: "manual".to_string(),
        created_at: "2026-10-04T12:00:00Z".to_string(),
        seed: 42,
        score: None,
        is_final: false,
        rank: None,
        score_report_json: None,
        run_params_json: None,
        data_hash: None,
        rules_hash: None,
    };

    let plan_details = PlanDetails {
        plan,
        assignments: original_assignments.clone(),
        score_report: None,
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let export_path = temp_dir.path().join("roundtrip_plan.xlsx");

    let settings = AppSettings {
        theme: "light".to_string(),
        language: "vi".to_string(),
        current_school_year_id: Some(SchoolYearId(1)),
        school_name: Some("TRƯỜNG THPT CHUYÊN".to_string()),
        department_name: Some("TỔ TOÁN - TIN".to_string()),
        signer_title: Some("TỔ TRƯỞNG CHUYÊN MÔN".to_string()),
        signer_name: Some("Nguyễn Văn A".to_string()),
        place_name: Some("Hà Nội".to_string()),
    };

    export_plan_workbook(
        &export_path,
        &plan_details,
        &problem.school_year,
        &problem.campuses,
        &problem.grades,
        &problem.exams,
        &problem.subjects,
        &problem.teachers,
        &settings,
        &problem.rule_settings,
    )
    .expect("export workbook");

    // Preview import from exported file
    let preview =
        preview_plan_import(&problem, Some(&export_path), None).expect("preview exported plan");

    assert!(
        preview.can_apply,
        "Preview should be valid: {:?}",
        preview.errors
    );
    assert_eq!(preview.errors.len(), 0);
    assert_eq!(preview.assignments.len(), 60);

    // Verify all 60 assignments match exactly
    for expected in &original_assignments {
        let matched = preview.assignments.iter().any(|actual| {
            actual.exam_id == expected.exam_id
                && actual.grade_id == expected.grade_id
                && actual.subject_id == expected.subject_id
                && actual.role == expected.role
                && actual.position == expected.position
                && actual.teacher_id == expected.teacher_id
        });
        assert!(
            matched,
            "Missing or mismatched assignment: exam={:?}, grade={:?}, subject={:?}, role={:?}, pos={}, teacher={:?}",
            expected.exam_id, expected.grade_id, expected.subject_id, expected.role, expected.position, expected.teacher_id
        );
    }
}

#[test]
fn test_plan_import_tsv_clipboard() {
    let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);

    // Build TSV text matching the grid (Col 0: Exam, Col 1: Role, Col 2..7: Grades & Subjects)
    let tsv_text = "Kì thi/khối\t\tKhối 10\t\tKhối 11\t\tKhối 12\t\n\
                    \t\tVL\tCN\tVL\tCN\tVL\tCN\n\
                    GK1\tĐề\tC Hiền\tT Nghĩa\tT Lộc\tT Nghĩa\tC Bình\tT Nghĩa\n\
                    \tĐề\tC Lài\t\tC Thư\t\tT Phúc\t\n\
                    \tP.Biện\tT Phúc\tC Hiền\tC Na\tC Lài\tC Quí\tT Lộc\n\
                    CK1\tĐề\tC Hiền\tT Nghĩa\tC Lài\tT Nghĩa\tC Lan\tT Nghĩa\n\
                    \tĐề\tC Tú\t\tC Bình\t\tC Quí\t\n\
                    \tP.Biện\tC Như\tC Tú\tC Thư\tC Lan\tC Lài\tC Bình\n\
                    GK2\tĐề\tC Tú\tT Nghĩa\tC Na\tT Nghĩa\tC Lan\tT Nghĩa\n\
                    \tĐề\tC Như\t\tC Thư\t\tC Lài\t\n\
                    \tP.Biện\tT Lộc\tC Như\tC Hiền\tC Thư\tC Quí\tT Phúc\n\
                    CK2\tĐề\tC Na\tT Nghĩa\tT Lộc\tT Nghĩa\tT Phúc\tT Nghĩa\n\
                    \tĐề\tC Như\t\tC Quí\t\tC Hiền\t\n\
                    \tP.Biện\tC Tú\tC Quí\tC Bình\tC Na\tC Lan\tC Quí";

    let preview = preview_plan_import(&problem, None, Some(tsv_text)).expect("preview TSV content");

    assert!(
        preview.can_apply,
        "TSV preview errors: {:?}",
        preview.errors
    );
    assert_eq!(preview.errors.len(), 0);
    assert_eq!(preview.assignments.len(), 60);
}

#[test]
fn test_plan_import_golden_q_table() {
    let report_xlsx_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reports/phase-12/plan-grid.xlsx");

    let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);

    let preview =
        preview_plan_import(&problem, Some(&report_xlsx_path), None).expect("preview golden plan");

    assert!(preview.can_apply, "Errors: {:?}", preview.errors);
    assert_eq!(preview.assignments.len(), 60);

    // Verify per-teacher totals:
    // C Hiền: 5, C Lài: 5, T Phúc: 4, T Lộc: 4, C Thư: 4, C Na: 4, C Bình: 4, C Quí: 6, C Tú: 4, C Như: 4, C Lan: 4, T Nghĩa: 12
    let expected = [
        ("C Hiền", 5),
        ("C Lài", 5),
        ("T Phúc", 4),
        ("T Lộc", 4),
        ("C Thư", 4),
        ("C Na", 4),
        ("C Bình", 4),
        ("C Quí", 6),
        ("C Tú", 4),
        ("C Như", 4),
        ("C Lan", 4),
        ("T Nghĩa", 12),
    ];

    let mut report = String::new();
    report.push_str("=== Báo cáo đối soát nhập bảng phân công Q (Golden Validation) ===\n\n");
    report.push_str(&format!(
        "Số lượng phân công được nạp: {} / 60\n",
        preview.assignments.len()
    ));
    report.push_str(&format!(
        "Trạng thái hợp lệ (can_apply): {}\n\n",
        preview.can_apply
    ));
    report.push_str("Bảng đối soát từng giáo viên:\n");
    report.push_str(&format!(
        "{:<12} | {:<10} | {:<10} | {:<8} | {:<6} | {:<6}\n",
        "Giáo viên", "Thực tế", "Kỳ vọng", "Khớp", "Ra đề", "P.Biện"
    ));
    report.push_str("-------------+------------+------------+---------+--------+-------\n");

    for (name, exp_total) in expected {
        let t_total = preview
            .teacher_totals
            .iter()
            .find(|t| t.display_name == name || t.teacher_name == name)
            .expect("teacher found");

        let matched = t_total.computed_total == exp_total;
        report.push_str(&format!(
            "{:<12} | {:<10} | {:<10} | {:<8} | {:<6} | {:<6}\n",
            name,
            t_total.computed_total,
            exp_total,
            if matched { "OK" } else { "FAIL" },
            t_total.setter_count,
            t_total.reviewer_count
        ));
        assert_eq!(t_total.computed_total, exp_total, "Mismatch for {name}");
    }

    report.push_str("-------------+------------+------------+---------+--------+-------\n");
    let total_seats: i64 = preview
        .teacher_totals
        .iter()
        .map(|t| t.computed_total)
        .sum();
    let total_setters: i64 = preview.teacher_totals.iter().map(|t| t.setter_count).sum();
    let total_reviewers: i64 = preview
        .teacher_totals
        .iter()
        .map(|t| t.reviewer_count)
        .sum();
    report.push_str(&format!(
        "{:<12} | {:<10} | {:<10} | {:<8} | {:<6} | {:<6}\n\n",
        "Tổng cộng",
        total_seats,
        60,
        if total_seats == 60 { "OK" } else { "FAIL" },
        total_setters,
        total_reviewers
    ));

    assert_eq!(total_seats, 60);
    assert_eq!(total_setters, 36);
    assert_eq!(total_reviewers, 24);

    // Test applying the plan via service with seeded demo data
    let service = AppService::open_in_memory().unwrap();
    service.seed_demo().expect("seed demo data");

    let plan_id = service
        .apply_imported_plan(ApplyPlanImportInput {
            school_year_id: SchoolYearId(1),
            plan_name: Some("Nhập từ bảng của tổ".to_string()),
            assignments: preview.assignments.clone(),
        })
        .expect("apply imported plan");

    let details = service.get_plan(plan_id).expect("get applied plan");
    assert_eq!(details.plan.source, "manual");
    assert_eq!(details.plan.name, "Nhập từ bảng của tổ");
    assert!(details
        .plan
        .run_params_json
        .unwrap()
        .contains("\"origin\":\"import\""));
    assert_eq!(details.assignments.len(), 60);

    report.push_str(&format!(
        "Apply thành công vào cơ sở dữ liệu: Plan ID = {}\n",
        plan_id.value()
    ));
    report.push_str("Thuộc tính phương án: source = 'manual', origin = 'import'\n");

    let output_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reports/phase-12/import-golden.txt");
    fs::write(&output_path, report).expect("write golden report");
}
