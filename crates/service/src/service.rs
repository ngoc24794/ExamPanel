//! Application service implementation orchestrating core algorithms and persistence.

use crate::dto::{
    AppInfo, AppSettings, CreateCampusInput, CreateExamInput, CreateGradeInput, CreateLockInput,
    CreateSchoolYearInput, CreateSubjectInput, CreateTeacherInput, DeleteCompetencyInput,
    EvaluationOutcome, FeasibilityReportWithQuotas, OptimizeBudget, OptimizeOutcome,
    OptimizeRequest, PlanDetails, PlanStatus, PreviewQuotasInput, ProblemDetails, QuotaPreviewItem,
    ReplaceTeacherCompetenciesInput, RulePresetItem, SetCompetencyInput, UpdateSubjectInput,
};
use crate::error::AppError;
use exam_panel_core::domain::{
    calculate_quotas, Assignment, Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, Lock,
    LockId, LockKind, Plan, PlanId, PlanSummary, Problem, RulePreset, RuleSetting, SchoolYear,
    SchoolYearId, Subject, SubjectId, Teacher, TeacherId, TeacherWithGrades, Unavailability,
};
use exam_panel_core::feasibility::check_feasibility;
use exam_panel_core::optimize::{
    optimize, Budget, CandidateEval, OptimizeOptions, Progress, SlotRef,
};
use exam_panel_core::score::{bounds::lower_bounds, evaluate, ScoreReport};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use exam_panel_storage::paths::{fallback_app_data_dir, resolve_data_dir, resolve_database_path};
use exam_panel_storage::Store;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// Thread-safe central application service layer.
pub struct AppService {
    store: Mutex<Store>,
    active_job: Mutex<Option<Arc<AtomicBool>>>,
    is_trial_mode: AtomicBool,
    original_db_path: Mutex<Option<std::path::PathBuf>>,
}

impl AppService {
    /// Creates a service wrapping the provided storage store.
    #[must_use]
    pub fn new(store: Store) -> Self {
        let initial_path = store.path().map(|p| p.to_path_buf());
        Self {
            store: Mutex::new(store),
            active_job: Mutex::new(None),
            is_trial_mode: AtomicBool::new(false),
            original_db_path: Mutex::new(initial_path),
        }
    }

    /// Initializes a service with an in-memory SQLite store.
    pub fn open_in_memory() -> Result<Self, AppError> {
        let store = Store::open_in_memory()?;
        Ok(Self::new(store))
    }

    /// Initializes a service opening a database at the specified path.
    pub fn open_at<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let store = Store::open_at(path)?;
        Ok(Self::new(store))
    }

    /// Initializes a service opening the default portable/app-data database.
    pub fn open_default() -> Result<Self, AppError> {
        let store = Store::open_default()?;
        let service = Self::new(store);
        service.ensure_default_school_year()?;
        Ok(service)
    }

    /// Ensures that at least one school year exists.
    /// If no school years exist (fresh database), initializes "2026 - 2027" as current,
    /// complete with 4 default exams (GK1, CK1, GK2, CK2) and standard rule settings.
    pub fn ensure_default_school_year(&self) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let years = store.get_school_years()?;
        if years.is_empty() {
            let sy = store.create_school_year("2026 - 2027", None)?;
            store.set_current_school_year(sy.id)?;
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // App & Settings
    // -------------------------------------------------------------------------

    pub fn is_trial_mode(&self) -> bool {
        self.is_trial_mode.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn enter_trial_mode(&self) -> Result<(), AppError> {
        let demo_path = exam_panel_storage::paths::resolve_demo_database_path();
        let demo_exists = demo_path.exists();
        let mut store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;

        if let Some(orig) = store.path() {
            let mut orig_slot = self
                .original_db_path
                .lock()
                .map_err(|_| AppError::new("lock_poisoned"))?;
            *orig_slot = Some(orig.to_path_buf());
        }

        let demo_store = Store::open_at(&demo_path)
            .map_err(|e| AppError::internal(format!("Lỗi mở cơ sở dữ liệu dùng thử: {e}")))?;

        if !demo_exists || demo_store.get_teachers().map_or(0, |t| t.len()) == 0 {
            exam_panel_storage::seed_demo(demo_store.conn()).map_err(|e| {
                AppError::internal(format!(
                    "Lỗi nạp dữ liệu mẫu vào cơ sở dữ liệu dùng thử: {e}"
                ))
            })?;
        }

        *store = demo_store;
        self.is_trial_mode
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub fn exit_trial_mode(&self) -> Result<(), AppError> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;

        let real_path = {
            let orig_slot = self
                .original_db_path
                .lock()
                .map_err(|_| AppError::new("lock_poisoned"))?;
            orig_slot
                .clone()
                .unwrap_or_else(exam_panel_storage::paths::resolve_database_path)
        };

        let real_store = Store::open_at(&real_path)
            .map_err(|e| AppError::internal(format!("Lỗi mở lại cơ sở dữ liệu chính: {e}")))?;
        *store = real_store;
        self.is_trial_mode
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub fn get_app_info(&self) -> Result<AppInfo, AppError> {
        let data_dir = resolve_data_dir();
        let fallback = fallback_app_data_dir();
        let is_portable = data_dir != fallback;
        let in_trial = self.is_trial_mode();
        let db_path = if in_trial {
            exam_panel_storage::paths::resolve_demo_database_path()
        } else {
            resolve_database_path()
        };

        let commit_hash = option_env!("EXAMPANEL_COMMIT_HASH").map(|s| s.to_string());
        let build_date = option_env!("EXAMPANEL_BUILD_DATE").map(|s| s.to_string());

        Ok(AppInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: data_dir.to_string_lossy().to_string(),
            is_portable,
            db_path: db_path.to_string_lossy().to_string(),
            name: Some("ExamPanel".to_string()),
            identifier: Some("vn.exampanel.app".to_string()),
            mode: Some(if is_portable { "portable" } else { "installed" }.to_string()),
            commit_hash,
            build_date,
            in_trial_mode: Some(in_trial),
        })
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let theme = store
            .get_setting("theme")?
            .unwrap_or_else(|| "system".to_string());
        let language = store
            .get_setting("language")?
            .unwrap_or_else(|| "vi".to_string());
        let current_sy = store.get_current_school_year()?.map(|sy| sy.id);
        let school_name = store.get_setting("school_name")?;
        let department_name = store.get_setting("department_name")?;
        let signer_title = store.get_setting("signer_title")?;
        let signer_name = store.get_setting("signer_name")?;
        let place_name = store.get_setting("place_name")?;

        Ok(AppSettings {
            theme,
            language,
            current_school_year_id: current_sy,
            school_name,
            department_name,
            signer_title,
            signer_name,
            place_name,
        })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.set_setting(key, value)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Campuses
    // -------------------------------------------------------------------------

    pub fn list_campuses(&self) -> Result<Vec<Campus>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let campuses = store.get_campuses()?;
        Ok(campuses)
    }

    pub fn create_campus(&self, input: CreateCampusInput) -> Result<Campus, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let campus = store.create_campus(&input.code, &input.name, &input.color)?;
        Ok(campus)
    }

    pub fn update_campus(&self, campus: Campus) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.update_campus(&campus)?;
        Ok(())
    }

    pub fn delete_campus(&self, id: CampusId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_campus(id)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Grades
    // -------------------------------------------------------------------------

    pub fn list_grades(&self) -> Result<Vec<Grade>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let grades = store.get_grades()?;
        Ok(grades)
    }

    pub fn create_grade(&self, input: CreateGradeInput) -> Result<Grade, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let grade = store.create_grade(input.code, &input.name, input.sort_order)?;
        Ok(grade)
    }

    pub fn update_grade(&self, grade: Grade) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.update_grade(&grade)?;
        Ok(())
    }

    pub fn delete_grade(&self, id: GradeId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_grade(id)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Teachers
    // -------------------------------------------------------------------------

    pub fn list_teachers(&self) -> Result<Vec<Teacher>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let teachers = store.get_teachers()?;
        Ok(teachers)
    }

    pub fn create_teacher(&self, input: CreateTeacherInput) -> Result<Teacher, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let teacher = store.create_teacher_full(
            &input.full_name,
            input.campus_id,
            input.load_weight,
            input.active,
            input.note.as_deref(),
            input.code.as_deref(),
            input.display_name.as_deref(),
            input.quota_override,
            input.max_tasks_per_exam_override,
        )?;
        Ok(teacher)
    }

    pub fn update_teacher(&self, teacher: Teacher) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.update_teacher(&teacher)?;
        Ok(())
    }

    pub fn delete_teacher(&self, id: TeacherId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_teacher(id)?;
        Ok(())
    }

    pub fn deactivate_teacher(&self, id: TeacherId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.deactivate_teacher(id)?;
        Ok(())
    }

    pub fn set_teacher_grades(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
        grades: Vec<GradeId>,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.set_teacher_grades(teacher_id, school_year_id, &grades)?;
        Ok(())
    }

    pub fn teachers_with_grades(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<TeacherWithGrades>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let result = store.teachers_with_grades(school_year_id)?;
        Ok(result)
    }

    // -------------------------------------------------------------------------
    // School Years
    // -------------------------------------------------------------------------

    pub fn list_school_years(&self) -> Result<Vec<SchoolYear>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let years = store.get_school_years()?;
        Ok(years)
    }

    pub fn create_school_year(&self, input: CreateSchoolYearInput) -> Result<SchoolYear, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let sy = store.create_school_year(&input.name, input.copy_grades_from)?;
        if input.is_current {
            store.set_current_school_year(sy.id)?;
        }
        let created = store
            .get_school_year(sy.id)?
            .ok_or_else(|| AppError::new("not_found"))?;
        Ok(created)
    }

    pub fn set_current_school_year(&self, id: SchoolYearId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.set_current_school_year(id)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Exams
    // -------------------------------------------------------------------------

    pub fn list_exams(&self, school_year_id: SchoolYearId) -> Result<Vec<Exam>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let exams = store.get_exams(school_year_id)?;
        Ok(exams)
    }

    pub fn update_exam(&self, exam: Exam) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.update_exam(&exam)?;
        Ok(())
    }

    pub fn create_exam(&self, input: CreateExamInput) -> Result<Exam, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let exam = store.create_exam(
            input.school_year_id,
            &input.code,
            &input.name,
            input.sort_order,
        )?;
        Ok(exam)
    }

    pub fn delete_exam(&self, id: ExamId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_exam(id)?;
        Ok(())
    }

    pub fn reorder_exams(&self, exam_ids: Vec<ExamId>) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.reorder_exams(&exam_ids)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Subjects
    // -------------------------------------------------------------------------

    pub fn list_subjects(&self, school_year_id: SchoolYearId) -> Result<Vec<Subject>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let subjects = store.get_subjects(school_year_id)?;
        Ok(subjects)
    }

    pub fn create_subject(&self, input: CreateSubjectInput) -> Result<Subject, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let subject = store.create_subject(
            input.school_year_id,
            &input.code,
            &input.name,
            &input.color,
            input.sort_order,
            input.setters,
            input.reviewers,
            input.min_campuses,
        )?;
        Ok(subject)
    }

    pub fn update_subject(&self, input: UpdateSubjectInput) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let subject = Subject {
            id: input.id,
            code: input.code,
            name: input.name,
            color: input.color,
            sort_order: input.sort_order,
            setters: input.setters,
            reviewers: input.reviewers,
            min_campuses: input.min_campuses,
        };
        store.update_subject(&subject)?;
        Ok(())
    }

    pub fn delete_subject(&self, id: SubjectId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_subject(id)?;
        Ok(())
    }

    pub fn reorder_subjects(
        &self,
        school_year_id: SchoolYearId,
        ordered_ids: Vec<SubjectId>,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.reorder_subjects(school_year_id, &ordered_ids)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Teacher Competencies
    // -------------------------------------------------------------------------

    pub fn list_competencies(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Competency>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let competencies = store.get_competencies(school_year_id)?;
        Ok(competencies)
    }

    pub fn get_teacher_competencies(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Competency>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let competencies = store.get_teacher_competencies(teacher_id, school_year_id)?;
        Ok(competencies)
    }

    pub fn set_competency(&self, input: SetCompetencyInput) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.set_competency(
            input.teacher_id,
            input.subject_id,
            input.role,
            input.grade_scope,
        )?;
        Ok(())
    }

    pub fn delete_competency(&self, input: DeleteCompetencyInput) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_competency(input.teacher_id, input.subject_id, input.role)?;
        Ok(())
    }

    pub fn replace_teacher_competencies(
        &self,
        input: ReplaceTeacherCompetenciesInput,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.replace_teacher_competencies(
            input.teacher_id,
            input.school_year_id,
            &input.competencies,
        )?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Unavailability
    // -------------------------------------------------------------------------

    pub fn list_unavailabilities(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Unavailability>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let unavs = store.get_unavailabilities(school_year_id)?;
        Ok(unavs)
    }

    pub fn set_unavailability(&self, unavailability: Unavailability) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.set_unavailability(
            unavailability.teacher_id,
            unavailability.exam_id,
            unavailability.reason.as_deref(),
        )?;
        Ok(())
    }

    pub fn delete_unavailability(
        &self,
        teacher_id: TeacherId,
        exam_id: ExamId,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_unavailability(teacher_id, exam_id)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Locks
    // -------------------------------------------------------------------------

    pub fn list_locks(&self, school_year_id: SchoolYearId) -> Result<Vec<Lock>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let locks = store.get_locks(school_year_id)?;
        Ok(locks)
    }

    pub fn create_lock(&self, input: CreateLockInput) -> Result<Lock, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let lock = store.create_lock(
            input.exam_id,
            input.grade_id,
            input.subject_id,
            input.teacher_id,
            input.role,
            input.kind,
        )?;
        Ok(lock)
    }

    pub fn delete_lock(&self, id: LockId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_lock(id)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Rule Settings
    // -------------------------------------------------------------------------

    pub fn get_rule_settings(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<RuleSetting>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let settings = store.get_rule_settings(school_year_id)?;
        Ok(settings)
    }

    pub fn save_rule_settings(
        &self,
        school_year_id: SchoolYearId,
        settings: Vec<RuleSetting>,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        for s in settings {
            store.save_rule_setting(school_year_id, &s)?;
        }
        Ok(())
    }

    pub fn reset_rule_settings_to_defaults(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.reset_rule_settings_to_defaults(school_year_id)?;
        Ok(())
    }

    pub fn preview_quotas(
        &self,
        input: PreviewQuotasInput,
    ) -> Result<Vec<QuotaPreviewItem>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let mut problem = store.load_problem(input.school_year_id)?;
        problem.rule_settings = input.rule_settings;

        let quotas = calculate_quotas(&problem);
        let teacher_map: std::collections::HashMap<_, _> =
            problem.teachers.iter().map(|t| (t.id, t)).collect();
        let campus_map: std::collections::HashMap<_, _> =
            problem.campuses.iter().map(|c| (c.id, c)).collect();

        let items = quotas
            .into_iter()
            .filter_map(|q| {
                let t = teacher_map.get(&q.teacher_id)?;
                let c_name = campus_map
                    .get(&t.campus_id)
                    .map_or("", |c| c.name.as_str())
                    .to_string();
                Some(QuotaPreviewItem {
                    teacher_id: q.teacher_id,
                    teacher_name: t.full_name.clone(),
                    campus_id: t.campus_id,
                    campus_name: c_name,
                    load_weight: t.load_weight,
                    available_exams: q.available_exams,
                    quota: q.quota,
                    lo: q.lo,
                    hi: q.hi,
                })
            })
            .collect();

        Ok(items)
    }

    pub fn get_rule_presets(&self) -> Vec<RulePresetItem> {
        vec![
            RulePresetItem {
                id: "balanced".to_string(),
                name: "Cân bằng (mặc định)".to_string(),
                settings: RulePreset::Balanced.settings(),
            },
            RulePresetItem {
                id: "workload_fairness".to_string(),
                name: "Ưu tiên công bằng khối lượng".to_string(),
                settings: RulePreset::WorkloadFairness.settings(),
            },
            RulePresetItem {
                id: "team_diversity".to_string(),
                name: "Ưu tiên đa dạng ê-kíp".to_string(),
                settings: RulePreset::TeamDiversity.settings(),
            },
        ]
    }

    // -------------------------------------------------------------------------
    // Analysis
    // -------------------------------------------------------------------------

    pub fn check_feasibility(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<FeasibilityReportWithQuotas, AppError> {
        let problem = self.load_problem_snapshot(school_year_id)?;
        let quotas = calculate_quotas(&problem);
        let report = check_feasibility(&problem);
        Ok(FeasibilityReportWithQuotas { report, quotas })
    }

    pub fn evaluate_assignments(
        &self,
        school_year_id: SchoolYearId,
        assignments: Vec<Assignment>,
    ) -> Result<EvaluationOutcome, AppError> {
        let problem = self.load_problem_snapshot(school_year_id)?;
        let total_slots = problem.exams.len() * problem.grades.len() * 3;
        let hard_violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: !assignments.is_empty() && assignments.len() >= total_slots,
            },
        );
        let score_report = evaluate(&problem, &assignments);
        Ok(EvaluationOutcome {
            hard_violations,
            score_report,
        })
    }

    // -------------------------------------------------------------------------
    // Optimization
    // -------------------------------------------------------------------------

    /// Runs multi-plan optimization without holding the storage database lock.
    /// Rejects concurrent optimize calls with AppError "optimize_busy".
    pub fn run_optimize(
        &self,
        school_year_id: SchoolYearId,
        request: OptimizeRequest,
        progress_sink: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<OptimizeOutcome, AppError> {
        let job_cancel = cancel_flag.unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
        {
            let mut guard = self
                .active_job
                .lock()
                .map_err(|_| AppError::new("lock_poisoned"))?;
            if guard.is_some() {
                return Err(AppError::new("optimize_busy"));
            }
            *guard = Some(Arc::clone(&job_cancel));
        }

        struct JobGuard<'a>(&'a Mutex<Option<Arc<AtomicBool>>>);
        impl<'a> Drop for JobGuard<'a> {
            fn drop(&mut self) {
                if let Ok(mut g) = self.0.lock() {
                    *g = None;
                }
            }
        }
        let _guard = JobGuard(&self.active_job);

        // Load problem snapshot while holding lock, then release immediately
        let problem = self.load_problem_snapshot(school_year_id)?;

        let budget = match request.budget {
            OptimizeBudget::Iterations(it) => Budget::Iterations(it),
            OptimizeBudget::TimeMs(ms) => Budget::TimeMs(ms),
        };

        let opts = OptimizeOptions {
            base_seed: request.base_seed.unwrap_or(42),
            budget,
            num_runs: request.runs,
            max_plans: request.k,
            diversity_threshold: request.diversity_threshold.unwrap_or(0.20),
            cancel: Some(job_cancel),
            progress: progress_sink,
            initial_assignments: None,
        };

        let opt_res = optimize(&problem, &opts)?;
        let bounds = lower_bounds(&problem);

        let run_params_json = serde_json::json!({
            "app_version": env!("CARGO_PKG_VERSION"),
            "base_seed": opts.base_seed,
            "runs": opts.num_runs,
            "budget": request.budget,
            "k": opts.max_plans,
            "diversity_threshold": opts.diversity_threshold,
        })
        .to_string();

        Ok(OptimizeOutcome {
            school_year_id,
            plans: opt_res.plans,
            initial_report: opt_res.initial_report,
            stats: opt_res.stats,
            lower_bounds: bounds,
            run_params_json,
        })
    }

    /// Signals the active optimization job (if any) to cancel.
    pub fn cancel_optimize(&self) -> bool {
        if let Ok(guard) = self.active_job.lock() {
            if let Some(flag) = guard.as_ref() {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    /// Checks whether an optimization job is currently running.
    pub fn is_optimizing(&self) -> bool {
        self.active_job.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    // -------------------------------------------------------------------------
    // Plans
    // -------------------------------------------------------------------------

    pub fn save_optimize_result(
        &self,
        school_year_id: SchoolYearId,
        outcome: OptimizeOutcome,
    ) -> Result<Vec<PlanId>, AppError> {
        let snapshot = self.load_problem_snapshot(school_year_id)?;
        let data_hash = snapshot.data_hash();
        let rules_hash = snapshot.rules_hash();
        let mut batch = Vec::with_capacity(outcome.plans.len());

        for rp in outcome.plans {
            let score_json = serde_json::to_string(&rp.report)?;
            let plan = Plan {
                id: PlanId(0),
                school_year_id,
                name: format!("Phương án #{}", rp.rank),
                created_at: String::new(),
                seed: rp.seed,
                score: Some(rp.report.total),
                is_final: false,
                rank: Some(rp.rank as u32),
                score_report_json: Some(score_json),
                run_params_json: Some(outcome.run_params_json.clone()),
                source: "optimizer".to_string(),
                data_hash: Some(data_hash.clone()),
                rules_hash: Some(rules_hash.clone()),
            };
            batch.push((plan, rp.assignments));
        }

        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let ids = store.save_plans_batch(&batch)?;
        Ok(ids)
    }

    pub fn list_plans(&self, school_year_id: SchoolYearId) -> Result<Vec<PlanSummary>, AppError> {
        let current_data_hash = self.load_problem_snapshot(school_year_id)?.data_hash();
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let mut plans = store.list_plans(school_year_id)?;
        for p in &mut plans {
            p.is_stale = p.data_hash.as_deref() != Some(&current_data_hash);
        }
        Ok(plans)
    }

    pub fn get_plan(&self, id: PlanId) -> Result<PlanDetails, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let (plan, assignments) = store.load_plan(id)?;
        let score_report = if let Some(ref json) = plan.score_report_json {
            serde_json::from_str::<ScoreReport>(json).ok()
        } else {
            None
        };

        Ok(PlanDetails {
            plan,
            assignments,
            score_report,
        })
    }

    pub fn rename_plan(&self, id: PlanId, new_name: String) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.rename_plan(id, &new_name)?;
        Ok(())
    }

    pub fn delete_plan(&self, id: PlanId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.delete_plan(id)?;
        Ok(())
    }

    pub fn duplicate_plan(&self, id: PlanId, new_name: String) -> Result<PlanId, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let new_id = store.duplicate_plan(id, &new_name)?;
        Ok(new_id)
    }

    pub fn plan_status(&self, plan_id: PlanId) -> Result<PlanStatus, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let (plan, assignments) = store.load_plan(plan_id)?;
        let current_problem = store.load_problem(plan.school_year_id)?;
        drop(store);

        let current_data_hash = current_problem.data_hash();
        let current_rules_hash = current_problem.rules_hash();
        let data_changed = plan.data_hash.as_deref() != Some(&current_data_hash);
        let rules_changed = plan.rules_hash.as_deref() != Some(&current_rules_hash);
        let total_slots = current_problem.exams.len() * current_problem.grades.len() * 3;
        let hard_violations_now = validate_assignments(
            &current_problem,
            &assignments,
            &ValidateOptions {
                require_complete: !assignments.is_empty() && assignments.len() >= total_slots,
            },
        );
        let score_now = evaluate(&current_problem, &assignments);
        Ok(PlanStatus {
            data_changed,
            rules_changed,
            hard_violations_now,
            score_now,
        })
    }

    pub fn create_manual_copy(&self, plan_id: PlanId, name: String) -> Result<PlanId, AppError> {
        self.duplicate_plan(plan_id, name)
    }

    pub fn update_plan_assignments(
        &self,
        plan_id: PlanId,
        assignments: Vec<Assignment>,
    ) -> Result<EvaluationOutcome, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let (mut plan, _) = store.load_plan(plan_id)?;
        if plan.source != "manual" && plan.source != "duplicate" {
            return Err(AppError::new("plan_immutable"));
        }
        if plan.is_final {
            return Err(AppError::new("plan_final"));
        }
        let problem = store.load_problem(plan.school_year_id)?;
        let total_slots = problem.exams.len() * problem.grades.len() * 3;
        let hard_violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: !assignments.is_empty() && assignments.len() >= total_slots,
            },
        );
        let score_report = evaluate(&problem, &assignments);
        plan.score = Some(score_report.total);
        plan.score_report_json = Some(serde_json::to_string(&score_report)?);
        plan.data_hash = Some(problem.data_hash());
        plan.rules_hash = Some(problem.rules_hash());
        store.save_plan(&plan, &assignments)?;
        Ok(EvaluationOutcome {
            hard_violations,
            score_report,
        })
    }

    pub fn mark_final(&self, id: PlanId) -> Result<(), AppError> {
        let status = self.plan_status(id)?;
        if status.data_changed {
            return Err(AppError::new("plan_stale"));
        }
        if !status.hard_violations_now.is_empty() {
            return Err(AppError::new("plan_invalid"));
        }
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.mark_final(id)?;
        Ok(())
    }

    pub fn evaluate_candidates(
        &self,
        school_year_id: SchoolYearId,
        assignments: Vec<Assignment>,
        slot: SlotRef,
    ) -> Result<Vec<CandidateEval>, AppError> {
        let problem = self.load_problem_snapshot(school_year_id)?;
        Ok(exam_panel_core::optimize::evaluate_candidates(
            &problem,
            &assignments,
            slot,
        ))
    }

    pub fn evaluate_swap(
        &self,
        school_year_id: SchoolYearId,
        assignments: Vec<Assignment>,
        slot_a: SlotRef,
        slot_b: SlotRef,
    ) -> Result<CandidateEval, AppError> {
        let problem = self.load_problem_snapshot(school_year_id)?;
        Ok(exam_panel_core::optimize::evaluate_swap(
            &problem,
            &assignments,
            slot_a,
            slot_b,
        ))
    }

    pub fn reoptimize_from(
        &self,
        plan_id: PlanId,
        keep: Vec<SlotRef>,
        request: OptimizeRequest,
        progress_sink: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Result<OptimizeOutcome, AppError> {
        let job_cancel = cancel_flag.unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
        {
            let mut guard = self
                .active_job
                .lock()
                .map_err(|_| AppError::new("lock_poisoned"))?;
            if guard.is_some() {
                return Err(AppError::new("optimize_busy"));
            }
            *guard = Some(Arc::clone(&job_cancel));
        }

        let res = (|| -> Result<OptimizeOutcome, AppError> {
            let (plan, assignments) = {
                let store = self
                    .store
                    .lock()
                    .map_err(|_| AppError::new("lock_poisoned"))?;
                store.load_plan(plan_id)?
            };
            let mut problem = self.load_problem_snapshot(plan.school_year_id)?;

            // Temporarily pin the kept slots without persisting locks
            for slot in &keep {
                for a in &assignments {
                    if a.exam_id == slot.exam_id
                        && a.grade_id == slot.grade_id
                        && a.subject_id == slot.subject_id
                        && a.role == slot.role
                        && a.position == slot.position
                    {
                        problem.locks.push(exam_panel_core::domain::Lock {
                            id: exam_panel_core::domain::LockId(0),
                            exam_id: slot.exam_id,
                            grade_id: slot.grade_id,
                            subject_id: slot.subject_id,
                            teacher_id: a.teacher_id,
                            role: Some(slot.role),
                            kind: LockKind::Pin,
                        });
                        break;
                    }
                }
            }

            let budget = match request.budget {
                OptimizeBudget::Iterations(it) => Budget::Iterations(it),
                OptimizeBudget::TimeMs(ms) => Budget::TimeMs(ms),
            };

            let opt_options = OptimizeOptions {
                base_seed: request.base_seed.unwrap_or(42),
                budget,
                num_runs: request.runs,
                max_plans: request.k,
                diversity_threshold: request.diversity_threshold.unwrap_or(0.20),
                cancel: Some(Arc::clone(&job_cancel)),
                progress: progress_sink,
                initial_assignments: Some(assignments),
            };

            let opt_res = optimize(&problem, &opt_options)?;
            let lb = lower_bounds(&problem);
            let run_params_json = serde_json::to_string(&request)?;

            Ok(OptimizeOutcome {
                school_year_id: plan.school_year_id,
                plans: opt_res.plans,
                initial_report: opt_res.initial_report,
                stats: opt_res.stats,
                lower_bounds: lb,
                run_params_json,
            })
        })();

        {
            let mut guard = self
                .active_job
                .lock()
                .map_err(|_| AppError::new("lock_poisoned"))?;
            *guard = None;
        }

        res
    }

    // -------------------------------------------------------------------------
    // Dev Tools & Helpers
    // -------------------------------------------------------------------------

    pub fn seed_demo(&self) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        exam_panel_storage::seed_demo(store.conn())?;
        Ok(())
    }

    pub fn load_problem(&self, school_year_id: SchoolYearId) -> Result<Problem, AppError> {
        self.load_problem_snapshot(school_year_id)
    }

    pub fn get_problem_details(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<ProblemDetails, AppError> {
        let problem = self.load_problem_snapshot(school_year_id)?;
        let forced = exam_panel_core::domain::find_forced_placements(&problem).unwrap_or_default();
        Ok(ProblemDetails { problem, forced })
    }

    // -------------------------------------------------------------------------
    // Excel Import & Export
    // -------------------------------------------------------------------------

    pub fn generate_import_template(&self, target_path: &Path) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let campuses = store.get_campuses()?;
        crate::excel::template::generate_import_template(target_path, &campuses)
            .map_err(|e| AppError::internal(format!("Lỗi tạo biểu mẫu Excel: {e}")))?;
        Ok(())
    }

    pub fn preview_import(
        &self,
        school_year_id: SchoolYearId,
        file_path: &Path,
        mode: &str,
    ) -> Result<crate::dto::ImportPreviewResult, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        crate::excel::import::preview_import(&store, school_year_id, file_path, mode)
    }

    pub fn apply_import(
        &self,
        school_year_id: SchoolYearId,
        preview: &crate::dto::ImportPreviewResult,
    ) -> Result<crate::dto::ImportApplyResult, AppError> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        crate::excel::import::apply_import(&mut store, school_year_id, preview)
    }

    pub fn export_plan_excel(&self, plan_id: PlanId, target_path: &Path) -> Result<(), AppError> {
        let plan_details = self.get_plan(plan_id)?;
        let school_year_id = plan_details.plan.school_year_id;

        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let school_year = store
            .get_school_years()?
            .into_iter()
            .find(|sy| sy.id == school_year_id)
            .ok_or_else(|| AppError::not_found("Không tìm thấy năm học"))?;
        let campuses = store.get_campuses()?;
        let grades = store.get_grades()?;
        let exams = store.get_exams(school_year_id)?;
        let teachers = store.get_teachers()?;
        let rule_settings = store.get_rule_settings(school_year_id)?;
        drop(store);

        let settings = self.get_settings()?;

        crate::excel::export::export_plan_workbook(
            target_path,
            &plan_details,
            &school_year,
            &campuses,
            &grades,
            &exams,
            &teachers,
            &settings,
            &rule_settings,
        )
        .map_err(|e| AppError::internal(format!("Lỗi xuất phương án Excel: {e}")))?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Backup & Restore
    // -------------------------------------------------------------------------

    pub fn backup_database(&self, target_path: &Path) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store
            .backup_to(target_path)
            .map_err(|e| AppError::internal(format!("Lỗi sao lưu cơ sở dữ liệu: {e}")))?;
        Ok(())
    }

    pub fn restore_database(&self, source_path: &Path) -> Result<(), AppError> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store
            .restore_from(source_path)
            .map_err(|e| AppError::internal(format!("Lỗi khôi phục cơ sở dữ liệu: {e}")))?;
        Ok(())
    }

    pub fn validate_backup(
        &self,
        path: &Path,
    ) -> Result<crate::dto::BackupValidationSummary, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let summary = store
            .validate_backup(path)
            .map_err(|e| AppError::internal(format!("Lỗi kiểm tra tệp sao lưu: {e}")))?;
        Ok(summary.into())
    }

    pub fn list_backups(&self) -> Result<Vec<crate::dto::BackupFileInfo>, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let list = store
            .list_backups()
            .map_err(|e| AppError::internal(format!("Lỗi lấy danh sách sao lưu: {e}")))?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    // -------------------------------------------------------------------------
    // Internal Helpers
    // -------------------------------------------------------------------------

    fn load_problem_snapshot(&self, school_year_id: SchoolYearId) -> Result<Problem, AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let problem = store.load_problem(school_year_id)?;
        Ok(problem)
    }
}
