use std::fs;
use std::path::Path;

use exam_panel_core::domain::{
    Assignment, Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, PlanId,
    Problem, Role, RuleKey, RuleSetting, SchoolYear, SchoolYearId, Subject, SubjectId, Teacher,
    TeacherGrade, TeacherId,
};
use exam_panel_core::feasibility::check_feasibility;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::evaluate;
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use serde_json::json;

fn make_canonical_problem(h3_enabled: bool) -> Problem {
    let school_year = SchoolYear {
        id: SchoolYearId(1),
        name: "2026-2027".to_string(),
        is_current: true,
    };

    let campuses = vec![
        Campus {
            id: CampusId(1),
            code: "PH1".to_string(),
            name: "Phân hiệu 1".to_string(),
            color: "palette-1".to_string(),
        },
        Campus {
            id: CampusId(2),
            code: "PH2".to_string(),
            name: "Phân hiệu 2".to_string(),
            color: "palette-2".to_string(),
        },
    ];

    let grades = vec![
        Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        },
        Grade {
            id: GradeId(2),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        },
        Grade {
            id: GradeId(3),
            code: 12,
            name: "Khối 12".to_string(),
            sort_order: 3,
        },
    ];

    let subjects = vec![
        Subject {
            id: SubjectId(1),
            code: "VL".to_string(),
            name: "Vật lí".to_string(),
            color: "palette-1".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        },
        Subject {
            id: SubjectId(2),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "palette-2".to_string(),
            sort_order: 2,
            setters: 1,
            reviewers: 1,
            min_campuses: 2,
        },
    ];

    let exams = vec![
        Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        },
        Exam {
            id: ExamId(2),
            school_year_id: SchoolYearId(1),
            code: "CK1".to_string(),
            name: "Cuối kỳ 1".to_string(),
            sort_order: 2,
        },
        Exam {
            id: ExamId(3),
            school_year_id: SchoolYearId(1),
            code: "GK2".to_string(),
            name: "Giữa kỳ 2".to_string(),
            sort_order: 3,
        },
        Exam {
            id: ExamId(4),
            school_year_id: SchoolYearId(1),
            code: "CK2".to_string(),
            name: "Cuối kỳ 2".to_string(),
            sort_order: 4,
        },
    ];

    let teachers = vec![
        Teacher {
            id: TeacherId(1),
            full_name: "Cô Hiền".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("HIEN".to_string()),
            display_name: Some("C Hiền".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(2),
            full_name: "Cô Lài".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("LAI".to_string()),
            display_name: Some("C Lài".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(3),
            full_name: "Thầy Phúc".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("PHUC".to_string()),
            display_name: Some("T Phúc".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(4),
            full_name: "Thầy Lộc".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("LOC".to_string()),
            display_name: Some("T Lộc".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(5),
            full_name: "Cô Thư".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("THU".to_string()),
            display_name: Some("C Thư".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(6),
            full_name: "Cô Na".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("NA".to_string()),
            display_name: Some("C Na".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(7),
            full_name: "Cô Bình".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("BINH".to_string()),
            display_name: Some("C Bình".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(8),
            full_name: "Cô Quí".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("QUI".to_string()),
            display_name: Some("C Quí".to_string()),
            quota_override: Some(6),
            max_tasks_per_exam_override: Some(3),
        },
        Teacher {
            id: TeacherId(9),
            full_name: "Cô Tú".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("TU".to_string()),
            display_name: Some("C Tú".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(10),
            full_name: "Cô Như".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("NHU".to_string()),
            display_name: Some("C Như".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(11),
            full_name: "Cô Lan".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("LAN".to_string()),
            display_name: Some("C Lan".to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        },
        Teacher {
            id: TeacherId(12),
            full_name: "Thầy Nghĩa".to_string(),
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some("NGHIA".to_string()),
            display_name: Some("T Nghĩa".to_string()),
            quota_override: Some(12),
            max_tasks_per_exam_override: Some(3),
        },
    ];

    let teacher_grades_spec = vec![
        (1, vec![1, 2, 3]),
        (2, vec![1, 2, 3]),
        (3, vec![1, 3]),
        (4, vec![1, 2]),
        (5, vec![2]),
        (6, vec![1, 2]),
        (7, vec![2, 3]),
        (8, vec![2, 3]),
        (9, vec![1]),
        (10, vec![1]),
        (11, vec![3]),
        (12, vec![1, 2, 3]),
    ];

    let mut teacher_grades = Vec::new();
    for (tid, gids) in teacher_grades_spec {
        for gid in gids {
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: SchoolYearId(1),
                grade_id: GradeId(gid),
            });
        }
    }

    let mut competencies = Vec::new();
    for tid in 1..=11 {
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(1),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        });
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(1),
            role: Role::Reviewer,
            grade_scope: GradeScope::Taught,
        });
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(2),
            role: Role::Reviewer,
            grade_scope: GradeScope::Any,
        });
    }
    competencies.push(Competency {
        teacher_id: TeacherId(12),
        subject_id: SubjectId(2),
        role: Role::Setter,
        grade_scope: GradeScope::Any,
    });

    let mut rule_settings = RuleSetting::default_settings();
    if let Some(h3) = rule_settings.iter_mut().find(|r| r.key == RuleKey::H3) {
        h3.enabled = h3_enabled;
        h3.params = json!({ "enabled": h3_enabled });
    }

    Problem {
        school_year,
        campuses,
        grades,
        subjects,
        teachers,
        teacher_grades,
        competencies,
        exams,
        unavailabilities: Vec::new(),
        locks: Vec::new(),
        rule_settings,
    }
}

fn make_q_assignments() -> Vec<Assignment> {
    let mut a = Vec::with_capacity(60);
    let mut add = |e: i64, g: i64, s: i64, r: Role, pos: usize, tid: i64| {
        a.push(Assignment {
            plan_id: PlanId(1),
            exam_id: ExamId(e),
            grade_id: GradeId(g),
            subject_id: SubjectId(s),
            teacher_id: TeacherId(tid),
            role: r,
            position: pos,
        });
    };

    // GK1 (e = 1)
    // 10 (g = 1): VL [1, 2 | 3], CN [12 | 1]
    add(1, 1, 1, Role::Setter, 0, 1);
    add(1, 1, 1, Role::Setter, 1, 2);
    add(1, 1, 1, Role::Reviewer, 0, 3);
    add(1, 1, 2, Role::Setter, 0, 12);
    add(1, 1, 2, Role::Reviewer, 0, 1);
    // 11 (g = 2): VL [4, 5 | 6], CN [12 | 2]
    add(1, 2, 1, Role::Setter, 0, 4);
    add(1, 2, 1, Role::Setter, 1, 5);
    add(1, 2, 1, Role::Reviewer, 0, 6);
    add(1, 2, 2, Role::Setter, 0, 12);
    add(1, 2, 2, Role::Reviewer, 0, 2);
    // 12 (g = 3): VL [7, 3 | 8], CN [12 | 4]
    add(1, 3, 1, Role::Setter, 0, 7);
    add(1, 3, 1, Role::Setter, 1, 3);
    add(1, 3, 1, Role::Reviewer, 0, 8);
    add(1, 3, 2, Role::Setter, 0, 12);
    add(1, 3, 2, Role::Reviewer, 0, 4);

    // CK1 (e = 2)
    // 10: VL [1, 9 | 10], CN [12 | 9]
    add(2, 1, 1, Role::Setter, 0, 1);
    add(2, 1, 1, Role::Setter, 1, 9);
    add(2, 1, 1, Role::Reviewer, 0, 10);
    add(2, 1, 2, Role::Setter, 0, 12);
    add(2, 1, 2, Role::Reviewer, 0, 9);
    // 11: VL [2, 7 | 5], CN [12 | 11]
    add(2, 2, 1, Role::Setter, 0, 2);
    add(2, 2, 1, Role::Setter, 1, 7);
    add(2, 2, 1, Role::Reviewer, 0, 5);
    add(2, 2, 2, Role::Setter, 0, 12);
    add(2, 2, 2, Role::Reviewer, 0, 11);
    // 12: VL [11, 8 | 2], CN [12 | 7]
    add(2, 3, 1, Role::Setter, 0, 11);
    add(2, 3, 1, Role::Setter, 1, 8);
    add(2, 3, 1, Role::Reviewer, 0, 2);
    add(2, 3, 2, Role::Setter, 0, 12);
    add(2, 3, 2, Role::Reviewer, 0, 7);

    // GK2 (e = 3)
    // 10: VL [9, 10 | 4], CN [12 | 10]
    add(3, 1, 1, Role::Setter, 0, 9);
    add(3, 1, 1, Role::Setter, 1, 10);
    add(3, 1, 1, Role::Reviewer, 0, 4);
    add(3, 1, 2, Role::Setter, 0, 12);
    add(3, 1, 2, Role::Reviewer, 0, 10);
    // 11: VL [6, 5 | 1], CN [12 | 5]
    add(3, 2, 1, Role::Setter, 0, 6);
    add(3, 2, 1, Role::Setter, 1, 5);
    add(3, 2, 1, Role::Reviewer, 0, 1);
    add(3, 2, 2, Role::Setter, 0, 12);
    add(3, 2, 2, Role::Reviewer, 0, 5);
    // 12: VL [11, 2 | 8], CN [12 | 3]
    add(3, 3, 1, Role::Setter, 0, 11);
    add(3, 3, 1, Role::Setter, 1, 2);
    add(3, 3, 1, Role::Reviewer, 0, 8);
    add(3, 3, 2, Role::Setter, 0, 12);
    add(3, 3, 2, Role::Reviewer, 0, 3);

    // CK2 (e = 4)
    // 10: VL [6, 10 | 9], CN [12 | 8]
    add(4, 1, 1, Role::Setter, 0, 6);
    add(4, 1, 1, Role::Setter, 1, 10);
    add(4, 1, 1, Role::Reviewer, 0, 9);
    add(4, 1, 2, Role::Setter, 0, 12);
    add(4, 1, 2, Role::Reviewer, 0, 8);
    // 11: VL [4, 8 | 7], CN [12 | 6]
    add(4, 2, 1, Role::Setter, 0, 4);
    add(4, 2, 1, Role::Setter, 1, 8);
    add(4, 2, 1, Role::Reviewer, 0, 7);
    add(4, 2, 2, Role::Setter, 0, 12);
    add(4, 2, 2, Role::Reviewer, 0, 6);
    // 12: VL [3, 1 | 11], CN [12 | 8]
    add(4, 3, 1, Role::Setter, 0, 3);
    add(4, 3, 1, Role::Setter, 1, 1);
    add(4, 3, 1, Role::Reviewer, 0, 11);
    add(4, 3, 2, Role::Setter, 0, 12);
    add(4, 3, 2, Role::Reviewer, 0, 8);

    a
}

fn main() {
    println!("=== Phase 11 Report Generation ===");

    // 1. Build Canonical Problem (with H3 disabled for Q table validation, since Q has no campus data)
    let problem_q = make_canonical_problem(false);
    let q_assignments = make_q_assignments();

    println!("Checking feasibility of canonical problem...");
    let feas = check_feasibility(&problem_q);
    println!(
        "Feasibility: feasible={}, errors={}, warnings={}",
        feas.is_feasible,
        feas.errors.len(),
        feas.warnings.len()
    );

    // 2. Evaluate Q's Table against hard constraints (H1-H7 with H3 disabled)
    println!("Evaluating Q's real manual table against hard rules...");
    let hard_violations_q =
        validate_assignments(&problem_q, &q_assignments, &ValidateOptions::default());
    println!(
        "Q Table Hard Violations (H3 off): {}",
        hard_violations_q.len()
    );
    for hv in &hard_violations_q {
        println!("  Hard violation: {:?} code={}", hv.rule, hv.code);
    }

    // Soft evaluation
    let q_score = evaluate(&problem_q, &q_assignments);
    println!("Q Table Total Soft Penalty: {:.2}", q_score.total);
    for r in &q_score.by_rule {
        println!(
            "  Rule {:?}: units={:.2}, penalty={:.2}, bound={:.2}",
            r.rule, r.units, r.penalty, r.lower_bound
        );
    }

    // Check invariants
    let nghia_tasks = q_assignments
        .iter()
        .filter(|a| a.teacher_id == TeacherId(12))
        .count();
    let nghia_reviews = q_assignments
        .iter()
        .filter(|a| a.teacher_id == TeacherId(12) && a.role == Role::Reviewer)
        .count();
    println!(
        "T Nghĩa: total tasks={}, reviews={}",
        nghia_tasks, nghia_reviews
    );

    let qui_reviews = q_assignments
        .iter()
        .filter(|a| a.teacher_id == TeacherId(8) && a.role == Role::Reviewer)
        .count();
    println!("C Quí: reviews={}", qui_reviews);

    // 3. Evaluate Q's Table with H3 enabled to report campus collision
    let problem_h3 = make_canonical_problem(true);
    let hard_violations_h3 =
        validate_assignments(&problem_h3, &q_assignments, &ValidateOptions::default());
    println!(
        "Q Table with H3 enabled: {} hard violations (expected campus collisions)",
        hard_violations_h3.len()
    );

    // 4. Run Optimizer Benchmark
    println!("Running Optimizer Benchmark on Canonical Problem (with H3 enabled)...");
    let opts_h3 = OptimizeOptions {
        base_seed: 42,
        budget: Budget::Iterations(50_000),
        num_runs: 8,
        max_plans: 3,
        diversity_threshold: 0.20,
        cancel: None,
        progress: None,
        initial_assignments: None,
    };
    let opt_outcome = optimize(&problem_h3, &opts_h3).expect("Optimizer failed");
    let best_plan = &opt_outcome.plans[0];
    println!("Optimizer Best Score: {:.2}", best_plan.report.total);
    println!(
        "Optimizer Stats: runs={}, iterations={}, elapsed={}ms",
        opt_outcome.stats.total_runs,
        opt_outcome.stats.total_iterations,
        opt_outcome.stats.elapsed_ms
    );
    for r in &best_plan.report.by_rule {
        println!(
            "  Rule {:?}: units={:.2}, penalty={:.2}, bound={:.2}",
            r.rule, r.units, r.penalty, r.lower_bound
        );
    }

    let opt_hard_violations = validate_assignments(
        &problem_h3,
        &best_plan.assignments,
        &ValidateOptions::default(),
    );
    println!("Optimizer Hard Violations: {}", opt_hard_violations.len());

    // 5. Run Optimizer Benchmark on Canonical Problem with H3 disabled (fair comparison with Q's table)
    println!("Running Optimizer Benchmark with H3 disabled (same rules as Q's table)...");
    let opts_no_h3 = OptimizeOptions {
        base_seed: 42,
        budget: Budget::Iterations(50_000),
        num_runs: 8,
        max_plans: 3,
        diversity_threshold: 0.20,
        cancel: None,
        progress: None,
        initial_assignments: None,
    };
    let opt_outcome_no_h3 = optimize(&problem_q, &opts_no_h3).expect("Optimizer failed");
    let best_plan_no_h3 = &opt_outcome_no_h3.plans[0];
    println!(
        "Optimizer (No H3) Best Score: {:.2}",
        best_plan_no_h3.report.total
    );

    // 6. Write Reports
    let out_dir = Path::new("docs/reports/phase-11");
    fs::create_dir_all(out_dir).expect("Failed to create docs/reports/phase-11");

    let q_table_report = json!({
        "dataset": "Q real manual table (canonical problem, 12 teachers, 24 panels, 60 seats)",
        "feasibility": {
            "is_feasible": feas.is_feasible,
            "errors_count": feas.errors.len(),
            "warnings_count": feas.warnings.len(),
            "quotas": feas.quotas
        },
        "q_eval_no_h3": {
            "hard_violations_count": hard_violations_q.len(),
            "hard_violations": hard_violations_q,
            "score": q_score,
        },
        "q_eval_with_h3": {
            "hard_violations_count": hard_violations_h3.len(),
            "hard_violations": hard_violations_h3,
        },
        "invariants_checked": {
            "total_seats": q_assignments.len(),
            "t_nghia_tasks": nghia_tasks,
            "t_nghia_reviews": nghia_reviews,
            "c_qui_reviews": qui_reviews,
        },
        "assignments": q_assignments
    });

    let opt_report = json!({
        "canonical_benchmark_with_h3": {
            "stats": opt_outcome.stats,
            "best_score": best_plan.report.total,
            "report": best_plan.report,
            "hard_violations_count": opt_hard_violations.len(),
            "assignments": best_plan.assignments
        },
        "canonical_benchmark_without_h3": {
            "stats": opt_outcome_no_h3.stats,
            "best_score": best_plan_no_h3.report.total,
            "report": best_plan_no_h3.report,
            "assignments": best_plan_no_h3.assignments
        }
    });

    fs::write(
        out_dir.join("q_table_eval.json"),
        serde_json::to_string_pretty(&q_table_report).unwrap(),
    )
    .expect("write q_table_eval.json");

    fs::write(
        out_dir.join("optimizer_benchmark.json"),
        serde_json::to_string_pretty(&opt_report).unwrap(),
    )
    .expect("write optimizer_benchmark.json");

    // Also write a markdown summary
    let summary_md = format!(
        r#"# Phase 11 Validation Report: Q's Real Manual Table and Optimizer Benchmark

## 1. Canonical Dataset Overview
- **Teachers**: 12 (11 Vật lí teachers + Thầy Nghĩa)
- **Campuses**: 2 (Phân hiệu 1, Phân hiệu 2)
- **Subjects**: 2
  - **VL (Vật lí)**: 2 setters + 1 reviewer, min 2 campuses
  - **CN (Công nghệ)**: 1 setter + 1 reviewer, min 2 campuses
- **Exams**: 4 terms (GK1, CK1, GK2, CK2)
- **Grades**: 3 (Khối 10, 11, 12)
- **Panels**: 4 exams × 3 grades × 2 subjects = 24 panels
- **Seats**: (12 VL × 3) + (12 CN × 2) = 36 + 24 = 60 seats

## 2. Invariants & Q's Manual Table Validation
- **Total seats placed**: {}
- **Thầy Nghĩa**: {} tasks (all CN setter), {} reviews (0)
- **Cô Quí**: {} reviews (2 VL + 2 CN)
- **Hard Violations under Q's process (H3 disabled)**: **{} violations** (Feasible: {})
- **Hard Violations with H3 enabled**: {} (Expected: Q's table did not record campuses; because Thầy Nghĩa is the sole CN setter and all 11 VL teachers review CN, CN multi-campus requires VL teachers on Campus 1, which conflicts with VL internal multi-campus).

## 3. Q's Table Soft Score vs Optimizer Benchmark
- **Q's Table Total Soft Penalty (No H3)**: {:.2}
- **Optimizer Best Total Soft Penalty (No H3)**: {:.2}
- **Optimizer Best Total Soft Penalty (With H3)**: {:.2}
- **Optimizer Runs / Iterations**: {} runs, {} total iterations
- **Optimizer Runtime**: {} ms
"#,
        q_assignments.len(),
        nghia_tasks,
        nghia_reviews,
        qui_reviews,
        hard_violations_q.len(),
        feas.is_feasible,
        hard_violations_h3.len(),
        q_score.total,
        best_plan_no_h3.report.total,
        best_plan.report.total,
        opt_outcome.stats.total_runs,
        opt_outcome.stats.total_iterations,
        opt_outcome.stats.elapsed_ms
    );

    fs::write(out_dir.join("summary.md"), summary_md).expect("write summary.md");

    println!("Reports written successfully to docs/reports/phase-11/");
}
