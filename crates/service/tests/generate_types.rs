use std::fs;
use std::path::Path;

fn expected_typescript_types() -> String {
    let content = include_str!("../../../ui/src/lib/api/generated/types.ts");
    content.replace("\r\n", "\n")
}

#[test]
fn test_typescript_types_freshness() {
    let types_path = Path::new("../../ui/src/lib/api/generated/types.ts");
    assert!(
        types_path.exists(),
        "ui/src/lib/api/generated/types.ts must exist"
    );

    let on_disk = fs::read_to_string(types_path)
        .expect("read ui/src/lib/api/generated/types.ts")
        .replace("\r\n", "\n");

    let expected = expected_typescript_types();
    assert_eq!(
        on_disk, expected,
        "ui/src/lib/api/generated/types.ts is out of date!"
    );

    // Verify key structs and types are present
    assert!(on_disk.contains("export interface AppInfo"));
    assert!(on_disk.contains("export interface AppSettings"));
    assert!(on_disk.contains("export interface Campus"));
    assert!(on_disk.contains("export interface Grade"));
    assert!(on_disk.contains("export interface Teacher"));
    assert!(on_disk.contains("export interface TeacherWithGrades"));
    assert!(on_disk.contains("export interface SchoolYear"));
    assert!(on_disk.contains("export interface Exam"));
    assert!(on_disk.contains("export interface Unavailability"));
    assert!(on_disk.contains("export interface Lock"));
    assert!(on_disk.contains("export interface RuleSetting"));
    assert!(on_disk.contains("export interface FeasibilityReportWithQuotas"));
    assert!(on_disk.contains("export interface OptimizeRequest"));
    assert!(on_disk.contains("export interface OptimizeOutcome"));
    assert!(on_disk.contains("export interface PlanDetails"));
    assert!(on_disk.contains("export interface AppError"));
    assert!(on_disk.contains("export interface CreateExamInput"));
    assert!(on_disk.contains("export interface QuotaPreviewItem"));
    assert!(on_disk.contains("export interface RulePresetItem"));
}
