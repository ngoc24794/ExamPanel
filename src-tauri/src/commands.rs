//! Tauri command handlers for ExamPanel.
//!
//! All commands are thin wrappers over `AppService`.

use exam_panel_core::domain::{
    Assignment, Campus, CampusId, Exam, ExamId, Grade, GradeId, Lock, LockId, PlanId, PlanSummary,
    RuleSetting, SchoolYear, SchoolYearId, Teacher, TeacherId, TeacherWithGrades, Unavailability,
};
use exam_panel_core::optimize::{CandidateEval, Progress, SlotRef};
use exam_panel_service::dto::{
    AppInfo, AppSettings, BackupFileInfo, BackupValidationSummary, CreateCampusInput,
    CreateExamInput, CreateGradeInput, CreateLockInput, CreateSchoolYearInput, CreateTeacherInput,
    EvaluationOutcome, FeasibilityReportWithQuotas, ImportApplyResult, ImportPreviewResult,
    OptimizeOutcome, OptimizeRequest, PlanDetails, PlanStatus, PreviewQuotasInput,
    QuotaPreviewItem, ReoptimizeRequest, RulePresetItem,
};
use exam_panel_service::error::AppError;
use exam_panel_service::service::AppService;
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::State;

// -----------------------------------------------------------------------------
// Ping / App Info / Settings
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn ping() -> String {
    format!(
        "pong from {} core v{}",
        crate::APP_NAME,
        exam_panel_core::core_version()
    )
}

#[tauri::command]
pub fn get_app_info(service: State<'_, Arc<AppService>>) -> Result<AppInfo, AppError> {
    service.get_app_info()
}

#[tauri::command]
pub fn get_settings(service: State<'_, Arc<AppService>>) -> Result<AppSettings, AppError> {
    service.get_settings()
}

#[tauri::command]
pub fn set_setting(
    service: State<'_, Arc<AppService>>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    service.set_setting(&key, &value)
}

// -----------------------------------------------------------------------------
// Campuses
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_campuses(service: State<'_, Arc<AppService>>) -> Result<Vec<Campus>, AppError> {
    service.list_campuses()
}

#[tauri::command]
pub fn create_campus(
    service: State<'_, Arc<AppService>>,
    input: CreateCampusInput,
) -> Result<Campus, AppError> {
    service.create_campus(input)
}

#[tauri::command]
pub fn update_campus(service: State<'_, Arc<AppService>>, campus: Campus) -> Result<(), AppError> {
    service.update_campus(campus)
}

#[tauri::command]
pub fn delete_campus(service: State<'_, Arc<AppService>>, id: CampusId) -> Result<(), AppError> {
    service.delete_campus(id)
}

// -----------------------------------------------------------------------------
// Grades
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_grades(service: State<'_, Arc<AppService>>) -> Result<Vec<Grade>, AppError> {
    service.list_grades()
}

#[tauri::command]
pub fn create_grade(
    service: State<'_, Arc<AppService>>,
    input: CreateGradeInput,
) -> Result<Grade, AppError> {
    service.create_grade(input)
}

#[tauri::command]
pub fn update_grade(service: State<'_, Arc<AppService>>, grade: Grade) -> Result<(), AppError> {
    service.update_grade(grade)
}

#[tauri::command]
pub fn delete_grade(service: State<'_, Arc<AppService>>, id: GradeId) -> Result<(), AppError> {
    service.delete_grade(id)
}

// -----------------------------------------------------------------------------
// Teachers
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_teachers(service: State<'_, Arc<AppService>>) -> Result<Vec<Teacher>, AppError> {
    service.list_teachers()
}

#[tauri::command]
pub fn create_teacher(
    service: State<'_, Arc<AppService>>,
    input: CreateTeacherInput,
) -> Result<Teacher, AppError> {
    service.create_teacher(input)
}

#[tauri::command]
pub fn update_teacher(
    service: State<'_, Arc<AppService>>,
    teacher: Teacher,
) -> Result<(), AppError> {
    service.update_teacher(teacher)
}

#[tauri::command]
pub fn delete_teacher(service: State<'_, Arc<AppService>>, id: TeacherId) -> Result<(), AppError> {
    service.delete_teacher(id)
}

#[tauri::command]
pub fn deactivate_teacher(
    service: State<'_, Arc<AppService>>,
    id: TeacherId,
) -> Result<(), AppError> {
    service.deactivate_teacher(id)
}

#[tauri::command]
pub fn set_teacher_grades(
    service: State<'_, Arc<AppService>>,
    teacher_id: TeacherId,
    school_year_id: SchoolYearId,
    grade_ids: Vec<GradeId>,
) -> Result<(), AppError> {
    service.set_teacher_grades(teacher_id, school_year_id, grade_ids)
}

#[tauri::command]
pub fn teachers_with_grades(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<TeacherWithGrades>, AppError> {
    service.teachers_with_grades(school_year_id)
}

// -----------------------------------------------------------------------------
// School Years
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_school_years(service: State<'_, Arc<AppService>>) -> Result<Vec<SchoolYear>, AppError> {
    service.list_school_years()
}

#[tauri::command]
pub fn create_school_year(
    service: State<'_, Arc<AppService>>,
    input: CreateSchoolYearInput,
) -> Result<SchoolYear, AppError> {
    service.create_school_year(input)
}

#[tauri::command]
pub fn set_current_school_year(
    service: State<'_, Arc<AppService>>,
    id: SchoolYearId,
) -> Result<(), AppError> {
    service.set_current_school_year(id)
}

// -----------------------------------------------------------------------------
// Exams
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_exams(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<Exam>, AppError> {
    service.list_exams(school_year_id)
}

#[tauri::command]
pub fn update_exam(service: State<'_, Arc<AppService>>, exam: Exam) -> Result<(), AppError> {
    service.update_exam(exam)
}

#[tauri::command]
pub fn create_exam(
    service: State<'_, Arc<AppService>>,
    input: CreateExamInput,
) -> Result<Exam, AppError> {
    service.create_exam(input)
}

#[tauri::command]
pub fn delete_exam(service: State<'_, Arc<AppService>>, id: ExamId) -> Result<(), AppError> {
    service.delete_exam(id)
}

#[tauri::command]
pub fn reorder_exams(
    service: State<'_, Arc<AppService>>,
    exam_ids: Vec<ExamId>,
) -> Result<(), AppError> {
    service.reorder_exams(exam_ids)
}

// -----------------------------------------------------------------------------
// Unavailability
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_unavailabilities(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<Unavailability>, AppError> {
    service.list_unavailabilities(school_year_id)
}

#[tauri::command]
pub fn set_unavailability(
    service: State<'_, Arc<AppService>>,
    unavailability: Unavailability,
) -> Result<(), AppError> {
    service.set_unavailability(unavailability)
}

#[tauri::command]
pub fn delete_unavailability(
    service: State<'_, Arc<AppService>>,
    teacher_id: TeacherId,
    exam_id: ExamId,
) -> Result<(), AppError> {
    service.delete_unavailability(teacher_id, exam_id)
}

// -----------------------------------------------------------------------------
// Locks
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn list_locks(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<Lock>, AppError> {
    service.list_locks(school_year_id)
}

#[tauri::command]
pub fn create_lock(
    service: State<'_, Arc<AppService>>,
    input: CreateLockInput,
) -> Result<Lock, AppError> {
    service.create_lock(input)
}

#[tauri::command]
pub fn delete_lock(service: State<'_, Arc<AppService>>, id: LockId) -> Result<(), AppError> {
    service.delete_lock(id)
}

// -----------------------------------------------------------------------------
// Rule Settings
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn get_rule_settings(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<RuleSetting>, AppError> {
    service.get_rule_settings(school_year_id)
}

#[tauri::command]
pub fn save_rule_settings(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    settings: Vec<RuleSetting>,
) -> Result<(), AppError> {
    service.save_rule_settings(school_year_id, settings)
}

#[tauri::command]
pub fn reset_rule_settings_to_defaults(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<(), AppError> {
    service.reset_rule_settings_to_defaults(school_year_id)
}

#[tauri::command]
pub fn preview_quotas(
    service: State<'_, Arc<AppService>>,
    input: PreviewQuotasInput,
) -> Result<Vec<QuotaPreviewItem>, AppError> {
    service.preview_quotas(input)
}

#[tauri::command]
pub fn get_rule_presets(service: State<'_, Arc<AppService>>) -> Vec<RulePresetItem> {
    service.get_rule_presets()
}

// -----------------------------------------------------------------------------
// Analysis
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn check_feasibility(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<FeasibilityReportWithQuotas, AppError> {
    service.check_feasibility(school_year_id)
}

#[tauri::command]
pub fn evaluate_assignments(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    assignments: Vec<Assignment>,
) -> Result<EvaluationOutcome, AppError> {
    service.evaluate_assignments(school_year_id, assignments)
}

// -----------------------------------------------------------------------------
// Optimization
// -----------------------------------------------------------------------------

#[tauri::command]
pub async fn start_optimize(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    request: OptimizeRequest,
    on_progress: Channel<Progress>,
) -> Result<OptimizeOutcome, AppError> {
    let service_clone = Arc::clone(&service);
    tauri::async_runtime::spawn_blocking(move || {
        let sink = Arc::new(move |p: Progress| {
            let _ = on_progress.send(p);
        });
        service_clone.run_optimize(school_year_id, request, Some(sink), None)
    })
    .await
    .map_err(|e| AppError::new("internal_error").with_param("detail", e.to_string()))?
}

#[tauri::command]
pub fn cancel_optimize(service: State<'_, Arc<AppService>>) -> bool {
    service.cancel_optimize()
}

// -----------------------------------------------------------------------------
// Plans
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn save_optimize_result(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    outcome: OptimizeOutcome,
) -> Result<Vec<PlanId>, AppError> {
    service.save_optimize_result(school_year_id, outcome)
}

#[tauri::command]
pub fn list_plans(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
) -> Result<Vec<PlanSummary>, AppError> {
    service.list_plans(school_year_id)
}

#[tauri::command]
pub fn get_plan(service: State<'_, Arc<AppService>>, id: PlanId) -> Result<PlanDetails, AppError> {
    service.get_plan(id)
}

#[tauri::command]
pub fn rename_plan(
    service: State<'_, Arc<AppService>>,
    id: PlanId,
    new_name: String,
) -> Result<(), AppError> {
    service.rename_plan(id, new_name)
}

#[tauri::command]
pub fn delete_plan(service: State<'_, Arc<AppService>>, id: PlanId) -> Result<(), AppError> {
    service.delete_plan(id)
}

#[tauri::command]
pub fn mark_final(service: State<'_, Arc<AppService>>, id: PlanId) -> Result<(), AppError> {
    service.mark_final(id)
}

#[tauri::command]
pub fn duplicate_plan(
    service: State<'_, Arc<AppService>>,
    id: PlanId,
    new_name: String,
) -> Result<PlanId, AppError> {
    service.duplicate_plan(id, new_name)
}

#[tauri::command]
pub fn plan_status(
    service: State<'_, Arc<AppService>>,
    id: PlanId,
) -> Result<PlanStatus, AppError> {
    service.plan_status(id)
}

#[tauri::command]
pub fn create_manual_copy(
    service: State<'_, Arc<AppService>>,
    id: PlanId,
    name: String,
) -> Result<PlanId, AppError> {
    service.create_manual_copy(id, name)
}

#[tauri::command]
pub fn update_plan_assignments(
    service: State<'_, Arc<AppService>>,
    id: PlanId,
    assignments: Vec<Assignment>,
) -> Result<EvaluationOutcome, AppError> {
    service.update_plan_assignments(id, assignments)
}

#[tauri::command]
pub fn evaluate_candidates(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    assignments: Vec<Assignment>,
    slot: SlotRef,
) -> Result<Vec<CandidateEval>, AppError> {
    service.evaluate_candidates(school_year_id, assignments, slot)
}

#[tauri::command]
pub fn evaluate_swap(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    assignments: Vec<Assignment>,
    slot_a: SlotRef,
    slot_b: SlotRef,
) -> Result<CandidateEval, AppError> {
    service.evaluate_swap(school_year_id, assignments, slot_a, slot_b)
}

#[tauri::command]
pub async fn reoptimize_from(
    service: State<'_, Arc<AppService>>,
    req: ReoptimizeRequest,
    on_progress: Channel<Progress>,
) -> Result<OptimizeOutcome, AppError> {
    let service_clone = Arc::clone(&service);
    tauri::async_runtime::spawn_blocking(move || {
        let sink = Arc::new(move |p: Progress| {
            let _ = on_progress.send(p);
        });
        service_clone.reoptimize_from(req.plan_id, req.keep, req.request, Some(sink), None)
    })
    .await
    .map_err(|e| AppError::new("internal_error").with_param("detail", e.to_string()))?
}

// -----------------------------------------------------------------------------
// Dev Tools
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn seed_demo(service: State<'_, Arc<AppService>>) -> Result<(), AppError> {
    #[cfg(feature = "dev-tools")]
    {
        service.seed_demo()
    }
    #[cfg(not(feature = "dev-tools"))]
    {
        Err(AppError::new("not_supported").with_param("detail", "dev-tools feature not enabled"))
    }
}

// -----------------------------------------------------------------------------
// Excel Import & Export
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn generate_import_template(
    service: State<'_, Arc<AppService>>,
    target_path: String,
) -> Result<(), AppError> {
    service.generate_import_template(std::path::Path::new(&target_path))
}

#[tauri::command]
pub fn preview_import(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    file_path: String,
    mode: String,
) -> Result<ImportPreviewResult, AppError> {
    service.preview_import(school_year_id, std::path::Path::new(&file_path), &mode)
}

#[tauri::command]
pub fn apply_import(
    service: State<'_, Arc<AppService>>,
    school_year_id: SchoolYearId,
    preview: ImportPreviewResult,
) -> Result<ImportApplyResult, AppError> {
    service.apply_import(school_year_id, &preview)
}

#[tauri::command]
pub fn export_plan_excel(
    service: State<'_, Arc<AppService>>,
    plan_id: PlanId,
    target_path: String,
) -> Result<(), AppError> {
    service.export_plan_excel(plan_id, std::path::Path::new(&target_path))
}

// -----------------------------------------------------------------------------
// Backup & Restore
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn backup_database(
    service: State<'_, Arc<AppService>>,
    target_path: String,
) -> Result<(), AppError> {
    service.backup_database(std::path::Path::new(&target_path))
}

#[tauri::command]
pub fn restore_database(
    service: State<'_, Arc<AppService>>,
    source_path: String,
) -> Result<(), AppError> {
    service.restore_database(std::path::Path::new(&source_path))
}

#[tauri::command]
pub fn validate_backup(
    service: State<'_, Arc<AppService>>,
    path: String,
) -> Result<BackupValidationSummary, AppError> {
    service.validate_backup(std::path::Path::new(&path))
}

#[tauri::command]
pub fn list_backups(service: State<'_, Arc<AppService>>) -> Result<Vec<BackupFileInfo>, AppError> {
    service.list_backups()
}
