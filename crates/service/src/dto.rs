//! Data Transfer Objects (DTOs) for the ExamPanel service layer.

use exam_panel_core::domain::{
    Assignment, CampusId, Competency, ExamId, GradeId, GradeScope, LockKind, Placement, Plan,
    Problem, Role, RuleSetting, SchoolYearId, SubjectId, TeacherId, TeacherQuota,
};
use exam_panel_core::feasibility::FeasibilityReport;
use exam_panel_core::optimize::{OptimizeStats, RankedPlan};
use exam_panel_core::score::{RuleBound, ScoreReport};
use exam_panel_core::validate::Violation;
use serde::{Deserialize, Serialize};

/// High-level runtime application and database storage details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub is_portable: bool,
    pub db_path: String,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub identifier: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub mode: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub commit_hash: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub build_date: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub in_trial_mode: Option<bool>,
}

/// Global user-configurable interface and state settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub current_school_year_id: Option<SchoolYearId>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub school_name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub department_name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub signer_title: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub signer_name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub place_name: Option<String>,
}

/// Input parameters for creating a new campus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateCampusInput {
    pub code: String,
    pub name: String,
    pub color: String,
}

/// Input parameters for creating a new grade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateGradeInput {
    pub code: i32,
    pub name: String,
    pub sort_order: i32,
}

/// Input parameters for creating a new exam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateExamInput {
    pub school_year_id: SchoolYearId,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
}

/// Input parameters for creating a new subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateSubjectInput {
    pub school_year_id: SchoolYearId,
    pub code: String,
    pub name: String,
    pub color: String,
    pub sort_order: u32,
    pub setters: u8,
    pub reviewers: u8,
    pub min_campuses: u8,
}

/// Input parameters for updating an existing subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct UpdateSubjectInput {
    pub id: SubjectId,
    pub code: String,
    pub name: String,
    pub color: String,
    pub sort_order: u32,
    pub setters: u8,
    pub reviewers: u8,
    pub min_campuses: u8,
}

/// Input parameters for setting teacher competency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SetCompetencyInput {
    pub teacher_id: TeacherId,
    pub subject_id: SubjectId,
    pub role: Role,
    pub grade_scope: GradeScope,
}

/// Input parameters for deleting teacher competency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct DeleteCompetencyInput {
    pub teacher_id: TeacherId,
    pub subject_id: SubjectId,
    pub role: Role,
}

/// Input parameters for replacing all competencies of a teacher in a school year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ReplaceTeacherCompetenciesInput {
    pub teacher_id: TeacherId,
    pub school_year_id: SchoolYearId,
    pub competencies: Vec<Competency>,
}

/// Input parameters for creating a new teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateTeacherInput {
    pub full_name: String,
    pub campus_id: CampusId,
    #[serde(default = "default_load_weight")]
    pub load_weight: f64,
    #[serde(default = "default_true")]
    pub active: bool,
    pub note: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub code: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub display_name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub quota_override: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub max_tasks_per_exam_override: Option<u32>,
}

impl CreateTeacherInput {
    #[must_use]
    pub fn new(full_name: impl Into<String>, campus_id: CampusId) -> Self {
        Self {
            full_name: full_name.into(),
            campus_id,
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        }
    }
}

fn default_load_weight() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

/// Input parameters for creating a new school year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateSchoolYearInput {
    pub name: String,
    #[serde(default)]
    pub is_current: bool,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub copy_grades_from: Option<SchoolYearId>,
}

/// Input parameters for creating a manual PIN / FORBID lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CreateLockInput {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub subject_id: SubjectId,
    pub teacher_id: TeacherId,
    pub role: Option<Role>,
    pub kind: LockKind,
}

/// Complete problem snapshot accompanied by structurally forced placements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ProblemDetails {
    pub problem: Problem,
    pub forced: Vec<Placement>,
}

/// Termination budget for optimization runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[serde(tag = "type", content = "value")]
pub enum OptimizeBudget {
    Iterations(#[cfg_attr(feature = "typegen", ts(type = "number"))] u64),
    TimeMs(#[cfg_attr(feature = "typegen", ts(type = "number"))] u64),
}

/// Request parameters for starting an optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct OptimizeRequest {
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional, type = "number"))]
    pub base_seed: Option<u64>,
    #[serde(default = "default_runs")]
    pub runs: usize,
    pub budget: OptimizeBudget,
    #[serde(default = "default_k")]
    pub k: usize,
    #[serde(default)]
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub diversity_threshold: Option<f64>,
}

fn default_runs() -> usize {
    8
}

fn default_k() -> usize {
    3
}

/// Complete outcome produced by an optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct OptimizeOutcome {
    pub school_year_id: SchoolYearId,
    pub plans: Vec<RankedPlan>,
    pub initial_report: ScoreReport,
    pub stats: OptimizeStats,
    pub lower_bounds: Vec<RuleBound>,
    pub run_params_json: String,
}

/// Feasibility analysis report accompanied by availability-scaled quotas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct FeasibilityReportWithQuotas {
    pub report: FeasibilityReport,
    pub quotas: Vec<TeacherQuota>,
}

/// Evaluation output combining hard-constraint violations and soft-constraint scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct EvaluationOutcome {
    pub hard_violations: Vec<Violation>,
    pub score_report: ScoreReport,
}

/// Complete plan schedule details including stored report and assignments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PlanDetails {
    pub plan: Plan,
    pub assignments: Vec<Assignment>,
    #[cfg_attr(feature = "typegen", ts(optional))]
    pub score_report: Option<ScoreReport>,
}

/// A teacher's workload quota projection in the rules & quotas preview table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct QuotaPreviewItem {
    pub teacher_id: TeacherId,
    pub teacher_name: String,
    pub campus_id: CampusId,
    pub campus_name: String,
    pub load_weight: f64,
    pub available_exams: usize,
    pub quota: f64,
    pub lo: usize,
    pub hi: usize,
}

/// Input parameters for previewing workload quotas under unsaved rule configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PreviewQuotasInput {
    pub school_year_id: SchoolYearId,
    pub rule_settings: Vec<RuleSetting>,
}

/// A named rule configuration preset for soft constraint weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct RulePresetItem {
    pub id: String,
    pub name: String,
    pub settings: Vec<RuleSetting>,
}

/// Current plan validity and staleness status evaluated against current problem data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PlanStatus {
    pub data_changed: bool,
    pub rules_changed: bool,
    pub hard_violations_now: Vec<Violation>,
    pub score_now: ScoreReport,
}

/// Request parameters for re-optimizing around a plan with kept slots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ReoptimizeRequest {
    pub plan_id: exam_panel_core::domain::PlanId,
    pub keep: Vec<exam_panel_core::optimize::SlotRef>,
    pub request: OptimizeRequest,
}

// -----------------------------------------------------------------------------
// Backup & Restore DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct BackupValidationSummary {
    pub valid: bool,
    pub user_version: i32,
    pub school_years_count: usize,
    pub teachers_count: usize,
    pub plans_count: usize,
    pub error: Option<String>,
}

impl From<exam_panel_storage::BackupValidationSummary> for BackupValidationSummary {
    fn from(s: exam_panel_storage::BackupValidationSummary) -> Self {
        Self {
            valid: s.valid,
            user_version: s.user_version,
            school_years_count: s.school_years_count,
            teachers_count: s.teachers_count,
            plans_count: s.plans_count,
            error: s.error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct BackupFileInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

impl From<exam_panel_storage::BackupFileInfo> for BackupFileInfo {
    fn from(s: exam_panel_storage::BackupFileInfo) -> Self {
        Self {
            filename: s.filename,
            path: s.path,
            size_bytes: s.size_bytes,
            modified_at: s.modified_at,
        }
    }
}

// -----------------------------------------------------------------------------
// Excel Import DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub enum ImportRowStatus {
    #[serde(rename = "new")]
    New,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "unchanged")]
    Unchanged,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "skipped")]
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportCellError {
    pub sheet: String,
    pub row: usize,
    pub column: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CampusImportRow {
    pub row_index: usize,
    pub status: ImportRowStatus,
    pub code: String,
    pub name: String,
    pub errors: Vec<ImportCellError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct TeacherImportRow {
    pub row_index: usize,
    pub status: ImportRowStatus,
    pub code: Option<String>,
    pub full_name: String,
    pub display_name: Option<String>,
    pub campus_code: String,
    pub grades_str: String,
    pub grade_codes: Vec<i32>,
    pub load_weight: f64,
    pub active: bool,
    pub note: Option<String>,
    pub quota_override: Option<u32>,
    pub max_tasks_per_exam_override: Option<u32>,
    pub matched_teacher_id: Option<i64>,
    pub errors: Vec<ImportCellError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct UnavailabilityImportRow {
    pub row_index: usize,
    pub status: ImportRowStatus,
    pub teacher_ref: String,
    pub exam_code: String,
    pub reason: Option<String>,
    pub matched_teacher_id: Option<i64>,
    pub errors: Vec<ImportCellError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct SubjectImportRow {
    pub row_index: usize,
    pub status: ImportRowStatus,
    pub code: String,
    pub name: String,
    pub setters: u8,
    pub reviewers: u8,
    pub min_campuses: u8,
    pub color: String,
    pub errors: Vec<ImportCellError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct CompetencyImportRow {
    pub row_index: usize,
    pub status: ImportRowStatus,
    pub teacher_ref: String,
    pub subject_code: String,
    pub role: String,
    pub grade_scope: String,
    pub matched_teacher_id: Option<i64>,
    pub matched_subject_id: Option<i64>,
    pub errors: Vec<ImportCellError>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportSummaryCounts {
    pub new_count: usize,
    pub update_count: usize,
    pub unchanged_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct DeactivatedTeacherPreview {
    pub id: i64,
    pub code: Option<String>,
    pub full_name: String,
    pub campus_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportPreviewResult {
    pub mode: String,
    pub can_apply: bool,
    pub campuses: Vec<CampusImportRow>,
    pub teachers: Vec<TeacherImportRow>,
    pub unavailabilities: Vec<UnavailabilityImportRow>,
    pub subjects: Vec<SubjectImportRow>,
    pub competencies: Vec<CompetencyImportRow>,
    pub campuses_summary: ImportSummaryCounts,
    pub teachers_summary: ImportSummaryCounts,
    pub unavailabilities_summary: ImportSummaryCounts,
    pub subjects_summary: ImportSummaryCounts,
    pub competencies_summary: ImportSummaryCounts,
    pub deactivated_teachers: Vec<DeactivatedTeacherPreview>,
    pub feasibility_report: Option<FeasibilityReportWithQuotas>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ImportApplyResult {
    pub backup_path: String,
    pub campuses_created: usize,
    pub campuses_updated: usize,
    pub teachers_created: usize,
    pub teachers_updated: usize,
    pub teachers_deactivated: usize,
    pub unavailabilities_created: usize,
    pub subjects_created: usize,
    pub subjects_updated: usize,
    pub competencies_created: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PlanImportTeacherTotal {
    pub teacher_id: TeacherId,
    pub teacher_name: String,
    pub display_name: String,
    #[cfg_attr(feature = "typegen", ts(type = "number | null"))]
    pub file_total: Option<i64>,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub computed_total: i64,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub setter_count: i64,
    #[cfg_attr(feature = "typegen", ts(type = "number"))]
    pub reviewer_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct PlanImportPreview {
    pub assignments: Vec<Assignment>,
    pub teacher_totals: Vec<PlanImportTeacherTotal>,
    pub errors: Vec<ImportCellError>,
    pub warnings: Vec<String>,
    pub can_apply: bool,
    pub hard_violations: Vec<Violation>,
    pub score_report: Option<ScoreReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct ApplyPlanImportInput {
    pub school_year_id: SchoolYearId,
    pub plan_name: Option<String>,
    pub assignments: Vec<Assignment>,
}

/// Generates the complete TypeScript declaration file contents from Rust types.
#[cfg(feature = "typegen")]
#[must_use]
pub fn generate_typescript_declarations() -> String {
    let cfg = ts_rs::Config::default();
    let mut out = String::new();

    out.push_str("// This file is generated by ts-rs from Rust DTOs.\n");
    out.push_str("// Do not edit manually. Verify with `cargo test -p exam-panel-service --features typegen --test generate_types`.\n\n");

    macro_rules! export_type {
        ($t:ty) => {
            let decl = <$t as ts_rs::TS>::decl(&cfg);
            let trimmed = decl.trim();
            if !trimmed.is_empty() {
                out.push_str(&format!("export {}\n\n", trimmed));
            }
        };
    }

    // Core Enums & Basics
    export_type!(Role);
    export_type!(LockKind);
    export_type!(exam_panel_core::domain::RuleKey);
    export_type!(exam_panel_core::optimize::OptimizationEffort);

    // DTOs & Settings
    export_type!(AppInfo);
    export_type!(AppSettings);
    export_type!(crate::error::AppError);

    // Core Master Data
    export_type!(CreateCampusInput);
    export_type!(exam_panel_core::domain::Campus);
    export_type!(CreateGradeInput);
    export_type!(exam_panel_core::domain::Grade);
    export_type!(CreateSubjectInput);
    export_type!(UpdateSubjectInput);
    export_type!(exam_panel_core::domain::Subject);
    export_type!(exam_panel_core::domain::GradeScope);
    export_type!(exam_panel_core::domain::Competency);
    export_type!(SetCompetencyInput);
    export_type!(DeleteCompetencyInput);
    export_type!(ReplaceTeacherCompetenciesInput);
    export_type!(CreateTeacherInput);
    export_type!(exam_panel_core::domain::Teacher);
    export_type!(exam_panel_core::domain::TeacherGrade);
    export_type!(exam_panel_core::domain::TeacherWithGrades);
    export_type!(CreateSchoolYearInput);
    export_type!(exam_panel_core::domain::SchoolYear);
    export_type!(CreateExamInput);
    export_type!(exam_panel_core::domain::Exam);
    export_type!(exam_panel_core::domain::Unavailability);
    export_type!(CreateLockInput);
    export_type!(exam_panel_core::domain::Lock);
    export_type!(RuleSetting);

    // Previews & Quotas
    export_type!(QuotaPreviewItem);
    export_type!(PreviewQuotasInput);
    export_type!(RulePresetItem);

    // Feasibility & Diagnostics
    export_type!(exam_panel_core::domain::PanelKey);
    export_type!(exam_panel_core::domain::Placement);
    export_type!(exam_panel_core::domain::Problem);
    export_type!(ProblemDetails);
    export_type!(Violation);
    export_type!(TeacherQuota);
    export_type!(exam_panel_core::feasibility::Diagnostic);
    export_type!(FeasibilityReport);
    export_type!(FeasibilityReportWithQuotas);

    // Plan & Scores
    export_type!(Assignment);
    export_type!(exam_panel_core::score::SoftViolation);
    export_type!(exam_panel_core::score::TeacherStats);
    export_type!(exam_panel_core::score::RuleScore);
    export_type!(ScoreReport);
    export_type!(EvaluationOutcome);

    // Optimization & Progress
    export_type!(OptimizeBudget);
    export_type!(OptimizeRequest);
    export_type!(exam_panel_core::optimize::Progress);
    export_type!(RankedPlan);
    export_type!(OptimizeStats);
    export_type!(RuleBound);
    export_type!(OptimizeOutcome);

    // Plan Entities & Workspace
    export_type!(exam_panel_core::domain::PlanSummary);
    export_type!(Plan);
    export_type!(PlanDetails);
    export_type!(PlanStatus);
    export_type!(exam_panel_core::optimize::SlotRef);
    export_type!(exam_panel_core::optimize::CandidateEval);
    export_type!(ReoptimizeRequest);

    // Backup & Restore
    export_type!(BackupValidationSummary);
    export_type!(BackupFileInfo);

    // Excel Import
    export_type!(ImportRowStatus);
    export_type!(ImportCellError);
    export_type!(CampusImportRow);
    export_type!(TeacherImportRow);
    export_type!(UnavailabilityImportRow);
    export_type!(SubjectImportRow);
    export_type!(CompetencyImportRow);
    export_type!(ImportSummaryCounts);
    export_type!(DeactivatedTeacherPreview);
    export_type!(ImportPreviewResult);
    export_type!(ImportApplyResult);
    export_type!(PlanImportTeacherTotal);
    export_type!(PlanImportPreview);
    export_type!(ApplyPlanImportInput);

    out.replace("\r\n", "\n")
}
