import type { Assignment } from '@/lib/api'

/**
 * True when some panel (exam × grade × subject) holds the same teacher in more than one seat.
 * The assignments table is keyed by (plan, exam, grade, subject, teacher), so such a plan can
 * never be saved and breaks hard rule H1 (RA-020).
 */
export function hasDuplicateTeacherInPanel(assignments: Assignment[]): boolean {
  const seen = new Set<string>()
  for (const a of assignments) {
    const key = `${a.exam_id}|${a.grade_id}|${a.subject_id ?? 1}|${a.teacher_id}`
    if (seen.has(key)) return true
    seen.add(key)
  }
  return false
}

/** Stable signature of a plan's seats, used as a query key for live evaluation. */
export function assignmentsSignature(assignments: Assignment[]): string {
  return assignments
    .map(
      (a) =>
        `${a.exam_id}.${a.grade_id}.${a.subject_id ?? 1}.${a.role}.${a.position ?? 0}.${a.teacher_id}`,
    )
    .sort()
    .join(',')
}
