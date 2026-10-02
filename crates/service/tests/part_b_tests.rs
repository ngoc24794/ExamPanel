use exam_panel_core::domain::{CampusId, ExamId, GradeId, Role, SchoolYearId};
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

    // Status immediately after optimize should be fresh (problem_changed: false)
    let status_initial = service.plan_status(plan_id).expect("plan_status");
    assert!(!status_initial.problem_changed);
    assert!(status_initial.hard_violations_now.is_empty());

    // Make a data change to make problem stale (e.g. create a new teacher)
    service
        .create_teacher(exam_panel_service::dto::CreateTeacherInput {
            full_name: "Teacher New Staleness".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
        })
        .expect("create teacher");

    // Status now must report problem_changed: true
    let status_stale = service.plan_status(plan_id).expect("plan_status");
    assert!(
        status_stale.problem_changed,
        "Plan should be marked stale after problem data changed!"
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
            role: Role::Setter,
            position: 0,
        },
        SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
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
