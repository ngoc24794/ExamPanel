import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  api,
  type Campus,
  type CreateCampusInput,
  type CreateExamInput,
  type CreateGradeInput,
  type CreateLockInput,
  type CreateSchoolYearInput,
  type CreateTeacherInput,
  type Exam,
  type Grade,
  type RuleSetting,
  type Teacher,
  type TeacherWithGrades,
  type Unavailability,
} from '@/lib/api'
import { toast } from 'sonner'
import i18n from '@/i18n'
import { getErrorMessage } from './query-client'

export const queryKeys = {
  appInfo: ['appInfo'] as const,
  settings: ['settings'] as const,
  schoolYears: ['schoolYears'] as const,
  campuses: ['campuses'] as const,
  grades: ['grades'] as const,
  teachers: (schoolYearId: number) => ['teachers', schoolYearId] as const,
  exams: (schoolYearId: number) => ['exams', schoolYearId] as const,
  unavailabilities: (schoolYearId: number) => ['unavailabilities', schoolYearId] as const,
  locks: (schoolYearId: number) => ['locks', schoolYearId] as const,
  ruleSettings: (schoolYearId: number) => ['ruleSettings', schoolYearId] as const,
  rulePresets: ['rulePresets'] as const,
  feasibility: (schoolYearId: number) => ['feasibility', schoolYearId] as const,
}

// Queries
export function useAppInfo() {
  return useQuery({
    queryKey: queryKeys.appInfo,
    queryFn: () => api.getAppInfo(),
  })
}

export function useSettings() {
  return useQuery({
    queryKey: queryKeys.settings,
    queryFn: () => api.getSettings(),
  })
}

export function useSchoolYears() {
  return useQuery({
    queryKey: queryKeys.schoolYears,
    queryFn: () => api.listSchoolYears(),
  })
}

export function useCampuses() {
  return useQuery({
    queryKey: queryKeys.campuses,
    queryFn: () => api.listCampuses(),
  })
}

export function useGrades() {
  return useQuery({
    queryKey: queryKeys.grades,
    queryFn: () => api.listGrades(),
  })
}

export function useTeachers(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.teachers(schoolYearId ?? 0),
    queryFn: () =>
      schoolYearId ? api.teachersWithGrades(schoolYearId) : Promise.resolve([]),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

export function useExams(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.exams(schoolYearId ?? 0),
    queryFn: () => (schoolYearId ? api.listExams(schoolYearId) : Promise.resolve([])),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

export function useUnavailabilities(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.unavailabilities(schoolYearId ?? 0),
    queryFn: () =>
      schoolYearId ? api.listUnavailabilities(schoolYearId) : Promise.resolve([]),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

export function useLocks(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.locks(schoolYearId ?? 0),
    queryFn: () => (schoolYearId ? api.listLocks(schoolYearId) : Promise.resolve([])),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

export function useRuleSettings(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.ruleSettings(schoolYearId ?? 0),
    queryFn: () =>
      schoolYearId ? api.getRuleSettings(schoolYearId) : Promise.resolve([]),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

export function useRulePresets() {
  return useQuery({
    queryKey: queryKeys.rulePresets,
    queryFn: () => api.getRulePresets(),
  })
}

export function useFeasibility(schoolYearId: number | undefined) {
  return useQuery({
    queryKey: queryKeys.feasibility(schoolYearId ?? 0),
    queryFn: () =>
      schoolYearId ? api.checkFeasibility(schoolYearId) : Promise.reject('No year'),
    enabled: typeof schoolYearId === 'number' && schoolYearId > 0,
  })
}

// Mutations - Campuses
export function useCreateCampus() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CreateCampusInput) => api.createCampus(input),
    onSuccess: (newCampus) => {
      qc.setQueryData<Campus[]>(queryKeys.campuses, (old = []) => [...old, newCampus])
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

export function useUpdateCampus() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (campus: Campus) => api.updateCampus(campus),
    onSuccess: (_, campus) => {
      qc.setQueryData<Campus[]>(queryKeys.campuses, (old = []) =>
        old.map((c) => (c.id === campus.id ? campus : c)),
      )
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

export function useDeleteCampus() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deleteCampus(id),
    onSuccess: (_, id) => {
      qc.setQueryData<Campus[]>(queryKeys.campuses, (old = []) =>
        old.filter((c) => c.id !== id),
      )
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

// Mutations - Teachers
export function useCreateTeacher(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({
      input,
      gradeIds,
    }: {
      input: CreateTeacherInput
      gradeIds: number[]
    }) => {
      const teacher = await api.createTeacher(input)
      if (gradeIds.length > 0) {
        await api.setTeacherGrades(teacher.id, schoolYearId, gradeIds)
      }
      return teacher
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

export function useUpdateTeacher(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({
      teacher,
      gradeIds,
    }: {
      teacher: Teacher
      gradeIds?: number[]
    }) => {
      await api.updateTeacher(teacher)
      if (gradeIds !== undefined) {
        await api.setTeacherGrades(teacher.id, schoolYearId, gradeIds)
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

export function useDeleteTeacher(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deleteTeacher(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.campuses })
    },
  })
}

export function useDeactivateTeacher(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deactivateTeacher(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

// Optimistic Grade Toggle with Rollback
export function useToggleTeacherGrade(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ teacherId, gradeIds }: { teacherId: number; gradeIds: number[] }) =>
      api.setTeacherGrades(teacherId, schoolYearId, gradeIds),
    onMutate: async ({ teacherId, gradeIds }) => {
      await qc.cancelQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      const previous = qc.getQueryData<TeacherWithGrades[]>(
        queryKeys.teachers(schoolYearId),
      )

      if (previous) {
        qc.setQueryData<TeacherWithGrades[]>(
          queryKeys.teachers(schoolYearId),
          previous.map((item) =>
            item.teacher.id === teacherId ? { ...item, grade_ids: gradeIds } : item,
          ),
        )
      }
      return { previous }
    },
    onError: (err, _variables, context) => {
      if (context?.previous) {
        qc.setQueryData(queryKeys.teachers(schoolYearId), context.previous)
      }
      toast.error(getErrorMessage(err))
    },
    onSettled: () => {
      qc.invalidateQueries({ queryKey: queryKeys.teachers(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

// School Years
export function useCreateSchoolYear() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CreateSchoolYearInput) => api.createSchoolYear(input),
    onSuccess: (newYear) => {
      qc.invalidateQueries({ queryKey: queryKeys.schoolYears })
      qc.invalidateQueries({ queryKey: queryKeys.settings })
      toast.success(
        i18n.t('schoolYear.createSuccess', { defaultValue: 'Tạo năm học thành công' }),
      )
      return newYear
    },
  })
}

export function useSetCurrentSchoolYear() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.setCurrentSchoolYear(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.schoolYears })
      qc.invalidateQueries({ queryKey: queryKeys.settings })
    },
  })
}

// Demo Data Seed
export function useSeedDemo() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: () => api.seedDemo(),
    onSuccess: () => {
      qc.invalidateQueries()
      toast.success(
        i18n.t('dev.seedSuccess', { defaultValue: 'Đã nạp dữ liệu mẫu thành công' }),
      )
    },
  })
}

// Grades
export function useCreateGrade() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CreateGradeInput) => api.createGrade(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.grades })
      qc.invalidateQueries({ queryKey: ['feasibility'] })
    },
  })
}

export function useUpdateGrade() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (grade: Grade) => api.updateGrade(grade),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.grades })
      qc.invalidateQueries({ queryKey: ['feasibility'] })
    },
  })
}

export function useDeleteGrade() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deleteGrade(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.grades })
      qc.invalidateQueries({ queryKey: ['feasibility'] })
    },
  })
}

// Exams
export function useCreateExam(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CreateExamInput) => api.createExam(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.exams(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

export function useUpdateExam(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (exam: Exam) => api.updateExam(exam),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.exams(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

export function useDeleteExam(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deleteExam(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.exams(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

export function useReorderExams(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (examIds: number[]) => api.reorderExams(examIds),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.exams(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

// Unavailability (Optimistic toggle with rollback)
export function useSetUnavailability(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (unavailability: Unavailability) => api.setUnavailability(unavailability),
    onMutate: async (unavail) => {
      await qc.cancelQueries({ queryKey: queryKeys.unavailabilities(schoolYearId) })
      const previous = qc.getQueryData<Unavailability[]>(
        queryKeys.unavailabilities(schoolYearId),
      )
      if (previous) {
        qc.setQueryData<Unavailability[]>(queryKeys.unavailabilities(schoolYearId), [
          ...previous.filter(
            (u) =>
              !(u.teacher_id === unavail.teacher_id && u.exam_id === unavail.exam_id),
          ),
          unavail,
        ])
      }
      return { previous }
    },
    onError: (err, _vars, context) => {
      if (context?.previous) {
        qc.setQueryData(queryKeys.unavailabilities(schoolYearId), context.previous)
      }
      toast.error(getErrorMessage(err))
    },
    onSettled: () => {
      qc.invalidateQueries({ queryKey: queryKeys.unavailabilities(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

export function useDeleteUnavailability(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ teacherId, examId }: { teacherId: number; examId: number }) =>
      api.deleteUnavailability(teacherId, examId),
    onMutate: async ({ teacherId, examId }) => {
      await qc.cancelQueries({ queryKey: queryKeys.unavailabilities(schoolYearId) })
      const previous = qc.getQueryData<Unavailability[]>(
        queryKeys.unavailabilities(schoolYearId),
      )
      if (previous) {
        qc.setQueryData<Unavailability[]>(
          queryKeys.unavailabilities(schoolYearId),
          previous.filter((u) => !(u.teacher_id === teacherId && u.exam_id === examId)),
        )
      }
      return { previous }
    },
    onError: (err, _vars, context) => {
      if (context?.previous) {
        qc.setQueryData(queryKeys.unavailabilities(schoolYearId), context.previous)
      }
      toast.error(getErrorMessage(err))
    },
    onSettled: () => {
      qc.invalidateQueries({ queryKey: queryKeys.unavailabilities(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

// Locks
export function useCreateLock(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CreateLockInput) => api.createLock(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.locks(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

export function useDeleteLock(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api.deleteLock(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.locks(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}

// Rule Settings
export function useSaveRuleSettings(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (settings: RuleSetting[]) => api.saveRuleSettings(schoolYearId, settings),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.ruleSettings(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
      toast.success(
        i18n.t('rules.saveSuccess', {
          defaultValue: 'Đã lưu cấu hình quy tắc thành công',
        }),
      )
    },
  })
}

export function useResetRuleSettings(schoolYearId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: () => api.resetRuleSettingsToDefaults(schoolYearId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.ruleSettings(schoolYearId) })
      qc.invalidateQueries({ queryKey: queryKeys.feasibility(schoolYearId) })
    },
  })
}
