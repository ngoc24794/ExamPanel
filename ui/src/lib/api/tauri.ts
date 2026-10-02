import { Channel, invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import type {
  AppInfo,
  AppSettings,
  Assignment,
  BackupFileInfo,
  BackupValidationSummary,
  Campus,
  CandidateEval,
  Competency,
  CreateCampusInput,
  CreateExamInput,
  CreateGradeInput,
  CreateLockInput,
  CreateSchoolYearInput,
  CreateSubjectInput,
  CreateTeacherInput,
  DeleteCompetencyInput,
  EvaluationOutcome,
  Exam,
  ExamPanelApi,
  FeasibilityReportWithQuotas,
  Grade,
  ImportApplyResult,
  ImportPreviewResult,
  Lock,
  OptimizeHandle,
  OptimizeOutcome,
  OptimizeRequest,
  PlanDetails,
  PlanStatus,
  PlanSummary,
  PreviewQuotasInput,
  ProblemDetails,
  Progress,
  QuotaPreviewItem,
  ReoptimizeRequest,
  ReplaceTeacherCompetenciesInput,
  RulePresetItem,
  RuleSetting,
  SchoolYear,
  SetCompetencyInput,
  SlotRef,
  Subject,
  Teacher,
  TeacherWithGrades,
  ThemeMode,
  Unavailability,
  UpdateSubjectInput,
} from './types'

export class TauriExamPanelApi implements ExamPanelApi {
  // App & Settings
  async ping(): Promise<string> {
    return await invoke<string>('ping')
  }

  async getAppInfo(): Promise<AppInfo> {
    const info = await invoke<AppInfo>('get_app_info')
    return {
      ...info,
      name: 'ExamPanel',
      identifier: 'vn.exampanel.app',
      mode: 'tauri',
    }
  }

  async getSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>('get_settings')
  }

  async setSetting(key: string, value: string): Promise<void> {
    await invoke<void>('set_setting', { key, value })
  }

  async getTheme(): Promise<ThemeMode | null> {
    const settings = await this.getSettings()
    const theme = settings.theme
    if (theme === 'light' || theme === 'dark' || theme === 'system') {
      return theme
    }
    return null
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    await this.setSetting('theme', theme)
  }

  async getLanguage(): Promise<string> {
    const settings = await this.getSettings()
    return settings.language || 'vi'
  }

  async setLanguage(lang: string): Promise<void> {
    await this.setSetting('language', lang)
  }

  async openDataFolder(): Promise<void> {
    const info = await this.getAppInfo()
    await openPath(info.data_dir)
  }

  async openLogFolder(): Promise<void> {
    await invoke<void>('open_log_folder')
  }

  async enterTrialMode(): Promise<void> {
    await invoke<void>('enter_trial_mode')
  }

  async exitTrialMode(): Promise<void> {
    await invoke<void>('exit_trial_mode')
  }

  // Campuses
  async listCampuses(): Promise<Campus[]> {
    return await invoke<Campus[]>('list_campuses')
  }

  async createCampus(input: CreateCampusInput): Promise<Campus> {
    return await invoke<Campus>('create_campus', { input })
  }

  async updateCampus(campus: Campus): Promise<void> {
    await invoke<void>('update_campus', { campus })
  }

  async deleteCampus(id: number): Promise<void> {
    await invoke<void>('delete_campus', { id })
  }

  // Grades
  async listGrades(): Promise<Grade[]> {
    return await invoke<Grade[]>('list_grades')
  }

  async createGrade(input: CreateGradeInput): Promise<Grade> {
    return await invoke<Grade>('create_grade', { input })
  }

  async updateGrade(grade: Grade): Promise<void> {
    await invoke<void>('update_grade', { grade })
  }

  async deleteGrade(id: number): Promise<void> {
    await invoke<void>('delete_grade', { id })
  }

  // Teachers
  async listTeachers(): Promise<Teacher[]> {
    return await invoke<Teacher[]>('list_teachers')
  }

  async createTeacher(input: CreateTeacherInput): Promise<Teacher> {
    return await invoke<Teacher>('create_teacher', { input })
  }

  async updateTeacher(teacher: Teacher): Promise<void> {
    await invoke<void>('update_teacher', { teacher })
  }

  async deleteTeacher(id: number): Promise<void> {
    await invoke<void>('delete_teacher', { id })
  }

  async deactivateTeacher(id: number): Promise<void> {
    await invoke<void>('deactivate_teacher', { id })
  }

  async setTeacherGrades(
    teacherId: number,
    schoolYearId: number,
    gradeIds: number[],
  ): Promise<void> {
    await invoke<void>('set_teacher_grades', {
      teacherId,
      schoolYearId,
      gradeIds,
    })
  }

  async teachersWithGrades(schoolYearId: number): Promise<TeacherWithGrades[]> {
    return await invoke<TeacherWithGrades[]>('teachers_with_grades', {
      schoolYearId,
    })
  }

  // School Years
  async listSchoolYears(): Promise<SchoolYear[]> {
    return await invoke<SchoolYear[]>('list_school_years')
  }

  async createSchoolYear(input: CreateSchoolYearInput): Promise<SchoolYear> {
    return await invoke<SchoolYear>('create_school_year', { input })
  }

  async setCurrentSchoolYear(id: number): Promise<void> {
    await invoke<void>('set_current_school_year', { id })
  }

  // Exams
  async listExams(schoolYearId: number): Promise<Exam[]> {
    return await invoke<Exam[]>('list_exams', { schoolYearId })
  }

  async createExam(input: CreateExamInput): Promise<Exam> {
    return await invoke<Exam>('create_exam', { input })
  }

  async updateExam(exam: Exam): Promise<void> {
    await invoke<void>('update_exam', { exam })
  }

  async deleteExam(id: number): Promise<void> {
    await invoke<void>('delete_exam', { id })
  }

  async reorderExams(examIds: number[]): Promise<void> {
    await invoke<void>('reorder_exams', { examIds })
  }

  // Subjects
  async listSubjects(schoolYearId: number): Promise<Subject[]> {
    return await invoke<Subject[]>('list_subjects', { schoolYearId })
  }

  async createSubject(input: CreateSubjectInput): Promise<Subject> {
    return await invoke<Subject>('create_subject', { input })
  }

  async updateSubject(input: UpdateSubjectInput): Promise<void> {
    await invoke<void>('update_subject', { input })
  }

  async deleteSubject(id: number): Promise<void> {
    await invoke<void>('delete_subject', { id })
  }

  async reorderSubjects(schoolYearId: number, subjectIds: number[]): Promise<void> {
    await invoke<void>('reorder_subjects', { schoolYearId, subjectIds })
  }

  // Competencies
  async listCompetencies(schoolYearId: number): Promise<Competency[]> {
    return await invoke<Competency[]>('list_competencies', { schoolYearId })
  }

  async getTeacherCompetencies(
    teacherId: number,
    schoolYearId: number,
  ): Promise<Competency[]> {
    return await invoke<Competency[]>('get_teacher_competencies', {
      teacherId,
      schoolYearId,
    })
  }

  async setCompetency(input: SetCompetencyInput): Promise<void> {
    await invoke<void>('set_competency', { input })
  }

  async deleteCompetency(input: DeleteCompetencyInput): Promise<void> {
    await invoke<void>('delete_competency', { input })
  }

  async replaceTeacherCompetencies(
    input: ReplaceTeacherCompetenciesInput,
  ): Promise<void> {
    await invoke<void>('replace_teacher_competencies', { input })
  }

  // Unavailability
  async listUnavailabilities(schoolYearId: number): Promise<Unavailability[]> {
    return await invoke<Unavailability[]>('list_unavailabilities', {
      schoolYearId,
    })
  }

  async setUnavailability(unavailability: Unavailability): Promise<void> {
    await invoke<void>('set_unavailability', { unavailability })
  }

  async deleteUnavailability(teacherId: number, examId: number): Promise<void> {
    await invoke<void>('delete_unavailability', { teacherId, examId })
  }

  // Locks
  async listLocks(schoolYearId: number): Promise<Lock[]> {
    return await invoke<Lock[]>('list_locks', { schoolYearId })
  }

  async createLock(input: CreateLockInput): Promise<Lock> {
    return await invoke<Lock>('create_lock', { input })
  }

  async deleteLock(id: number): Promise<void> {
    await invoke<void>('delete_lock', { id })
  }

  // Rule Settings
  async getRuleSettings(schoolYearId: number): Promise<RuleSetting[]> {
    return await invoke<RuleSetting[]>('get_rule_settings', {
      schoolYearId,
    })
  }

  async saveRuleSettings(schoolYearId: number, settings: RuleSetting[]): Promise<void> {
    await invoke<void>('save_rule_settings', { schoolYearId, settings })
  }

  async resetRuleSettingsToDefaults(schoolYearId: number): Promise<void> {
    await invoke<void>('reset_rule_settings_to_defaults', { schoolYearId })
  }

  async getRulePresets(): Promise<RulePresetItem[]> {
    return await invoke<RulePresetItem[]>('get_rule_presets')
  }

  async previewQuotas(input: PreviewQuotasInput): Promise<QuotaPreviewItem[]> {
    return await invoke<QuotaPreviewItem[]>('preview_quotas', { input })
  }

  // Analysis
  async getProblemDetails(schoolYearId: number): Promise<ProblemDetails> {
    return await invoke<ProblemDetails>('get_problem_details', { schoolYearId })
  }

  async checkFeasibility(schoolYearId: number): Promise<FeasibilityReportWithQuotas> {
    const res = await invoke<FeasibilityReportWithQuotas>('check_feasibility', {
      schoolYearId,
    })
    res.report.is_feasible = res.report.errors.length === 0
    return res
  }

  async evaluateAssignments(
    schoolYearId: number,
    assignments: Assignment[],
  ): Promise<EvaluationOutcome> {
    return await invoke<EvaluationOutcome>('evaluate_assignments', {
      schoolYearId,
      assignments,
    })
  }

  // Optimization
  startOptimize(
    schoolYearId: number,
    request: OptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle {
    const channel = new Channel<Progress>()
    if (onProgress) {
      channel.onmessage = (message) => {
        onProgress(message)
      }
    }

    const promise = invoke<OptimizeOutcome>('start_optimize', {
      schoolYearId,
      request,
      onProgress: channel,
    })

    return {
      promise,
      cancel: () => {
        void this.cancelOptimize()
      },
    }
  }

  async cancelOptimize(): Promise<boolean> {
    return await invoke<boolean>('cancel_optimize')
  }

  // Plans
  async saveOptimizeResult(
    schoolYearId: number,
    outcome: OptimizeOutcome,
  ): Promise<number[]> {
    return await invoke<number[]>('save_optimize_result', {
      schoolYearId,
      outcome,
    })
  }

  async listPlans(schoolYearId: number): Promise<PlanSummary[]> {
    return await invoke<PlanSummary[]>('list_plans', { schoolYearId })
  }

  async getPlan(id: number): Promise<PlanDetails> {
    return await invoke<PlanDetails>('get_plan', { id })
  }

  async renamePlan(id: number, newName: string): Promise<void> {
    await invoke<void>('rename_plan', { id, newName })
  }

  async deletePlan(id: number): Promise<void> {
    await invoke<void>('delete_plan', { id })
  }

  async markFinal(id: number): Promise<void> {
    await invoke<void>('mark_final', { id })
  }

  async duplicatePlan(id: number, newName: string): Promise<number> {
    return await invoke<number>('duplicate_plan', { id, newName })
  }

  async planStatus(id: number): Promise<PlanStatus> {
    return await invoke<PlanStatus>('plan_status', { id })
  }

  async createManualCopy(id: number, name: string): Promise<number> {
    return await invoke<number>('create_manual_copy', { id, name })
  }

  async updatePlanAssignments(
    id: number,
    assignments: Assignment[],
  ): Promise<EvaluationOutcome> {
    return await invoke<EvaluationOutcome>('update_plan_assignments', {
      id,
      assignments,
    })
  }

  async evaluateCandidates(
    schoolYearId: number,
    assignments: Assignment[],
    slot: SlotRef,
  ): Promise<CandidateEval[]> {
    return await invoke<CandidateEval[]>('evaluate_candidates', {
      schoolYearId,
      assignments,
      slot,
    })
  }

  async evaluateSwap(
    schoolYearId: number,
    assignments: Assignment[],
    slotA: SlotRef,
    slotB: SlotRef,
  ): Promise<CandidateEval> {
    return await invoke<CandidateEval>('evaluate_swap', {
      schoolYearId,
      assignments,
      slotA,
      slotB,
    })
  }

  reoptimizeFrom(
    req: ReoptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle {
    const channel = new Channel<Progress>()
    if (onProgress) {
      channel.onmessage = (message) => {
        onProgress(message)
      }
    }

    const promise = invoke<OptimizeOutcome>('reoptimize_from', {
      req,
      onProgress: channel,
    })

    return {
      promise,
      cancel: () => {
        void this.cancelOptimize()
      },
    }
  }

  // Excel Import & Export
  async generateImportTemplate(targetPath: string): Promise<void> {
    await invoke<void>('generate_import_template', { targetPath })
  }

  async previewImport(
    schoolYearId: number,
    filePath: string,
    mode: string,
  ): Promise<ImportPreviewResult> {
    return await invoke<ImportPreviewResult>('preview_import', {
      schoolYearId,
      filePath,
      mode,
    })
  }

  async applyImport(
    schoolYearId: number,
    preview: ImportPreviewResult,
  ): Promise<ImportApplyResult> {
    return await invoke<ImportApplyResult>('apply_import', {
      schoolYearId,
      preview,
    })
  }

  async exportPlanExcel(planId: number, targetPath: string): Promise<void> {
    await invoke<void>('export_plan_excel', { planId, targetPath })
  }

  // Backup & Restore
  async backupDatabase(targetPath: string): Promise<void> {
    await invoke<void>('backup_database', { targetPath })
  }

  async restoreDatabase(sourcePath: string): Promise<void> {
    await invoke<void>('restore_database', { sourcePath })
  }

  async validateBackup(path: string): Promise<BackupValidationSummary> {
    return await invoke<BackupValidationSummary>('validate_backup', { path })
  }

  async listBackups(): Promise<BackupFileInfo[]> {
    return await invoke<BackupFileInfo[]>('list_backups')
  }

  // Dev Tools
  async seedDemo(): Promise<void> {
    await invoke<void>('seed_demo')
  }
}

export const tauriApi = new TauriExamPanelApi()
