use exam_panel_core::domain::{LockKind, PlanId, Role, Unavailability};
use exam_panel_service::dto::{
    CreateCampusInput, CreateGradeInput, CreateLockInput, CreateSchoolYearInput,
    CreateTeacherInput, OptimizeBudget, OptimizeRequest,
};
use exam_panel_service::service::AppService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::NamedTempFile;

fn create_temp_service() -> (AppService, NamedTempFile) {
    let tmp = NamedTempFile::new().expect("create temp file");
    let service = AppService::open_at(tmp.path()).expect("open temp service");
    (service, tmp)
}

fn create_demo_service() -> (AppService, NamedTempFile) {
    let (service, tmp) = create_temp_service();
    service.seed_demo().expect("seed demo");
    (service, tmp)
}

#[test]
fn test_app_info_and_settings() {
    let (service, _tmp) = create_temp_service();

    let info = service.get_app_info().expect("get app info");
    assert!(!info.version.is_empty());
    assert!(!info.data_dir.is_empty());

    let settings = service.get_settings().expect("get settings");
    assert_eq!(settings.theme, "system");
    assert_eq!(settings.language, "vi");
    assert!(settings.current_school_year_id.is_none());

    service.set_setting("theme", "dark").expect("set theme");
    service.set_setting("language", "en").expect("set language");

    let updated = service.get_settings().expect("get updated settings");
    assert_eq!(updated.theme, "dark");
    assert_eq!(updated.language, "en");
}

#[test]
fn test_campuses_and_grades_crud() {
    let (service, _tmp) = create_temp_service();

    // Campuses
    let campuses = service.list_campuses().expect("list campuses");
    assert!(campuses.is_empty());

    let created_c = service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "#3b82f6".to_string(),
        })
        .expect("create campus");
    assert_eq!(created_c.code, "PH1");

    let mut campus_to_update = created_c.clone();
    campus_to_update.name = "Phân hiệu 1 (Cập nhật)".to_string();
    service
        .update_campus(campus_to_update)
        .expect("update campus");

    let campuses_after = service.list_campuses().expect("list campuses");
    assert_eq!(campuses_after.len(), 1);
    assert_eq!(campuses_after[0].name, "Phân hiệu 1 (Cập nhật)");

    // Grades (seed_defaults initializes grades 10, 11, 12)
    let grades = service.list_grades().expect("list grades");
    assert_eq!(grades.len(), 3);

    let created_g = service
        .create_grade(CreateGradeInput {
            code: 13,
            name: "Khối 13".to_string(),
            sort_order: 4,
        })
        .expect("create grade");
    assert_eq!(created_g.code, 13);

    let mut grade_to_update = created_g.clone();
    grade_to_update.name = "Khối 13 Nâng Cao".to_string();
    service.update_grade(grade_to_update).expect("update grade");

    let grades_after = service.list_grades().expect("list grades");
    assert_eq!(grades_after.len(), 4);
    assert!(grades_after.iter().any(|g| g.name == "Khối 13 Nâng Cao"));

    // Deletions
    service.delete_grade(created_g.id).expect("delete grade");
    assert_eq!(service.list_grades().expect("list").len(), 3);

    service.delete_campus(created_c.id).expect("delete campus");
    assert!(service.list_campuses().expect("list").is_empty());
}

#[test]
fn test_teachers_and_grades() {
    let (service, _tmp) = create_temp_service();

    let campus = service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "#3b82f6".to_string(),
        })
        .expect("create campus");

    let grades = service.list_grades().expect("list grades");
    let grade = &grades[0];

    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2024-2025".to_string(),
            is_current: false,
            copy_grades_from: None,
        })
        .expect("create school year");

    // Create teacher
    let teacher = service
        .create_teacher(CreateTeacherInput {
            full_name: "Nguyễn Văn A".to_string(),
            campus_id: campus.id,
            load_weight: 1.0,
            active: true,
            note: Some("Trưởng bộ môn".to_string()),
            code: Some("GV001".to_string()),
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .expect("create teacher");
    assert_eq!(teacher.full_name, "Nguyễn Văn A");
    assert_eq!(teacher.code.as_deref(), Some("GV001"));

    // Update teacher
    let mut updated_teacher = teacher.clone();
    updated_teacher.load_weight = 0.5;
    service
        .update_teacher(updated_teacher)
        .expect("update teacher");

    // Deactivate teacher
    service
        .deactivate_teacher(teacher.id)
        .expect("deactivate teacher");
    let teachers = service.list_teachers().expect("list teachers");
    assert!(!teachers[0].active);

    // Teacher grades
    service
        .set_teacher_grades(teacher.id, sy.id, vec![grade.id])
        .expect("set teacher grades");

    let twg = service
        .teachers_with_grades(sy.id)
        .expect("teachers with grades");
    assert_eq!(twg.len(), 1);
    assert_eq!(twg[0].grade_ids, vec![grade.id]);

    // Delete teacher
    service.delete_teacher(teacher.id).expect("delete teacher");
    assert!(service.list_teachers().expect("list").is_empty());
}

#[test]
fn test_school_years_and_exams() {
    let (service, _tmp) = create_temp_service();

    let sy = service
        .create_school_year(CreateSchoolYearInput {
            name: "2024-2025".to_string(),
            is_current: false,
            copy_grades_from: None,
        })
        .expect("create school year");

    service.set_current_school_year(sy.id).expect("set current");
    let settings = service.get_settings().expect("get settings");
    assert_eq!(settings.current_school_year_id, Some(sy.id));

    let exams = service.list_exams(sy.id).expect("list exams");
    assert_eq!(exams.len(), 4); // GK1, CK1, GK2, CK2 generated by default

    let mut exam = exams[0].clone();
    exam.name = "Giữa kỳ 1 (Đổi tên)".to_string();
    service.update_exam(exam).expect("update exam");

    let updated_exams = service.list_exams(sy.id).expect("list exams");
    assert_eq!(updated_exams[0].name, "Giữa kỳ 1 (Đổi tên)");
}

#[test]
fn test_unavailabilities_locks_and_rule_settings() {
    let (service, _tmp) = create_demo_service();
    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;
    let teachers = service.list_teachers().expect("teachers");
    let exams = service.list_exams(sy_id).expect("exams");
    let grades = service.list_grades().expect("grades");

    let tid = teachers[0].id;
    let eid = exams[0].id;
    let gid = grades[0].id;

    // Unavailabilities
    service
        .set_unavailability(Unavailability {
            teacher_id: tid,
            exam_id: eid,
            reason: None,
        })
        .expect("set unavail");
    let unavails = service.list_unavailabilities(sy_id).expect("list unavail");
    assert!(unavails
        .iter()
        .any(|u| u.teacher_id == tid && u.exam_id == eid));

    service
        .delete_unavailability(tid, eid)
        .expect("del unavail");
    let unavails2 = service.list_unavailabilities(sy_id).expect("list unavail");
    assert!(!unavails2
        .iter()
        .any(|u| u.teacher_id == tid && u.exam_id == eid));

    // Locks
    let subjects = service.list_subjects(sy_id).expect("list subjects");
    let lock = service
        .create_lock(CreateLockInput {
            exam_id: eid,
            grade_id: gid,
            subject_id: subjects[0].id,
            teacher_id: tid,
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        })
        .expect("create lock");

    let locks = service.list_locks(sy_id).expect("list locks");
    assert_eq!(locks.len(), 1);
    assert_eq!(locks[0].id, lock.id);
    assert_eq!(locks[0].subject_id, subjects[0].id);

    service.delete_lock(lock.id).expect("delete lock");
    assert!(service.list_locks(sy_id).expect("list locks").is_empty());

    // Rule settings
    let default_rules = service.get_rule_settings(sy_id).expect("get rules");
    assert!(!default_rules.is_empty());

    let mut modified_rules = default_rules.clone();
    modified_rules[0].weight = 999.0;
    service
        .save_rule_settings(sy_id, modified_rules)
        .expect("save rules");

    let fetched_rules = service.get_rule_settings(sy_id).expect("get rules");
    assert_eq!(fetched_rules[0].weight, 999.0);

    service
        .reset_rule_settings_to_defaults(sy_id)
        .expect("reset rules");
    let reset_rules = service.get_rule_settings(sy_id).expect("get rules");
    assert_ne!(reset_rules[0].weight, 999.0);
}

#[test]
fn test_feasibility_and_evaluation() {
    let (service, _tmp) = create_demo_service();
    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;

    let feas = service.check_feasibility(sy_id).expect("check feas");
    assert!(feas.report.is_feasible());
    assert!(!feas.quotas.is_empty());

    // Evaluate empty assignments (in partial mode)
    let eval = service
        .evaluate_assignments(sy_id, vec![])
        .expect("evaluate assignments");
    assert!(eval.hard_violations.is_empty());
}

#[test]
fn test_optimization_persistence_and_plan_crud() {
    let (service, _tmp) = create_demo_service();
    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;

    let req = OptimizeRequest {
        base_seed: Some(42),
        runs: 2,
        budget: OptimizeBudget::Iterations(500),
        k: 2,
        diversity_threshold: Some(0.20),
    };

    let outcome = service
        .run_optimize(sy_id, req, None, None)
        .expect("run optimize");
    assert_eq!(outcome.plans.len(), 2);
    assert!(!outcome.lower_bounds.is_empty());

    // Save optimize result
    let saved_ids = service
        .save_optimize_result(sy_id, outcome.clone())
        .expect("save optimize result");
    assert_eq!(saved_ids.len(), 2);

    // List plans
    let plans = service.list_plans(sy_id).expect("list plans");
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].source, "optimizer");
    assert_eq!(plans[0].rank, Some(1));
    assert_eq!(plans[1].rank, Some(2));
    assert!(!plans[0].is_final);

    // Get plan details
    let p1 = service.get_plan(saved_ids[0]).expect("get plan");
    assert_eq!(p1.assignments.len(), 60); // 4 exams * 3 grades * (3 VL + 2 CN) = 60 seats
    assert!(p1.score_report.is_some());

    // Rename plan
    service
        .rename_plan(saved_ids[0], "Phương án chính thức".to_string())
        .expect("rename");
    let updated_p1 = service.get_plan(saved_ids[0]).expect("get plan");
    assert_eq!(updated_p1.plan.name, "Phương án chính thức");

    // Mark final
    service.mark_final(saved_ids[0]).expect("mark final");
    let final_p1 = service.get_plan(saved_ids[0]).expect("get plan");
    assert!(final_p1.plan.is_final);

    // Duplicate plan
    let dup_id = service
        .duplicate_plan(saved_ids[0], "Bản sao chỉnh tay".to_string())
        .expect("duplicate");
    let dup = service.get_plan(dup_id).expect("get dup");
    assert_eq!(dup.plan.source, "duplicate");
    assert_eq!(dup.assignments.len(), p1.assignments.len());

    // Delete plan
    service.delete_plan(dup_id).expect("delete dup");
    let remaining_plans = service.list_plans(sy_id).expect("list plans");
    assert_eq!(remaining_plans.len(), 2);
}

#[test]
fn test_lock_not_held_during_optimize_and_cancellation() {
    let (service, _tmp) = create_demo_service();
    let service = Arc::new(service);
    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;

    let s_clone = Arc::clone(&service);
    let started = Arc::new(AtomicBool::new(false));
    let started_clone = Arc::clone(&started);

    let handle = std::thread::spawn(move || {
        let req = OptimizeRequest {
            base_seed: Some(123),
            runs: 1,
            budget: OptimizeBudget::Iterations(200_000),
            k: 1,
            diversity_threshold: Some(0.20),
        };
        started_clone.store(true, Ordering::SeqCst);
        s_clone.run_optimize(sy_id, req, None, None)
    });

    // Wait until background thread starts
    while !started.load(Ordering::SeqCst) || !service.is_optimizing() {
        std::thread::sleep(Duration::from_millis(5));
    }

    // Verify DB lock is NOT held: reading campuses succeeds immediately!
    let start_read = std::time::Instant::now();
    let campuses = service.list_campuses().expect("read during optimize");
    let read_duration = start_read.elapsed();
    assert!(!campuses.is_empty());
    assert!(
        read_duration < Duration::from_millis(100),
        "DB read took too long: {:?}",
        read_duration
    );

    // Cancel optimization
    let cancelled = service.cancel_optimize();
    assert!(cancelled);

    let res = handle.join().expect("thread join");
    assert!(res.is_ok());
    assert!(!service.is_optimizing());
}

#[test]
fn test_busy_job_rejection() {
    let (service, _tmp) = create_demo_service();
    let service = Arc::new(service);
    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;

    let s_clone = Arc::clone(&service);
    let started = Arc::new(AtomicBool::new(false));
    let started_clone = Arc::clone(&started);

    let handle = std::thread::spawn(move || {
        let req = OptimizeRequest {
            base_seed: Some(101),
            runs: 1,
            budget: OptimizeBudget::Iterations(200_000),
            k: 1,
            diversity_threshold: Some(0.20),
        };
        started_clone.store(true, Ordering::SeqCst);
        s_clone.run_optimize(sy_id, req, None, None)
    });

    while !started.load(Ordering::SeqCst) || !service.is_optimizing() {
        std::thread::sleep(Duration::from_millis(5));
    }

    // Calling run_optimize while active must return optimize_busy error
    let req2 = OptimizeRequest {
        base_seed: Some(102),
        runs: 1,
        budget: OptimizeBudget::Iterations(100),
        k: 1,
        diversity_threshold: Some(0.20),
    };
    let err = service
        .run_optimize(sy_id, req2, None, None)
        .expect_err("should reject concurrent optimize");
    assert_eq!(err.code, "optimize_busy");

    service.cancel_optimize();
    let _ = handle.join();
}

#[test]
fn test_error_mapping() {
    let (service, _tmp) = create_temp_service();

    // Duplicate campus code error
    service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "#000".to_string(),
        })
        .expect("create first");

    let err = service
        .create_campus(CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phân hiệu 1 trùng".to_string(),
            color: "#111".to_string(),
        })
        .expect_err("duplicate code");

    assert_eq!(err.code, "duplicate_entry");

    // Not found
    let err2 = service
        .get_plan(PlanId::new(99999))
        .expect_err("nonexistent plan");
    assert_eq!(err2.code, "not_found");
}

#[test]
fn test_rule_presets_and_quota_preview() {
    let (service, _tmp) = create_demo_service();
    let sy_id = exam_panel_core::domain::SchoolYearId::new(1);

    // Rule presets
    let presets = service.get_rule_presets();
    assert_eq!(presets.len(), 4);
    assert_eq!(presets[0].id, "balanced");
    assert_eq!(presets[1].id, "workload_fairness");
    assert_eq!(presets[2].id, "team_diversity");
    assert_eq!(presets[3].id, "allow_task_crowding");

    // Preview quotas under default settings
    let default_settings = service.get_rule_settings(sy_id).expect("rule settings");
    let preview = service
        .preview_quotas(exam_panel_service::dto::PreviewQuotasInput {
            school_year_id: sy_id,
            rule_settings: default_settings.clone(),
        })
        .expect("preview quotas");
    assert!(!preview.is_empty());
    // Active teachers have non-zero available exams
    for item in &preview {
        assert!(item.available_exams > 0);
        assert!(item.hi >= item.lo);
    }
}

#[test]
fn test_exam_crud_and_reorder() {
    let (service, _tmp) = create_demo_service();
    let sy_id = exam_panel_core::domain::SchoolYearId::new(1);

    let exams = service.list_exams(sy_id).expect("list exams");
    let initial_count = exams.len();
    assert!(initial_count >= 2);

    // Reorder: reverse the IDs
    let reversed_ids: Vec<exam_panel_core::domain::ExamId> =
        exams.iter().rev().map(|e| e.id).collect();
    service.reorder_exams(reversed_ids).expect("reorder exams");

    let reordered = service.list_exams(sy_id).expect("list reordered");
    assert_eq!(reordered[0].id, exams.last().unwrap().id);

    // Create a new exam
    let new_exam = service
        .create_exam(exam_panel_service::dto::CreateExamInput {
            school_year_id: sy_id,
            code: "TEST_EXAM".to_string(),
            name: "Kỳ thi thử nghiệm".to_string(),
            sort_order: 99,
        })
        .expect("create exam");
    assert_eq!(new_exam.code, "TEST_EXAM");

    // Delete the new exam (no assignments attached) -> succeeds
    service
        .delete_exam(new_exam.id)
        .expect("delete unused exam");

    let after_delete = service.list_exams(sy_id).expect("list after delete");
    assert_eq!(after_delete.len(), initial_count);
}

#[test]
fn test_ensure_default_school_year() {
    let (service, _tmp) = create_temp_service();
    // Initially empty
    let years = service.list_school_years().expect("list school years");
    assert!(years.is_empty());

    // Ensure default
    service
        .ensure_default_school_year()
        .expect("ensure default");
    let after = service
        .list_school_years()
        .expect("list school years after");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].name, "2026 - 2027");
    assert!(after[0].is_current);

    // Exams created
    let exams = service.list_exams(after[0].id).expect("list exams");
    assert_eq!(exams.len(), 4);

    // Calling again is idempotent
    service
        .ensure_default_school_year()
        .expect("ensure default again");
    let after2 = service
        .list_school_years()
        .expect("list school years after 2");
    assert_eq!(after2.len(), 1);
}
