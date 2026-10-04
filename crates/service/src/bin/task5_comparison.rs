//! Task 5 Comparison Tool: Q's manual plan vs Cold Start vs Warm Start on nocampus.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, make_q_assignments, QVariant};
use exam_panel_core::domain::RuleKey;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use exam_panel_core::score::evaluate;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let problem = make_canonical_q_problem(QVariant::NoCampus);
    let q_assigns = make_q_assignments();
    let q_report = evaluate(&problem, &q_assigns);

    // 1. Cold start optimizer
    eprintln!("Running Cold Start Optimizer (seed 42, R = 8, 200,000 iters)...");
    let cold_opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 8,
        budget: Budget::Iterations(200_000),
        max_plans: 3,
        diversity_threshold: 0.20,
        initial_assignments: None,
        cancel: None,
        progress: None,
    };
    let cold_res = optimize(&problem, &cold_opts)?;
    let cold_plan = &cold_res.plans[0];
    let cold_report = &cold_plan.report;

    // 2. Warm start optimizer from Q's plan
    eprintln!("Running Warm Start Optimizer (seed 42, R = 8, 200,000 iters from Q's plan)...");
    let warm_opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 8,
        budget: Budget::Iterations(200_000),
        max_plans: 3,
        diversity_threshold: 0.20,
        initial_assignments: Some(q_assigns.clone()),
        cancel: None,
        progress: None,
    };
    let warm_res = optimize(&problem, &warm_opts)?;
    let warm_plan = &warm_res.plans[0];
    let warm_report = &warm_plan.report;

    // Precompute bounds
    let bounds = lower_bounds(&problem);
    let mut bounds_map: HashMap<RuleKey, f64> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();
    // Correct S10 bound: exactly 0.0 because 11 teachers can review 12 VL and 12 CN panels
    bounds_map.insert(RuleKey::S10, 0.0);
    // Correct S5 bound: exactly 1.0 by Pigeonhole Principle on CN (12 panels / 11 teachers with fixed setter T Nghĩa)
    bounds_map.insert(RuleKey::S5, 1.0);

    let soft_rules = [
        (RuleKey::S1, 10.0, "S1 Reviewer frequency"),
        (RuleKey::S2, 3.0, "S2 Role ratio balance"),
        (RuleKey::S3, 4.0, "S3 Campus independence"),
        (RuleKey::S4, 6.0, "S4 Setter pair diversity"),
        (RuleKey::S5, 6.0, "S5 Review relation diversity"),
        (RuleKey::S6, 2.0, "S6 Setter consecutive"),
        (RuleKey::S7, 1.0, "S7 Grade rotation"),
        (RuleKey::S8, 8.0, "S8 Workload quota balance"),
        (RuleKey::S9, 5.0, "S9 Avoidable exam crowding"),
        (RuleKey::S10, 4.0, "S10 Review subject missing"),
    ];

    let q_s3_pen = q_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);
    let cold_s3_pen = cold_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);
    let warm_s3_pen = warm_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);

    println!("\n# Task 5 Comparison: Q Manual Plan vs Optimizer (nocampus)\n");

    println!("## Warm vs Cold Optimization Summary");
    println!("- Base Seed: 42, Runs: 8 (seeds 42..49), Iterations: 200,000 per run");
    println!("- Initial Score of Warm Run (Q's manual plan on nocampus with C Quí's override):");
    println!("  * With S3: {:.2}", q_report.total);
    println!("  * Without S3: {:.2}", q_report.total - q_s3_pen);
    println!(
        "- Final Best Score (Cold Start): With S3 = {:.2}, Without S3 = {:.2}",
        cold_report.total,
        cold_report.total - cold_s3_pen
    );
    println!(
        "- Final Best Score (Warm Start): With S3 = {:.2}, Without S3 = {:.2}",
        warm_report.total,
        warm_report.total - warm_s3_pen
    );

    let identical_in_every_rule = soft_rules.iter().all(|(key, _, _)| {
        let cu = cold_report
            .by_rule
            .iter()
            .find(|r| r.rule == *key)
            .map_or(0.0, |r| r.units);
        let wu = warm_report
            .by_rule
            .iter()
            .find(|r| r.rule == *key)
            .map_or(0.0, |r| r.units);
        (cu - wu).abs() < 1e-4
    });
    if identical_in_every_rule {
        println!("- Evidence of Identicality: Cold and Warm optimization runs produce IDENTICAL unit scores across all 10 rules.\n");
    }
    println!("| Rule | Weight | LB Units | Q Units | Q Pen | Q Gap | Cold Units | Cold Pen | Cold Gap | Warm Units | Warm Pen | Warm Gap |");
    println!("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |");

    let mut q_tot_with_s3 = 0.0;
    let mut q_tot_no_s3 = 0.0;
    let mut cold_tot_with_s3 = 0.0;
    let mut cold_tot_no_s3 = 0.0;
    let mut warm_tot_with_s3 = 0.0;
    let mut warm_tot_no_s3 = 0.0;

    for (key, default_w, label) in soft_rules {
        let lb = bounds_map.get(&key).copied().unwrap_or(0.0);
        let q_u = q_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(0.0, |r| r.units);
        let q_w = q_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(default_w, |r| r.weight);
        let q_p = q_u * q_w;
        let q_gap = (q_u - lb).max(0.0);

        let cold_u = cold_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(0.0, |r| r.units);
        let cold_w = cold_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(default_w, |r| r.weight);
        let cold_p = cold_u * cold_w;
        let cold_gap = (cold_u - lb).max(0.0);

        let warm_u = warm_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(0.0, |r| r.units);
        let warm_w = warm_report
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(default_w, |r| r.weight);
        let warm_p = warm_u * warm_w;
        let warm_gap = (warm_u - lb).max(0.0);

        println!(
            "| {:<28} | {:>4.1} | {:>6.2} | {:>7.2} | {:>6.2} | {:>5.2} | {:>10.2} | {:>8.2} | {:>8.2} | {:>10.2} | {:>8.2} | {:>8.2} |",
            label, default_w, lb, q_u, q_p, q_gap, cold_u, cold_p, cold_gap, warm_u, warm_p, warm_gap
        );

        q_tot_with_s3 += q_p;
        cold_tot_with_s3 += cold_p;
        warm_tot_with_s3 += warm_p;

        if key != RuleKey::S3 {
            q_tot_no_s3 += q_p;
            cold_tot_no_s3 += cold_p;
            warm_tot_no_s3 += warm_p;
        }
    }

    println!(
        "| **Total (with S3)** | | | | **{:.2}** | | | **{:.2}** | | | **{:.2}** |",
        q_tot_with_s3, cold_tot_with_s3, warm_tot_with_s3
    );
    println!(
        "| **Total (without S3)** | | | | **{:.2}** | | | **{:.2}** | | | **{:.2}** |",
        q_tot_no_s3, cold_tot_no_s3, warm_tot_no_s3
    );

    println!("\n## Lower Bound Reasoning");
    println!("- **S6 Lower Bound = 2.0**: Across the 4 exams, 24 non-forced setter presences are distributed over 11 eligible teachers (k_t <= 4), with Teacher 12 excluded as his 12 setter seats are strictly forced. Minimizing sum f(k_t) where f(k) = max(0, 2k - 5) allocates 9 teachers 2 presences (f=0) and 2 teachers 3 presences (f=1 each), establishing an exact lower bound of 2.0 units.");
    println!("- **S10 Lower Bound = 0.0**: 11 teachers (Teachers 1..11) possess competencies to review both Vật lí (VL) and Công nghệ (CN). Across the schedule there are 12 VL review seats and 12 CN review seats. Since 11 <= 12 in both subjects, it is mathematically possible for every teacher to review each subject at least once.");
    println!("- **S5 Lower Bound >= 1.0**: In subject CN, all 12 panels require exactly 1 reviewer, and Teacher 12 (Thầy Nghĩa) is the unique forced setter. Only Teachers 1..11 can review CN. By the Pigeonhole Principle, assigning 12 seats among 11 teachers requires at least one teacher to review CN at least ceil(12/11) = 2 times. Each repeated review of Thầy Nghĩa incurs max(0, count - 1) >= 1 unit of penalty. Hence S5 >= 1.0.");
    println!("- **S3 Single-Campus Explanation**: On nocampus, all teachers reside at Campus 1. For each of the 12 Vật lí (VL) panels (2 setters, 1 reviewer), the reviewer shares campus with 2 setters, contributing 12 x 2 = 24 units. For each of the 12 Công nghệ (CN) panels (1 setter, 1 reviewer), the reviewer shares campus with 1 setter, contributing 12 x 1 = 12 units. Total single-campus S3 units = 24 + 12 = 36.0 units.");

    Ok(())
}
