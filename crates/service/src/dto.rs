//! Data Transfer Objects (DTOs) for the ExamPanel service layer.

use exam_panel_core::domain::{
    Assignment, CampusId, ExamId, GradeId, LockKind, Plan, Role, RuleSetting, SchoolYearId,
    TeacherId, TeacherQuota,
};
use exam_panel_core::feasibility::FeasibilityReport;
use exam_panel_core::optimize::{OptimizeStats, RankedPlan};
use exam_panel_core::score::{RuleBound, ScoreReport};
use exam_panel_core::validate::Violation;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// High-level runtime application and database storage details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub is_portable: bool,
    pub db_path: String,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub identifier: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub mode: Option<String>,
}

/// Global user-configurable interface and state settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub current_school_year_id: Option<SchoolYearId>,
}

/// Input parameters for creating a new campus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CreateCampusInput {
    pub code: String,
    pub name: String,
    pub color: String,
}

/// Input parameters for creating a new grade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CreateGradeInput {
    pub code: i32,
    pub name: String,
    pub sort_order: i32,
}

/// Input parameters for creating a new exam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CreateExamInput {
    pub school_year_id: SchoolYearId,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
}

/// Input parameters for creating a new teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CreateTeacherInput {
    pub full_name: String,
    pub campus_id: CampusId,
    #[serde(default = "default_load_weight")]
    pub load_weight: f64,
    #[serde(default = "default_true")]
    pub active: bool,
    pub note: Option<String>,
}

fn default_load_weight() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

/// Input parameters for creating a new school year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CreateSchoolYearInput {
    pub name: String,
    #[serde(default)]
    pub is_current: bool,
    #[serde(default)]
    #[ts(optional)]
    pub copy_grades_from: Option<SchoolYearId>,
}

/// Input parameters for creating a manual PIN / FORBID lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CreateLockInput {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub teacher_id: TeacherId,
    pub role: Option<Role>,
    pub kind: LockKind,
}

/// Termination budget for optimization runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "value")]
pub enum OptimizeBudget {
    Iterations(#[ts(type = "number")] u64),
    TimeMs(#[ts(type = "number")] u64),
}

/// Request parameters for starting an optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OptimizeRequest {
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub base_seed: Option<u64>,
    #[serde(default = "default_runs")]
    pub runs: usize,
    pub budget: OptimizeBudget,
    #[serde(default = "default_k")]
    pub k: usize,
    #[serde(default)]
    #[ts(optional)]
    pub diversity_threshold: Option<f64>,
}

fn default_runs() -> usize {
    8
}

fn default_k() -> usize {
    3
}

/// Complete outcome produced by an optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OptimizeOutcome {
    pub school_year_id: SchoolYearId,
    pub plans: Vec<RankedPlan>,
    pub initial_report: ScoreReport,
    pub stats: OptimizeStats,
    pub lower_bounds: Vec<RuleBound>,
    pub run_params_json: String,
}

/// Feasibility analysis report accompanied by availability-scaled quotas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FeasibilityReportWithQuotas {
    pub report: FeasibilityReport,
    pub quotas: Vec<TeacherQuota>,
}

/// Evaluation output combining hard-constraint violations and soft-constraint scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EvaluationOutcome {
    pub hard_violations: Vec<Violation>,
    pub score_report: ScoreReport,
}

/// Complete plan schedule details including stored report and assignments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PlanDetails {
    pub plan: Plan,
    pub assignments: Vec<Assignment>,
    #[ts(optional)]
    pub score_report: Option<ScoreReport>,
}

/// A teacher's workload quota projection in the rules & quotas preview table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreviewQuotasInput {
    pub school_year_id: SchoolYearId,
    pub rule_settings: Vec<RuleSetting>,
}

/// A named rule configuration preset for soft constraint weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RulePresetItem {
    pub id: String,
    pub name: String,
    pub settings: Vec<RuleSetting>,
}

/// Current plan validity and staleness status evaluated against current problem data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PlanStatus {
    pub problem_changed: bool,
    pub hard_violations_now: Vec<Violation>,
    pub score_now: ScoreReport,
}

/// Request parameters for re-optimizing around a plan with kept slots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReoptimizeRequest {
    pub plan_id: exam_panel_core::domain::PlanId,
    pub keep: Vec<exam_panel_core::optimize::SlotRef>,
    pub request: OptimizeRequest,
}

/// Generates the complete TypeScript declaration file contents from Rust types.
#[must_use]
pub fn generate_typescript_declarations() -> String {
    let cfg = ts_rs::Config::default();
    let mut out = String::new();

    out.push_str("// This file is generated by ts-rs from Rust DTOs.\n");
    out.push_str("// Do not edit manually. Verify with `cargo test -p exam-panel-service --test generate_types`.\n\n");

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
    export_type!(CreateTeacherInput);
    export_type!(exam_panel_core::domain::Teacher);
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

    out.replace("\r\n", "\n")
}
