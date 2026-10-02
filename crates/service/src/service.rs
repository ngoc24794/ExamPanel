//! Application service implementation orchestrating core algorithms and persistence.

use crate::dto::{
    AppInfo, AppSettings, CreateCampusInput, CreateGradeInput, CreateLockInput,
    CreateSchoolYearInput, CreateTeacherInput, EvaluationOutcome, FeasibilityReportWithQuotas,
    OptimizeBudget, OptimizeOutcome, OptimizeRequest, PlanDetails,
};
use crate::error::AppError;
use exam_panel_core::domain::{
    calculate_quotas, Assignment, Campus, CampusId, Exam, ExamId, Grade, GradeId, Lock, LockId,
    Plan, PlanId, PlanSummary, Problem, RuleSetting, SchoolYear, SchoolYearId, Teacher, TeacherId,
    TeacherWithGrades, Unavailability,
};
use exam_panel_core::feasibility::check_feasibility;
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions, Progress};
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
}

impl AppService {
    /// Creates a service wrapping the provided storage store.
    #[must_use]
    pub fn new(store: Store) -> Self {
        Self {
            store: Mutex::new(store),
            active_job: Mutex::new(None),
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
        Ok(Self::new(store))
    }

    // -------------------------------------------------------------------------
    // App & Settings
    // -------------------------------------------------------------------------

    pub fn get_app_info(&self) -> Result<AppInfo, AppError> {
        let data_dir = resolve_data_dir();
        let fallback = fallback_app_data_dir();
        let is_portable = data_dir != fallback;
        let db_path = resolve_database_path();

        Ok(AppInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: data_dir.to_string_lossy().to_string(),
            is_portable,
            db_path: db_path.to_string_lossy().to_string(),
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

        Ok(AppSettings {
            theme,
            language,
            current_school_year_id: current_sy,
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
        let teacher = store.create_teacher(
            &input.full_name,
            input.campus_id,
            input.load_weight,
            input.active,
            input.note.as_deref(),
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
        let mut batch = Vec::with_capacity(outcome.plans.len());

        for rp in outcome.plans {
            let score_json = serde_json::to_string(&rp.report)?;
            let plan = Plan {
                id: PlanId(0),
                school_year_id,
                name: format!("Kế hoạch #{}", rp.rank),
                created_at: String::new(),
                seed: rp.seed,
                score: Some(rp.report.total),
                is_final: false,
                rank: Some(rp.rank as u32),
                score_report_json: Some(score_json),
                run_params_json: Some(outcome.run_params_json.clone()),
                source: "optimizer".to_string(),
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
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        let plans = store.list_plans(school_year_id)?;
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

    pub fn mark_final(&self, id: PlanId) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        store.mark_final(id)?;
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

    // -------------------------------------------------------------------------
    // Dev Tools
    // -------------------------------------------------------------------------

    #[cfg(feature = "dev-tools")]
    pub fn seed_demo(&self) -> Result<(), AppError> {
        let store = self
            .store
            .lock()
            .map_err(|_| AppError::new("lock_poisoned"))?;
        exam_panel_storage::seed_demo(store.conn())?;
        Ok(())
    }

    #[cfg(feature = "dev-tools")]
    pub fn load_problem(&self, school_year_id: SchoolYearId) -> Result<Problem, AppError> {
        self.load_problem_snapshot(school_year_id)
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
