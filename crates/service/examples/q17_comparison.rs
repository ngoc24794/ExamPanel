//! Q17 comparison: 17 simulated teachers, 4 campuses, 60 panel seats.
//!
//! This example is self-contained and does not touch the product binaries:
//! - builds a synthetic 17-teacher problem (10 Vật lí + 7 Công nghệ, 4 phân hiệu),
//! - builds ONE deterministic round-robin manual plan (program-generated, not a human plan),
//! - evaluates that manual plan (soft score + hard violations + crowding),
//! - runs the cold-start optimizer (seed 42, R = 8, 200,000 iterations per run),
//! - writes a Markdown report and JSON artifacts into the requested directory.
//!
//! All teacher names are fake (GV01..GV17). No real personal data is used.
//!
//! Usage:
//!   cargo run -p exam-panel-service --example q17_comparison -- [output_dir]
//!
//! The default output directory is `docs/reports/edu-2026/q17`.

use exam_panel_core::domain::{
    calculate_quotas, Assignment, Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId,
    GradeScope, PanelKey, Problem, Role, RuleKey, RuleSetting, SchoolYear, SchoolYearId, Subject,
    SubjectId, Teacher, TeacherGrade, TeacherId,
};
use exam_panel_core::feasibility::check_feasibility;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::score::bounds::lower_bounds;
use exam_panel_core::score::{evaluate, ScoreReport};
use exam_panel_core::validate::{validate_assignments, ValidateOptions, Violation};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const VL: i64 = 1;
const CN: i64 = 2;

const BASE_SEED: u64 = 42;
const NUM_RUNS: usize = 8;
const ITERATIONS: u64 = 200_000;

/// Teacher code, campus id, subject id. Names are intentionally fake.
fn teacher_specs() -> Vec<(i64, &'static str, i64, i64)> {
    vec![
        (1, "GV01", VL, 1),
        (2, "GV02", VL, 2),
        (3, "GV03", VL, 3),
        (4, "GV04", VL, 4),
        (5, "GV05", VL, 1),
        (6, "GV06", VL, 2),
        (7, "GV07", VL, 3),
        (8, "GV08", VL, 4),
        (9, "GV09", VL, 1),
        (10, "GV10", VL, 2),
        (11, "GV11", CN, 3),
        (12, "GV12", CN, 4),
        (13, "GV13", CN, 1),
        (14, "GV14", CN, 2),
        (15, "GV15", CN, 3),
        (16, "GV16", CN, 4),
        (17, "GV17", CN, 2),
    ]
}

/// Builds the synthetic 17-teacher problem.
fn make_q17_problem() -> Problem {
    let school_year = SchoolYear {
        id: SchoolYearId(1),
        name: "2026-2027".to_string(),
        is_current: true,
    };

    let campuses = (1..=4)
        .map(|c| Campus {
            id: CampusId(c),
            code: format!("PH{c}"),
            name: format!("Phân hiệu {c} (mô phỏng)"),
            color: format!("palette-{c}"),
        })
        .collect();

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
            id: SubjectId(VL),
            code: "VL".to_string(),
            name: "Vật lí".to_string(),
            color: "palette-1".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        },
        Subject {
            id: SubjectId(CN),
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

    let teachers: Vec<Teacher> = teacher_specs()
        .into_iter()
        .map(|(id, code, _subject, campus)| Teacher {
            id: TeacherId(id),
            full_name: format!("Giáo viên mô phỏng {code}"),
            campus_id: CampusId(campus),
            load_weight: 1.0,
            active: true,
            note: Some("Dữ liệu mô phỏng".to_string()),
            code: Some(code.to_string()),
            display_name: Some(code.to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .collect();

    let mut teacher_grades = Vec::new();
    for t in &teachers {
        for g in 1..=3 {
            teacher_grades.push(TeacherGrade {
                teacher_id: t.id,
                school_year_id: SchoolYearId(1),
                grade_id: GradeId(g),
            });
        }
    }

    let mut competencies = Vec::new();
    for (id, _code, subject, _campus) in teacher_specs() {
        competencies.push(Competency {
            teacher_id: TeacherId(id),
            subject_id: SubjectId(subject),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        });
        competencies.push(Competency {
            teacher_id: TeacherId(id),
            subject_id: SubjectId(subject),
            role: Role::Reviewer,
            grade_scope: GradeScope::Taught,
        });
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
        rule_settings: RuleSetting::default_settings(),
    }
}

/// One deterministic round-robin manual plan.
///
/// Vật lí roster = GV01..GV10 in code order, Công nghệ roster = GV11..GV17 in code order.
/// For exam index e (0-based):
/// - VL: cursor = (9 * e) mod 10; fill 9 seats in panel order
///   (K10 setter 1, K10 setter 2, K10 reviewer, K11 setter 1, ..., K12 reviewer).
/// - CN: cursor = (6 * e) mod 7; fill 6 seats in panel order
///   (K10 setter, K10 reviewer, K11 setter, K11 reviewer, K12 setter, K12 reviewer).
///
/// The cursor keeps moving forward; the procedure never checks soft rules and never
/// calls the solver. It is a program-generated reference, not a human-made plan.
fn make_manual_plan(problem: &Problem) -> Vec<Assignment> {
    let roster = |subject: i64| -> Vec<TeacherId> {
        let mut ids: Vec<(String, TeacherId)> = problem
            .teachers
            .iter()
            .filter(|t| {
                problem
                    .competencies
                    .iter()
                    .any(|c| c.teacher_id == t.id && c.subject_id == SubjectId(subject))
            })
            .map(|t| (t.code.clone().unwrap_or_default(), t.id))
            .collect();
        ids.sort();
        ids.into_iter().map(|(_, id)| id).collect()
    };

    let vl = roster(VL);
    let cn = roster(CN);
    let mut assignments = Vec::with_capacity(60);

    for exam_idx in 0..problem.exams.len() {
        let exam = &problem.exams[exam_idx];

        let vl_start = (exam_idx * 9) % vl.len();
        for local_pos in 0..9 {
            let teacher = vl[(vl_start + local_pos) % vl.len()];
            let grade_id = GradeId((local_pos / 3 + 1) as i64);
            let within = local_pos % 3;
            let (role, position) = if within < 2 {
                (Role::Setter, within)
            } else {
                (Role::Reviewer, 0)
            };
            assignments.push(Assignment::new(
                exam.id,
                grade_id,
                SubjectId(VL),
                teacher,
                role,
                position,
            ));
        }

        let cn_start = (exam_idx * 6) % cn.len();
        for local_pos in 0..6 {
            let teacher = cn[(cn_start + local_pos) % cn.len()];
            let grade_id = GradeId((local_pos / 2 + 1) as i64);
            let within = local_pos % 2;
            let (role, position) = if within == 0 {
                (Role::Setter, 0)
            } else {
                (Role::Reviewer, 0)
            };
            assignments.push(Assignment::new(
                exam.id,
                grade_id,
                SubjectId(CN),
                teacher,
                role,
                position,
            ));
        }
    }

    assignments
}

fn rule_label(key: RuleKey) -> &'static str {
    match key {
        RuleKey::S1 => "S1 Tần suất phản biện",
        RuleKey::S2 => "S2 Cân bằng vai trò",
        RuleKey::S3 => "S3 Độc lập cơ sở",
        RuleKey::S4 => "S4 Đa dạng cặp ra đề",
        RuleKey::S5 => "S5 Đa dạng quan hệ phản biện",
        RuleKey::S6 => "S6 Ra đề liên tiếp",
        RuleKey::S7 => "S7 Xoay vòng khối",
        RuleKey::S8 => "S8 Cân bằng khối lượng",
        RuleKey::S9 => "S9 Dồn việc trong kỳ",
        RuleKey::S10 => "S10 Thiếu môn phản biện",
        _ => "?",
    }
}

fn rule_of(report: &ScoreReport, key: RuleKey) -> Option<&exam_panel_core::score::RuleScore> {
    report.by_rule.iter().find(|r| r.rule == key)
}

fn rule_units(report: &ScoreReport, key: RuleKey) -> f64 {
    rule_of(report, key).map_or(0.0, |r| r.units)
}

fn rule_penalty(report: &ScoreReport, key: RuleKey) -> f64 {
    rule_of(report, key).map_or(0.0, |r| r.penalty)
}

fn rule_weight(report: &ScoreReport, key: RuleKey) -> f64 {
    rule_of(report, key).map_or(0.0, |r| r.weight)
}

fn teacher_code(problem: &Problem, id: TeacherId) -> String {
    problem
        .teachers
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.code.clone().unwrap_or_else(|| t.full_name.clone()))
        .unwrap_or_else(|| format!("?{}", id.0))
}

fn campus_code(problem: &Problem, id: CampusId) -> String {
    problem
        .campuses
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.code.clone())
        .unwrap_or_else(|| format!("?{}", id.0))
}

fn teacher_subject(problem: &Problem, id: TeacherId) -> String {
    problem
        .competencies
        .iter()
        .find(|c| c.teacher_id == id)
        .and_then(|c| problem.subjects.iter().find(|s| s.id == c.subject_id))
        .map(|s| s.code.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn panel_label(problem: &Problem, panel: PanelKey) -> String {
    let exam = problem
        .exams
        .iter()
        .find(|e| e.id == panel.exam_id)
        .map(|e| e.code.clone())
        .unwrap_or_else(|| format!("E{}", panel.exam_id.0));
    let grade = problem
        .grades
        .iter()
        .find(|g| g.id == panel.grade_id)
        .map(|g| g.code.to_string())
        .unwrap_or_else(|| format!("G{}", panel.grade_id.0));
    let subject = problem
        .subjects
        .iter()
        .find(|s| s.id == panel.subject_id)
        .map(|s| s.code.clone())
        .unwrap_or_else(|| format!("S{}", panel.subject_id.0));
    format!("{exam}/K{grade}/{subject}")
}

fn panel_members(
    problem: &Problem,
    assignments: &[Assignment],
    panel: PanelKey,
    role: Role,
) -> String {
    let mut members: Vec<String> = assignments
        .iter()
        .filter(|a| {
            a.exam_id == panel.exam_id
                && a.grade_id == panel.grade_id
                && a.subject_id == panel.subject_id
                && a.role == role
        })
        .map(|a| {
            let campus = problem
                .teachers
                .iter()
                .find(|t| t.id == a.teacher_id)
                .map(|t| campus_code(problem, t.campus_id))
                .unwrap_or_else(|| "-".to_string());
            format!("{}({campus})", teacher_code(problem, a.teacher_id))
        })
        .collect();
    members.sort();
    if members.is_empty() {
        "-".to_string()
    } else {
        members.join(", ")
    }
}

/// Counts avoidable crowding: teacher/exam cells with at least 2 tasks.
fn crowding_cases(problem: &Problem, assignments: &[Assignment]) -> Vec<(String, String, usize)> {
    let mut counts: HashMap<(TeacherId, ExamId), usize> = HashMap::new();
    for a in assignments {
        *counts.entry((a.teacher_id, a.exam_id)).or_default() += 1;
    }
    let mut rows: Vec<(String, String, usize)> = counts
        .into_iter()
        .filter(|(_, n)| *n >= 2)
        .map(|((t, e), n)| {
            let exam = problem
                .exams
                .iter()
                .find(|x| x.id == e)
                .map(|x| x.code.clone())
                .unwrap_or_else(|| format!("E{}", e.0));
            (teacher_code(problem, t), exam, n)
        })
        .collect();
    rows.sort();
    rows
}

fn h3_violations(problem: &Problem, assignments: &[Assignment]) -> Vec<Violation> {
    validate_assignments(problem, assignments, &ValidateOptions::default())
        .into_iter()
        .filter(|v| v.rule == RuleKey::H3)
        .collect()
}

fn write_plan_json(
    path: &Path,
    problem: &Problem,
    assignments: &[Assignment],
    report: &ScoreReport,
    violations: &[Violation],
) {
    let payload = serde_json::json!({
        "problem": problem,
        "assignments": assignments,
        "score": report,
        "hard_violations": violations,
    });
    fs::write(
        path,
        serde_json::to_string_pretty(&payload).expect("serialize plan"),
    )
    .expect("write plan json");
}

fn main() {
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "docs/reports/edu-2026/q17".to_string()),
    );
    fs::create_dir_all(&out_dir).expect("create output dir");

    eprintln!("[q17] building synthetic 17-teacher problem...");
    let problem = make_q17_problem();

    let structural_errors = problem.validate();
    if !structural_errors.is_empty() {
        eprintln!("[q17] structural validation failed: {structural_errors:?}");
        std::process::exit(1);
    }

    let feasibility = check_feasibility(&problem);
    let quotas = calculate_quotas(&problem);

    eprintln!("[q17] building deterministic round-robin manual plan...");
    let manual = make_manual_plan(&problem);
    assert_eq!(manual.len(), 60, "manual plan must fill 60 seats");
    let manual_report = evaluate(&problem, &manual);
    let manual_violations = validate_assignments(&problem, &manual, &ValidateOptions::default());
    let manual_h3 = h3_violations(&problem, &manual);
    let manual_crowding = crowding_cases(&problem, &manual);

    eprintln!(
        "[q17] running cold-start optimizer (seed {BASE_SEED}, R = {NUM_RUNS}, {ITERATIONS} iters/run)..."
    );
    let opts = OptimizeOptions {
        base_seed: BASE_SEED,
        num_runs: NUM_RUNS,
        budget: Budget::Iterations(ITERATIONS),
        max_plans: 3,
        diversity_threshold: 0.20,
        initial_assignments: None,
        cancel: None,
        progress: None,
    };
    let result = match optimize(&problem, &opts) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[q17] optimizer failed: {err}");
            std::process::exit(2);
        }
    };
    let best = &result.plans[0];
    let opt_assignments = &best.assignments;
    let opt_report = &best.report;
    let opt_violations =
        validate_assignments(&problem, opt_assignments, &ValidateOptions::default());
    let opt_h3 = h3_violations(&problem, opt_assignments);
    let opt_crowding = crowding_cases(&problem, opt_assignments);

    let bounds = lower_bounds(&problem);
    let soft_keys = [
        RuleKey::S1,
        RuleKey::S2,
        RuleKey::S3,
        RuleKey::S4,
        RuleKey::S5,
        RuleKey::S6,
        RuleKey::S7,
        RuleKey::S8,
        RuleKey::S9,
        RuleKey::S10,
    ];

    let mut lb_units = HashMap::new();
    for b in &bounds {
        lb_units.insert(b.rule, b.units_lower_bound);
    }
    let lb_penalty: f64 = soft_keys
        .iter()
        .map(|k| lb_units.get(k).copied().unwrap_or(0.0) * rule_weight(&manual_report, *k))
        .sum();

    let manual_s3 = rule_penalty(&manual_report, RuleKey::S3);
    let opt_s3 = rule_penalty(opt_report, RuleKey::S3);
    let manual_total = manual_report.total;
    let opt_total = opt_report.total;
    let manual_no_s3 = manual_total - manual_s3;
    let opt_no_s3 = opt_total - opt_s3;
    let reduction = if manual_total.abs() > 1e-12 {
        (manual_total - opt_total) / manual_total * 100.0
    } else {
        0.0
    };
    let reduction_no_s3 = if manual_no_s3.abs() > 1e-12 {
        (manual_no_s3 - opt_no_s3) / manual_no_s3 * 100.0
    } else {
        0.0
    };

    let mut md = String::new();
    writeln!(
        md,
        "# Q17 — Kiểm định 17 giáo viên, 4 phân hiệu (dữ liệu mô phỏng)"
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "> Toàn bộ dữ liệu trong tệp này là **dữ liệu mô phỏng**: tên giáo viên là GV01…GV17, \
         phân hiệu là PH1…PH4. Không có dữ liệu cá nhân thật. Tệp được sinh trực tiếp bởi \
         `crates/service/examples/q17_comparison.rs`; mọi con số do chương trình in ra, \
         không viết tay. Đầu ra lệnh gốc: `run.log` cùng thư mục."
    )
    .unwrap();
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 1. Dataset
    // ------------------------------------------------------------------
    writeln!(md, "## 1. Cấu hình bộ dữ liệu").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "- Quy mô: **{} giáo viên** ({} Vật lí + {} Công nghệ), {} phân hiệu, {} kỳ × {} khối.",
        problem.teachers.len(),
        teacher_specs().iter().filter(|s| s.2 == VL).count(),
        teacher_specs().iter().filter(|s| s.2 == CN).count(),
        problem.campuses.len(),
        problem.exams.len(),
        problem.grades.len()
    )
    .unwrap();
    let panels_per_subject = problem.exams.len() * problem.grades.len();
    let total_slots: usize = problem
        .subjects
        .iter()
        .map(|s| panels_per_subject * (s.setters as usize + s.reviewers as usize))
        .sum();
    writeln!(
        md,
        "- Vị trí: {} ban đề Vật lí × 3 người + {} ban đề Công nghệ × 2 người = **{} vị trí**.",
        panels_per_subject, panels_per_subject, total_slots
    )
    .unwrap();
    writeln!(
        md,
        "- Ràng buộc cứng H1–H7 bật mặc định; H3 yêu cầu mỗi ban đề có giáo viên từ ≥ 2 phân hiệu \
         (áp dụng cho cả VL và CN). H4 mặc định: tối đa 2 việc/kỳ, tối đa 1 lượt ra đề/kỳ. \
         H7: dung sai 1. Không dùng quota override hay max-task override."
    )
    .unwrap();
    writeln!(
        md,
        "- Trọng số mềm mặc định: S1 = 10, S2 = 3, S3 = 4, S4 = 6, S5 = 6, S6 = 2, S7 = 1, \
         S8 = 8, S9 = 5, S10 = 4."
    )
    .unwrap();
    writeln!(
        md,
        "- Kiểm tra khả thi trước khi giải: **{}** ({} lỗi, {} cảnh báo).",
        if feasibility.is_feasible() {
            "KHẢ THI"
        } else {
            "KHÔNG KHẢ THI"
        },
        feasibility.errors.len(),
        feasibility.warnings.len()
    )
    .unwrap();
    for e in &feasibility.errors {
        writeln!(md, "  - Lỗi: `{}` {:?}", e.code, e.params).unwrap();
    }
    for w in &feasibility.warnings {
        writeln!(md, "  - Cảnh báo: `{}` {:?}", w.code, w.params).unwrap();
    }
    writeln!(md).unwrap();

    writeln!(
        md,
        "| Mã GV | Môn | Phân hiệu | Khối dạy | Hệ số tải | Chỉ tiêu q | lo | hi |"
    )
    .unwrap();
    writeln!(
        md,
        "| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |"
    )
    .unwrap();
    for t in &problem.teachers {
        let code = t.code.clone().unwrap_or_default();
        let subject = teacher_subject(&problem, t.id);
        let grades = problem
            .teacher_grades
            .iter()
            .filter(|tg| tg.teacher_id == t.id)
            .filter_map(|tg| problem.grades.iter().find(|g| g.id == tg.grade_id))
            .map(|g| g.code.to_string())
            .collect::<Vec<_>>()
            .join("/");
        let q = quotas.iter().find(|x| x.teacher_id == t.id);
        writeln!(
            md,
            "| {} | {} | {} | {} | {:.1} | {:.4} | {} | {} |",
            code,
            subject,
            campus_code(&problem, t.campus_id),
            grades,
            t.load_weight,
            q.map_or(0.0, |x| x.quota),
            q.map_or(0, |x| x.lo),
            q.map_or(0, |x| x.hi)
        )
        .unwrap();
    }
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 2. Manual plan
    // ------------------------------------------------------------------
    writeln!(
        md,
        "## 2. Phương án thủ công (do chương trình sinh, không phải người thật)"
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "Đây là **một** phương án tham chiếu sinh bằng quy trình xoay vòng tất định, không dùng \
         bộ giải và không tinh chỉnh theo quy tắc mềm. Mô tả:"
    )
    .unwrap();
    writeln!(
        md,
        "- Danh sách Vật lí lấy theo mã GV tăng dần (GV01…GV10); danh sách Công nghệ lấy theo mã \
         GV tăng dần (GV11…GV17)."
    )
    .unwrap();
    writeln!(
        md,
        "- Với kỳ thứ e (e = 0…3), con trỏ Vật lí bắt đầu tại `(9 × e) mod 10`; lần lượt lấp 9 ghế \
         theo thứ tự K10-ra đề 1, K10-ra đề 2, K10-phản biện, K11-ra đề 1, …, K12-phản biện."
    )
    .unwrap();
    writeln!(
        md,
        "- Con trỏ Công nghệ bắt đầu tại `(6 × e) mod 7`; lần lượt lấp 6 ghế theo thứ tự \
         K10-ra đề, K10-phản biện, K11-ra đề, K11-phản biện, K12-ra đề, K12-phản biện."
    )
    .unwrap();
    writeln!(
        md,
        "- Con trỏ chỉ chạy tiếp qua các kỳ; quy trình không kiểm tra lại ràng buộc mềm, không \
         hoán đổi để giảm điểm, và không gọi thuật toán tối ưu."
    )
    .unwrap();
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 3. Lower bounds
    // ------------------------------------------------------------------
    writeln!(md, "## 3. Cận dưới lý thuyết (do `lower_bounds` tính)").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "| Quy tắc | Cận dưới (đơn vị) | Phạt tại trọng số mặc định | Phương pháp |"
    )
    .unwrap();
    writeln!(md, "| :--- | :---: | :---: | :--- |").unwrap();
    for key in soft_keys {
        let lb = lb_units.get(&key).copied().unwrap_or(0.0);
        let w = rule_weight(&manual_report, key);
        let method = bounds
            .iter()
            .find(|b| b.rule == key)
            .map(|b| b.method.clone())
            .unwrap_or_default();
        writeln!(
            md,
            "| {} | {:.4} | {:.2} | `{}` |",
            rule_label(key),
            lb,
            lb * w,
            method
        )
        .unwrap();
    }
    writeln!(
        md,
        "| **Tổng cận dưới** | **{:.4}** | **{:.2}** | |",
        lb_units.values().sum::<f64>(),
        lb_penalty
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "Ghi chú: cận dưới S3 = 0 là cận độc lập (không xét xung đột cơ sở cụ thể); S8 dùng phân \
         bổ số nguyên tối ưu theo chỉ tiêu q. Vì vậy tổng cận dưới ở trên là mốc tham chiếu cho \
         phần mềm, không phải giá trị mà mọi phương án đều đạt."
    )
    .unwrap();
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 4. Per-rule comparison
    // ------------------------------------------------------------------
    writeln!(
        md,
        "## 4. Điểm phạt từng quy tắc: thủ công vs tối ưu (cold start)"
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "- Tham số tối ưu: base seed = {}, R = {}, {} vòng/lần chạy, K = 3 phương án giữ lại.",
        BASE_SEED, NUM_RUNS, ITERATIONS
    )
    .unwrap();
    writeln!(
        md,
        "- Thời gian chạy tối ưu: **{} ms**; tổng vòng lặp: **{}**; số phương án trả về: **{}**.",
        result.stats.elapsed_ms,
        result.stats.total_iterations,
        result.plans.len()
    )
    .unwrap();
    writeln!(
        md,
        "- Phương án tối ưu tốt nhất: hạng {}, seed thắng = **{}**.",
        best.rank, best.seed
    )
    .unwrap();
    writeln!(md).unwrap();

    writeln!(
        md,
        "| Quy tắc | Trọng số | Thủ công đơn vị | Thủ công phạt | Tối ưu đơn vị | Tối ưu phạt | Giảm phạt |"
    )
    .unwrap();
    writeln!(
        md,
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: |"
    )
    .unwrap();
    for key in soft_keys {
        let w = rule_weight(&manual_report, key);
        let mu = rule_units(&manual_report, key);
        let mp = rule_penalty(&manual_report, key);
        let ou = rule_units(opt_report, key);
        let op = rule_penalty(opt_report, key);
        writeln!(
            md,
            "| {} | {:.1} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            rule_label(key),
            w,
            mu,
            mp,
            ou,
            op,
            mp - op
        )
        .unwrap();
    }
    writeln!(
        md,
        "| **Tổng (có S3)** | | | **{:.2}** | | **{:.2}** | **{:.2}** |",
        manual_total,
        opt_total,
        manual_total - opt_total
    )
    .unwrap();
    writeln!(
        md,
        "| **Tổng (không S3)** | | | **{:.2}** | | **{:.2}** | **{:.2}** |",
        manual_no_s3,
        opt_no_s3,
        manual_no_s3 - opt_no_s3
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "**Tỉ lệ giảm điểm phạt (có S3): {:.2}%.** Tỉ lệ giảm khi không tính S3: **{:.2}%**.",
        reduction, reduction_no_s3
    )
    .unwrap();
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 5. Hard constraints
    // ------------------------------------------------------------------
    writeln!(md, "## 5. Ràng buộc cứng và vi phạm H3").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "| Phương án | Tổng vi phạm cứng | Vi phạm H3 | Số ban đề H3 không đạt |"
    )
    .unwrap();
    writeln!(md, "| :--- | :---: | :---: | :---: |").unwrap();
    let manual_h3_panels = manual_h3
        .iter()
        .filter_map(|v| v.panel)
        .collect::<std::collections::HashSet<_>>()
        .len();
    let opt_h3_panels = opt_h3
        .iter()
        .filter_map(|v| v.panel)
        .collect::<std::collections::HashSet<_>>()
        .len();
    writeln!(
        md,
        "| Thủ công | {} | {} | {} |",
        manual_violations.len(),
        manual_h3.len(),
        manual_h3_panels
    )
    .unwrap();
    writeln!(
        md,
        "| Tối ưu | {} | {} | {} |",
        opt_violations.len(),
        opt_h3.len(),
        opt_h3_panels
    )
    .unwrap();
    writeln!(md).unwrap();

    for (title, viols, plan) in [
        ("thủ công", &manual_h3, manual.as_slice()),
        ("tối ưu", &opt_h3, opt_assignments.as_slice()),
    ] {
        if viols.is_empty() {
            writeln!(md, "- Phương án {title}: không có ban đề nào vi phạm H3.").unwrap();
        } else {
            writeln!(md, "- Phương án {title}: các ban đề vi phạm H3:").unwrap();
            writeln!(
                md,
                "  | Ban đề | Số phân hiệu | Yêu cầu | Ra đề (cơ sở) | Phản biện (cơ sở) |"
            )
            .unwrap();
            writeln!(md, "  | :--- | :---: | :---: | :--- | :--- |").unwrap();
            for v in viols {
                if let Some(panel) = v.panel {
                    let campus_count = v
                        .params
                        .get("campus_count")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    let min_campuses = v
                        .params
                        .get("min_campuses")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    writeln!(
                        md,
                        "  | {} | {} | {} | {} | {} |",
                        panel_label(&problem, panel),
                        campus_count,
                        min_campuses,
                        panel_members(&problem, plan, panel, Role::Setter),
                        panel_members(&problem, plan, panel, Role::Reviewer)
                    )
                    .unwrap();
                }
            }
        }
    }
    writeln!(md).unwrap();

    // ------------------------------------------------------------------
    // 6. Crowding
    // ------------------------------------------------------------------
    writeln!(md, "## 6. Dồn việc trong cùng một kỳ").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "\"Trường hợp dồn việc\" ở đây là một cặp (giáo viên, kỳ thi) có từ 2 việc trở lên. \
         S9 đo phần dồn việc có thể tránh được sau khi trừ phần bắt buộc."
    )
    .unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "| Phương án | Số trường hợp dồn việc | Số GV bị dồn việc | S9 đơn vị | S9 phạt |"
    )
    .unwrap();
    writeln!(md, "| :--- | :---: | :---: | :---: | :---: |").unwrap();
    let manual_crowded_teachers = manual_crowding
        .iter()
        .map(|(t, _, _)| t.clone())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let opt_crowded_teachers = opt_crowding
        .iter()
        .map(|(t, _, _)| t.clone())
        .collect::<std::collections::HashSet<_>>()
        .len();
    writeln!(
        md,
        "| Thủ công | {} | {} | {:.2} | {:.2} |",
        manual_crowding.len(),
        manual_crowded_teachers,
        rule_units(&manual_report, RuleKey::S9),
        rule_penalty(&manual_report, RuleKey::S9)
    )
    .unwrap();
    writeln!(
        md,
        "| Tối ưu | {} | {} | {:.2} | {:.2} |",
        opt_crowding.len(),
        opt_crowded_teachers,
        rule_units(opt_report, RuleKey::S9),
        rule_penalty(opt_report, RuleKey::S9)
    )
    .unwrap();
    writeln!(md).unwrap();
    if !manual_crowding.is_empty() {
        writeln!(md, "Chi tiết dồn việc của phương án thủ công:").unwrap();
        writeln!(md, "| Giáo viên | Kỳ | Số việc |").unwrap();
        writeln!(md, "| :---: | :---: | :---: |").unwrap();
        for (t, e, n) in &manual_crowding {
            writeln!(md, "| {} | {} | {} |", t, e, n).unwrap();
        }
        writeln!(md).unwrap();
    }

    // ------------------------------------------------------------------
    // 7. Workload per teacher
    // ------------------------------------------------------------------
    writeln!(md, "## 7. Khối lượng theo giáo viên").unwrap();
    writeln!(md).unwrap();
    for (title, assigns, report) in [
        ("thủ công", manual.as_slice(), &manual_report),
        ("tối ưu", opt_assignments.as_slice(), opt_report),
    ] {
        writeln!(md, "### Phương án {title}").unwrap();
        writeln!(md).unwrap();
        writeln!(
            md,
            "| Mã GV | Môn | Phân hiệu | q | Tổng | Ra đề | Phản biện | GK1 | CK1 | GK2 | CK2 |"
        )
        .unwrap();
        writeln!(md, "| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |").unwrap();
        for t in &problem.teachers {
            let mine: Vec<&Assignment> = assigns.iter().filter(|a| a.teacher_id == t.id).collect();
            let per_exam: Vec<usize> = problem
                .exams
                .iter()
                .map(|e| mine.iter().filter(|a| a.exam_id == e.id).count())
                .collect();
            let q = report
                .per_teacher
                .iter()
                .find(|x| x.teacher_id == t.id)
                .map_or(0.0, |x| x.quota);
            writeln!(
                md,
                "| {} | {} | {} | {:.4} | {} | {} | {} | {} | {} | {} | {} |",
                t.code.clone().unwrap_or_default(),
                teacher_subject(&problem, t.id),
                campus_code(&problem, t.campus_id),
                q,
                mine.len(),
                mine.iter().filter(|a| a.role == Role::Setter).count(),
                mine.iter().filter(|a| a.role == Role::Reviewer).count(),
                per_exam[0],
                per_exam[1],
                per_exam[2],
                per_exam[3]
            )
            .unwrap();
        }
        writeln!(md).unwrap();
    }

    // ------------------------------------------------------------------
    // 8. Artifacts
    // ------------------------------------------------------------------
    writeln!(md, "## 8. Tệp sinh kèm").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "- `q17_problem.json`: ảnh chụp bài toán 17 GV (cấu hình, quy tắc, chỉ tiêu)."
    )
    .unwrap();
    writeln!(
        md,
        "- `q17_manual_plan.json`: 60 phân công thủ công + điểm + vi phạm cứng."
    )
    .unwrap();
    writeln!(
        md,
        "- `q17_optimized_plan.json`: 60 phân công tối ưu + điểm + vi phạm cứng."
    )
    .unwrap();
    writeln!(
        md,
        "- `q17_scores.json`: bảng điểm hai phương án, cận dưới, thống kê dồn việc."
    )
    .unwrap();
    writeln!(md).unwrap();

    // Write artifacts.
    let problem_json = serde_json::json!({
        "teachers": &problem.teachers,
        "campuses": &problem.campuses,
        "grades": &problem.grades,
        "subjects": &problem.subjects,
        "exams": &problem.exams,
        "rule_settings": &problem.rule_settings,
        "quotas": &quotas,
        "feasibility": &feasibility,
    });
    fs::write(
        out_dir.join("q17_problem.json"),
        serde_json::to_string_pretty(&problem_json).expect("serialize problem"),
    )
    .expect("write problem json");

    write_plan_json(
        &out_dir.join("q17_manual_plan.json"),
        &problem,
        &manual,
        &manual_report,
        &manual_violations,
    );
    write_plan_json(
        &out_dir.join("q17_optimized_plan.json"),
        &problem,
        opt_assignments,
        opt_report,
        &opt_violations,
    );

    let scores_json = serde_json::json!({
        "parameters": {
            "base_seed": BASE_SEED,
            "num_runs": NUM_RUNS,
            "iterations_per_run": ITERATIONS,
            "max_plans": 3,
            "diversity_threshold": 0.20,
        },
        "manual": {
            "total": manual_total,
            "total_without_s3": manual_no_s3,
            "h3_violations": manual_h3.len(),
            "hard_violations_total": manual_violations.len(),
            "crowding_cases": manual_crowding.len(),
            "score": &manual_report,
        },
        "optimized": {
            "rank": best.rank,
            "seed": best.seed,
            "total": opt_total,
            "total_without_s3": opt_no_s3,
            "h3_violations": opt_h3.len(),
            "hard_violations_total": opt_violations.len(),
            "crowding_cases": opt_crowding.len(),
            "score": opt_report,
        },
        "lower_bounds": &bounds,
        "lower_bound_total_penalty": lb_penalty,
        "reduction_percent": reduction,
        "reduction_percent_without_s3": reduction_no_s3,
        "optimizer_stats": &result.stats,
    });
    fs::write(
        out_dir.join("q17_scores.json"),
        serde_json::to_string_pretty(&scores_json).expect("serialize scores"),
    )
    .expect("write scores json");

    fs::write(out_dir.join("q17_comparison.md"), &md).expect("write markdown report");

    print!("{md}");
}
