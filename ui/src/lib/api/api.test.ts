import { beforeEach, describe, expect, it } from 'vitest'
import { MockExamPanelApi } from './mock'
import type { AppError } from './types'

describe('MockExamPanelApi Contract Tests', () => {
  let api: MockExamPanelApi

  beforeEach(() => {
    localStorage.clear()
    api = new MockExamPanelApi()
  })

  // ---------------------------------------------------------------------------
  // Ping & App Info & Settings
  // ---------------------------------------------------------------------------
  it('ping returns expected mock string', async () => {
    const res = await api.ping()
    expect(res).toContain('pong from Mock Engine')
  })

  it('getAppInfo returns app metadata with mock mode', async () => {
    const info = await api.getAppInfo()
    expect(info.name).toBe('ExamPanel')
    expect(info.identifier).toBe('vn.exampanel.app')
    expect(info.mode).toBe('mock')
    expect(info.is_portable).toBe(true)
  })

  it('persists and retrieves theme', async () => {
    expect(await api.getTheme()).toBeNull()
    await api.setTheme('dark')
    expect(await api.getTheme()).toBe('dark')
    await api.setTheme('light')
    expect(await api.getTheme()).toBe('light')
  })

  it('persists and retrieves settings and language', async () => {
    const settings = await api.getSettings()
    expect(settings.theme).toBe('system')
    expect(settings.language).toBe('vi')

    await api.setSetting('theme', 'dark')
    await api.setSetting('language', 'en')

    const updated = await api.getSettings()
    expect(updated.theme).toBe('dark')
    expect(updated.language).toBe('en')
    expect(await api.getLanguage()).toBe('en')
  })

  // ---------------------------------------------------------------------------
  // Campuses & Grades CRUD
  // ---------------------------------------------------------------------------
  it('performs campuses CRUD and prevents duplicate code', async () => {
    const initial = await api.listCampuses()
    expect(initial.length).toBe(4)

    const created = await api.createCampus({
      code: 'CS_NEW',
      name: 'Cơ sở Mới',
      color: '#ffffff',
    })
    expect(created.code).toBe('CS_NEW')

    await api.updateCampus({
      ...created,
      name: 'Cơ sở Đã Đổi Tên',
    })
    const afterUpdate = await api.listCampuses()
    expect(afterUpdate.find((c) => c.id === created.id)?.name).toBe(
      'Cơ sở Đã Đổi Tên'
    )

    await api.deleteCampus(created.id)
    const afterDelete = await api.listCampuses()
    expect(afterDelete.length).toBe(initial.length)

    // Duplicate code rejection
    try {
      await api.createCampus({
        code: 'CS1',
        name: 'Trùng CS1',
        color: '#000',
      })
      expect.unreachable('Should fail with duplicate_entry')
    } catch (err) {
      const appErr = err as AppError
      expect(appErr.code).toBe('duplicate_entry')
    }
  })

  it('performs grades CRUD and prevents duplicate code', async () => {
    const initial = await api.listGrades()
    expect(initial.length).toBe(3)

    const created = await api.createGrade({
      code: 13,
      name: 'Khối 13',
      sort_order: 4,
    })
    expect(created.code).toBe(13)

    await api.updateGrade({ ...created, name: 'Khối 13 VIP' })
    const list = await api.listGrades()
    expect(list.find((g) => g.id === created.id)?.name).toBe('Khối 13 VIP')

    await api.deleteGrade(created.id)
    expect((await api.listGrades()).length).toBe(initial.length)

    // Duplicate code rejection
    try {
      await api.createGrade({
        code: 10,
        name: 'Trùng khối 10',
        sort_order: 1,
      })
      expect.unreachable('Should fail with duplicate_entry')
    } catch (err) {
      const appErr = err as AppError
      expect(appErr.code).toBe('duplicate_entry')
    }
  })

  // ---------------------------------------------------------------------------
  // Teachers & Grades CRUD
  // ---------------------------------------------------------------------------
  it('performs teachers CRUD, deactivation, and grade qualifications', async () => {
    const teachers = await api.listTeachers()
    expect(teachers.length).toBe(11)

    const newTeacher = await api.createTeacher({
      full_name: 'Phạm Thị Mới',
      campus_id: 1,
      load_weight: 1.0,
      active: true,
      note: 'Thử nghiệm',
    })
    expect(newTeacher.full_name).toBe('Phạm Thị Mới')

    await api.deactivateTeacher(newTeacher.id)
    const afterDeact = await api.listTeachers()
    expect(afterDeact.find((t) => t.id === newTeacher.id)?.active).toBe(false)

    await api.setTeacherGrades(newTeacher.id, 1, [1, 2])
    const twg = await api.teachersWithGrades(1)
    const record = twg.find((t) => t.teacher.id === newTeacher.id)
    expect(record?.grade_ids).toEqual([1, 2])

    await api.deleteTeacher(newTeacher.id)
    expect((await api.listTeachers()).length).toBe(teachers.length)
  })

  // ---------------------------------------------------------------------------
  // School Years, Exams, Unavailability, Locks, Rules
  // ---------------------------------------------------------------------------
  it('manages school years, exams, unavailabilities, locks, and rules', async () => {
    // School years
    const newYear = await api.createSchoolYear({
      name: '2027-2028',
      is_current: false,
    })
    expect(newYear.name).toBe('2027-2028')

    await api.setCurrentSchoolYear(newYear.id)
    const settings = await api.getSettings()
    expect(settings.current_school_year_id).toBe(newYear.id)

    // Exams
    const exams = await api.listExams(1)
    expect(exams.length).toBe(4)
    await api.updateExam({ ...exams[0], name: 'Giữa kỳ 1 (Mới)' })
    const updatedExams = await api.listExams(1)
    expect(updatedExams[0].name).toBe('Giữa kỳ 1 (Mới)')

    // Unavailability
    await api.setUnavailability({ teacher_id: 1, exam_id: 1, reason: 'Bận công tác' })
    const unavs = await api.listUnavailabilities(1)
    expect(unavs.some((u) => u.teacher_id === 1 && u.exam_id === 1)).toBe(true)
    await api.deleteUnavailability(1, 1)
    const unavsAfter = await api.listUnavailabilities(1)
    expect(unavsAfter.some((u) => u.teacher_id === 1 && u.exam_id === 1)).toBe(false)

    // Locks
    const lock = await api.createLock({
      exam_id: 1,
      grade_id: 1,
      teacher_id: 1,
      role: 'setter',
      kind: 'pin',
    })
    expect(lock.kind).toBe('pin')
    const locks = await api.listLocks(1)
    expect(locks.length).toBe(1)
    await api.deleteLock(lock.id)
    expect((await api.listLocks(1)).length).toBe(0)

    // Rules
    const rules = await api.getRuleSettings(1)
    expect(rules.length).toBeGreaterThan(0)
    await api.saveRuleSettings(1, [{ ...rules[0], weight: 50.0 }])
    const modified = await api.getRuleSettings(1)
    expect(modified[0].weight).toBe(50.0)
    await api.resetRuleSettingsToDefaults(1)
  })

  // ---------------------------------------------------------------------------
  // Feasibility & Evaluation
  // ---------------------------------------------------------------------------
  it('checks feasibility and evaluates assignments', async () => {
    const feas = await api.checkFeasibility(1)
    expect(feas.report.is_feasible).toBe(true)
    expect(feas.quotas.length).toBe(11)

    const evalRes = await api.evaluateAssignments(1, [])
    expect(evalRes.hard_violations).toEqual([])
    expect(evalRes.score_report).toBeDefined()
  })

  // ---------------------------------------------------------------------------
  // Optimization Streaming, Progress, Cancel, and Busy Rejection
  // ---------------------------------------------------------------------------
  it('streams optimization progress to completion', async () => {
    const progressList: number[] = []
    const handle = api.startOptimize(
      1,
      {
        base_seed: 42,
        runs: 8,
        budget: { type: 'Iterations', value: 200000 },
        k: 3,
        diversity_threshold: 0.20,
      },
      (p) => {
        progressList.push(p.iteration)
      }
    )

    const outcome = await handle.promise
    expect(outcome.plans.length).toBe(3)
    expect(outcome.lower_bounds.length).toBe(8)
    expect(progressList.length).toBeGreaterThanOrEqual(10)
  })

  it('cancels optimization promptly when requested', async () => {
    const handle = api.startOptimize(
      1,
      {
        base_seed: 42,
        runs: 8,
        budget: { type: 'Iterations', value: 200000 },
        k: 3,
        diversity_threshold: 0.20,
      }
    )

    // Cancel shortly after start
    setTimeout(() => {
      handle.cancel()
    }, 250)

    try {
      await handle.promise
      expect.unreachable('Should reject upon cancellation')
    } catch (err) {
      const appErr = err as AppError
      expect(appErr.code).toBe('cancelled')
    }
  })

  it('rejects concurrent optimization with optimize_busy', async () => {
    const handle1 = api.startOptimize(1, {
      base_seed: 42,
      runs: 8,
      budget: { type: 'Iterations', value: 200000 },
      k: 3,
      diversity_threshold: 0.20,
    })

    try {
      api.startOptimize(1, {
        base_seed: 43,
        runs: 8,
        budget: { type: 'Iterations', value: 200000 },
        k: 3,
        diversity_threshold: 0.20,
      })
      expect.unreachable('Concurrent start should throw optimize_busy')
    } catch (err) {
      const appErr = err as AppError
      expect(appErr.code).toBe('optimize_busy')
    }

    handle1.cancel()
    try {
      await handle1.promise
    } catch {
      // Expected cancellation
    }
  })

  // ---------------------------------------------------------------------------
  // Plans Persistence & Management
  // ---------------------------------------------------------------------------
  it('saves, retrieves, duplicates, renames, marks final, and deletes plans', async () => {
    const handle = api.startOptimize(1, {
      base_seed: 42,
      runs: 8,
      budget: { type: 'Iterations', value: 200000 },
      k: 3,
      diversity_threshold: 0.20,
    })
    const outcome = await handle.promise

    const savedIds = await api.saveOptimizeResult(1, outcome)
    expect(savedIds.length).toBe(3)

    const plans = await api.listPlans(1)
    expect(plans.length).toBeGreaterThanOrEqual(3)

    const planDetails = await api.getPlan(savedIds[0])
    expect(planDetails.assignments.length).toBe(36)
    expect(planDetails.plan.source).toBe('optimizer')

    await api.renamePlan(savedIds[0], 'Phương án tốt nhất đã chọn')
    const renamed = await api.getPlan(savedIds[0])
    expect(renamed.plan.name).toBe('Phương án tốt nhất đã chọn')

    await api.markFinal(savedIds[0])
    const finalPlan = await api.getPlan(savedIds[0])
    expect(finalPlan.plan.is_final).toBe(true)

    const dupId = await api.duplicatePlan(savedIds[0], 'Bản sao để sửa tay')
    const dup = await api.getPlan(dupId)
    expect(dup.plan.source).toBe('duplicate')
    expect(dup.assignments.length).toBe(36)

    await api.deletePlan(dupId)
    try {
      await api.getPlan(dupId)
      expect.unreachable('Deleted plan should not be found')
    } catch (err) {
      const appErr = err as AppError
      expect(appErr.code).toBe('not_found')
    }
  })
})
