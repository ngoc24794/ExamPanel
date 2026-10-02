//! Data Transfer Objects (DTOs) for the ExamPanel service layer.

use exam_panel_core::domain::{
    Assignment, CampusId, ExamId, GradeId, LockKind, Plan, Role, SchoolYearId, TeacherId,
    TeacherQuota,
};
use exam_panel_core::feasibility::FeasibilityReport;
use exam_panel_core::optimize::{OptimizeStats, RankedPlan};
use exam_panel_core::score::{RuleBound, ScoreReport};
use exam_panel_core::validate::Violation;
use serde::{Deserialize, Serialize};

/// High-level runtime application and database storage details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub is_portable: bool,
    pub db_path: String,
}

/// Global user-configurable interface and state settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub current_school_year_id: Option<SchoolYearId>,
}

/// Input parameters for creating a new campus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateCampusInput {
    pub code: String,
    pub name: String,
    pub color: String,
}

/// Input parameters for creating a new grade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateGradeInput {
    pub code: i32,
    pub name: String,
    pub sort_order: i32,
}

/// Input parameters for creating a new teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSchoolYearInput {
    pub name: String,
    #[serde(default)]
    pub is_current: bool,
    pub copy_grades_from: Option<SchoolYearId>,
}

/// Input parameters for creating a manual PIN / FORBID lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLockInput {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub teacher_id: TeacherId,
    pub role: Option<Role>,
    pub kind: LockKind,
}

/// Termination budget for optimization runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OptimizeBudget {
    Iterations(u64),
    TimeMs(u64),
}

/// Request parameters for starting an optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizeRequest {
    pub base_seed: Option<u64>,
    #[serde(default = "default_runs")]
    pub runs: usize,
    pub budget: OptimizeBudget,
    #[serde(default = "default_k")]
    pub k: usize,
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
pub struct FeasibilityReportWithQuotas {
    pub report: FeasibilityReport,
    pub quotas: Vec<TeacherQuota>,
}

/// Evaluation output combining hard-constraint violations and soft-constraint scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationOutcome {
    pub hard_violations: Vec<Violation>,
    pub score_report: ScoreReport,
}

/// Complete plan schedule details including stored report and assignments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanDetails {
    pub plan: Plan,
    pub assignments: Vec<Assignment>,
    pub score_report: Option<ScoreReport>,
}
