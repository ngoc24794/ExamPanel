//! Binary to benchmark and report all 4 canonical rule presets on nocampus Q-shaped data.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, QVariant};
use exam_panel_core::domain::{RuleKey, RulePreset};
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("# Đánh giá 4 bộ cấu hình mẫu (Rule Presets Analysis)");
    println!("\nKiểm định trên tập dữ liệu canonical Q (`QVariant::NoCampus`):");
    println!("- Thông số: Cold start, base seed = 42 (seeds 42..49), R = 8, Budget = 200,000 bước lặp / luồng, K = 3.");
    println!("- Cận dưới: S5 LB = 1.00 (Dirichlet), S6 LB = 2.00 (Parity), S8 LB = 2.5455 (28/11), S3 = 0.00 (độc lập) / 36.00 (đơn cơ sở).\n");

    let base_problem = make_canonical_q_problem(QVariant::NoCampus);

    // Precompute bounds
    let bounds = lower_bounds(&base_problem);
    let mut bounds_map: HashMap<RuleKey, f64> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();
    bounds_map.insert(RuleKey::S10, 0.0);
    bounds_map.insert(RuleKey::S5, 1.0);
    bounds_map.insert(RuleKey::S6, 2.0);

    let soft_rules = [
        (RuleKey::S1, "S1 Reviewer frequency"),
        (RuleKey::S2, "S2 Role ratio balance"),
        (RuleKey::S3, "S3 Campus independence"),
        (RuleKey::S4, "S4 Setter pair diversity"),
        (RuleKey::S5, "S5 Review relation diversity"),
        (RuleKey::S6, "S6 Setter consecutive"),
        (RuleKey::S7, "S7 Grade rotation"),
        (RuleKey::S8, "S8 Workload quota balance"),
        (RuleKey::S9, "S9 Avoidable exam crowding"),
        (RuleKey::S10, "S10 Review subject missing"),
    ];

    let presets = [
        (RulePreset::Balanced, "Cân bằng (mặc định)"),
        (RulePreset::WorkloadFairness, "Ưu tiên công bằng khối lượng"),
        (RulePreset::TeamDiversity, "Ưu tiên đa dạng ê-kíp"),
        (
            RulePreset::AllowTaskCrowding,
            "Cho phép dồn việc trong một kỳ",
        ),
    ];

    let opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 8,
        budget: Budget::Iterations(200_000),
        max_plans: 3,
        diversity_threshold: 0.20,
        initial_assignments: None,
        cancel: None,
        progress: None,
    };

    for (preset, name) in presets {
        let mut problem = base_problem.clone();
        problem.rule_settings = preset.settings();

        eprintln!("Evaluating preset '{}' ({})...", preset.id(), name);
        let res = optimize(&problem, &opts)?;
        let best_plan = &res.plans[0];
        let report = &best_plan.report;

        println!("## Preset: {} (`{}`)", name, preset.id());
        println!("| Quy tắc | Trạng thái | Trọng số | Cận dưới (LB) | Đơn vị vi phạm | Điểm phạt | Chênh lệch (Gap) |");
        println!("| :--- | :---: | :---: | :---: | :---: | :---: | :---: |");

        let mut sum_lb_no_s3 = 0.0;
        let mut sum_lb_with_s3 = 0.0;
        let mut tot_pen_no_s3 = 0.0;
        let mut tot_pen_with_s3 = 0.0;

        for (k, label) in soft_rules {
            let setting = problem.rule_settings.iter().find(|s| s.key == k);
            let enabled = setting.is_none_or(|s| s.enabled);
            let weight = setting.map_or(0.0, |s| s.weight);

            let lb_u = bounds_map.get(&k).copied().unwrap_or(0.0);
            let rule_score = report.by_rule.iter().find(|r| r.rule == k);
            let u = rule_score.map_or(0.0, |r| r.units);
            let p = rule_score.map_or(0.0, |r| r.penalty);
            let gap = (u - lb_u).max(0.0);

            println!(
                "| {:<28} | {:^10} | {:>8.1} | {:>13.2} | {:>14.2} | {:>9.2} | {:>16.2} |",
                label,
                if enabled { "BẬT" } else { "TẮT" },
                weight,
                lb_u,
                u,
                p,
                gap
            );

            if k != RuleKey::S3 {
                sum_lb_no_s3 += lb_u * weight;
                sum_lb_with_s3 += lb_u * weight;
                tot_pen_no_s3 += p;
                tot_pen_with_s3 += p;
            } else {
                sum_lb_with_s3 += 36.0 * weight; // single campus 36 units
                tot_pen_with_s3 += p;
            }
        }

        println!(
            "| **Tổng cộng (Không tính S3)** | | | **{:.2}** | | **{:.2}** | **{:.2}** |",
            sum_lb_no_s3,
            tot_pen_no_s3,
            tot_pen_no_s3 - sum_lb_no_s3
        );
        println!(
            "| **Tổng cộng (Có S3)** | | | **{:.2}** | | **{:.2}** | **{:.2}** |\n",
            sum_lb_with_s3,
            tot_pen_with_s3,
            tot_pen_with_s3 - sum_lb_with_s3
        );
    }

    Ok(())
}
