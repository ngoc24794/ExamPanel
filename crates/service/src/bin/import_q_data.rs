//! Dev tool to apply Q real data JSON to a SQLite database.
//!
//! Usage:
//!     cargo run --bin import_q_data -- [path/to/q_real_data.json] [path/to/db.sqlite]

use exam_panel_core::domain::fixtures::{
    load_q_real_data_from_file_or_env, make_canonical_q_problem_with_real_data, QRealData,
    QVariant, DEFAULT_Q_REAL_DATA_PATH,
};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let json_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or(DEFAULT_Q_REAL_DATA_PATH);

    println!("ExamPanel Dev Tool: Import Q Real Data");
    println!("Loading JSON from: {json_path}");

    let real_data = if Path::new(json_path).exists() {
        let content = fs::read_to_string(json_path)?;
        let data = QRealData::from_json_str(&content)?;
        println!("Successfully parsed Q real data:");
        println!("  - Campuses: {}", data.campuses.len());
        println!("  - Teacher campuses: {}", data.teacher_campuses.len());
        println!("  - Teacher grades: {}", data.teacher_grades.len());
        Some(data)
    } else {
        println!("File not found at '{json_path}'. Checking env/fallback...");
        load_q_real_data_from_file_or_env()
    };

    let problem =
        make_canonical_q_problem_with_real_data(QVariant::SyntheticCampuses, real_data.as_ref());

    println!("\nConstructed problem snapshot with real data overlay:");
    println!("  - Campuses: {}", problem.campuses.len());
    for c in &problem.campuses {
        println!("    • [{}] {}: {}", c.id.0, c.code, c.name);
    }
    println!("  - Teachers: {}", problem.teachers.len());
    for t in &problem.teachers {
        let c = problem.campuses.iter().find(|c| c.id == t.campus_id);
        let c_str = c.map(|c| c.code.as_str()).unwrap_or("???");
        let tg_count = problem
            .teacher_grades
            .iter()
            .filter(|tg| tg.teacher_id == t.id)
            .count();
        println!(
            "    • [{}] {:<12} (campus: {:<4}, grades taught: {})",
            t.id.0, t.full_name, c_str, tg_count
        );
    }

    let validation = problem.validate();
    if validation.is_empty() {
        println!("\nProblem structural validation: PASSED (0 errors)");
    } else {
        println!(
            "\nProblem structural validation: {} errors:",
            validation.len()
        );
        for err in validation {
            println!("  ! {err:?}");
        }
    }

    Ok(())
}
