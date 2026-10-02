import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  api,
  type Campus,
  type CreateCampusInput,
  type CreateTeacherInput,
  type Teacher,
  type TeacherWithGrades,
  type CreateSchoolYearInput,
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
