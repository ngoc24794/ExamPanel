use std::fs;
use std::path::PathBuf;

fn main() {
    let content = exam_panel_service::dto::generate_typescript_declarations();

    // Resolve target path relative to cargo manifest dir
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_file = manifest_dir.join("../../ui/src/lib/api/generated/types.ts");

    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directory for generated types");
    }

    fs::write(&target_file, &content).expect("Failed to write generated TypeScript types");
    println!(
        "Successfully generated TypeScript types to {:?}",
        target_file
    );
}
