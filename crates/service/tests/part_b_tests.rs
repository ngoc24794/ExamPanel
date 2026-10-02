use exam_panel_core::domain::{CampusId, ExamId, GradeId, Role, RuleKey, SchoolYearId, SubjectId};
use exam_panel_core::optimize::{OptimizationEffort, SlotRef};
use exam_panel_service::dto::{OptimizeBudget, OptimizeRequest};
use exam_panel_service::service::AppService;

#[test]
fn test_part_b_plan_status_and_staleness() {
    let service = AppService::open_in_memory().expect("open service");
    service.seed_demo().expect("seed demo data");

    let sy_id = SchoolYearId(1);
    let (runs, iters) = OptimizationEffort::Fast.runs_and_iterations();
    let outcome = service
        .run_optimize(
            sy_id,
            OptimizeRequest {
                base_seed: Some(42),
                runs,
                budget: OptimizeBudget::Iterations(iters),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("run optimize");

    let plan_ids = service
        .save_optimize_result(sy_id, outcome)
        .expect("save optimize result");
    let plan_id = plan_ids[0];

    // Status immediately after optimize should be fresh (data_changed: false, rules_changed: false)
    let status_initial = service.plan_status(plan_id).expect("plan_status");
    assert!(!status_initial.data_changed);
    assert!(!status_initial.rules_changed);
    assert!(status_initial.hard_violations_now.is_empty());

    // Make a data change to make problem stale (e.g. create a new teacher)
    service
        .create_teacher(exam_panel_service::dto::CreateTeacherInput {
            full_name: "Teacher New Staleness".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .expect("create teacher");

    // Status now must report data_changed: true
    let status_stale = service.plan_status(plan_id).expect("plan_status");
    assert!(
        status_stale.data_changed,
        "Plan should have data_changed: true after problem data changed!"
    );

    // List plans must also show is_stale: true
    let list = service.list_plans(sy_id).expect("list_plans");
    let summary = list.iter().find(|p| p.id == plan_id).unwrap();
    assert!(summary.is_stale, "PlanSummary should have is_stale: true");

    // mark_final must be rejected with error "plan_stale"
    let err = service.mark_final(plan_id).unwrap_err();
    assert_eq!(err.code, "plan_stale");
}

#[test]
fn test_part_a1_staleness_split_data_vs_rules() {
    let service = AppService::open_in_memory().expect("open service");
    service.seed_demo().expect("seed demo data");

    let sy_id = SchoolYearId(1);
    let outcome = service
        .run_optimize(
            sy_id,
            OptimizeRequest {
                base_seed: Some(42),
                runs: 2,
                budget: OptimizeBudget::Iterations(1_000),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("run optimize");

    let plan_ids = service
        .save_optimize_result(sy_id, outcome)
        .expect("save optimize result");
    let plan_id = plan_ids[0];

    // Initial state: both hashes match
    let st0 = service.plan_status(plan_id).expect("status 0");
    assert!(!st0.data_changed);
    assert!(!st0.rules_changed);

    // 1. Change soft rule weight only: rules_changed = true, data_changed = false
    let mut rules = service.get_rule_settings(sy_id).expect("get_rule_settings");
    if let Some(r) = rules.iter_mut().find(|r| r.key == RuleKey::S1) {
        r.weight = 99.0;
    }
    service
        .save_rule_settings(sy_id, rules)
        .expect("save_rule_settings");

    let st1 = service.plan_status(plan_id).expect("status 1");
    assert!(
        !st1.data_changed,
        "data_changed should be false when only soft rule weight changed"
    );
    assert!(
        st1.rules_changed,
        "rules_changed should be true when soft rule weight changed"
    );

    // mark_final does NOT block on rules_changed
    service
        .mark_final(plan_id)
        .expect("mark_final should succeed when only rules_changed");

    // Unmark final by creating a duplicate
    let copy_id = service
        .create_manual_copy(plan_id, "Bản thử".to_string())
        .expect("copy");

    // 2. Change data: data_changed = true
    service
        .create_teacher(exam_panel_service::dto::CreateTeacherInput {
            full_name: "Thầy Giáo Mới".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("GV_TEST_A1".to_string()),
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .expect("create teacher");

    let st2 = service.plan_status(copy_id).expect("status 2");
    assert!(
        st2.data_changed,
        "data_changed should be true when master data changed"
    );

    // mark_final MUST be blocked on data_changed
    let err = service.mark_final(copy_id).unwrap_err();
    assert_eq!(
        err.code, "plan_stale",
        "mark_final must fail when data_changed is true"
    );

    // 3. Editing and saving a repaired copy refreshes both hashes and unblocks mark_final
    let details = service.get_plan(copy_id).expect("get_plan");
    service
        .update_plan_assignments(copy_id, details.assignments)
        .expect("update_plan_assignments refreshes data_hash and rules_hash");

    let st3 = service.plan_status(copy_id).expect("status 3");
    assert!(
        !st3.data_changed,
        "data_changed should be false after save refreshed hashes"
    );
    assert!(
        !st3.rules_changed,
        "rules_changed should be false after save refreshed hashes"
    );

    service
        .mark_final(copy_id)
        .expect("mark_final should now succeed after saving repaired copy");
}

#[test]
fn test_part_b_manual_editing_and_immutability() {
    let service = AppService::open_in_memory().expect("open service");
    service.seed_demo().expect("seed demo data");

    let sy_id = SchoolYearId(1);
    let outcome = service
        .run_optimize(
            sy_id,
            OptimizeRequest {
                base_seed: Some(42),
                runs: 2,
                budget: OptimizeBudget::Iterations(1_000),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("run optimize");

    let plan_ids = service
        .save_optimize_result(sy_id, outcome)
        .expect("save optimize result");
    let opt_plan_id = plan_ids[0];

    let details = service.get_plan(opt_plan_id).expect("get_plan");
    let mut assignments = details.assignments;

    // 1. Optimizer plans are immutable: update_plan_assignments must fail with "plan_immutable"
    let err = service
        .update_plan_assignments(opt_plan_id, assignments.clone())
        .unwrap_err();
    assert_eq!(err.code, "plan_immutable");

    // 2. Duplicate plan for manual editing
    let manual_plan_id = service
        .create_manual_copy(opt_plan_id, "Bản chỉnh tay".to_string())
        .expect("create_manual_copy");

    // 3. Evaluate candidates on a slot
    let slot = SlotRef {
        exam_id: ExamId(1),
        grade_id: GradeId(1),
        subject_id: SubjectId(1),
        role: Role::Setter,
        position: 0,
    };
    let candidates = service
        .evaluate_candidates(sy_id, assignments.clone(), slot)
        .expect("evaluate_candidates");
    assert!(!candidates.is_empty());

    // 4. Evaluate swap between two slots
    let slot_b = SlotRef {
        exam_id: ExamId(1),
        grade_id: GradeId(1),
        subject_id: SubjectId(1),
        role: Role::Setter,
        position: 1,
    };
    let swap_eval = service
        .evaluate_swap(sy_id, assignments.clone(), slot, slot_b)
        .expect("evaluate_swap");
    assert!(swap_eval.new_total >= 0.0);

    // 5. Apply an invalid assignment (H4 violation: same teacher assigned to multiple panels in same exam)
    let first_exam = assignments[0].exam_id;
    let first_grade = assignments[0].grade_id;
    let t_id = assignments[0].teacher_id;
    for a in &mut assignments {
        if a.exam_id == first_exam && a.grade_id != first_grade {
            a.teacher_id = t_id;
            break;
        }
    }

    // Saving is allowed with hard violations
    let update_res = service
        .update_plan_assignments(manual_plan_id, assignments.clone())
        .expect("update_plan_assignments allows saving with violations");
    assert!(!update_res.hard_violations.is_empty());

    // But mark_final is blocked with "plan_invalid"
    let err = service.mark_final(manual_plan_id).unwrap_err();
    assert_eq!(err.code, "plan_invalid");
}

#[test]
fn test_part_b_reoptimize_from_plan() {
    let service = AppService::open_in_memory().expect("open service");
    service.seed_demo().expect("seed demo data");

    let sy_id = SchoolYearId(1);
    let outcome = service
        .run_optimize(
            sy_id,
            OptimizeRequest {
                base_seed: Some(42),
                runs: 2,
                budget: OptimizeBudget::Iterations(2_000),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("run optimize");

    let plan_ids = service
        .save_optimize_result(sy_id, outcome)
        .expect("save optimize result");
    let plan_id = plan_ids[0];

    let keep_slots = vec![
        SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            subject_id: SubjectId(1),
            role: Role::Setter,
            position: 0,
        },
        SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            subject_id: SubjectId(1),
            role: Role::Reviewer,
            position: 0,
        },
    ];

    let reopt_outcome = service
        .reoptimize_from(
            plan_id,
            keep_slots,
            OptimizeRequest {
                base_seed: Some(999),
                runs: 2,
                budget: OptimizeBudget::Iterations(2_000),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("reoptimize_from");

    assert!(!reopt_outcome.plans.is_empty());
}
