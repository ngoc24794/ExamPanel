//! ExamPanel Tauri Application Shell
//!
//! Exposes IPC commands from Rust to the React frontend.
//! Calls into `exam_panel_service`.

pub mod commands;

use exam_panel_service::dto::CreateSchoolYearInput;
use exam_panel_service::service::AppService;
use std::sync::Arc;

/// Application identifier as a constant for easy modification.
pub const APP_IDENTIFIER: &str = "vn.exampanel.app";

/// Display name of the application.
pub const APP_NAME: &str = "ExamPanel";

/// Executes headless smoke test mode, printing JSON summary to stdout and exiting with code 0.
pub fn run_smoke_test() {
    let service = AppService::open_default().expect("failed to initialize database in smoke mode");
    let info = service
        .get_app_info()
        .expect("failed to get app info in smoke mode");

    let sy = service
        .list_school_years()
        .unwrap_or_default()
        .into_iter()
        .find(|y| y.name == "Smoke Test Year")
        .unwrap_or_else(|| {
            service
                .create_school_year(CreateSchoolYearInput {
                    name: "Smoke Test Year".to_string(),
                    is_current: false,
                    copy_grades_from: None,
                })
                .expect("failed to create smoke test school year")
        });

    let feas = service
        .check_feasibility(sy.id)
        .expect("failed to run feasibility check in smoke mode");

    let summary = serde_json::json!({
        "status": "ok",
        "mode": "smoke_test",
        "app_name": APP_NAME,
        "app_version": info.version,
        "data_dir": info.data_dir,
        "db_path": info.db_path,
        "is_portable": info.is_portable,
        "feasibility": {
            "school_year_id": sy.id,
            "is_feasible": feas.report.is_feasible(),
            "errors": feas.report.errors.len(),
            "warnings": feas.report.warnings.len(),
            "quotas_count": feas.quotas.len(),
        }
    });

    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    std::process::exit(0);
}

/// Executes headless smoke test mode on seeded demo data, printing JSON summary to stdout and exiting with code 0.
pub fn run_smoke_demo_test() {
    let service = AppService::open_default().expect("failed to initialize database in smoke mode");
    let info = service
        .get_app_info()
        .expect("failed to get app info in smoke mode");

    service
        .seed_demo()
        .expect("failed to seed demo data in smoke demo mode");

    let sy = service
        .list_school_years()
        .unwrap_or_default()
        .into_iter()
        .find(|y| y.name == "2026-2027")
        .or_else(|| {
            service
                .list_school_years()
                .unwrap_or_default()
                .into_iter()
                .find(|y| y.is_current)
        })
        .or_else(|| {
            service
                .list_school_years()
                .unwrap_or_default()
                .into_iter()
                .next()
        })
        .expect("expected at least one school year after seeding demo");

    let _ = service.set_current_school_year(sy.id);

    let feas = service
        .check_feasibility(sy.id)
        .expect("failed to run feasibility check in smoke demo mode");

    let summary = serde_json::json!({
        "status": "ok",
        "mode": "smoke_demo",
        "app_name": APP_NAME,
        "app_version": info.version,
        "data_dir": info.data_dir,
        "db_path": info.db_path,
        "is_portable": info.is_portable,
        "feasibility": {
            "school_year_id": sy.id,
            "school_year_name": sy.name,
            "is_feasible": feas.report.is_feasible(),
            "errors": feas.report.errors.len(),
            "warnings": feas.report.warnings.len(),
            "quotas_count": feas.quotas.len(),
        }
    });

    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--smoke-test") {
        run_smoke_test();
        return;
    }
    if args.iter().any(|arg| arg == "--smoke-demo") {
        run_smoke_demo_test();
        return;
    }

    let service =
        Arc::new(AppService::open_default().expect("failed to open database and apply migrations"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(service)
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_app_info,
            commands::get_settings,
            commands::set_setting,
            commands::list_campuses,
            commands::create_campus,
            commands::update_campus,
            commands::delete_campus,
            commands::list_grades,
            commands::create_grade,
            commands::update_grade,
            commands::delete_grade,
            commands::list_teachers,
            commands::create_teacher,
            commands::update_teacher,
            commands::delete_teacher,
            commands::deactivate_teacher,
            commands::set_teacher_grades,
            commands::teachers_with_grades,
            commands::list_school_years,
            commands::create_school_year,
            commands::set_current_school_year,
            commands::list_exams,
            commands::create_exam,
            commands::update_exam,
            commands::delete_exam,
            commands::reorder_exams,
            commands::list_unavailabilities,
            commands::set_unavailability,
            commands::delete_unavailability,
            commands::list_locks,
            commands::create_lock,
            commands::delete_lock,
            commands::get_rule_settings,
            commands::save_rule_settings,
            commands::reset_rule_settings_to_defaults,
            commands::preview_quotas,
            commands::get_rule_presets,
            commands::check_feasibility,
            commands::evaluate_assignments,
            commands::start_optimize,
            commands::cancel_optimize,
            commands::save_optimize_result,
            commands::list_plans,
            commands::get_plan,
            commands::rename_plan,
            commands::delete_plan,
            commands::mark_final,
            commands::duplicate_plan,
            commands::plan_status,
            commands::create_manual_copy,
            commands::update_plan_assignments,
            commands::evaluate_candidates,
            commands::evaluate_swap,
            commands::reoptimize_from,
            commands::seed_demo,
            commands::generate_import_template,
            commands::preview_import,
            commands::apply_import,
            commands::export_plan_excel,
            commands::backup_database,
            commands::restore_database,
            commands::validate_backup,
            commands::list_backups,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_returns_expected_format() {
        let msg = commands::ping();
        assert!(msg.starts_with("pong from ExamPanel"));
    }
}
