import demoCampuses from './fixtures/demo_campuses.json'
import demoExams from './fixtures/demo_exams.json'
import demoGrades from './fixtures/demo_grades.json'
import demoOutcome from './fixtures/demo_outcome.json'
import demoPlanDetails from './fixtures/demo_plan_details.json'
import demoPlans from './fixtures/demo_plans.json'
import demoRules from './fixtures/demo_rule_settings.json'
import demoTeachers from './fixtures/demo_teachers.json'
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
  PlanImportPreview,
  PlanImportTeacherTotal,
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
  TeacherGrade,
  TeacherQuota,
  TeacherWithGrades,
  ThemeMode,
  Unavailability,
  UpdateSubjectInput,
  Violation,
} from './types'

export class MockExamPanelApi implements ExamPanelApi {
  private settings: AppSettings = {
    theme: 'system',
    language: 'vi',
    current_school_year_id: 1,
  }

  private campuses: Campus[] = JSON.parse(JSON.stringify(demoCampuses))
  private grades: Grade[] = JSON.parse(JSON.stringify(demoGrades))
  private teachers: Teacher[] = JSON.parse(JSON.stringify(demoTeachers))
  private schoolYears: SchoolYear[] = [{ id: 1, name: '2026-2027', is_current: true }]
  private exams: Exam[] = JSON.parse(JSON.stringify(demoExams))
  private unavailabilities: Unavailability[] = [
    { teacher_id: 6, exam_id: 3, reason: null },
  ]
  private locks: Lock[] = []
  private ruleSettings: RuleSetting[] = JSON.parse(JSON.stringify(demoRules))
  private plans: PlanSummary[] = JSON.parse(JSON.stringify(demoPlans))
  private planDetailsMap: Map<number, PlanDetails> = new Map([
    [1, JSON.parse(JSON.stringify(demoPlanDetails))],
    [
      2,
      JSON.parse(
        JSON.stringify({
          ...demoPlanDetails,
          plan: { ...demoPlanDetails.plan, id: 2, name: 'Phương án #2', rank: 2 },
        }),
      ),
    ],
    [
      3,
      JSON.parse(
        JSON.stringify({
          ...demoPlanDetails,
          plan: { ...demoPlanDetails.plan, id: 3, name: 'Phương án #3', rank: 3 },
        }),
      ),
    ],
  ])
  private teacherGradesMap: Map<number, number[]> = new Map([
    [1, [1, 2]],
    [2, [1, 2]],
    [3, [1, 3]],
    [4, [2, 3]],
    [5, [1, 2, 3]],
    [6, [1, 2]],
    [7, [1, 3]],
    [8, [1, 2, 3]],
    [9, [1, 2]],
    [10, [2, 3]],
    [11, [1, 3]],
  ])

  private subjects: Subject[] = [
    {
      id: 1,
      code: 'VL',
      name: 'Vật lí',
      color: 'palette-1',
      sort_order: 1,
      setters: 2,
      reviewers: 1,
      min_campuses: 2,
    },
    {
      id: 2,
      code: 'CN',
      name: 'Công nghệ',
      color: 'palette-2',
      sort_order: 2,
      setters: 1,
      reviewers: 1,
      min_campuses: 2,
    },
  ]
  private competencies: Competency[] = []

  private isOptimizing = false
  private activeCancelCallback: (() => void) | null = null
  private savedTheme: ThemeMode | null = null
  private inTrialMode = false

  // App & Settings
  async ping(): Promise<string> {
    return 'pong from Mock Engine (browser mode)'
  }

  async getAppInfo(): Promise<AppInfo> {
    return {
      name: 'ExamPanel',
      version: '0.1.0',
      identifier: 'vn.exampanel.app',
      mode: 'mock',
      data_dir: '/mock/data',
      is_portable: true,
      db_path: this.inTrialMode ? '/mock/data/demo.db' : '/mock/data/exam-panel.db',
      commit_hash: '0c24c8e',
      build_date: '2026-10-02',
      in_trial_mode: this.inTrialMode,
    }
  }

  async getSettings(): Promise<AppSettings> {
    return { ...this.settings }
  }

  async setSetting(key: string, value: string): Promise<void> {
    if (key === 'theme') {
      this.settings.theme = value
      this.savedTheme = value as ThemeMode
    } else if (key === 'language') {
      this.settings.language = value
    } else if (key === 'school_name') {
      this.settings.school_name = value
    } else if (key === 'department_name') {
      this.settings.department_name = value
    } else if (key === 'signer_title') {
      this.settings.signer_title = value
    } else if (key === 'signer_name') {
      this.settings.signer_name = value
    } else if (key === 'place_name') {
      this.settings.place_name = value
    }
  }

  async getTheme(): Promise<ThemeMode | null> {
    return this.savedTheme
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    this.savedTheme = theme
    this.settings.theme = theme
  }

  async getLanguage(): Promise<string> {
    return this.settings.language || 'vi'
  }

  async setLanguage(lang: string): Promise<void> {
    this.settings.language = lang
  }

  async openDataFolder(): Promise<void> {
    // In mock mode, no native filesystem to open
  }

  async printPage(): Promise<void> {
    window.print()
  }

  async openLogFolder(): Promise<void> {
    // In mock mode, no native filesystem to open
  }

  async enterTrialMode(): Promise<void> {
    this.inTrialMode = true
  }

  async exitTrialMode(): Promise<void> {
    this.inTrialMode = false
  }

  // Campuses
  async listCampuses(): Promise<Campus[]> {
    return JSON.parse(JSON.stringify(this.campuses))
  }

  async createCampus(input: CreateCampusInput): Promise<Campus> {
    if (this.campuses.some((c) => c.code === input.code)) {
      throw {
        code: 'duplicate_entry',
        params: { detail: `Campus code ${input.code} already exists` },
      }
    }
    const maxId = this.campuses.reduce((max, c) => Math.max(max, c.id), 0)
    const newCampus: Campus = {
      id: maxId + 1,
      code: input.code,
      name: input.name,
      color: input.color,
    }
    this.campuses.push(newCampus)
    return { ...newCampus }
  }

  async updateCampus(campus: Campus): Promise<void> {
    const idx = this.campuses.findIndex((c) => c.id === campus.id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Campus not found' } }
    }
    this.campuses[idx] = { ...campus }
  }

  async deleteCampus(id: number): Promise<void> {
    if (this.teachers.some((t) => t.campus_id === id)) {
      throw {
        code: 'campus_in_use',
        params: { message: 'Campus has assigned teachers' },
      }
    }
    const idx = this.campuses.findIndex((c) => c.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Campus not found' } }
    }
    this.campuses.splice(idx, 1)
  }

  // Grades
  async listGrades(): Promise<Grade[]> {
    return JSON.parse(JSON.stringify(this.grades))
  }

  async createGrade(input: CreateGradeInput): Promise<Grade> {
    if (this.grades.some((g) => g.code === input.code)) {
      throw {
        code: 'duplicate_entry',
        params: { detail: `Grade code ${input.code} already exists` },
      }
    }
    const maxId = this.grades.reduce((max, g) => Math.max(max, g.id), 0)
    const newGrade: Grade = {
      id: maxId + 1,
      code: input.code,
      name: input.name,
      sort_order: input.sort_order,
    }
    this.grades.push(newGrade)
    return { ...newGrade }
  }

  async updateGrade(grade: Grade): Promise<void> {
    const idx = this.grades.findIndex((g) => g.id === grade.id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Grade not found' } }
    }
    this.grades[idx] = { ...grade }
  }

  async deleteGrade(id: number): Promise<void> {
    const hasTeachers = Array.from(this.teacherGradesMap.values()).some((grades) =>
      grades.includes(id),
    )
    const hasLocks = this.locks.some((l) => l.grade_id === id)
    if (hasTeachers || hasLocks) {
      throw {
        code: 'grade_in_use',
        params: { message: 'Grade is in use in teacher qualifications or locks' },
      }
    }
    const idx = this.grades.findIndex((g) => g.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Grade not found' } }
    }
    this.grades.splice(idx, 1)
  }

  // Teachers
  async listTeachers(): Promise<Teacher[]> {
    return JSON.parse(JSON.stringify(this.teachers))
  }

  async createTeacher(input: CreateTeacherInput): Promise<Teacher> {
    const maxId = this.teachers.reduce((max, t) => Math.max(max, t.id), 0)
    const newTeacher: Teacher = {
      id: maxId + 1,
      full_name: input.full_name,
      campus_id: input.campus_id,
      load_weight: input.load_weight ?? 1.0,
      active: input.active ?? true,
      note: input.note ?? null,
      code: input.code ?? undefined,
      display_name: input.display_name ?? undefined,
      quota_override: input.quota_override ?? undefined,
      max_tasks_per_exam_override: input.max_tasks_per_exam_override ?? undefined,
    }
    this.teachers.push(newTeacher)
    return { ...newTeacher }
  }

  async updateTeacher(teacher: Teacher): Promise<void> {
    const idx = this.teachers.findIndex((t) => t.id === teacher.id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Teacher not found' } }
    }
    this.teachers[idx] = { ...teacher }
  }

  async deleteTeacher(id: number): Promise<void> {
    const idx = this.teachers.findIndex((t) => t.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Teacher not found' } }
    }
    const hasAssignments = Array.from(this.planDetailsMap.values()).some((d) =>
      d.assignments.some((a) => a.teacher_id === id),
    )
    if (
      this.unavailabilities.some((u) => u.teacher_id === id) ||
      this.locks.some((l) => l.teacher_id === id) ||
      hasAssignments
    ) {
      throw {
        code: 'teacher_in_use',
        params: { message: 'Teacher is in use in exams, locks, or assignments' },
      }
    }
    this.teachers.splice(idx, 1)
    this.teacherGradesMap.delete(id)
  }

  async deactivateTeacher(id: number): Promise<void> {
    const teacher = this.teachers.find((t) => t.id === id)
    if (!teacher) {
      throw { code: 'not_found', params: { message: 'Teacher not found' } }
    }
    teacher.active = false
  }

  async setTeacherGrades(
    teacherId: number,
    _schoolYearId: number,
    gradeIds: number[],
  ): Promise<void> {
    this.teacherGradesMap.set(teacherId, [...gradeIds])
  }

  async teachersWithGrades(_schoolYearId: number): Promise<TeacherWithGrades[]> {
    return this.teachers.map((t) => ({
      teacher: { ...t },
      grade_ids: this.teacherGradesMap.get(t.id) || [],
    }))
  }

  // School Years
  async listSchoolYears(): Promise<SchoolYear[]> {
    return JSON.parse(JSON.stringify(this.schoolYears))
  }

  async createSchoolYear(input: CreateSchoolYearInput): Promise<SchoolYear> {
    const maxId = this.schoolYears.reduce((max, s) => Math.max(max, s.id), 0)
    const newYear: SchoolYear = {
      id: maxId + 1,
      name: input.name,
      is_current: input.is_current ?? false,
    }
    if (newYear.is_current) {
      for (const y of this.schoolYears) {
        y.is_current = false
      }
      this.settings.current_school_year_id = newYear.id
    }
    this.schoolYears.push(newYear)
    return { ...newYear }
  }

  async setCurrentSchoolYear(id: number): Promise<void> {
    const year = this.schoolYears.find((y) => y.id === id)
    if (!year) {
      throw {
        code: 'not_found',
        params: { message: 'School year not found' },
      }
    }
    for (const y of this.schoolYears) {
      y.is_current = y.id === id
    }
    this.settings.current_school_year_id = id
  }

  // Exams
  async listExams(_schoolYearId: number): Promise<Exam[]> {
    return JSON.parse(JSON.stringify(this.exams))
  }

  async createExam(input: CreateExamInput): Promise<Exam> {
    if (this.exams.some((e) => e.code === input.code)) {
      throw {
        code: 'duplicate_entry',
        params: { detail: `Exam code ${input.code} already exists` },
      }
    }
    const maxId = this.exams.reduce((max, e) => Math.max(max, e.id), 0)
    const newExam: Exam = {
      id: maxId + 1,
      school_year_id: input.school_year_id,
      code: input.code,
      name: input.name,
      sort_order: input.sort_order,
    }
    this.exams.push(newExam)
    return { ...newExam }
  }

  async updateExam(exam: Exam): Promise<void> {
    const idx = this.exams.findIndex((e) => e.id === exam.id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Exam not found' } }
    }
    this.exams[idx] = { ...exam }
  }

  async deleteExam(id: number): Promise<void> {
    const hasUnavailability = this.unavailabilities.some((u) => u.exam_id === id)
    const hasLocks = this.locks.some((l) => l.exam_id === id)
    if (hasUnavailability || hasLocks) {
      throw {
        code: 'exam_in_use',
        params: { message: 'Exam is in use in unavailabilities or locks' },
      }
    }
    const idx = this.exams.findIndex((e) => e.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Exam not found' } }
    }
    this.exams.splice(idx, 1)
  }

  async reorderExams(examIds: number[]): Promise<void> {
    examIds.forEach((id, idx) => {
      const exam = this.exams.find((e) => e.id === id)
      if (exam) {
        exam.sort_order = idx + 1
      }
    })
    this.exams.sort((a, b) => a.sort_order - b.sort_order)
  }

  // Subjects
  async listSubjects(_schoolYearId: number): Promise<Subject[]> {
    return JSON.parse(JSON.stringify(this.subjects))
  }

  async createSubject(input: CreateSubjectInput): Promise<Subject> {
    if (this.subjects.some((s) => s.code === input.code)) {
      throw {
        code: 'duplicate_entry',
        params: { detail: `Subject code ${input.code} already exists` },
      }
    }
    const maxId = this.subjects.reduce((max, s) => Math.max(max, s.id), 0)
    const newSubject: Subject = {
      id: maxId + 1,
      code: input.code,
      name: input.name,
      color: input.color,
      sort_order: input.sort_order,
      setters: input.setters,
      reviewers: input.reviewers,
      min_campuses: input.min_campuses,
    }
    this.subjects.push(newSubject)
    return { ...newSubject }
  }

  async updateSubject(input: UpdateSubjectInput): Promise<void> {
    const idx = this.subjects.findIndex((s) => s.id === input.id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Subject not found' } }
    }
    this.subjects[idx] = { ...this.subjects[idx], ...input }
  }

  async deleteSubject(id: number): Promise<void> {
    const idx = this.subjects.findIndex((s) => s.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Subject not found' } }
    }
    this.subjects.splice(idx, 1)
  }

  async reorderSubjects(_schoolYearId: number, subjectIds: number[]): Promise<void> {
    subjectIds.forEach((id, idx) => {
      const sub = this.subjects.find((s) => s.id === id)
      if (sub) {
        sub.sort_order = idx + 1
      }
    })
  }

  // Competencies
  private ensureCompetencies() {
    if (this.competencies.length === 0) {
      for (const t of this.teachers) {
        for (const s of this.subjects) {
          this.competencies.push({
            teacher_id: t.id,
            subject_id: s.id,
            role: 'setter',
            grade_scope: 'taught',
          })
          this.competencies.push({
            teacher_id: t.id,
            subject_id: s.id,
            role: 'reviewer',
            grade_scope: 'taught',
          })
        }
      }
    }
  }

  async listCompetencies(_schoolYearId: number): Promise<Competency[]> {
    this.ensureCompetencies()
    return JSON.parse(JSON.stringify(this.competencies))
  }

  async getTeacherCompetencies(
    teacherId: number,
    _schoolYearId: number,
  ): Promise<Competency[]> {
    this.ensureCompetencies()
    return JSON.parse(
      JSON.stringify(this.competencies.filter((c) => c.teacher_id === teacherId)),
    )
  }

  async setCompetency(input: SetCompetencyInput): Promise<void> {
    this.ensureCompetencies()
    const idx = this.competencies.findIndex(
      (c) =>
        c.teacher_id === input.teacher_id &&
        c.subject_id === input.subject_id &&
        c.role === input.role,
    )
    if (idx !== -1) {
      this.competencies[idx].grade_scope = input.grade_scope
    } else {
      this.competencies.push({
        teacher_id: input.teacher_id,
        subject_id: input.subject_id,
        role: input.role,
        grade_scope: input.grade_scope,
      })
    }
  }

  async deleteCompetency(input: DeleteCompetencyInput): Promise<void> {
    this.ensureCompetencies()
    const idx = this.competencies.findIndex(
      (c) =>
        c.teacher_id === input.teacher_id &&
        c.subject_id === input.subject_id &&
        c.role === input.role,
    )
    if (idx !== -1) {
      this.competencies.splice(idx, 1)
    }
  }

  async replaceTeacherCompetencies(
    input: ReplaceTeacherCompetenciesInput,
  ): Promise<void> {
    this.ensureCompetencies()
    this.competencies = this.competencies.filter((c) => c.teacher_id !== input.teacher_id)
    for (const c of input.competencies) {
      this.competencies.push({ ...c })
    }
  }

  // Unavailability
  async listUnavailabilities(_schoolYearId: number): Promise<Unavailability[]> {
    return JSON.parse(JSON.stringify(this.unavailabilities))
  }

  async setUnavailability(unavailability: Unavailability): Promise<void> {
    this.unavailabilities = this.unavailabilities.filter(
      (u) =>
        !(
          u.teacher_id === unavailability.teacher_id &&
          u.exam_id === unavailability.exam_id
        ),
    )
    this.unavailabilities.push({ ...unavailability })
  }

  async deleteUnavailability(teacherId: number, examId: number): Promise<void> {
    this.unavailabilities = this.unavailabilities.filter(
      (u) => !(u.teacher_id === teacherId && u.exam_id === examId),
    )
  }

  // Locks
  async listLocks(_schoolYearId: number): Promise<Lock[]> {
    return JSON.parse(JSON.stringify(this.locks))
  }

  async createLock(input: CreateLockInput): Promise<Lock> {
    const maxId = this.locks.reduce((max, l) => Math.max(max, l.id), 0)
    const newLock: Lock = {
      id: maxId + 1,
      exam_id: input.exam_id,
      grade_id: input.grade_id,
      subject_id: input.subject_id,
      teacher_id: input.teacher_id,
      role: input.role ?? null,
      kind: input.kind,
    }
    this.locks.push(newLock)
    return { ...newLock }
  }

  async deleteLock(id: number): Promise<void> {
    const idx = this.locks.findIndex((l) => l.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Lock not found' } }
    }
    this.locks.splice(idx, 1)
  }

  // Rule Settings
  async getRuleSettings(_schoolYearId: number): Promise<RuleSetting[]> {
    return JSON.parse(JSON.stringify(this.ruleSettings))
  }

  async saveRuleSettings(_schoolYearId: number, settings: RuleSetting[]): Promise<void> {
    this.ruleSettings = JSON.parse(JSON.stringify(settings))
  }

  async resetRuleSettingsToDefaults(_schoolYearId: number): Promise<void> {
    this.ruleSettings = JSON.parse(JSON.stringify(demoRules))
  }

  async getRulePresets(): Promise<RulePresetItem[]> {
    const makeSettings = (weights: Record<string, number>): RuleSetting[] => {
      const base: RuleSetting[] = JSON.parse(JSON.stringify(demoRules))
      for (const s of base) {
        if (s.key in weights) {
          s.weight = weights[s.key]
        }
      }
      return base
    }

    return [
      {
        id: 'balanced',
        name: 'Cân bằng (mặc định)',
        settings: makeSettings({
          s1: 10,
          s2: 10,
          s3: 10,
          s4: 10,
          s5: 10,
          s6: 10,
          s7: 10,
          s8: 10,
        }),
      },
      {
        id: 'workload_fairness',
        name: 'Ưu tiên công bằng khối lượng',
        settings: makeSettings({
          s8: 20,
          s1: 18,
          s2: 10,
          s3: 8,
          s4: 8,
          s5: 8,
          s6: 8,
          s7: 8,
        }),
      },
      {
        id: 'team_diversity',
        name: 'Ưu tiên đa dạng ê-kíp',
        settings: makeSettings({
          s4: 20,
          s5: 18,
          s3: 16,
          s1: 8,
          s2: 8,
          s6: 8,
          s7: 8,
          s8: 10,
        }),
      },
      {
        id: 'allow_task_crowding',
        name: 'Cho phép dồn việc trong một kỳ',
        settings: (() => {
          const s = makeSettings({})
          for (const r of s) {
            if (r.key === 's9') {
              r.enabled = false
              r.weight = 0
            }
          }
          return s
        })(),
      },
    ]
  }

  async previewQuotas(input: PreviewQuotasInput): Promise<QuotaPreviewItem[]> {
    const activeTeachers = this.teachers.filter((t) => t.active && t.load_weight > 0)
    const totalSlots = this.exams.length * this.grades.length * 3
    const totalWeight = activeTeachers.reduce((sum, t) => sum + t.load_weight, 0)

    const h7Setting = input.rule_settings.find((s) => s.key === 'h7')
    const tolerance =
      h7Setting && typeof h7Setting.params?.tolerance === 'number'
        ? (h7Setting.params.tolerance as number)
        : 1

    return activeTeachers.map((t) => {
      const unavailableCount = this.unavailabilities.filter(
        (u) => u.teacher_id === t.id,
      ).length
      const availableExams = Math.max(0, this.exams.length - unavailableCount)
      const q = totalWeight > 0 ? (totalSlots * t.load_weight) / totalWeight : 0
      const lo = Math.max(0, Math.floor(q) - tolerance)
      const hi = Math.min(availableExams, Math.ceil(q) + tolerance)
      const campus = this.campuses.find((c) => c.id === t.campus_id)

      return {
        teacher_id: t.id,
        teacher_name: t.full_name,
        campus_id: t.campus_id,
        campus_name: campus?.name || '',
        load_weight: t.load_weight,
        available_exams: availableExams,
        quota: Math.round(q * 100) / 100,
        lo,
        hi,
      }
    })
  }

  // Analysis
  async getProblemDetails(schoolYearId: number): Promise<ProblemDetails> {
    this.ensureCompetencies()
    const teachers = await this.listTeachers()
    const teacherGrades: TeacherGrade[] = []
    this.teacherGradesMap.forEach((gids, tid) => {
      for (const gid of gids) {
        teacherGrades.push({
          teacher_id: tid,
          school_year_id: schoolYearId,
          grade_id: gid,
        })
      }
    })
    return {
      problem: {
        school_year: this.schoolYears[0],
        campuses: this.campuses,
        grades: this.grades,
        subjects: this.subjects,
        teachers,
        teacher_grades: teacherGrades,
        competencies: this.competencies,
        exams: this.exams,
        unavailabilities: this.unavailabilities,
        locks: this.locks,
        rule_settings: this.ruleSettings,
      },
      forced: [],
    }
  }

  async checkFeasibility(_schoolYearId: number): Promise<FeasibilityReportWithQuotas> {
    const activeTeachers = this.teachers.filter((t) => t.active && t.load_weight > 0)
    const totalSlots = this.exams.length * this.grades.length * 3
    const totalWeight = activeTeachers.reduce((sum, t) => sum + t.load_weight, 0)

    // Degenerate database diagnostics (Part A3)
    if (this.campuses.length === 0) {
      return {
        report: {
          is_feasible: false,
          errors: [{ rule: 'h3', code: 'no_campuses', params: {} }],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      }
    }
    if (activeTeachers.length === 0) {
      return {
        report: {
          is_feasible: false,
          errors: [{ rule: 'h2', code: 'no_active_teachers', params: {} }],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      }
    }
    const campusesWithActiveTeachers = new Set(activeTeachers.map((t) => t.campus_id))
    if (campusesWithActiveTeachers.size < 2) {
      return {
        report: {
          is_feasible: false,
          errors: [{ rule: 'h3', code: 'single_campus', params: {} }],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      }
    }
    if (this.grades.length === 0) {
      return {
        report: {
          is_feasible: false,
          errors: [{ rule: 'h2', code: 'no_grades', params: {} }],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      }
    }
    if (this.exams.length === 0) {
      return {
        report: {
          is_feasible: false,
          errors: [{ rule: 'h2', code: 'no_exams', params: {} }],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      }
    }

    const errors: Violation[] = []
    const warnings: Violation[] = []

    for (const grade of this.grades) {
      const eligible = activeTeachers.filter((t) =>
        (this.teacherGradesMap.get(t.id) || []).includes(grade.id),
      )
      if (eligible.length < 3) {
        errors.push({
          rule: 'h2',
          code: 'insufficient_panel_teachers',
          panel: {
            exam_id: this.exams[0]?.id ?? 1,
            grade_id: grade.id,
            subject_id: this.subjects[0]?.id ?? 1,
          },
          params: {
            exam: 'Các kỳ thi',
            grade: grade.code.toString(),
            count: eligible.length.toString(),
          },
        })
      } else {
        const campuses = new Set(eligible.map((t) => t.campus_id))
        if (campuses.size < 2) {
          errors.push({
            rule: 'h3',
            code: 'insufficient_campuses',
            panel: {
              exam_id: this.exams[0]?.id ?? 1,
              grade_id: grade.id,
              subject_id: this.subjects[0]?.id ?? 1,
            },
            params: {
              exam: 'Các kỳ thi',
              grade: grade.code.toString(),
              count: campuses.size.toString(),
            },
          })
        }
        if (eligible.length === 3) {
          warnings.push({
            rule: 'h2',
            code: 'tight_panel_roster',
            panel: {
              exam_id: this.exams[0]?.id ?? 1,
              grade_id: grade.id,
              subject_id: this.subjects[0]?.id ?? 1,
            },
            params: {
              exam: 'Các kỳ thi',
              grade: grade.code.toString(),
              count: eligible.length.toString(),
            },
          })
        }
      }
    }

    // Live lock diagnostics (Part F)
    for (const lock of this.locks) {
      const teacher = this.teachers.find((t) => t.id === lock.teacher_id)
      const exam = this.exams.find((e) => e.id === lock.exam_id)
      const grade = this.grades.find((g) => g.id === lock.grade_id)
      const teacherName = teacher?.full_name || `Teacher #${lock.teacher_id}`
      const examCode = exam?.code || `Exam #${lock.exam_id}`
      const gradeCode = grade?.code.toString() || `Grade #${lock.grade_id}`

      if (lock.kind === 'pin') {
        const isUnavailable = this.unavailabilities.some(
          (u) => u.teacher_id === lock.teacher_id && u.exam_id === lock.exam_id,
        )
        if (isUnavailable) {
          errors.push({
            rule: 'h6',
            code: 'pinned_teacher_ineligible_due_to_unavailability',
            panel: {
              exam_id: lock.exam_id,
              grade_id: lock.grade_id,
              subject_id: lock.subject_id,
            },
            teacher: lock.teacher_id,
            params: {
              teacher: teacherName,
              exam: examCode,
              grade: gradeCode,
            },
          })
        }

        const isQualified = (this.teacherGradesMap.get(lock.teacher_id) || []).includes(
          lock.grade_id,
        )
        if (!isQualified) {
          errors.push({
            rule: 'h6',
            code: 'pinned_teacher_not_qualified',
            panel: {
              exam_id: lock.exam_id,
              grade_id: lock.grade_id,
              subject_id: lock.subject_id,
            },
            teacher: lock.teacher_id,
            params: {
              teacher: teacherName,
              exam: examCode,
              grade: gradeCode,
            },
          })
        }
      }
    }

    const lockMap = new Map<string, Lock[]>()
    for (const lock of this.locks) {
      const key = `${lock.exam_id}-${lock.grade_id}-${lock.teacher_id}`
      if (!lockMap.has(key)) lockMap.set(key, [])
      lockMap.get(key)!.push(lock)
    }
    for (const [, group] of lockMap) {
      const hasPin = group.some((l) => l.kind === 'pin')
      const hasForbid = group.some((l) => l.kind === 'forbid')
      if (hasPin && hasForbid) {
        const lock = group[0]
        const teacher = this.teachers.find((t) => t.id === lock.teacher_id)
        errors.push({
          rule: 'h6',
          code: 'lock_conflict',
          panel: {
            exam_id: lock.exam_id,
            grade_id: lock.grade_id,
            subject_id: lock.subject_id,
          },
          teacher: lock.teacher_id,
          params: {
            teacher: teacher?.full_name || `Teacher #${lock.teacher_id}`,
          },
        })
      }
    }

    const panelPins = new Map<string, Lock[]>()
    for (const lock of this.locks.filter((l) => l.kind === 'pin')) {
      const key = `${lock.exam_id}-${lock.grade_id}`
      if (!panelPins.has(key)) panelPins.set(key, [])
      panelPins.get(key)!.push(lock)
    }
    for (const [, pins] of panelPins) {
      if (pins.length > 3) {
        const lock = pins[0]
        errors.push({
          rule: 'h6',
          code: 'excess_pinned_setters_and_reviewers_and_pins',
          panel: {
            exam_id: lock.exam_id,
            grade_id: lock.grade_id,
            subject_id: lock.subject_id,
          },
          params: { count: pins.length.toString() },
        })
      }
      const pinnedSetters = pins.filter((l) => l.role === 'setter')
      if (pinnedSetters.length > 2) {
        const lock = pinnedSetters[0]
        errors.push({
          rule: 'h6',
          code: 'excess_pinned_setters_and_reviewers_and_pins',
          panel: {
            exam_id: lock.exam_id,
            grade_id: lock.grade_id,
            subject_id: lock.subject_id,
          },
          params: { count: pinnedSetters.length.toString() },
        })
      }
      const pinnedReviewers = pins.filter((l) => l.role === 'reviewer')
      if (pinnedReviewers.length > 1) {
        const lock = pinnedReviewers[0]
        errors.push({
          rule: 'h6',
          code: 'excess_pinned_setters_and_reviewers_and_pins',
          panel: {
            exam_id: lock.exam_id,
            grade_id: lock.grade_id,
            subject_id: lock.subject_id,
          },
          params: { count: pinnedReviewers.length.toString() },
        })
      }
    }

    const quotas: TeacherQuota[] = activeTeachers.map((t) => {
      const unavailableCount = this.unavailabilities.filter(
        (u) => u.teacher_id === t.id,
      ).length
      const availableExams = Math.max(0, this.exams.length - unavailableCount)
      const q = totalWeight > 0 ? (totalSlots * t.load_weight) / totalWeight : 0
      const lo = Math.max(0, Math.floor(q) - 1)
      const hi = Math.min(availableExams, Math.ceil(q) + 1)
      return {
        teacher_id: t.id,
        available_exams: availableExams,
        quota: q,
        lo,
        hi,
      }
    })

    const totalMax = quotas.reduce((sum, q) => sum + q.hi, 0)
    if (totalMax < totalSlots) {
      errors.push({
        rule: 'h7',
        code: 'insufficient_total_capacity',
        params: {
          max_capacity: totalMax.toString(),
          total_slots: totalSlots.toString(),
        },
      })
    }

    return {
      report: {
        is_feasible: errors.length === 0,
        errors,
        warnings,
        quotas,
      },
      quotas,
    }
  }

  async evaluateAssignments(
    _schoolYearId: number,
    _assignments: Assignment[],
  ): Promise<EvaluationOutcome> {
    const outcome = demoOutcome as unknown as OptimizeOutcome
    return {
      hard_violations: [],
      score_report: outcome.plans[0].report,
    }
  }

  // Optimization
  startOptimize(
    schoolYearId: number,
    request: OptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle {
    if (this.isOptimizing) {
      throw {
        code: 'optimize_busy',
        params: { message: 'Optimization is already in progress' },
      }
    }

    this.isOptimizing = true
    let isCancelled = false
    let timerId: ReturnType<typeof setInterval> | null = null
    let rejectPromise: ((err: unknown) => void) | null = null

    const cancel = () => {
      isCancelled = true
      this.isOptimizing = false
      if (timerId !== null) {
        clearInterval(timerId)
        timerId = null
      }
      this.activeCancelCallback = null
      if (rejectPromise) {
        rejectPromise({
          code: 'cancelled',
          params: { message: 'Optimization was cancelled' },
        })
        rejectPromise = null
      }
    }

    this.activeCancelCallback = cancel

    const promise = new Promise<OptimizeOutcome>((resolve, reject) => {
      rejectPromise = reject
      let step = 0
      const totalSteps = 10
      const stepDurationMs = 200

      timerId = setInterval(() => {
        if (isCancelled) {
          if (timerId !== null) clearInterval(timerId)
          return
        }

        step += 1
        // Real shape: every event carries the counters of ONE run; all runs advance together.
        const budget = request.budget.value
        const best_score = Math.max(2.6, 25.0 - step * 2.2)
        if (onProgress) {
          for (let run = 0; run < Math.max(1, request.runs); run++) {
            const current_score = best_score + Math.random() * 3.0
            onProgress({
              run,
              iteration: Math.floor((step / totalSteps) * budget),
              best_score: Number((best_score + run * 0.01).toFixed(2)),
              current_score: Number(current_score.toFixed(2)),
              elapsed_ms: step * stepDurationMs,
            })
          }
        }

        if (step >= totalSteps) {
          if (timerId !== null) clearInterval(timerId)
          this.isOptimizing = false
          this.activeCancelCallback = null
          const result: OptimizeOutcome = JSON.parse(
            JSON.stringify(demoOutcome),
          ) as unknown as OptimizeOutcome
          result.school_year_id = schoolYearId
          resolve(result)
        }
      }, stepDurationMs)
    })

    return {
      promise,
      cancel,
    }
  }

  async cancelOptimize(): Promise<boolean> {
    if (this.isOptimizing && this.activeCancelCallback) {
      this.activeCancelCallback()
      return true
    }
    return false
  }

  // Plans
  async saveOptimizeResult(
    _schoolYearId: number,
    outcome: OptimizeOutcome,
  ): Promise<number[]> {
    const ids: number[] = []
    for (const p of outcome.plans) {
      const nextId = this.plans.reduce((max, pl) => Math.max(max, pl.id), 0) + 1
      ids.push(nextId)
      const summary: PlanSummary = {
        id: nextId,
        name: `Phương án #${p.rank}`,
        rank: p.rank,
        score: p.report.total,
        created_at: new Date().toISOString(),
        is_final: false,
        source: 'optimizer',
        is_stale: false,
        data_hash: undefined,
        rules_hash: undefined,
      }
      this.plans.push(summary)
      this.planDetailsMap.set(nextId, {
        plan: {
          ...summary,
          school_year_id: 1,
          seed: p.seed,
          score_report_json: JSON.stringify(p.report),
          run_params_json: outcome.run_params_json,
          data_hash: undefined,
          rules_hash: undefined,
        },
        assignments: p.assignments,
        score_report: p.report,
      })
    }
    return ids
  }

  async listPlans(_schoolYearId: number): Promise<PlanSummary[]> {
    return JSON.parse(JSON.stringify(this.plans))
  }

  async getPlan(id: number): Promise<PlanDetails> {
    const details = this.planDetailsMap.get(id)
    if (!details) {
      throw { code: 'not_found', params: { message: `Plan ${id} not found` } }
    }
    return JSON.parse(JSON.stringify(details))
  }

  async renamePlan(id: number, newName: string): Promise<void> {
    const plan = this.plans.find((p) => p.id === id)
    if (!plan) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    plan.name = newName
    const details = this.planDetailsMap.get(id)
    if (details) {
      details.plan.name = newName
    }
  }

  async deletePlan(id: number): Promise<void> {
    const idx = this.plans.findIndex((p) => p.id === id)
    if (idx === -1) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    this.plans.splice(idx, 1)
    this.planDetailsMap.delete(id)
  }

  private mockValidateAssignments(assignments: Assignment[]): Violation[] {
    const violations: Violation[] = []
    // H1: Duplicate teacher in the same panel
    const panelTeacherCount = new Map<string, number>()
    for (const a of assignments) {
      const key = `${a.exam_id}_${a.grade_id}_${a.teacher_id}`
      panelTeacherCount.set(key, (panelTeacherCount.get(key) || 0) + 1)
      if (panelTeacherCount.get(key)! > 1) {
        violations.push({
          rule: 'h1',
          code: 'duplicate_teacher_in_panel',
          teacher: a.teacher_id,
          panel: { exam_id: a.exam_id, grade_id: a.grade_id, subject_id: a.subject_id },
          params: {
            teacher_id: a.teacher_id.toString(),
            exam_id: a.exam_id.toString(),
            grade_id: a.grade_id.toString(),
          },
        })
      }
    }

    // H2: Teacher inactive or unqualified
    const activeTeacherMap = new Map(this.teachers.map((t) => [t.id, t]))
    for (const a of assignments) {
      const t = activeTeacherMap.get(a.teacher_id)
      if (!t || !t.active || t.load_weight <= 0) {
        violations.push({
          rule: 'h2',
          code: !t || !t.active ? 'inactive_teacher' : 'zero_weight_teacher',
          teacher: a.teacher_id,
          panel: { exam_id: a.exam_id, grade_id: a.grade_id, subject_id: a.subject_id },
          params: { teacher_id: a.teacher_id.toString() },
        })
      }
      const tg = this.teacherGradesMap.get(a.teacher_id) || []
      if (!tg.includes(a.grade_id)) {
        violations.push({
          rule: 'h2',
          code: 'unqualified_grade',
          teacher: a.teacher_id,
          panel: { exam_id: a.exam_id, grade_id: a.grade_id, subject_id: a.subject_id },
          params: {
            teacher_id: a.teacher_id.toString(),
            grade_id: a.grade_id.toString(),
          },
        })
      }
    }

    // H4: Same teacher assigned to multiple panels in same exam
    const examTeacherPanels = new Map<string, Set<number>>()
    for (const a of assignments) {
      const key = `${a.exam_id}_${a.teacher_id}`
      if (!examTeacherPanels.has(key)) {
        examTeacherPanels.set(key, new Set())
      }
      const set = examTeacherPanels.get(key)!
      set.add(a.grade_id)
      if (set.size > 1) {
        violations.push({
          rule: 'h4',
          code: 'multiple_panels_in_exam',
          teacher: a.teacher_id,
          params: { exam_id: a.exam_id, count: set.size, limit: 1 },
        })
      }
    }

    // H5: Teacher unavailable
    for (const a of assignments) {
      const unavail = this.unavailabilities.some(
        (u) => u.teacher_id === a.teacher_id && u.exam_id === a.exam_id,
      )
      if (unavail) {
        violations.push({
          rule: 'h5',
          code: 'teacher_unavailable',
          teacher: a.teacher_id,
          panel: { exam_id: a.exam_id, grade_id: a.grade_id, subject_id: a.subject_id },
          params: { exam_id: a.exam_id },
        })
      }
    }

    return violations
  }

  async markFinal(id: number): Promise<void> {
    const details = this.planDetailsMap.get(id)
    if (!details) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    const isStale = this.plans.find((p) => p.id === id)?.is_stale ?? false
    if (isStale) {
      throw { code: 'plan_stale', params: { id: id.toString() } }
    }
    const hardViolations = this.mockValidateAssignments(details.assignments)
    if (hardViolations.length > 0) {
      throw { code: 'plan_invalid', params: { count: hardViolations.length.toString() } }
    }
    for (const p of this.plans) {
      p.is_final = p.id === id
      const d = this.planDetailsMap.get(p.id)
      if (d) d.plan.is_final = p.is_final
    }
  }

  async duplicatePlan(id: number, newName: string): Promise<number> {
    return this.createManualCopy(id, newName)
  }

  async planStatus(id: number): Promise<PlanStatus> {
    const details = this.planDetailsMap.get(id)
    if (!details) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    const hardViolations = this.mockValidateAssignments(details.assignments)
    const isStale = this.plans.find((p) => p.id === id)?.is_stale ?? false
    const scoreReport = details.score_report ?? {
      total: 0,
      by_rule: [],
      violations: [],
      per_teacher: [],
    }
    return {
      data_changed: isStale,
      rules_changed: false,
      hard_violations_now: hardViolations,
      score_now: scoreReport,
    }
  }

  async createManualCopy(id: number, name: string): Promise<number> {
    const orig = this.planDetailsMap.get(id)
    if (!orig) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    const nextId = this.plans.reduce((max, pl) => Math.max(max, pl.id), 0) + 1
    const summary: PlanSummary = {
      id: nextId,
      name,
      rank: undefined,
      score: orig.plan.score,
      created_at: new Date().toISOString(),
      is_final: false,
      source: 'duplicate',
      is_stale: false,
      data_hash: orig.plan.data_hash ?? undefined,
      rules_hash: orig.plan.rules_hash ?? undefined,
    }
    this.plans.push(summary)
    this.planDetailsMap.set(nextId, {
      plan: {
        ...summary,
        school_year_id: orig.plan.school_year_id,
        seed: orig.plan.seed,
        score_report_json: orig.plan.score_report_json,
        run_params_json: orig.plan.run_params_json,
      },
      assignments: JSON.parse(JSON.stringify(orig.assignments)),
      score_report: orig.score_report,
    })
    return nextId
  }

  async updatePlanAssignments(
    id: number,
    assignments: Assignment[],
  ): Promise<EvaluationOutcome> {
    const details = this.planDetailsMap.get(id)
    if (!details) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    if (details.plan.source === 'optimizer') {
      throw { code: 'plan_immutable', params: { id: id.toString() } }
    }
    if (details.plan.is_final) {
      throw { code: 'plan_final_immutable', params: { id: id.toString() } }
    }

    const hardViolations = this.mockValidateAssignments(assignments)
    details.assignments = JSON.parse(JSON.stringify(assignments))
    const scoreReport = details.score_report ?? {
      total: 0,
      by_rule: [],
      violations: [],
      per_teacher: [],
    }
    return {
      hard_violations: hardViolations,
      score_report: scoreReport,
    }
  }

  async evaluateCandidates(
    _schoolYearId: number,
    assignments: Assignment[],
    slot: SlotRef,
  ): Promise<CandidateEval[]> {
    const currentBaseScore = 150.0
    const results: CandidateEval[] = []

    for (const teacher of this.teachers) {
      const simAssignments = assignments.map((a) => ({ ...a }))
      let replaced = false
      for (const a of simAssignments) {
        if (
          a.exam_id === slot.exam_id &&
          a.grade_id === slot.grade_id &&
          a.subject_id === slot.subject_id &&
          a.role === slot.role &&
          a.position === slot.position
        ) {
          a.teacher_id = teacher.id
          replaced = true
          break
        }
      }
      if (!replaced) {
        simAssignments.push({
          plan_id: assignments[0]?.plan_id ?? 0,
          exam_id: slot.exam_id,
          grade_id: slot.grade_id,
          subject_id: slot.subject_id,
          teacher_id: teacher.id,
          role: slot.role,
          position: slot.position,
        })
      }

      const hardViolations = this.mockValidateAssignments(simAssignments)
      // Deterministic placeholder delta for candidates (mock only)
      const delta = ((teacher.id * 7) % 17) - 8.5
      const newTotal = Math.max(0, currentBaseScore + delta)

      results.push({
        teacher_id: teacher.id,
        hard_violations: hardViolations,
        delta_score: delta,
        new_total: newTotal,
      })
    }

    results.sort((a, b) => a.delta_score - b.delta_score)
    return results
  }

  async evaluateSwap(
    _schoolYearId: number,
    assignments: Assignment[],
    slotA: SlotRef,
    slotB: SlotRef,
  ): Promise<CandidateEval> {
    const currentBaseScore = 150.0
    let tA: number | undefined
    let tB: number | undefined
    let sCountA = 0
    let sCountB = 0
    for (const a of assignments) {
      if (
        a.exam_id === slotA.exam_id &&
        a.grade_id === slotA.grade_id &&
        a.role === slotA.role
      ) {
        if (slotA.role === 'reviewer' || sCountA === slotA.position) {
          tA = a.teacher_id
        }
        sCountA += 1
      }
      if (
        a.exam_id === slotB.exam_id &&
        a.grade_id === slotB.grade_id &&
        a.role === slotB.role
      ) {
        if (slotB.role === 'reviewer' || sCountB === slotB.position) {
          tB = a.teacher_id
        }
        sCountB += 1
      }
    }

    const simAssignments = assignments.map((a) => ({ ...a }))
    if (tA !== undefined && tB !== undefined) {
      for (const a of simAssignments) {
        if (
          a.exam_id === slotA.exam_id &&
          a.grade_id === slotA.grade_id &&
          a.role === slotA.role &&
          a.teacher_id === tA
        ) {
          a.teacher_id = tB
          break
        }
      }
      for (const a of simAssignments) {
        if (
          a.exam_id === slotB.exam_id &&
          a.grade_id === slotB.grade_id &&
          a.role === slotB.role &&
          a.teacher_id === tB
        ) {
          a.teacher_id = tA
          break
        }
      }
    }

    const hardViolations = this.mockValidateAssignments(simAssignments)
    const delta = (((tA ?? 1) + (tB ?? 2)) % 11) - 5.0
    return {
      teacher_id: tB ?? 0,
      hard_violations: hardViolations,
      delta_score: delta,
      new_total: Math.max(0, currentBaseScore + delta),
    }
  }

  reoptimizeFrom(
    req: ReoptimizeRequest,
    onProgress?: (progress: Progress) => void,
  ): OptimizeHandle {
    let cancelled = false
    let rejectRun: ((err: unknown) => void) | null = null
    const runs = Math.max(1, req.request.runs)
    const budget = req.request.budget.value
    const promise = new Promise<OptimizeOutcome>((resolve, reject) => {
      rejectRun = reject
      let step = 0
      const totalSteps = 5
      const interval = setInterval(() => {
        if (cancelled) {
          clearInterval(interval)
          return
        }
        step += 1
        if (onProgress) {
          for (let run = 0; run < runs; run++) {
            onProgress({
              run,
              iteration: Math.floor((step / totalSteps) * budget),
              elapsed_ms: step * 100,
              current_score: 120.0 - step * 10,
              best_score: 100.0 - step * 8 + run * 0.01,
            })
          }
        }
        if (step >= totalSteps) {
          clearInterval(interval)
          const outcome: OptimizeOutcome = JSON.parse(JSON.stringify(demoOutcome))
          resolve(outcome)
        }
      }, 100)
    })

    return {
      promise,
      cancel: async () => {
        cancelled = true
        // Like the real backend (RA-011): a cancelled job rejects with `cancelled`.
        rejectRun?.({
          code: 'cancelled',
          params: { message: 'Optimization was cancelled' },
        })
        return true
      },
    }
  }

  // Excel Import & Export
  async generateImportTemplate(_targetPath: string): Promise<void> {
    // In mock/browser mode, simulate template generation
  }

  async previewImport(
    _schoolYearId: number,
    _filePath: string,
    mode: string,
  ): Promise<ImportPreviewResult> {
    const isSync = mode === 'sync'
    return {
      mode,
      can_apply: true,
      campuses: [
        {
          row_index: 2,
          status: 'unchanged',
          code: 'PH1',
          name: 'Phân hiệu 1',
          errors: [],
        },
        {
          row_index: 3,
          status: 'new',
          code: 'PH3',
          name: 'Phân hiệu 3',
          errors: [],
        },
      ],
      teachers: [
        {
          row_index: 2,
          status: 'update',
          code: 'GV001',
          full_name: 'Nguyễn Văn A',
          display_name: null,
          campus_code: 'PH1',
          grades_str: '10, 11',
          grade_codes: [10, 11],
          load_weight: 1.0,
          active: true,
          note: null,
          matched_teacher_id: BigInt(1),
          quota_override: null,
          max_tasks_per_exam_override: null,
          errors: [],
        },
        {
          row_index: 3,
          status: 'new',
          code: 'GV020',
          full_name: 'Trần Thị Mới',
          display_name: null,
          campus_code: 'PH2',
          grades_str: '12',
          grade_codes: [12],
          load_weight: 0.5,
          active: true,
          note: 'Giáo viên thỉnh giảng',
          matched_teacher_id: null,
          quota_override: null,
          max_tasks_per_exam_override: null,
          errors: [],
        },
      ],
      unavailabilities: [
        {
          row_index: 2,
          status: 'new',
          teacher_ref: 'GV001',
          exam_code: 'GK1',
          reason: 'Bận công tác',
          matched_teacher_id: BigInt(1),
          errors: [],
        },
      ],
      subjects: [
        {
          row_index: 2,
          status: 'new',
          code: 'VL',
          name: 'Vật lí',
          setters: 2,
          reviewers: 1,
          min_campuses: 2,
          color: '#2563EB',
          errors: [],
        },
        {
          row_index: 3,
          status: 'new',
          code: 'CN',
          name: 'Công nghệ',
          setters: 1,
          reviewers: 1,
          min_campuses: 2,
          color: '#10B981',
          errors: [],
        },
      ],
      competencies: [
        {
          row_index: 2,
          status: 'new',
          teacher_ref: 'GV001',
          subject_code: 'VL',
          role: 'Cả hai',
          grade_scope: 'Theo khối dạy',
          matched_teacher_id: BigInt(1),
          matched_subject_id: null,
          errors: [],
        },
      ],
      campuses_summary: {
        new_count: 1,
        update_count: 0,
        unchanged_count: 1,
        error_count: 0,
      },
      teachers_summary: {
        new_count: 1,
        update_count: 1,
        unchanged_count: 0,
        error_count: 0,
      },
      unavailabilities_summary: {
        new_count: 1,
        update_count: 0,
        unchanged_count: 0,
        error_count: 0,
      },
      subjects_summary: {
        new_count: 2,
        update_count: 0,
        unchanged_count: 0,
        error_count: 0,
      },
      competencies_summary: {
        new_count: 1,
        update_count: 0,
        unchanged_count: 0,
        error_count: 0,
      },
      deactivated_teachers: isSync
        ? [
            {
              id: BigInt(99),
              code: 'GV099',
              full_name: 'Lê Văn Cũ',
              campus_name: 'Phân hiệu 1',
            },
          ]
        : [],
      feasibility_report: {
        report: {
          is_feasible: true,
          errors: [],
          warnings: [],
          quotas: [],
        },
        quotas: [],
      },
    }
  }

  async applyImport(
    _schoolYearId: number,
    preview: ImportPreviewResult,
  ): Promise<ImportApplyResult> {
    return {
      backup_path: 'data/backups/exampanel-backup-auto-import-preview.db',
      campuses_created: preview.campuses_summary.new_count,
      campuses_updated: preview.campuses_summary.update_count,
      teachers_created: preview.teachers_summary.new_count,
      teachers_updated: preview.teachers_summary.update_count,
      teachers_deactivated: preview.deactivated_teachers.length,
      unavailabilities_created: preview.unavailabilities_summary.new_count,
      subjects_created: preview.subjects_summary.new_count,
      subjects_updated: preview.subjects_summary.update_count,
      competencies_created: preview.competencies_summary.new_count,
    }
  }

  async exportPlanExcel(_planId: number, _targetPath: string): Promise<void> {
    // In mock mode, simulate export
  }

  async previewImportPlan(
    _schoolYearId: number,
    _filePath?: string,
    _tsvContent?: string,
  ): Promise<PlanImportPreview> {
    const demoPlan = this.planDetailsMap.get(1)
    const assignments: Assignment[] = demoPlan ? demoPlan.assignments : []
    const teacherTotals: PlanImportTeacherTotal[] = this.teachers.map((t) => {
      const count = assignments.filter((a: Assignment) => a.teacher_id === t.id).length
      const de = assignments.filter(
        (a: Assignment) => a.teacher_id === t.id && a.role === 'setter',
      ).length
      const pb = assignments.filter(
        (a: Assignment) => a.teacher_id === t.id && a.role === 'reviewer',
      ).length
      return {
        teacher_id: t.id,
        teacher_name: t.full_name,
        display_name: t.display_name || t.full_name,
        file_total: count,
        computed_total: count,
        setter_count: de,
        reviewer_count: pb,
      }
    })
    return {
      assignments: JSON.parse(JSON.stringify(assignments)),
      teacher_totals: teacherTotals,
      errors: [],
      warnings: [],
      can_apply: true,
      hard_violations: [],
      score_report: demoPlan?.score_report || null,
    }
  }

  async applyImportedPlan(input: ApplyPlanImportInput): Promise<number> {
    const nextId = this.plans.reduce((max, pl) => Math.max(max, pl.id), 0) + 1
    const summary: PlanSummary = {
      id: nextId,
      name: input.plan_name?.trim() || 'Nhập từ bảng của tổ',
      created_at: new Date().toISOString(),
      score: 0,
      is_final: false,
      is_stale: false,
      source: 'manual',
    }
    this.plans.push(summary)

    const newPlanDetails: PlanDetails = {
      plan: {
        id: nextId,
        school_year_id: input.school_year_id,
        name: summary.name,
        created_at: summary.created_at,
        seed: 0,
        score: 0,
        is_final: false,
        source: 'manual',
        run_params_json: JSON.stringify({ origin: 'import' }),
      },
      assignments: input.assignments.map((a: Assignment) => ({
        ...a,
        plan_id: nextId,
      })),
      score_report: undefined,
    }
    this.planDetailsMap.set(nextId, newPlanDetails)
    return nextId
  }

  // Backup & Restore
  private mockBackups: BackupFileInfo[] = [
    {
      filename: 'exampanel-backup-20261001-1400.db',
      path: 'data/backups/exampanel-backup-20261001-1400.db',
      size_bytes: BigInt(262144),
      modified_at: '2026-10-01 14:00:00',
    },
    {
      filename: 'exampanel-backup-20260928-0915.db',
      path: 'data/backups/exampanel-backup-20260928-0915.db',
      size_bytes: BigInt(245760),
      modified_at: '2026-09-28 09:15:00',
    },
  ]

  async backupDatabase(targetPath: string): Promise<void> {
    const filename = targetPath.split(/[\\/]/).pop() || 'backup.db'
    this.mockBackups.unshift({
      filename,
      path: targetPath,
      size_bytes: BigInt(250000),
      modified_at: new Date().toISOString().replace('T', ' ').substring(0, 19),
    })
  }

  async restoreDatabase(_sourcePath: string): Promise<void> {
    // In mock mode, simulate restore
  }

  async validateBackup(path: string): Promise<BackupValidationSummary> {
    const filename = path.split(/[\\/]/).pop() || 'backup.db'
    if (filename.includes('corrupted') || filename.includes('invalid')) {
      return {
        valid: false,
        user_version: 0,
        school_years_count: 0,
        teachers_count: 0,
        plans_count: 0,
        error: 'Database integrity failure: *** in database main ***',
        error_code: 'corrupted',
        supported_version: 5,
      }
    }
    if (filename.includes('newer')) {
      return {
        valid: false,
        user_version: 99,
        school_years_count: 0,
        teachers_count: 0,
        plans_count: 0,
        error: 'Unsupported future database version: 99 > supported 5',
        error_code: 'newer_version',
        supported_version: 5,
      }
    }
    return {
      valid: true,
      user_version: 4,
      school_years_count: 2,
      teachers_count: 18,
      plans_count: 3,
      error: null,
      error_code: null,
      supported_version: 5,
    }
  }

  async listBackups(): Promise<BackupFileInfo[]> {
    return [...this.mockBackups]
  }

  // Dev Tools
  async seedDemo(): Promise<void> {
    // Re-initialize from demo fixtures
    this.campuses = JSON.parse(JSON.stringify(demoCampuses))
    this.grades = JSON.parse(JSON.stringify(demoGrades))
    this.teachers = JSON.parse(JSON.stringify(demoTeachers))
    this.exams = JSON.parse(JSON.stringify(demoExams))
    this.ruleSettings = JSON.parse(JSON.stringify(demoRules))
    this.plans = JSON.parse(JSON.stringify(demoPlans))
    this.planDetailsMap = new Map([
      [1, JSON.parse(JSON.stringify(demoPlanDetails))],
      [
        2,
        JSON.parse(
          JSON.stringify({
            ...demoPlanDetails,
            plan: { ...demoPlanDetails.plan, id: 2, name: 'Phương án #2', rank: 2 },
          }),
        ),
      ],
      [
        3,
        JSON.parse(
          JSON.stringify({
            ...demoPlanDetails,
            plan: { ...demoPlanDetails.plan, id: 3, name: 'Phương án #3', rank: 3 },
          }),
        ),
      ],
    ])
    this.subjects = [
      {
        id: 1,
        code: 'VL',
        name: 'Vật lí',
        color: 'palette-1',
        sort_order: 1,
        setters: 2,
        reviewers: 1,
        min_campuses: 2,
      },
      {
        id: 2,
        code: 'CN',
        name: 'Công nghệ',
        color: 'palette-2',
        sort_order: 2,
        setters: 1,
        reviewers: 1,
        min_campuses: 2,
      },
    ]
    this.competencies = []
  }
}

export const mockApi = new MockExamPanelApi()
