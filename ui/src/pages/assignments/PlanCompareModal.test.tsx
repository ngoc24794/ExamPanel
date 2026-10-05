import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import '@/i18n'
import {
  api,
  type Assignment,
  type Exam,
  type Grade,
  type PlanDetails,
  type PlanSummary,
  type Subject,
  type TeacherWithGrades,
} from '@/lib/api'
import { PlanCompareModal } from './PlanCompareModal'

const exams: Exam[] = [
  { id: 1, school_year_id: 1, code: 'GK1', name: 'Giữa kỳ 1', sort_order: 1 },
]
const grades: Grade[] = [{ id: 1, name: 'Khối 10', sort_order: 1 } as Grade]
const subjects: Subject[] = [
  {
    id: 1,
    code: 'VL',
    name: 'Vật lí',
    color: 'blue',
    sort_order: 1,
    setters: 2,
    reviewers: 1,
    min_campuses: 0,
  },
  {
    id: 2,
    code: 'CN',
    name: 'Công nghệ',
    color: 'green',
    sort_order: 2,
    setters: 1,
    reviewers: 1,
    min_campuses: 0,
  },
]
const names = ['', 'Cô Hiền', 'Cô Lài', 'Thầy Phúc', 'Thầy Nghĩa', 'Cô Bình']
const teachers: TeacherWithGrades[] = [1, 2, 3, 4, 5].map((id) => ({
  teacher: {
    id,
    full_name: names[id],
    campus_id: 1,
    load_weight: 1,
    active: true,
    note: null,
  },
  grade_ids: [1],
}))
const plans: PlanSummary[] = [1, 2].map((id) => ({
  id,
  name: `Plan ${id}`,
  created_at: '2026-01-01T00:00:00Z',
  is_final: false,
  source: 'optimizer',
  is_stale: false,
  score: 10,
}))

const seat = (
  subject_id: number,
  role: 'setter' | 'reviewer',
  position: number,
  teacher_id: number,
): Assignment => ({
  plan_id: 0,
  exam_id: 1,
  grade_id: 1,
  subject_id,
  teacher_id,
  role,
  position,
})

// Plan A: VL [Hiền, Lài | Phúc]  CN [Nghĩa | Bình]
const planA = [
  seat(1, 'setter', 0, 1),
  seat(1, 'setter', 1, 2),
  seat(1, 'reviewer', 0, 3),
  seat(2, 'setter', 0, 4),
  seat(2, 'reviewer', 0, 5),
]
// Plan B: the two reviewers trade subjects: VL [Hiền, Lài | Bình]  CN [Nghĩa | Phúc]
const planB = [
  seat(1, 'setter', 0, 1),
  seat(1, 'setter', 1, 2),
  seat(1, 'reviewer', 0, 5),
  seat(2, 'setter', 0, 4),
  seat(2, 'reviewer', 0, 3),
]
const details = (id: number, assignments: Assignment[]): PlanDetails => ({
  plan: {
    id,
    school_year_id: 1,
    name: `Plan ${id}`,
    created_at: '',
    seed: 1,
    is_final: false,
    source: 'optimizer',
  },
  assignments,
})

describe('PlanCompareModal with two subjects (RA-013)', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.spyOn(api, 'getPlan').mockImplementation(async (id: number) =>
      details(id, id === 1 ? planA : planB),
    )
  })

  it('counts every seat whose teacher differs, per subject', async () => {
    render(
      <PlanCompareModal
        open
        onOpenChange={() => {}}
        plans={plans}
        initialPlanAId={1}
        initialPlanBId={2}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
      />,
    )
    // VL reviewer and CN reviewer both changed => 2 seats (subject-blind code reported 1).
    expect(await screen.findByText(/Khoảng cách sai khác: 2 ô/)).toBeInTheDocument()
  })

  it('shows the VL and CN panels separately in the changed cell', async () => {
    render(
      <PlanCompareModal
        open
        onOpenChange={() => {}}
        plans={plans}
        initialPlanAId={1}
        initialPlanBId={2}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
      />,
    )
    const vlA = await screen.findByTestId('compare-cell-1-1-VL-A')
    await waitFor(() => expect(vlA.textContent).toMatch(/Phúc/))
    expect(vlA.textContent).toMatch(/Hiền/)
    expect(vlA.textContent).not.toMatch(/Nghĩa/) // CN setter must not leak into the VL panel
    const cnB = screen.getByTestId('compare-cell-1-1-CN-B')
    expect(within(cnB).getByText(/Phúc/)).toBeInTheDocument()
  })
})
