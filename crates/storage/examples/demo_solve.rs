use exam_panel_core::domain::{Role, SchoolYearId};
use exam_panel_core::feasibility::check_feasibility;
use exam_panel_core::solver::{solve_hard, SolveOptions};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use exam_panel_storage::{open_in_memory, run_migrations, seed_demo, Store};

fn main() {
    println!("============================================================");
    println!("      EXAMPANEL - DEMO RUN: FEASIBILITY & HARD SOLVER       ");
    println!("============================================================");

    // 1. Initialize Database & Seed Demo Data
    let mut conn = open_in_memory().expect("Failed to open in-memory SQLite database");
    run_migrations(&mut conn).expect("Failed to run schema migrations");
    seed_demo(&mut conn).expect("Failed to seed demo data");
    println!(" [OK] Database initialized and seeded with demo dataset.");

    // 2. Load Problem
    let store = Store::new(conn);
    let school_year_id = SchoolYearId(1);
    let problem = store
        .load_problem(school_year_id)
        .expect("Failed to load problem from storage");

    println!(" [OK] Problem loaded successfully:");
    println!("      - School Year: {}", problem.school_year.name);
    println!("      - Campuses:    {}", problem.campuses.len());
    println!("      - Exams:       {}", problem.exams.len());
    println!("      - Grades:      {}", problem.grades.len());
    println!("      - Teachers:    {}", problem.teachers.len());
    println!(
        "      - Total Panels: {}",
        problem.exams.len() * problem.grades.len()
    );

    // 3. Feasibility Checker
    println!("\n--- [STEP 1: FEASIBILITY CHECK] ---");
    let report = check_feasibility(&problem);
    if report.is_feasible() {
        println!(" Feasibility Status: FEASIBLE (0 blocking errors)");
    } else {
        println!(
            " Feasibility Status: INFEASIBLE ({} errors)",
            report.errors.len()
        );
        for err in &report.errors {
            println!(
                "   [ERROR] {:?}: {}",
                err.code,
                serde_json::to_string(&err.params).unwrap()
            );
        }
        return;
    }

    if !report.warnings.is_empty() {
        println!(" Warnings ({}):", report.warnings.len());
        for w in &report.warnings {
            println!(
                "   [WARN] {:?}: {}",
                w.code,
                serde_json::to_string(&w.params).unwrap()
            );
        }
    }

    println!("\n Teacher Quota Allocation (with availability scaling):");
    println!(" | ID  | Full Name            | Weight | Target Quota | Min (lo) | Max (hi) |");
    println!(" |:---:|:---------------------|:------:|:------------:|:--------:|:--------:|");
    for q in &report.quotas {
        let t = problem
            .teachers
            .iter()
            .find(|t| t.id == q.teacher_id)
            .unwrap();
        println!(
            " | T{:<2} | {:<20} | {:<6.1} | {:<12.2} | {:<8} | {:<8} |",
            t.id.0, t.full_name, t.load_weight, q.quota, q.lo, q.hi
        );
    }

    // 4. Hard Solver Execution
    println!("\n--- [STEP 2: HARD-CONSTRAINT SOLVER] ---");
    let opts = SolveOptions {
        seed: 42,
        time_limit_ms: 2000,
        max_nodes: 500_000,
    };

    let start = std::time::Instant::now();
    let solution = match solve_hard(&problem, &opts) {
        Ok(sol) => sol,
        Err(e) => {
            println!(" Solver error: {:?}", e);
            return;
        }
    };
    let elapsed = start.elapsed();

    println!(
        " Solver completed in: {:.2} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    println!(" Search nodes explored: {}", solution.stats.nodes);
    println!(" Backtracks:            {}", solution.stats.backtracks);
    println!(" Total assignments:     {}", solution.assignments.len());

    // 5. Solution Validation
    let violations = validate_assignments(
        &problem,
        &solution.assignments,
        &ValidateOptions {
            require_complete: true,
        },
    );

    if violations.is_empty() {
        println!(" Validation Status: 100% VALID (0 hard constraint violations)");
    } else {
        println!(" Validation Failed: {} violations found!", violations.len());
        for v in &violations {
            println!("   [VIOLATION] {:?} - {}", v.rule, v.code);
        }
        return;
    }

    // 6. Display Schedule Table
    println!("\n--- [STEP 3: ASSIGNMENT SCHEDULE (4 Exams x 3 Grades)] ---");
    println!(" | Exam | Grade | Setters (Ra de)                        | Reviewer (Phan bien)         | Campuses |");
    println!(" |:----:|:-----:|:---------------------------------------|:-----------------------------|:---------|");

    for exam in &problem.exams {
        for grade in &problem.grades {
            let panel_assigns: Vec<_> = solution
                .assignments
                .iter()
                .filter(|a| a.exam_id == exam.id && a.grade_id == grade.id)
                .collect();

            let setters: Vec<_> = panel_assigns
                .iter()
                .filter(|a| a.role == Role::Setter)
                .map(|a| {
                    let t = problem
                        .teachers
                        .iter()
                        .find(|t| t.id == a.teacher_id)
                        .unwrap();
                    let campus = problem
                        .campuses
                        .iter()
                        .find(|c| c.id == t.campus_id)
                        .unwrap();
                    format!("{} ({})", t.full_name, campus.code)
                })
                .collect();

            let reviewer: Vec<_> = panel_assigns
                .iter()
                .filter(|a| a.role == Role::Reviewer)
                .map(|a| {
                    let t = problem
                        .teachers
                        .iter()
                        .find(|t| t.id == a.teacher_id)
                        .unwrap();
                    let campus = problem
                        .campuses
                        .iter()
                        .find(|c| c.id == t.campus_id)
                        .unwrap();
                    format!("{} ({})", t.full_name, campus.code)
                })
                .collect();

            let campuses: Vec<_> = panel_assigns
                .iter()
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
                    c.code.as_str()
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();

            println!(
                " | {:<4} | {:<5} | {:<38} | {:<28} | {:<8} |",
                exam.code,
                grade.code,
                setters.join(", "),
                reviewer.join(", "),
                campuses.join(", ")
            );
        }
    }

    println!("\n============================================================");
    println!("                    DEMO RUN COMPLETED!                     ");
    println!("============================================================");
}
