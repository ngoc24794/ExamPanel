//! Unit test for comparison integrity: lower bound assertions and Q-plan score validation.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, make_q_assignments, QVariant};
use exam_panel_core::domain::RuleKey;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use exam_panel_core::score::evaluate;
use std::collections::HashMap;

#[test]
fn test_comparison_integrity_and_lower_bounds() {
    let problem = make_canonical_q_problem(QVariant::NoCampus);
    let q_assigns = make_q_assignments();

    // 1. Evaluate Q's plan
    let q_report = evaluate(&problem, &q_assigns);
    let q_s3_pen = q_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);
    let q_total_no_s3 = q_report.total - q_s3_pen;

    // Verify expected score under defaults: Q without S3 = 116.36
    assert!(
        (q_total_no_s3 - 116.3636).abs() < 0.01,
        "Expected Q's plan without S3 to be 116.36, got {:.2}",
        q_total_no_s3
    );

    // 2. Compute lower bounds
    let bounds = lower_bounds(&problem);
    let mut bounds_map: HashMap<RuleKey, f64> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();
    // Correct S10 bound: exactly 0.0
    bounds_map.insert(RuleKey::S10, 0.0);
    // Correct S5 bound: 1.0 (Pigeonhole principle on CN: 12 panels / 11 reviewers)
    bounds_map.insert(RuleKey::S5, 1.0);
    // Correct S6 bound: 2.0 (consecutive setting parity)
    bounds_map.insert(RuleKey::S6, 2.0);

    let soft_rules = [
        (RuleKey::S1, 10.0),
        (RuleKey::S2, 3.0),
        (RuleKey::S3, 4.0),
        (RuleKey::S4, 6.0),
        (RuleKey::S5, 6.0),
        (RuleKey::S6, 2.0),
        (RuleKey::S7, 1.0),
        (RuleKey::S8, 8.0),
        (RuleKey::S9, 5.0),
        (RuleKey::S10, 4.0),
    ];

    let mut sum_lb_no_s3 = 0.0;
    for (k, w) in &soft_rules {
        if *k != RuleKey::S3 {
            let u = bounds_map.get(k).copied().unwrap_or(0.0);
            sum_lb_no_s3 += u * w;
        }
    }

    // Verify expected Σ LB without S3 = 30.36
    assert!(
        (sum_lb_no_s3 - 30.3636).abs() < 0.01,
        "Expected Σ LB without S3 to be 30.36, got {:.2}",
        sum_lb_no_s3
    );

    // 3. Assert for Q plan: per-rule units >= LB units and total penalty >= Σ LB penalties
    for (k, _) in &soft_rules {
        let lb_u = bounds_map.get(k).copied().unwrap_or(0.0);
        let q_u = q_report
            .by_rule
            .iter()
            .find(|r| r.rule == *k)
            .map_or(0.0, |r| r.units);
        assert!(
            q_u >= lb_u - 1e-4,
            "Q plan rule {:?} units ({:.4}) < LB ({:.4})",
            k,
            q_u,
            lb_u
        );
    }
    assert!(
        q_total_no_s3 >= sum_lb_no_s3 - 1e-4,
        "Q plan total w/o S3 ({:.4}) < Σ LB ({:.4})",
        q_total_no_s3,
        sum_lb_no_s3
    );

    // 4. Assert for an optimizer plan (short run to keep test fast)
    let opt_res = optimize(
        &problem,
        &OptimizeOptions {
            base_seed: 42,
            num_runs: 2,
            budget: Budget::Iterations(5000),
            max_plans: 1,
            diversity_threshold: 0.20,
            initial_assignments: None,
            cancel: None,
            progress: None,
        },
    )
    .expect("optimize");

    let opt_plan = &opt_res.plans[0];
    let opt_s3_pen = opt_plan
        .report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);
    let opt_total_no_s3 = opt_plan.report.total - opt_s3_pen;

    for (k, _) in &soft_rules {
        let lb_u = bounds_map.get(k).copied().unwrap_or(0.0);
        let opt_u = opt_plan
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == *k)
            .map_or(0.0, |r| r.units);
        assert!(
            opt_u >= lb_u - 1e-4,
            "Optimizer plan rule {:?} units ({:.4}) < LB ({:.4})",
            k,
            opt_u,
            lb_u
        );
    }
    assert!(
        opt_total_no_s3 >= sum_lb_no_s3 - 1e-4,
        "Optimizer plan total w/o S3 ({:.4}) < Σ LB ({:.4})",
        opt_total_no_s3,
        sum_lb_no_s3
    );
}
