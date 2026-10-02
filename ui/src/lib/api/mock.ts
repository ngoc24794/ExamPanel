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
  Assignment,
  Campus,
  CreateCampusInput,
  CreateExamInput,
  CreateGradeInput,
  CreateLockInput,
  CreateSchoolYearInput,
  CreateTeacherInput,
  EvaluationOutcome,
  Exam,
  ExamPanelApi,
  FeasibilityReportWithQuotas,
  Grade,
  Lock,
  OptimizeHandle,
  OptimizeOutcome,
  OptimizeRequest,
  PlanDetails,
  PlanSummary,
  PreviewQuotasInput,
  Progress,
  QuotaPreviewItem,
  RulePresetItem,
  RuleSetting,
  SchoolYear,
  Teacher,
  TeacherQuota,
  TeacherWithGrades,
  ThemeMode,
  Unavailability,
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
  ])
  private teacherGradesMap: Map<number, number[]> = new Map([
    [1, [1, 2]],
    [2, [1, 2]],
    [3, [1, 3]],
    [4, [2, 3]],
    [5, [2, 3]],
    [6, [1, 2]],
    [7, [1, 3]],
    [8, [2, 3]],
    [9, [1, 2]],
    [10, [2, 3]],
    [11, [1, 3]],
  ])

  private isOptimizing = false
  private activeCancelCallback: (() => void) | null = null
  private savedTheme: ThemeMode | null = null

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
      db_path: '/mock/data/exampanel.db',
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
      school_year_id: 1,
      exam_id: input.exam_id,
      grade_id: input.grade_id,
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
          capacity: { target_slots: 0, active_teachers: 0, total_weight: 0 },
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
          capacity: { target_slots: 0, active_teachers: 0, total_weight: 0 },
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
          capacity: {
            target_slots: 0,
            active_teachers: activeTeachers.length,
            total_weight: totalWeight,
          },
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
          capacity: {
            target_slots: 0,
            active_teachers: activeTeachers.length,
            total_weight: totalWeight,
          },
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
          capacity: {
            target_slots: 0,
            active_teachers: activeTeachers.length,
            total_weight: totalWeight,
          },
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
          panel: { exam_id: this.exams[0]?.id ?? 1, grade_id: grade.id },
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
            panel: { exam_id: this.exams[0]?.id ?? 1, grade_id: grade.id },
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
            panel: { exam_id: this.exams[0]?.id ?? 1, grade_id: grade.id },
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
            panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
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
            panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
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
          panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
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
          panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
          params: { count: pins.length.toString() },
        })
      }
      const pinnedSetters = pins.filter((l) => l.role === 'setter')
      if (pinnedSetters.length > 2) {
        const lock = pinnedSetters[0]
        errors.push({
          rule: 'h6',
          code: 'excess_pinned_setters_and_reviewers_and_pins',
          panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
          params: { count: pinnedSetters.length.toString() },
        })
      }
      const pinnedReviewers = pins.filter((l) => l.role === 'reviewer')
      if (pinnedReviewers.length > 1) {
        const lock = pinnedReviewers[0]
        errors.push({
          rule: 'h6',
          code: 'excess_pinned_setters_and_reviewers_and_pins',
          panel: { exam_id: lock.exam_id, grade_id: lock.grade_id },
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
        capacity: {
          target_slots: totalSlots,
          active_teachers: activeTeachers.length,
          total_weight: totalWeight,
        },
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
    _request: OptimizeRequest,
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
        const iteration = Math.floor((step / totalSteps) * 200000)
        const best_score = Math.max(2.6, 25.0 - step * 2.2)
        const current_score = best_score + Math.random() * 3.0

        if (onProgress) {
          onProgress({
            run: 1 + Math.floor(step / 3),
            iteration,
            best_score: Number(best_score.toFixed(2)),
            current_score: Number(current_score.toFixed(2)),
            elapsed_ms: step * stepDurationMs,
          })
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
        name: `Kế hoạch #${p.rank}`,
        rank: p.rank,
        score: p.report.total,
        created_at: new Date().toISOString(),
        is_final: false,
        source: 'optimizer',
      }
      this.plans.push(summary)
      this.planDetailsMap.set(nextId, {
        plan: {
          ...summary,
          school_year_id: 1,
          seed: p.seed,
          score_report_json: JSON.stringify(p.report),
          run_params_json: outcome.run_params_json,
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

  async markFinal(id: number): Promise<void> {
    const plan = this.plans.find((p) => p.id === id)
    if (!plan) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    for (const p of this.plans) {
      p.is_final = p.id === id
      const d = this.planDetailsMap.get(p.id)
      if (d) d.plan.is_final = p.is_final
    }
  }

  async duplicatePlan(id: number, newName: string): Promise<number> {
    const orig = this.planDetailsMap.get(id)
    if (!orig) {
      throw { code: 'not_found', params: { message: 'Plan not found' } }
    }
    const nextId = this.plans.reduce((max, pl) => Math.max(max, pl.id), 0) + 1
    const summary: PlanSummary = {
      id: nextId,
      name: newName,
      rank: null,
      score: orig.plan.score,
      created_at: new Date().toISOString(),
      is_final: false,
      source: 'duplicate',
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

  async seedDemo(): Promise<void> {
    // Re-initialize from demo fixtures
    this.campuses = JSON.parse(JSON.stringify(demoCampuses))
    this.grades = JSON.parse(JSON.stringify(demoGrades))
    this.teachers = JSON.parse(JSON.stringify(demoTeachers))
    this.exams = JSON.parse(JSON.stringify(demoExams))
    this.ruleSettings = JSON.parse(JSON.stringify(demoRules))
    this.plans = JSON.parse(JSON.stringify(demoPlans))
  }
}

export const mockApi = new MockExamPanelApi()
