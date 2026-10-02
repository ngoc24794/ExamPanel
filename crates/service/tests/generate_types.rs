use std::fs;
use std::path::PathBuf;

#[test]
fn test_typescript_types_freshness() {
    let generated = exam_panel_service::dto::generate_typescript_declarations();

    // Write to a temporary directory to exercise temp file write path
    let temp_dir = std::env::temp_dir().join(format!("exam_panel_typegen_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("failed to create temp dir");
    let temp_file = temp_dir.join("types.ts");
    fs::write(&temp_file, &generated).expect("failed to write temp generated types");

    let temp_content = fs::read_to_string(&temp_file)
        .expect("failed to read temp file")
        .replace("\r\n", "\n");

    let _ = fs::remove_dir_all(&temp_dir);

    // Read committed types file
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let committed_file = manifest_dir.join("../../ui/src/lib/api/generated/types.ts");
    let committed_content = fs::read_to_string(&committed_file)
        .expect("failed to read committed types.ts")
        .replace("\r\n", "\n");

    assert_eq!(
        temp_content, committed_content,
        "Committed TypeScript types in ui/src/lib/api/generated/types.ts are stale! Run `cargo run -p exam-panel-service --bin generate_types` or `pnpm run typegen` to update."
    );
}
