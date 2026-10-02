use exam_panel_service::service::AppService;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let reports_dir = manifest_dir.join("../../docs/reports/phase-9");
    fs::create_dir_all(&reports_dir).expect("Failed to create reports directory");

    let template_path = reports_dir.join("mau-nhap-du-lieu.xlsx");
    let export_path = reports_dir.join("phan-cong-demo.xlsx");

    let service = AppService::open_in_memory().expect("open in-memory service");
    service.seed_demo().expect("seed demo data");

    // 1. Generate template
    service
        .generate_import_template(&template_path)
        .expect("generate template");
    println!("Generated template at {:?}", template_path);

    // 2. Export plan
    let sy = service
        .list_school_years()
        .expect("list years")
        .into_iter()
        .find(|y| y.is_current)
        .expect("current year");

    let outcome = service
        .run_optimize(
            sy.id,
            exam_panel_service::dto::OptimizeRequest {
                base_seed: Some(42),
                runs: 1,
                budget: exam_panel_service::dto::OptimizeBudget::Iterations(200),
                k: 1,
                diversity_threshold: None,
            },
            None,
            None,
        )
        .expect("run optimize");

    let plan_ids = service
        .save_optimize_result(sy.id, outcome)
        .expect("save plan");
    let plan_id = plan_ids[0];

    service
        .export_plan_excel(plan_id, &export_path)
        .expect("export plan");
    println!("Generated plan export at {:?}", export_path);
}
