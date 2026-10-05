//! Unit tests for warm start optimizer behavior and Q-plan initial score parity.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, make_q_assignments, QVariant};
use exam_panel_core::domain::RuleKey;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::evaluate;

#[test]
fn test_warm_start_initial_score_and_convergence() {
    let problem = make_canonical_q_problem(QVariant::NoCampus);
    let q_assigns = make_q_assignments();

    // 1. Initial score computed by evaluator on Q's plan
    let initial_report = evaluate(&problem, &q_assigns);
    let s3_pen = initial_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);

    assert_eq!(
        format!("{:.2}", initial_report.total),
        "260.36",
        "Initial score with S3 must be exactly 260.36"
    );
    assert_eq!(
        format!("{:.2}", initial_report.total - s3_pen),
        "116.36",
        "Initial score without S3 must be exactly 116.36"
    );

    // 2. Warm run with seed 42
    let warm_opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 2,
        budget: Budget::Iterations(10_000),
        max_plans: 1,
        diversity_threshold: 0.20,
        initial_assignments: Some(q_assigns),
        cancel: None,
        progress: None,
    };

    let warm_res = optimize(&problem, &warm_opts).expect("warm optimize");
    let warm_plan = &warm_res.plans[0];

    // Warm start must improve upon or equal initial score
    assert!(
        warm_plan.report.total <= initial_report.total,
        "Warm start final score ({}) must be <= initial score ({})",
        warm_plan.report.total,
        initial_report.total
    );

    // Verify seed is one of the valid run seeds generated from base_seed 42
    let valid_seeds: Vec<u64> = (0..2)
        .map(|r| 42u64.wrapping_add((r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) + 1))
        .collect();
    assert!(
        valid_seeds.contains(&warm_plan.seed),
        "Winning seed ({}) must be one of the generated run seeds: {:?}",
        warm_plan.seed,
        valid_seeds
    );
}
