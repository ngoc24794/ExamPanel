import type { TFunction } from 'i18next'
import i18n from '@/i18n'
import type { Violation } from '@/lib/api'
import { useExams, useGrades, useSubjects, useTeachers } from '@/lib/query/hooks'

/** Lookups used to put names into a violation sentence; every entry is optional. */
export interface ViolationLookup {
  teacherName?: (id: number) => string | undefined
  examName?: (id: number) => string | undefined
  gradeName?: (id: number) => string | undefined
  subjectName?: (id: number) => string | undefined
}

/**
 * Human sentence for an engine violation (hard rules H1–H7), built from `violations.<code>` so
 * the UI never shows engine codes such as "h4: max_tasks_per_exam_exceeded" (RA-010, RA-021).
 */
export function describeViolation(
  t: TFunction,
  v: Violation,
  lookup: ViolationLookup = {},
): string {
  const teacher =
    v.teacher !== undefined ? (lookup.teacherName?.(v.teacher) ?? `#${v.teacher}`) : ''
  const panelParts: string[] = []
  const examId = v.panel?.exam_id ?? (v.params?.exam_id as number | undefined)
  if (examId !== undefined) panelParts.push(lookup.examName?.(examId) ?? `#${examId}`)
  if (v.panel) {
    panelParts.push(lookup.gradeName?.(v.panel.grade_id) ?? `#${v.panel.grade_id}`)
    panelParts.push(lookup.subjectName?.(v.panel.subject_id) ?? `#${v.panel.subject_id}`)
  }
  const key = `violations.${v.code}`
  if (!i18n.exists(key)) {
    return t('violations.unknown', { rule: v.rule.toUpperCase(), code: v.code })
  }
  const sentence = t(key, {
    ...(v.params ?? {}),
    teacher,
    exam: panelParts[0] ?? '',
  })
  // The panel (exam · grade · subject) is appended once instead of being repeated in each text.
  return v.panel ? `${sentence} (${panelParts.join(' · ')})` : sentence
}

/** Names for violation sentences, loaded from the cached master data of the school year. */
export function useViolationLookup(schoolYearId: number): ViolationLookup {
  const { data: teachers = [] } = useTeachers(schoolYearId)
  const { data: exams = [] } = useExams(schoolYearId)
  const { data: grades = [] } = useGrades()
  const { data: subjects = [] } = useSubjects(schoolYearId)
  return {
    teacherName: (id) => {
      const t = teachers.find((x) => x.teacher.id === id)?.teacher
      return t ? t.display_name || t.full_name : undefined
    },
    examName: (id) => exams.find((e) => e.id === id)?.code,
    gradeName: (id) => grades.find((g) => g.id === id)?.name,
    subjectName: (id) => subjects.find((s) => s.id === id)?.code,
  }
}
