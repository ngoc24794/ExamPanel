use exam_panel_core::optimize::{optimize, plan_distance, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use exam_panel_core::score::evaluate;
use exam_panel_core::solver::{solve_hard, SolveOptions};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use exam_panel_service::service::AppService;
use std::collections::HashMap;
use tempfile::NamedTempFile;

#[test]
fn test_pipeline_report_zero_hard_violations_and_bounds_respected() {
    let tmp = NamedTempFile::new().expect("create temp db");
    let service = AppService::open_at(tmp.path()).expect("open temp service");
    service.seed_demo().expect("seed demo data");

    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;
    let problem = service.load_problem(sy_id).expect("load problem snapshot");

    // 1. Solve Hard Stage
    let hard_opts = SolveOptions {
        seed: 42,
        ..Default::default()
    };
    let hard_res = solve_hard(&problem, &hard_opts).expect("solve_hard must succeed");
    let hard_report = evaluate(&problem, &hard_res.assignments);

    // 2. Optimize Stage (base seed 42, R = 8, 200k iterations, K = 3)
    let opt_opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 8,
        budget: Budget::Iterations(200_000),
        max_plans: 3,
        diversity_threshold: 0.20,
        ..Default::default()
    };
    let opt_res = optimize(&problem, &opt_opts).expect("multi-plan optimize must succeed");

    // 3. Validation (require complete = true)
    let val_opts = ValidateOptions {
        require_complete: true,
        ..Default::default()
    };
    let hard_val = validate_assignments(&problem, &hard_res.assignments, &val_opts);
    assert!(
        hard_val.is_empty(),
        "Stage 1 solve_hard must have 0 hard violations, got: {hard_val:?}"
    );

    let bounds = lower_bounds(&problem);
    let bounds_map: HashMap<_, _> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();

    // Verify solve_hard scores >= lower bounds
    for r in &hard_report.by_rule {
        let lb = bounds_map.get(&r.rule).copied().unwrap_or(0.0);
        assert!(
            r.units >= lb - 1e-6,
            "Hard solve units ({:.2}) for rule {:?} is LESS than lower bound ({:.2})",
            r.units,
            r.rule,
            lb
        );
    }

    // Verify each optimized plan
    assert_eq!(opt_res.plans.len(), 3, "Expected 3 diverse plans returned");
    for (idx, plan) in opt_res.plans.iter().enumerate() {
        let plan_val = validate_assignments(&problem, &plan.assignments, &val_opts);
        assert!(
            plan_val.is_empty(),
            "Optimized Plan {} (Rank {}) must have 0 hard violations, got: {plan_val:?}",
            idx + 1,
            plan.rank
        );

        // Every rule score >= lower bound
        for r in &plan.report.by_rule {
            let lb = bounds_map.get(&r.rule).copied().unwrap_or(0.0);
            assert!(
                r.units >= lb - 1e-6,
                "Plan {} units ({:.2}) for rule {:?} is LESS than lower bound ({:.2})",
                plan.rank,
                r.units,
                r.rule,
                lb
            );
        }
    }

    // Pairwise diversity threshold
    for i in 0..opt_res.plans.len() {
        for j in (i + 1)..opt_res.plans.len() {
            let d = plan_distance(&opt_res.plans[i].assignments, &opt_res.plans[j].assignments);
            assert!(
                d >= 0.20,
                "Distance between Plan {} and Plan {} ({:.3}) must be >= 0.20",
                opt_res.plans[i].rank,
                opt_res.plans[j].rank,
                d
            );
        }
    }
}
