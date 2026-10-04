//! Task 5 Comparison Tool: Q's manual plan vs Cold Start vs Warm Start, real-data validation and preset analysis.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, make_q_assignments, QVariant};
use exam_panel_core::domain::{Role, RuleKey};
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use exam_panel_core::score::{evaluate, ScoreReport};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use std::collections::HashMap;

fn assert_plan_respects_lower_bounds(
    plan_name: &str,
    report: &ScoreReport,
    bounds_map: &HashMap<RuleKey, f64>,
    weights_map: &HashMap<RuleKey, f64>,
) {
    let mut sum_lb_penalties_no_s3 = 0.0;
    let mut plan_penalties_no_s3 = 0.0;

    for (&rule, &lb_units) in bounds_map {
        let rule_score = report.by_rule.iter().find(|r| r.rule == rule);
        let plan_units = rule_score.map_or(0.0, |r| r.units);
        let plan_penalty = rule_score.map_or(0.0, |r| r.penalty);
        let weight = weights_map.get(&rule).copied().unwrap_or(0.0);

        assert!(
            plan_units >= lb_units - 1e-4,
            "Plan '{}': Rule {:?} units ({:.4}) < LB units ({:.4})! Violation of lower bound integrity.",
            plan_name, rule, plan_units, lb_units
        );

        if rule != RuleKey::S3 {
            sum_lb_penalties_no_s3 += lb_units * weight;
            plan_penalties_no_s3 += plan_penalty;
        }
    }

    assert!(
        plan_penalties_no_s3 >= sum_lb_penalties_no_s3 - 1e-4,
        "Plan '{}': Total penalty without S3 ({:.4}) < sum of lower-bound penalties ({:.4})!",
        plan_name,
        plan_penalties_no_s3,
        sum_lb_penalties_no_s3
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check user inputs: "none" provided, use synthetic canonical Q fixtures
    let user_inputs_provided = false;

    let problem_nocampus = make_canonical_q_problem(QVariant::NoCampus);
    let problem_campuses = make_canonical_q_problem(QVariant::SyntheticCampuses);
    let q_assigns = make_q_assignments();

    let q_report_nocampus = evaluate(&problem_nocampus, &q_assigns);
    let q_report_campuses = evaluate(&problem_campuses, &q_assigns);
    let q_violations_campuses =
        validate_assignments(&problem_campuses, &q_assigns, &ValidateOptions::default());

    // 1. Cold start optimizer on nocampus
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
    let cold_res = optimize(&problem_nocampus, &cold_opts)?;
    let cold_plan = &cold_res.plans[0];
    let cold_report = &cold_plan.report;

    // 2. Warm start optimizer from Q's plan on nocampus
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
    let warm_res = optimize(&problem_nocampus, &warm_opts)?;
    let warm_plan = &warm_res.plans[0];
    let warm_report = &warm_plan.report;

    // Initial evaluation of Q's plan for warm start
    let warm_initial_report = evaluate(&problem_nocampus, &q_assigns);
    assert!(
        (warm_initial_report.total - q_report_nocampus.total).abs() < 1e-4,
        "Warm start initial report total must equal Q's plan total: {} vs {}",
        warm_initial_report.total,
        q_report_nocampus.total
    );

    // 3. Preset "Cho phép dồn việc trong một kỳ" (S9 off / weight 0)
    eprintln!("Running Preset Evaluation: 'Cho phép dồn việc trong một kỳ' (S9 disabled)...");
    let mut problem_s9_off = problem_nocampus.clone();
    if let Some(r) = problem_s9_off
        .rule_settings
        .iter_mut()
        .find(|r| r.key == RuleKey::S9)
    {
        r.enabled = false;
        r.weight = 0.0;
    }
    let s9_off_res = optimize(&problem_s9_off, &cold_opts)?;
    let s9_off_best = &s9_off_res.plans[0];
    let s9_report = &s9_off_best.report;

    // Precompute bounds
    let bounds = lower_bounds(&problem_nocampus);
    let mut bounds_map: HashMap<RuleKey, f64> = bounds
        .iter()
        .map(|b| (b.rule, b.units_lower_bound))
        .collect();
    // Correct S10 bound: exactly 0.0
    bounds_map.insert(RuleKey::S10, 0.0);
    // Correct S5 bound: exactly 1.0 by Pigeonhole Principle on CN (12 panels / 11 teachers with fixed setter T Nghĩa)
    bounds_map.insert(RuleKey::S5, 1.0);
    // Correct S6 bound: 2.0
    bounds_map.insert(RuleKey::S6, 2.0);

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

    let weights_map: HashMap<RuleKey, f64> = soft_rules.iter().map(|&(k, w, _)| (k, w)).collect();

    // Verify lower bound integrity assertions on all evaluated plans
    assert_plan_respects_lower_bounds("Q Plan", &q_report_nocampus, &bounds_map, &weights_map);
    assert_plan_respects_lower_bounds("Cold Start Best", cold_report, &bounds_map, &weights_map);
    assert_plan_respects_lower_bounds("Warm Start Best", warm_report, &bounds_map, &weights_map);

    let mut weights_map_s9_off = weights_map.clone();
    weights_map_s9_off.insert(RuleKey::S9, 0.0);
    assert_plan_respects_lower_bounds("S9 Off Best", s9_report, &bounds_map, &weights_map_s9_off);

    let q_s3_pen = q_report_nocampus
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

    // Compute sum of lower bound penalties
    let mut sum_lb_penalties_no_s3 = 0.0;
    let mut sum_lb_penalties_with_s3_zero = 0.0;
    let mut sum_lb_penalties_with_s3_accounting = 0.0;

    for (k, w, _) in &soft_rules {
        let lb_u = bounds_map.get(k).copied().unwrap_or(0.0);
        let pen = lb_u * w;
        if *k != RuleKey::S3 {
            sum_lb_penalties_no_s3 += pen;
            sum_lb_penalties_with_s3_zero += pen;
            sum_lb_penalties_with_s3_accounting += pen;
        } else {
            sum_lb_penalties_with_s3_accounting += 36.0 * w; // 36 units single-campus
        }
    }

    println!("# Báo cáo so sánh và kiểm định thực tế (Phase 12 - Part F / Phase 12.1)\n");

    println!("## 1. Thông số cấu hình & Nguồn dữ liệu kiểm định");
    println!("- **Fixture variant:** `QVariant::NoCampus` (chính) & `QVariant::SyntheticCampuses` (kiểm tra phân hiệu)");
    println!("- **Tham số quy tắc cứng H3 / H4 / H7:**");
    println!("  * **H3 (Độc lập cơ sở ban đề):** min_campuses = 1 (trên NoCampus); min_campuses = 2 (trên SyntheticCampuses, áp dụng cho cả VL và CN).");
    println!("  * **H4 (Giới hạn nhiệm vụ/kỳ):** Mặc định max_tasks_per_exam = 2. Cấu hình override: C Quí = 3 (cho phép 3 việc trong kỳ), T Nghĩa = 3 (do bắt buộc 3 ban CN mỗi kỳ).");
    println!("  * **H7 (Giới hạn ra đề liên tiếp):** max_consecutive_setter = 2.");
    println!("- **Trạng thái kích hoạt và trọng số của toàn bộ quy tắc:**");
    println!("  * Quy tắc cứng: H1 (Chuyên môn) = ON, H2 (Khối lớp) = ON, H3 (Cơ sở) = ON (min_campuses tùy cấu hình), H4 (Giới hạn/kỳ) = ON, H5 (Trùng giờ) = ON, H6 (Bận) = ON, H7 (Liên tiếp) = ON.");
    println!("  * Quy tắc mềm: S1 = 10.0, S2 = 3.0, S3 = 4.0, S4 = 6.0, S5 = 6.0, S6 = 2.0, S7 = 1.0, S8 = 8.0, S9 = 5.0, S10 = 4.0 (Tất cả đều ON).");
    println!("- **Thông số tối ưu hoá:** Hạt giống base seed = 42 (dải seeds 42..49), Số luồng R = 8, Số bước lặp = 200,000 / luồng (tổng 1,600,000 bước lặp), Số phương án giữ lại K = 3.");
    println!("- **Cận dưới lý thuyết Σ LB:**");
    println!("  * S5 LB = 1.00 đơn vị (phạt: 6.00) [Nguyên lý Dirichlet trên CN: 12 ban đề / 11 GV phản biện]");
    println!("  * S6 LB = 2.00 đơn vị (phạt: 4.00) [Cân bằng lượt ra đề 24 lượt VL / 11 GV: 9 người 2 lượt, 2 người 3 lượt => 2 x (2*3-5) = 2.0]");
    println!("  * S8 LB = 2.5455 đơn vị (phạt: 20.36) [48 lượt / 11 GV, quota = 48/11: 4 người 5 việc, 7 người 4 việc => Σ(c-q)² = 28/11 ≈ 2.5455]");
    println!(
        "  * Các quy tắc mềm khác (S1, S2, S4, S7, S9, S10): Cận dưới = 0.00 đơn vị (phạt: 0.00)"
    );
    println!(
        "  * **Tổng cận dưới Σ LB (Không tính S3):** **{:.2}** (S5: 6.00 + S6: 4.00 + S8: 20.36)",
        sum_lb_penalties_no_s3
    );
    println!(
        "  * **Tổng cận dưới Σ LB (Tính S3 = 0.00 theo cận dưới độc lập):** **{:.2}**",
        sum_lb_penalties_with_s3_zero
    );
    println!(
        "  * **Tổng cận dưới Σ LB (Tính S3 đơn cơ sở = 36 đơn vị x 4.0 = 144.00):** **{:.2}**",
        sum_lb_penalties_with_s3_accounting
    );

    println!("\n### Bảng cấu hình giáo viên (Quota override & Max tasks override)");
    println!("| Mã GV | Tên đầy đủ | Cách gọi | Cơ sở | Hệ số tải | Quota Override | Max Tasks/Exam Override | Ghi chú |");
    println!("| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :--- |");
    for t in &problem_nocampus.teachers {
        println!(
            "| {:<5} | {:<16} | {:<8} | {:^5} | {:^9.1} | {:^14} | {:^23} | {:<15} |",
            t.code.as_deref().unwrap_or("-"),
            t.full_name,
            t.display_name.as_deref().unwrap_or("-"),
            t.campus_id.0,
            t.load_weight,
            t.quota_override
                .map(|q| q.to_string())
                .unwrap_or_else(|| "None".to_string()),
            t.max_tasks_per_exam_override
                .map(|m| m.to_string())
                .unwrap_or_else(|| "None".to_string()),
            if t.id.value() == 8 {
                "C Quí: max-tasks=3, quota=None"
            } else if t.id.value() == 12 {
                "T Nghĩa: quota=12, max-tasks=3"
            } else {
                "Mặc định"
            }
        );
    }

    println!("\n## 2. Nguồn dữ liệu kiểm định");
    if user_inputs_provided {
        println!("- Dữ liệu: Phân hiệu và phân công thực tế từ tổ bộ môn (đã được ẩn danh hoá theo quy định bảo mật).");
    } else {
        println!("- Dữ liệu đầu vào: Người dùng chọn **'none'** (chưa cung cấp số liệu thực tế phân hiệu cá nhân).");
        println!("- Thực hiện kiểm định chuẩn tắc trên bộ dữ liệu kiểm thử canonical Q fixture (`make_canonical_q_problem(QVariant::SyntheticCampuses)` và `QVariant::NoCampus`).");
        println!(
            "- Không có thông tin cá nhân thực tế nào được lưu trữ hoặc commit vào kho mã nguồn."
        );
    }

    println!("\n## 3. Kiểm định bảng phân công của Thầy Q");
    println!(
        "- Tổng số phân công: {} chỗ (12 ban đề Vật lí x 3 = 36; 12 ban đề Công nghệ x 2 = 24).",
        q_assigns.len()
    );
    println!(
        "- Điểm phạt mềm (nocampus): **{:.2}** (không tính S3: **{:.2}**).",
        q_report_nocampus.total,
        q_report_nocampus.total - q_s3_pen
    );
    println!(
        "- Điểm phạt mềm (synthetic campuses): **{:.2}**.",
        q_report_campuses.total
    );
    println!(
        "- Trạng thái thoả mãn ràng buộc cứng (Hard constraints): {}",
        if q_violations_campuses.is_empty() {
            "HOÀN TOÀN HỢP LỆ (0 vi phạm)"
        } else {
            "CÓ VI PHẠM RÀNG BUỘC CỨNG"
        }
    );

    // H3 panel inspection on synthetic campuses
    let mut h3_violating_panels = Vec::new();
    for exam in &problem_campuses.exams {
        for grade in &problem_campuses.grades {
            for subj in &problem_campuses.subjects {
                let panel_assigns: Vec<_> = q_assigns
                    .iter()
                    .filter(|a| {
                        a.exam_id == exam.id && a.grade_id == grade.id && a.subject_id == subj.id
                    })
                    .collect();
                let mut campuses_in_panel = std::collections::HashSet::new();
                for a in &panel_assigns {
                    if let Some(t) = problem_campuses
                        .teachers
                        .iter()
                        .find(|t| t.id == a.teacher_id)
                    {
                        campuses_in_panel.insert(t.campus_id);
                    }
                }
                if campuses_in_panel.len() < subj.min_campuses as usize {
                    let setter_names: Vec<String> = panel_assigns
                        .iter()
                        .filter(|a| a.role == Role::Setter)
                        .map(|a| {
                            let t = problem_campuses
                                .teachers
                                .iter()
                                .find(|t| t.id == a.teacher_id)
                                .unwrap();
                            let camp = problem_campuses
                                .campuses
                                .iter()
                                .find(|c| c.id == t.campus_id)
                                .unwrap();
                            format!(
                                "{} ({})",
                                t.display_name.as_deref().unwrap_or(&t.full_name),
                                camp.code
                            )
                        })
                        .collect();
                    let reviewer_names: Vec<String> = panel_assigns
                        .iter()
                        .filter(|a| a.role == Role::Reviewer)
                        .map(|a| {
                            let t = problem_campuses
                                .teachers
                                .iter()
                                .find(|t| t.id == a.teacher_id)
                                .unwrap();
                            let camp = problem_campuses
                                .campuses
                                .iter()
                                .find(|c| c.id == t.campus_id)
                                .unwrap();
                            format!(
                                "{} ({})",
                                t.display_name.as_deref().unwrap_or(&t.full_name),
                                camp.code
                            )
                        })
                        .collect();
                    h3_violating_panels.push((
                        exam.code.clone(),
                        grade.code,
                        subj.code.clone(),
                        campuses_in_panel.len(),
                        subj.min_campuses,
                        setter_names.join(", "),
                        reviewer_names.join(", "),
                    ));
                }
            }
        }
    }

    if h3_violating_panels.is_empty() {
        println!("- Kiểm tra H3 trên synthetic campuses: Tất cả 24 ban đề đều đạt số phân hiệu tối thiểu (>= 2).");
    } else {
        println!(
            "- Danh sách ban đề không đạt H3 trên synthetic campuses ({} ban đề):",
            h3_violating_panels.len()
        );
        println!("  | Kỳ thi | Khối | Môn | Phân hiệu thực tế | Yêu cầu | Ra đề (Cơ sở) | Phản biện (Cơ sở) | Lý do |");
        println!("  | :---: | :---: | :---: | :---: | :---: | :--- | :--- | :--- |");
        for (ex, gr, sub, act, req, setters, revs) in &h3_violating_panels {
            println!(
                "  | {} | {} | {} | {} | {} | {} | {} | Phản biện trùng cơ sở với người ra đề |",
                ex, gr, sub, act, req, setters, revs
            );
        }
    }

    println!("\n## 4. So sánh Tối ưu Hoá: Khởi động Lạnh (Cold) vs Khởi động Ấm (Warm)");
    println!("- Tham số: Seed cơ sở = 42, Số luồng R = 8 (hạt giống 42..49), Số bước lặp = 200,000 / luồng, K = 3.");
    println!("- Điểm khởi tạo của Warm Run (tính bởi cùng bộ đánh giá trên phương án của Thầy Q):");
    println!("  * Bao gồm S3: **{:.2}**", warm_initial_report.total);
    println!(
        "  * Không tính S3: **{:.2}** (chính xác bằng điểm phương án Q: **{:.2}**)",
        warm_initial_report.total - q_s3_pen,
        q_report_nocampus.total - q_s3_pen
    );
    println!(
        "- Điểm tối ưu nhất (Cold Start, winning seed {}): Có S3 = **{:.2}**, Không tính S3 = **{:.2}**",
        cold_plan.seed,
        cold_report.total,
        cold_report.total - cold_s3_pen
    );
    println!(
        "- Điểm tối ưu nhất (Warm Start, winning seed {}): Có S3 = **{:.2}**, Không tính S3 = **{:.2}**",
        warm_plan.seed,
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
        println!("- **Bằng chứng trùng khớp tuyệt đối:** Cả Cold Start và Warm Start đều hội tụ về điểm số và đơn vị vi phạm hoàn toàn đồng nhất trên tất cả 10 quy tắc mềm.\n");
    }

    println!("### Bảng so sánh chi tiết theo 10 quy tắc mềm (12 cột chuẩn)");
    println!("| Quy tắc | Trọng số | Cận dưới (LB) | Q Đơn vị | Q Phạt | Q Chênh | Cold Đơn vị | Cold Phạt | Cold Chênh | Warm Đơn vị | Warm Phạt | Warm Chênh |");
    println!("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |");

    let mut q_tot_with_s3 = 0.0;
    let mut q_tot_no_s3 = 0.0;
    let mut cold_tot_with_s3 = 0.0;
    let mut cold_tot_no_s3 = 0.0;
    let mut warm_tot_with_s3 = 0.0;
    let mut warm_tot_no_s3 = 0.0;

    for (key, default_w, label) in soft_rules {
        let lb = bounds_map.get(&key).copied().unwrap_or(0.0);
        let q_u = q_report_nocampus
            .by_rule
            .iter()
            .find(|r| r.rule == key)
            .map_or(0.0, |r| r.units);
        let q_w = q_report_nocampus
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
            "| {:<28} | {:>8.1} | {:>13.2} | {:>8.2} | {:>6.2} | {:>7.2} | {:>11.2} | {:>9.2} | {:>10.2} | {:>11.2} | {:>9.2} | {:>10.2} |",
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
        "| **Tổng cộng (Có S3)** | | | | **{:.2}** | | | **{:.2}** | | | **{:.2}** | |",
        q_tot_with_s3, cold_tot_with_s3, warm_tot_with_s3
    );
    println!(
        "| **Tổng cộng (Không S3)** | | | | **{:.2}** | | | **{:.2}** | | | **{:.2}** | |",
        q_tot_no_s3, cold_tot_no_s3, warm_tot_no_s3
    );

    println!("\n## 5. Thống kê phân công theo giáo viên (Phương án tối ưu tốt nhất)");
    println!("| Mã GV | Cách gọi | Phân hiệu | Chỉ tiêu q | Tổng lượt | Ra đề | Phản biện | GK1 | CK1 | GK2 | CK2 | Chênh lệch | Ghi chú |");
    println!("| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |");

    let best_assigns = &cold_plan.assignments;
    for t in &problem_campuses.teachers {
        let t_assigns: Vec<_> = best_assigns
            .iter()
            .filter(|a| a.teacher_id == t.id)
            .collect();
        let total = t_assigns.len();
        let setters = t_assigns.iter().filter(|a| a.role == Role::Setter).count();
        let reviewers = t_assigns
            .iter()
            .filter(|a| a.role == Role::Reviewer)
            .count();

        let gk1 = t_assigns
            .iter()
            .filter(|a| a.exam_id == problem_campuses.exams[0].id)
            .count();
        let ck1 = t_assigns
            .iter()
            .filter(|a| a.exam_id == problem_campuses.exams[1].id)
            .count();
        let gk2 = t_assigns
            .iter()
            .filter(|a| a.exam_id == problem_campuses.exams[2].id)
            .count();
        let ck2 = t_assigns
            .iter()
            .filter(|a| a.exam_id == problem_campuses.exams[3].id)
            .count();

        let camp_name = problem_campuses
            .campuses
            .iter()
            .find(|c| c.id == t.campus_id)
            .map(|c| c.code.as_str())
            .unwrap_or("PH1");
        let quota = t
            .quota_override
            .unwrap_or(if t.id.value() == 12 { 12 } else { 4 });
        let diff = total as i32 - quota as i32;
        let note = if t.id.value() == 12 {
            "Cố định CN"
        } else if t.max_tasks_per_exam_override == Some(3) {
            "Tối đa 3 việc"
        } else {
            ""
        };

        println!(
            "| {:<5} | {:<8} | {:^9} | {:^10} | {:^9} | {:^5} | {:^9} | {:^3} | {:^3} | {:^3} | {:^3} | {:^10} | {:<12} |",
            t.code.as_deref().unwrap_or(""),
            t.display_name.as_deref().unwrap_or(&t.full_name),
            camp_name,
            quota,
            total,
            setters,
            reviewers,
            gk1,
            ck1,
            gk2,
            ck2,
            if diff >= 0 { format!("+{diff}") } else { diff.to_string() },
            note
        );
    }

    println!("\n## 6. Bảng phân công phương án tối ưu (Dạng bảng tổ - Q-Style Grid)");
    println!("| Kì thi | Vai trò | Khối 10 (VL) | Khối 10 (CN) | Khối 11 (VL) | Khối 11 (CN) | Khối 12 (VL) | Khối 12 (CN) |");
    println!("| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |");

    let get_cell =
        |exam_idx: usize, grade_code: i32, subj_code: &str, role: Role, pos: usize| -> String {
            let exam_id = problem_campuses.exams[exam_idx].id;
            let grade_id = problem_campuses
                .grades
                .iter()
                .find(|g| g.code == grade_code)
                .unwrap()
                .id;
            let subj_id = problem_campuses
                .subjects
                .iter()
                .find(|s| s.code == subj_code)
                .unwrap()
                .id;

            let matching: Vec<_> = best_assigns
                .iter()
                .filter(|a| {
                    a.exam_id == exam_id
                        && a.grade_id == grade_id
                        && a.subject_id == subj_id
                        && a.role == role
                })
                .collect();

            if let Some(a) = matching.get(pos) {
                let t = problem_campuses
                    .teachers
                    .iter()
                    .find(|t| t.id == a.teacher_id)
                    .unwrap();
                t.display_name
                    .clone()
                    .unwrap_or_else(|| t.full_name.clone())
            } else {
                "-".to_string()
            }
        };

    for (exam_idx, exam) in problem_campuses.exams.iter().enumerate() {
        println!(
            "| **{}** | Đề 1 | {} | {} | {} | {} | {} | {} |",
            exam.code,
            get_cell(exam_idx, 10, "VL", Role::Setter, 0),
            get_cell(exam_idx, 10, "CN", Role::Setter, 0),
            get_cell(exam_idx, 11, "VL", Role::Setter, 0),
            get_cell(exam_idx, 11, "CN", Role::Setter, 0),
            get_cell(exam_idx, 12, "VL", Role::Setter, 0),
            get_cell(exam_idx, 12, "CN", Role::Setter, 0),
        );
        println!(
            "| | Đề 2 | {} | | {} | | {} | |",
            get_cell(exam_idx, 10, "VL", Role::Setter, 1),
            get_cell(exam_idx, 11, "VL", Role::Setter, 1),
            get_cell(exam_idx, 12, "VL", Role::Setter, 1),
        );
        println!(
            "| | P.Biện | {} | {} | {} | {} | {} | {} |",
            get_cell(exam_idx, 10, "VL", Role::Reviewer, 0),
            get_cell(exam_idx, 10, "CN", Role::Reviewer, 0),
            get_cell(exam_idx, 11, "VL", Role::Reviewer, 0),
            get_cell(exam_idx, 11, "CN", Role::Reviewer, 0),
            get_cell(exam_idx, 12, "VL", Role::Reviewer, 0),
            get_cell(exam_idx, 12, "CN", Role::Reviewer, 0),
        );
    }

    println!("\n## 7. Đánh giá Preset 'Cho phép dồn việc trong một kỳ' (Tắt S9)");
    println!(
        "- **Mục đích:** Kiểm tra khả năng đạt 0 điểm phạt mềm khi cho phép dồn việc (S9 = 0)."
    );
    let s9_s3_pen = s9_report
        .by_rule
        .iter()
        .find(|r| r.rule == RuleKey::S3)
        .map_or(0.0, |r| r.penalty);
    println!(
        "- Tổng điểm phạt với S9 tắt (nocampus): Có S3 = **{:.2}**, Không tính S3 = **{:.2}**",
        s9_report.total,
        s9_report.total - s9_s3_pen
    );
    println!("- Chi tiết các quy tắc còn điểm phạt:");
    for r in &s9_report.by_rule {
        if r.rule != RuleKey::S3 && r.penalty > 0.0 {
            println!(
                "  * **{:?}**: {:.2} đơn vị vi phạm (Phạt: {:.2})",
                r.rule, r.units, r.penalty
            );
        }
    }
    println!(
        "- **Kết luận:** Khi tắt S9, optimizer có đạt 0 điểm phạt mềm không? -> {}",
        if (s9_report.total - s9_s3_pen) < 1e-4 {
            "**CÓ (Đạt 0.00 điểm phạt mềm)**"
        } else {
            "**KHÔNG (Vẫn còn vi phạm cận dưới không thể giảm thêm của S5 và S6)**"
        }
    );
    println!("  * Giải thích: Do S5 có cận dưới lý thuyết >= 1.0 (nguyên lý Dirichlet trên môn Công nghệ) và S6 có cận dưới lý thuyết = 2.0 (bài toán cân bằng lượt ra đề 24 lượt / 11 GV), nên ngay cả khi dồn việc tự do, tổng điểm phạt vẫn không thể đạt 0 tuyệt đối.");

    println!("\n## 8. Giải trình cơ sở toán học của các cận dưới (Lower Bounds)");
    println!("1. **Cận dưới S6 = 2.0:** Trong 4 kỳ thi, có 24 lượt giáo viên ra đề không bắt buộc (12 ban đề VL x 2) cần phân bổ cho 11 giáo viên đủ điều kiện (k_t <= 4, Thầy Nghĩa được loại trừ do 12 ghế ra đề CN của thầy là bắt buộc cố định). Để cực tiểu hoá tổng hàm lồi f(k_t) với f(k) = max(0, 2k - 5), thuật toán tham lam cân bằng tối ưu phân bổ 9 giáo viên nhận 2 lượt ra đề (f = 0) và 2 giáo viên nhận 3 lượt ra đề (f = 1 mỗi người), xác lập cận dưới chính xác tuyệt đối là 2.0 đơn vị.");
    println!("2. **Cận dưới S10 = 0.0:** Cả 11 giáo viên (GV001..GV011) đều có chuyên môn phản biện cho cả 2 môn Vật lí (VL) và Công nghệ (CN). Toàn bộ năm học có 12 ghế phản biện VL và 12 ghế phản biện CN. Vì 11 <= 12 ở cả hai môn, về mặt toán học hoàn toàn tồn tại phân công để mỗi giáo viên đều được phản biện mỗi môn ít nhất 1 lần.");
    println!("3. **Cận dưới S5 >= 1.0:** Ở môn Công nghệ, toàn bộ 12 ban đề đều do Thầy Nghĩa ra đề độc quyền, và chỉ có 11 giáo viên khác có thể làm phản biện. Theo nguyên lý Dirichlet (Pigeonhole Principle), phân phối 12 lượt phản biện cho 11 người chắc chắn sẽ có ít nhất một giáo viên phải phản biện cho Thầy Nghĩa ít nhất ceil(12/11) = 2 lần. Mỗi lượt phản biện lặp lại tạo ra max(0, count - 1) >= 1 đơn vị vi phạm. Do đó S5 >= 1.0.");
    println!("4. **Giải trình S3 trên mô hình đơn cơ sở (nocampus) = 36.0 đơn vị:** Trên mô hình nocampus, tất cả giáo viên cùng thuộc Cơ sở 1. Đối với mỗi ban đề Vật lí (2 người ra đề, 1 người phản biện), người phản biện trùng cơ sở với 2 người ra đề, tạo ra 12 x 2 = 24 đơn vị vi phạm. Đối với mỗi ban đề Công nghệ (1 người ra đề, 1 người phản biện), người phản biện trùng cơ sở với 1 người ra đề, tạo ra 12 x 1 = 12 đơn vị vi phạm. Tổng số đơn vị vi phạm S3 trên nocampus = 24 + 12 = 36.0 đơn vị.");

    Ok(())
}
