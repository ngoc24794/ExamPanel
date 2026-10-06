import type { Assignment } from '@/lib/api'

/** Seats grouped by subject panel (exam × grade × subject). */
function groupByPanel(assignments: Assignment[]): Map<string, Assignment[]> {
  const panels = new Map<string, Assignment[]>()
  for (const a of assignments) {
    const key = `${a.exam_id}_${a.grade_id}_${a.subject_id ?? 1}`
    const list = panels.get(key)
    if (list) list.push(a)
    else panels.set(key, [a])
  }
  return panels
}

/**
 * Teacher × teacher: number of subject panels the two sit on together. A teacher who works on
 * both the VL and the CN panel of the same exam × grade is a member of two different panels, so
 * panels of different subjects are never merged (RA-027).
 */
export function coWorkingCounts(assignments: Assignment[]): Map<string, number> {
  const matrix = new Map<string, number>()
  for (const seats of groupByPanel(assignments).values()) {
    const members = Array.from(new Set(seats.map((a) => a.teacher_id)))
    for (const x of members) {
      for (const y of members) {
        if (x === y) continue
        const key = `${x}_${y}`
        matrix.set(key, (matrix.get(key) ?? 0) + 1)
      }
    }
  }
  return matrix
}

/** Reviewer → setter: number of subject panels where the reviewer reviews the setter. */
export function reviewRelationCounts(assignments: Assignment[]): Map<string, number> {
  const matrix = new Map<string, number>()
  for (const seats of groupByPanel(assignments).values()) {
    const reviewers = seats.filter((a) => a.role === 'reviewer').map((a) => a.teacher_id)
    const setters = seats.filter((a) => a.role === 'setter').map((a) => a.teacher_id)
    for (const r of reviewers) {
      for (const s of setters) {
        const key = `${r}_${s}`
        matrix.set(key, (matrix.get(key) ?? 0) + 1)
      }
    }
  }
  return matrix
}

/** Number of distinct campuses in each non-empty subject panel. */
export function campusMix(
  assignments: Assignment[],
  campusOf: (teacherId: number) => number | undefined,
): { mix1: number; mix2: number; mix3: number; total: number } {
  let mix1 = 0
  let mix2 = 0
  let mix3 = 0
  let total = 0
  for (const seats of groupByPanel(assignments).values()) {
    total += 1
    const campuses = new Set(
      seats.map((a) => campusOf(a.teacher_id)).filter((c) => c !== undefined),
    )
    if (campuses.size === 1) mix1 += 1
    else if (campuses.size === 2) mix2 += 1
    else if (campuses.size >= 3) mix3 += 1
  }
  return { mix1, mix2, mix3, total }
}
