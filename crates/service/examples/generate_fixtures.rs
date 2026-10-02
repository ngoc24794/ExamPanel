use exam_panel_service::dto::{OptimizeBudget, OptimizeRequest};
use exam_panel_service::service::AppService;
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

fn main() {
    println!("Generating fixtures and types for ui/src/lib/api/...");

    let tmp = NamedTempFile::new().expect("create temp db");
    let service = AppService::open_at(tmp.path()).expect("open temp service");
    service.seed_demo().expect("seed demo data");

    let years = service.list_school_years().expect("list school years");
    let sy_id = years[0].id;

    let fixtures_dir = Path::new("ui/src/lib/api/fixtures");
    fs::create_dir_all(fixtures_dir).expect("create fixtures dir");

    // 1. Demo Feasibility Report
    let feas = service.check_feasibility(sy_id).expect("check feasibility");
    let feas_json = serde_json::to_string_pretty(&feas).expect("serialize feasibility");
    fs::write(fixtures_dir.join("demo_feasibility.json"), feas_json).expect("write feasibility");
    println!("✓ Generated demo_feasibility.json");

    // 2. Demo Optimize Outcome (8 runs, 200k iterations, k=3)
    let req = OptimizeRequest {
        base_seed: Some(42),
        runs: 8,
        budget: OptimizeBudget::Iterations(200_000),
        k: 3,
        diversity_threshold: Some(0.20),
    };
    println!("Running optimizer to generate demo_outcome.json...");
    let outcome = service
        .run_optimize(sy_id, req, None, None)
        .expect("run optimize");
    let outcome_json = serde_json::to_string_pretty(&outcome).expect("serialize outcome");
    fs::write(fixtures_dir.join("demo_outcome.json"), outcome_json).expect("write outcome");
    println!("✓ Generated demo_outcome.json");

    // 3. Save optimize outcome to get stored plans
    let saved_ids = service
        .save_optimize_result(sy_id, outcome.clone())
        .expect("save optimize result");
    let plan_summaries = service.list_plans(sy_id).expect("list plans");
    let summaries_json =
        serde_json::to_string_pretty(&plan_summaries).expect("serialize summaries");
    fs::write(fixtures_dir.join("demo_plans.json"), summaries_json).expect("write plans");
    println!("✓ Generated demo_plans.json");

    // 4. Campuses, Grades, Teachers, Exams, SchoolYears
    let campuses = service.list_campuses().expect("list campuses");
    fs::write(
        fixtures_dir.join("demo_campuses.json"),
        serde_json::to_string_pretty(&campuses).unwrap(),
    )
    .unwrap();

    let grades = service.list_grades().expect("list grades");
    fs::write(
        fixtures_dir.join("demo_grades.json"),
        serde_json::to_string_pretty(&grades).unwrap(),
    )
    .unwrap();

    let teachers = service.list_teachers().expect("list teachers");
    fs::write(
        fixtures_dir.join("demo_teachers.json"),
        serde_json::to_string_pretty(&teachers).unwrap(),
    )
    .unwrap();

    let exams = service.list_exams(sy_id).expect("list exams");
    fs::write(
        fixtures_dir.join("demo_exams.json"),
        serde_json::to_string_pretty(&exams).unwrap(),
    )
    .unwrap();

    let twg = service
        .teachers_with_grades(sy_id)
        .expect("teachers with grades");
    fs::write(
        fixtures_dir.join("demo_teachers_with_grades.json"),
        serde_json::to_string_pretty(&twg).unwrap(),
    )
    .unwrap();

    let rules = service.get_rule_settings(sy_id).expect("rule settings");
    fs::write(
        fixtures_dir.join("demo_rule_settings.json"),
        serde_json::to_string_pretty(&rules).unwrap(),
    )
    .unwrap();

    let details_p1 = service.get_plan(saved_ids[0]).expect("get plan 1");
    fs::write(
        fixtures_dir.join("demo_plan_details.json"),
        serde_json::to_string_pretty(&details_p1).unwrap(),
    )
    .unwrap();

    println!("All fixtures successfully written to ui/src/lib/api/fixtures/!");
}
