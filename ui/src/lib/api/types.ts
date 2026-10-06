export * from './generated/types'

import type {
  AppInfo,
  AppSettings,
  ApplyPlanImportInput,
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
  FeasibilityReportWithQuotas,
  Grade,
  ImportApplyResult,
  ImportPreviewResult,
  Lock,
  OptimizeOutcome,
  OptimizeRequest,
  PlanDetails,
  PlanImportPreview,
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
  Unavailability,
  UpdateSubjectInput,
} from './generated/types'

export type ThemeMode = 'light' | 'dark' | 'system'

export interface OptimizeHandle {
  promise: Promise<OptimizeOutcome>
  cancel: () => void
}

export interface ExamPanelApi {
  // App & Settings
  ping(): Promise<string>
  getAppInfo(): Promise<AppInfo>
  getSettings(): Promise<AppSettings>
  setSetting(key: string, value: string): Promise<void>
  getTheme(): Promise<ThemeMode | null>
  setTheme(theme: ThemeMode): Promise<void>
  getLanguage(): Promise<string>
  setLanguage(lang: string): Promise<void>
  openDataFolder(): Promise<void>
  openLogFolder(): Promise<void>
  /** Opens the system print dialog for the current page (print / save as PDF). */
  printPage(): Promise<void>
  enterTrialMode(): Promise<void>
  exitTrialMode(): Promise<void>

  // Campuses
  listCampuses(): Promise<Campus[]>
  createCampus(input: CreateCampusInput): Promise<Campus>
  updateCampus(campus: Campus): Promise<void>
  deleteCampus(id: number): Promise<void>

  // Grades
  listGrades(): Promise<Grade[]>
  createGrade(input: CreateGradeInput): Promise<Grade>
  updateGrade(grade: Grade): Promise<void>
  deleteGrade(id: number): Promise<void>

  // Teachers
  listTeachers(): Promise<Teacher[]>
  createTeacher(input: CreateTeacherInput): Promise<Teacher>
  updateTeacher(teacher: Teacher): Promise<void>
  deleteTeacher(id: number): Promise<void>
  deactivateTeacher(id: number): Promise<void>
  setTeacherGrades(
    teacherId: number,
    schoolYearId: number,
    gradeIds: number[],
  ): Promise<void>
  teachersWithGrades(schoolYearId: number): Promise<TeacherWithGrades[]>

  // School Years
  listSchoolYears(): Promise<SchoolYear[]>
  createSchoolYear(input: CreateSchoolYearInput): Promise<SchoolYear>
  setCurrentSchoolYear(id: number): Promise<void>

  // Exams
  listExams(schoolYearId: number): Promise<Exam[]>
  createExam(input: CreateExamInput): Promise<Exam>
  updateExam(exam: Exam): Promise<void>
  deleteExam(id: number): Promise<void>
  reorderExams(examIds: number[]): Promise<void>

  // Subjects
  listSubjects(schoolYearId: number): Promise<Subject[]>
  createSubject(input: CreateSubjectInput): Promise<Subject>
  updateSubject(input: UpdateSubjectInput): Promise<void>
  deleteSubject(id: number): Promise<void>
  reorderSubjects(schoolYearId: number, subjectIds: number[]): Promise<void>

  // Competencies
  listCompetencies(schoolYearId: number): Promise<Competency[]>
  getTeacherCompetencies(teacherId: number, schoolYearId: number): Promise<Competency[]>
  setCompetency(input: SetCompetencyInput): Promise<void>
  deleteCompetency(input: DeleteCompetencyInput): Promise<void>
  replaceTeacherCompetencies(input: ReplaceTeacherCompetenciesInput): Promise<void>

  // Unavailability
  listUnavailabilities(schoolYearId: number): Promise<Unavailability[]>
  setUnavailability(unavailability: Unavailability): Promise<void>
  deleteUnavailability(teacherId: number, examId: number): Promise<void>

  // Locks
  listLocks(schoolYearId: number): Promise<Lock[]>
  createLock(input: CreateLockInput): Promise<Lock>
  deleteLock(id: number): Promise<void>

  // Rule Settings
  getRuleSettings(schoolYearId: number): Promise<RuleSetting[]>
  saveRuleSettings(schoolYearId: number, settings: RuleSetting[]): Promise<void>
  resetRuleSettingsToDefaults(schoolYearId: number): Promise<void>
  getRulePresets(): Promise<RulePresetItem[]>
  previewQuotas(input: PreviewQuotasInput): Promise<QuotaPreviewItem[]>

  // Analysis
  getProblemDetails(schoolYearId: number): Promise<ProblemDetails>
  checkFeasibility(schoolYearId: number): Promise<FeasibilityReportWithQuotas>
  evaluateAssignments(
    schoolYearId: number,
    assignments: Assignment[],
  ): Promise<EvaluationOutcome>

  // Optimization
  startOptimize(
    schoolYearId: number,
    request: OptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle
  cancelOptimize(): Promise<boolean>

  // Plans
  saveOptimizeResult(schoolYearId: number, outcome: OptimizeOutcome): Promise<number[]>
  listPlans(schoolYearId: number): Promise<PlanSummary[]>
  getPlan(id: number): Promise<PlanDetails>
  renamePlan(id: number, newName: string): Promise<void>
  deletePlan(id: number): Promise<void>
  markFinal(id: number): Promise<void>
  duplicatePlan(id: number, newName: string): Promise<number>
  planStatus(id: number): Promise<PlanStatus>
  createManualCopy(id: number, name: string): Promise<number>
  updatePlanAssignments(id: number, assignments: Assignment[]): Promise<EvaluationOutcome>
  evaluateCandidates(
    schoolYearId: number,
    assignments: Assignment[],
    slot: SlotRef,
  ): Promise<CandidateEval[]>
  evaluateSwap(
    schoolYearId: number,
    assignments: Assignment[],
    slotA: SlotRef,
    slotB: SlotRef,
  ): Promise<CandidateEval>
  reoptimizeFrom(
    req: ReoptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle

  // Excel Import & Export
  generateImportTemplate(targetPath: string): Promise<void>
  previewImport(
    schoolYearId: number,
    filePath: string,
    mode: string,
  ): Promise<ImportPreviewResult>
  applyImport(
    schoolYearId: number,
    preview: ImportPreviewResult,
  ): Promise<ImportApplyResult>
  exportPlanExcel(planId: number, targetPath: string): Promise<void>
  previewImportPlan(
    schoolYearId: number,
    filePath?: string,
    tsvContent?: string,
  ): Promise<PlanImportPreview>
  applyImportedPlan(input: ApplyPlanImportInput): Promise<number>

  // Backup & Restore
  backupDatabase(targetPath: string): Promise<void>
  restoreDatabase(sourcePath: string): Promise<void>
  validateBackup(path: string): Promise<BackupValidationSummary>
  listBackups(): Promise<BackupFileInfo[]>

  // Dev Tools
  seedDemo(): Promise<void>
}
