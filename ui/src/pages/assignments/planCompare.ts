import type { Assignment, Exam, Grade, Subject } from '@/lib/api'

export interface PanelMembers {
  setters: number[]
  reviewers: number[]
}

export interface PlanComparison {
  /** Number of seats of plan A whose (panel, role, teacher) does not exist in plan B. */
  distance: number
  /** Panel keys `${exam}_${grade}_${subject}` with at least one changed seat. */
  diffPanels: Set<string>
}

export const panelKey = (examId: number, gradeId: number, subjectId: number) =>
  `${examId}_${gradeId}_${subjectId}`

/** Members of one subject panel (setters by position, then reviewers). */
export function panelMembers(
  assignments: Assignment[],
  examId: number,
  gradeId: number,
  subjectId: number,
): PanelMembers {
  const inPanel = assignments.filter(
    (a) =>
      a.exam_id === examId && a.grade_id === gradeId && (a.subject_id ?? 1) === subjectId,
  )
  return {
    setters: inPanel
      .filter((a) => a.role === 'setter')
      .sort((x, y) => (x.position ?? 0) - (y.position ?? 0))
      .map((a) => a.teacher_id),
    reviewers: inPanel.filter((a) => a.role === 'reviewer').map((a) => a.teacher_id),
  }
}

/**
 * Seat-aware plan distance. Setters of a panel are an unordered set (the engine treats the two
 * "Đề" seats as interchangeable); each subject has its own panel, so a VL seat is never matched
 * against a CN seat.
 */
export function comparePlans(
  a: Assignment[],
  b: Assignment[],
  exams: Exam[],
  grades: Grade[],
  subjects: Subject[],
): PlanComparison {
  let distance = 0
  const diffPanels = new Set<string>()
  for (const exam of exams) {
    for (const grade of grades) {
      for (const subject of subjects) {
        const pa = panelMembers(a, exam.id, grade.id, subject.id)
        const pb = panelMembers(b, exam.id, grade.id, subject.id)
        const changed =
          countMissing(pa.setters, pb.setters) + countMissing(pa.reviewers, pb.reviewers)
        if (changed > 0) {
          distance += changed
          diffPanels.add(panelKey(exam.id, grade.id, subject.id))
        }
      }
    }
  }
  return { distance, diffPanels }
}

/** How many entries of `from` are missing from `to` (multiset difference). */
function countMissing(from: number[], to: number[]): number {
  const pool = [...to]
  let missing = 0
  for (const t of from) {
    const i = pool.indexOf(t)
    if (i >= 0) pool.splice(i, 1)
    else missing += 1
  }
  return missing
}
