import * as React from 'react'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { api } from '@/lib/api'
import {
  usePlans,
  useSetUnavailability,
  useDeleteUnavailability,
  useCreateTeacher,
  useUpdateTeacher,
  useDeleteTeacher,
  useDeactivateTeacher,
  useCreateExam,
  useUpdateExam,
  useDeleteExam,
  useReorderExams,
  useSaveRuleSettings,
  useResetRuleSettings,
  useCreateLock,
  useDeleteLock,
  useUpdateSubject,
  useSetCompetency,
  useToggleTeacherGrade,
} from './hooks'

vi.mock('sonner', () => ({ toast: { success: vi.fn(), info: vi.fn(), error: vi.fn() } }))

const SY = 1

// RA-022: changing problem data makes saved plans stale on the backend, but the cached plans
// list (staleTime 5 min) was never refreshed, so the "Dữ liệu đã đổi" badge only appeared after
// a window reload. Every mutation that changes problem data must refresh the plans list.
const cases: Array<{
  name: string
  api: keyof typeof api
  run: (h: ReturnType<typeof useAllHooks>) => Promise<unknown>
}> = [
  {
    name: 'set unavailability',
    api: 'setUnavailability',
    run: (h) => h.setUnavail.mutateAsync({ teacher_id: 1, exam_id: 1, reason: null }),
  },
  {
    name: 'delete unavailability',
    api: 'deleteUnavailability',
    run: (h) => h.delUnavail.mutateAsync({ teacherId: 1, examId: 1 }),
  },
  {
    name: 'create teacher',
    api: 'createTeacher',
    run: (h) =>
      h.createTeacher.mutateAsync({
        input: { full_name: 'X', campus_id: 1, load_weight: 1, active: true, note: null },
        gradeIds: [],
      }),
  },
  {
    name: 'update teacher',
    api: 'updateTeacher',
    run: (h) =>
      h.updateTeacher.mutateAsync({
        teacher: {
          id: 1,
          full_name: 'X',
          campus_id: 1,
          load_weight: 1,
          active: true,
          note: null,
        },
      }),
  },
  {
    name: 'delete teacher',
    api: 'deleteTeacher',
    run: (h) => h.delTeacher.mutateAsync(1),
  },
  {
    name: 'deactivate teacher',
    api: 'deactivateTeacher',
    run: (h) => h.deactTeacher.mutateAsync(1),
  },
  {
    name: 'toggle teacher grade',
    api: 'setTeacherGrades',
    run: (h) => h.toggleGrade.mutateAsync({ teacherId: 1, gradeIds: [1] }),
  },
  {
    name: 'create exam',
    api: 'createExam',
    run: (h) =>
      h.createExam.mutateAsync({
        school_year_id: SY,
        code: 'X',
        name: 'X',
        sort_order: 9,
      }),
  },
  {
    name: 'update exam',
    api: 'updateExam',
    run: (h) =>
      h.updateExam.mutateAsync({
        id: 1,
        school_year_id: SY,
        code: 'X',
        name: 'X',
        sort_order: 1,
      }),
  },
  { name: 'delete exam', api: 'deleteExam', run: (h) => h.delExam.mutateAsync(1) },
  {
    name: 'reorder exams',
    api: 'reorderExams',
    run: (h) => h.reorderExams.mutateAsync([1]),
  },
  {
    name: 'save rules',
    api: 'saveRuleSettings',
    run: (h) => h.saveRules.mutateAsync([]),
  },
  {
    name: 'reset rules',
    api: 'resetRuleSettingsToDefaults',
    run: (h) => h.resetRules.mutateAsync(),
  },
  {
    name: 'create lock',
    api: 'createLock',
    run: (h) =>
      h.createLock.mutateAsync({
        exam_id: 1,
        grade_id: 1,
        subject_id: 1,
        teacher_id: 1,
        role: 'setter',
        kind: 'pin',
      }),
  },
  { name: 'delete lock', api: 'deleteLock', run: (h) => h.delLock.mutateAsync(1) },
  {
    name: 'update subject',
    api: 'updateSubject',
    run: (h) =>
      h.updateSubject.mutateAsync({
        id: 1,
        code: 'VL',
        name: 'VL',
        setters: 2,
        reviewers: 1,
        min_campuses: 0,
        sort_order: 1,
      } as never),
  },
  {
    name: 'set competency',
    api: 'setCompetency',
    run: (h) =>
      h.setCompetency.mutateAsync({
        teacher_id: 1,
        subject_id: 1,
        role: 'setter',
        grade_scope: 'taught',
        school_year_id: SY,
      } as never),
  },
]

function useAllHooks() {
  return {
    plans: usePlans(SY),
    setUnavail: useSetUnavailability(SY),
    delUnavail: useDeleteUnavailability(SY),
    createTeacher: useCreateTeacher(SY),
    updateTeacher: useUpdateTeacher(SY),
    delTeacher: useDeleteTeacher(SY),
    deactTeacher: useDeactivateTeacher(SY),
    toggleGrade: useToggleTeacherGrade(SY),
    createExam: useCreateExam(SY),
    updateExam: useUpdateExam(SY),
    delExam: useDeleteExam(SY),
    reorderExams: useReorderExams(SY),
    saveRules: useSaveRuleSettings(SY),
    resetRules: useResetRuleSettings(SY),
    createLock: useCreateLock(SY),
    delLock: useDeleteLock(SY),
    updateSubject: useUpdateSubject(SY),
    setCompetency: useSetCompetency(SY),
  }
}

describe('plans list is refreshed after problem data changes (RA-022)', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it.each(cases)('$name refetches the plans list', async ({ api: method, run }) => {
    const listPlans = vi.spyOn(api, 'listPlans').mockResolvedValue([])
    ;(
      vi.spyOn(api, method as never) as unknown as {
        mockResolvedValue: (v: unknown) => void
      }
    ).mockResolvedValue(method === 'createTeacher' ? { id: 99 } : undefined)
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
    })
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    )
    const { result } = renderHook(() => useAllHooks(), { wrapper })
    await waitFor(() => expect(listPlans).toHaveBeenCalledTimes(1))

    await act(async () => {
      await run(result.current)
    })

    await waitFor(() => expect(listPlans.mock.calls.length).toBeGreaterThan(1))
  })
})
