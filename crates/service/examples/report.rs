use exam_panel_core::domain::{calculate_quotas, Role, RuleKey};
use exam_panel_core::optimize::{optimize, plan_distance, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::{lower_bounds, optimal_s8_counts};
use exam_panel_core::score::evaluate;
use exam_panel_core::solver::{solve_hard, SolveOptions};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use exam_panel_service::service::AppService;
use std::collections::HashMap;
use tempfile::NamedTempFile;

fn main() {
    let tmp = NamedTempFile::new().expect("create temp db");
    let service = AppService::open_at(tmp.path()).expect("open temp service");
    service.seed_demo().expect("seed demo data");

    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;
    let problem = service.load_problem(sy_id).expect("load problem snapshot");

    // 1. Solve Hard Stage (seed 42)
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

    // 3. Validation
    let val_opts = ValidateOptions {
        require_complete: true,
        ..Default::default()
    };
    let hard_val = validate_assignments(&problem, &hard_res.assignments, &val_opts);
    let mut plan_vals = Vec::new();
    for plan in &opt_res.plans {
        plan_vals.push(validate_assignments(&problem, &plan.assignments, &val_opts));
    }

    // Precompute bounds & quotas
    let bounds = lower_bounds(&problem);
    let bounds_map: HashMap<RuleKey, f64> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();
    let s8_opt_counts = optimal_s8_counts(&problem);
    let quotas = calculate_quotas(&problem);

    println!("# Reproducible Phase 5 & 6 Verification Report\n");
    println!(
        "> Generated automatically by `crates/service/examples/report.rs` on {}\n",
        chrono_or_date()
    );

    // -------------------------------------------------------------------------
    // 1) RULE WEIGHTS
    // -------------------------------------------------------------------------
    println!("## 1. Rule Weights: Database Defaults vs SPEC Defaults\n");
    println!("| Rule | Name | DB Weight | SPEC Weight | Status |");
    println!("|------|------|-----------|-------------|--------|");

    let spec_defaults = [
        (RuleKey::H4, "H4: Single Panel Per Exam", 100.0),
        (RuleKey::H7, "H7: Workload Quota Compliance", 100.0),
        (RuleKey::S1, "S1: Reviewer Count / Frequency", 10.0),
        (RuleKey::S2, "S2: Role Balance (1/3 Reviewer)", 3.0),
        (RuleKey::S3, "S3: Independent Reviewer Campus", 4.0),
        (RuleKey::S4, "S4: Repeated Setter Pair Diversity", 6.0),
        (RuleKey::S5, "S5: Repeated Review Relation Diversity", 6.0),
        (RuleKey::S6, "S6: Consecutive Setter Exam Relief", 2.0),
        (RuleKey::S7, "S7: Multi-Grade Rotation", 1.0),
        (RuleKey::S8, "S8: Discrete Load Balance", 8.0),
    ];

    let mut weight_mismatch = false;
    for (key, name, spec_w) in spec_defaults {
        let db_setting = problem.rule_settings.iter().find(|r| r.key == key);
        let db_w = db_setting.map_or(0.0, |r| r.weight);
        let status = if (db_w - spec_w).abs() < 1e-6 {
            "MATCH"
        } else {
            weight_mismatch = true;
            "MISMATCH"
        };
        println!(
            "| {:?} | {} | {:.1} | {:.1} | {} |",
            key, name, db_w, spec_w, status
        );
    }
    println!();
    if weight_mismatch {
        println!("**ALERT:** Found rule weight mismatch between DB defaults and SPEC!\n");
    } else {
        println!("**All rule weights in DB match SPEC defaults perfectly.**\n");
    }

    // -------------------------------------------------------------------------
    // 2) PER-RULE TABLE
    // -------------------------------------------------------------------------
    println!("## 2. Per-Rule Score & Lower Bound Comparison\n");
    println!("| Rule | Weight | LB Units | Hard Units | Hard Pen | Plan 1 Units | Plan 1 Pen | Plan 2 Units | Plan 2 Pen | Plan 3 Units | Plan 3 Pen |");
    println!("|------|--------|----------|------------|----------|--------------|------------|--------------|------------|--------------|------------|");

    let soft_rules = [
        RuleKey::S1,
        RuleKey::S2,
        RuleKey::S3,
        RuleKey::S4,
        RuleKey::S5,
        RuleKey::S6,
        RuleKey::S7,
        RuleKey::S8,
    ];

    let mut lb_total_pen = 0.0;
    for rule in soft_rules {
        let w = problem
            .rule_settings
            .iter()
            .find(|r| r.key == rule)
            .map_or(0.0, |r| r.weight);
        let lb_u = bounds_map.get(&rule).copied().unwrap_or(0.0);
        lb_total_pen += lb_u * w;

        let hard_sc = hard_report.by_rule.iter().find(|r| r.rule == rule).unwrap();
        let p1_sc = opt_res.plans[0]
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == rule)
            .unwrap();
        let p2_sc = opt_res.plans[1]
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == rule)
            .unwrap();
        let p3_sc = opt_res.plans[2]
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == rule)
            .unwrap();

        println!(
            "| {:?} | {:.1} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            rule,
            w,
            lb_u,
            hard_sc.units,
            hard_sc.penalty,
            p1_sc.units,
            p1_sc.penalty,
            p2_sc.units,
            p2_sc.penalty,
            p3_sc.units,
            p3_sc.penalty
        );
    }

    println!(
        "| **TOTAL** | — | — | — | **{:.2}** | — | **{:.2}** | — | **{:.2}** | — | **{:.2}** |",
        hard_report.total,
        opt_res.plans[0].report.total,
        opt_res.plans[1].report.total,
        opt_res.plans[2].report.total
    );
    println!(
        "\n*Theoretical Lower Bound Penalty Sum: {:.2}*\n",
        lb_total_pen
    );

    // -------------------------------------------------------------------------
    // 3) PER-TEACHER TABLE (PLAN 1)
    // -------------------------------------------------------------------------
    println!("## 3. Per-Teacher Workload & Quota Allocation (Plan 1)\n");
    println!("| Teacher | Campus | Load | Available | Quota (q) | [lo, hi] | S8-Opt | Actual | Setter | Reviewer | Assigned Grades |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|");

    let best_plan = &opt_res.plans[0];
    for t in &problem.teachers {
        let campus = problem
            .campuses
            .iter()
            .find(|c| c.id == t.campus_id)
            .unwrap();
        let q = quotas.iter().find(|q| q.teacher_id == t.id).unwrap();
        let s8_opt = s8_opt_counts.get(&t.id).copied().unwrap_or(0);
        let ts = best_plan
            .report
            .per_teacher
            .iter()
            .find(|ts| ts.teacher_id == t.id)
            .unwrap();

        let unavail_count = problem
            .unavailabilities
            .iter()
            .filter(|u| u.teacher_id == t.id)
            .count();
        let avail_exams = problem.exams.len() - unavail_count;

        let grades_str = ts
            .grades_assigned
            .iter()
            .map(|gid| {
                let g = problem.grades.iter().find(|g| g.id == *gid).unwrap();
                format!("Khối {}", g.code)
            })
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "| {} | {} ({}) | {:.2} | {}/{} | {:.2} | [{}, {}] | {} | {} | {} | {} | {} |",
            t.full_name,
            campus.name,
            campus.code,
            t.load_weight,
            avail_exams,
            problem.exams.len(),
            q.quota,
            q.lo,
            q.hi,
            s8_opt,
            ts.count,
            ts.setter,
            ts.reviewer,
            grades_str
        );
    }
    println!();

    // -------------------------------------------------------------------------
    // 4) 4 × 3 ASSIGNMENT MATRIX WITH CAMPUSES (BEST PLAN)
    // -------------------------------------------------------------------------
    println!("## 4. Assignment Matrix: 4 Exams × 3 Grades (Plan 1)\n");
    println!("| Exam | Khối 10 | Khối 11 | Khối 12 |");
    println!("|---|---|---|---|");

    for exam in &problem.exams {
        print!("| **{}** ({}) |", exam.name, exam.code);
        for grade in &problem.grades {
            let setters: Vec<String> = best_plan
                .assignments
                .iter()
                .filter(|a| {
                    a.exam_id == exam.id && a.grade_id == grade.id && a.role == Role::Setter
                })
                .map(|a| {
                    let t = problem
                        .teachers
                        .iter()
                        .find(|t| t.id == a.teacher_id)
                        .unwrap();
                    let c = problem
                        .campuses
                        .iter()
                        .find(|c| c.id == t.campus_id)
                        .unwrap();
                    format!("{} ({})", t.full_name, c.code)
                })
                .collect();
            let reviewer = best_plan
                .assignments
                .iter()
                .find(|a| {
                    a.exam_id == exam.id && a.grade_id == grade.id && a.role == Role::Reviewer
                })
                .map(|a| {
                    let t = problem
                        .teachers
                        .iter()
                        .find(|t| t.id == a.teacher_id)
                        .unwrap();
                    let c = problem
                        .campuses
                        .iter()
                        .find(|c| c.id == t.campus_id)
                        .unwrap();
                    format!("{} ({})", t.full_name, c.code)
                })
                .unwrap_or_else(|| "None".to_string());

            print!(
                " **Ra đề:** {}<br/>**Phản biện:** {} |",
                setters.join(", "),
                reviewer
            );
        }
        println!();
    }
    println!();

    // -------------------------------------------------------------------------
    // 5) HARD VIOLATIONS CHECK
    // -------------------------------------------------------------------------
    println!("## 5. Constraint Validator Output\n");
    println!(
        "- **Stage 1 (Solve Hard):** {} hard violations (valid = {})",
        hard_val.len(),
        hard_val.is_empty()
    );
    for v in &hard_val {
        println!("  - [{:?}] {}: {:?}", v.rule, v.code, v.params);
    }
    for (i, v_res) in plan_vals.iter().enumerate() {
        println!(
            "- **Plan {} (Rank {}):** {} hard violations (valid = {})",
            i + 1,
            opt_res.plans[i].rank,
            v_res.len(),
            v_res.is_empty()
        );
        for v in v_res {
            println!("  - [{:?}] {}: {:?}", v.rule, v.code, v.params);
        }
    }
    println!();

    // -------------------------------------------------------------------------
    // 6) PAIRWISE PLAN DISTANCES
    // -------------------------------------------------------------------------
    println!("## 6. Pairwise Plan Distances\n");
    println!("| Pair | Distance | Similarity | Interpretation |");
    println!("|---|---|---|---|");
    for i in 0..opt_res.plans.len() {
        for j in (i + 1)..opt_res.plans.len() {
            let d = plan_distance(&opt_res.plans[i].assignments, &opt_res.plans[j].assignments);
            println!(
                "| Plan {} vs Plan {} | {:.3} | {:.1}% | Exceeds 20% threshold ({}) |",
                opt_res.plans[i].rank,
                opt_res.plans[j].rank,
                d,
                (1.0 - d) * 100.0,
                if d >= 0.20 { "PASS" } else { "FAIL" }
            );
        }
    }
    let d_hard_best = plan_distance(&hard_res.assignments, &opt_res.plans[0].assignments);
    println!(
        "| Solve Hard vs Plan 1 | {:.3} | {:.1}% | Divergence after 200k anneal moves |",
        d_hard_best,
        (1.0 - d_hard_best) * 100.0
    );
    println!();
}

fn chrono_or_date() -> &'static str {
    "2026-10-02"
}
