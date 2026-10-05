import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api, type Assignment, type TeacherWithGrades } from '@/lib/api'
import { StatisticsPage } from './StatisticsPage'
import { coWorkingCounts, reviewRelationCounts } from './statsMatrices'

const seat = (
  subject_id: number,
  role: 'setter' | 'reviewer',
  position: number,
  teacher_id: number,
  exam_id = 1,
  grade_id = 1,
): Assignment => ({
  plan_id: 1,
  exam_id,
  grade_id,
  subject_id,
  teacher_id,
  role,
  position,
})

// VL [t1, t2 | t3]  CN [t4 | t1]  — t1 is on BOTH panels of the same exam × grade.
const assignments = [
  seat(1, 'setter', 0, 1),
  seat(1, 'setter', 1, 2),
  seat(1, 'reviewer', 0, 3),
  seat(2, 'setter', 0, 4),
  seat(2, 'reviewer', 0, 1),
]

describe('statistics matrices are computed per subject panel (RA-027)', () => {
  it('co-working counts members of the same subject panel only', () => {
    const m = coWorkingCounts(assignments)
    expect(m.get('1_2')).toBe(1) // subject-blind code reported 2
    expect(m.get('2_1')).toBe(1)
    expect(m.get('1_4')).toBe(1) // CN panel
    expect(m.get('2_4')).toBeUndefined() // never share a panel
  })

  it('review relation pairs the reviewer with the setters of ITS panel', () => {
    const m = reviewRelationCounts(assignments)
    expect(m.get('3_1')).toBe(1)
    expect(m.get('3_2')).toBe(1)
    expect(m.get('1_4')).toBe(1) // CN reviewer t1 reviews t4 (first-reviewer-only lost this)
    expect(m.get('3_4')).toBeUndefined()
  })
})

describe('StatisticsPage heatmaps with 12 teachers', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.restoreAllMocks()
    await api.seedDemo()
  })

  it('lists every teacher (no truncation at 11) with names as column headers', async () => {
    const twg: TeacherWithGrades[] = Array.from({ length: 12 }, (_, i) => ({
      teacher: {
        id: i + 1,
        full_name: `Giáo viên ${i + 1}`,
        campus_id: 1,
        load_weight: 1,
        active: true,
        note: null,
      },
      grade_ids: [1],
    }))
    vi.spyOn(api, 'teachersWithGrades').mockResolvedValue(twg)
    const details = await api.getPlan((await api.listPlans(1))[0].id)
    vi.spyOn(api, 'getPlan').mockResolvedValue({ ...details, assignments })

    render(
      <ThemeProvider>
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <StatisticsPage />
        </QueryClientProvider>
      </ThemeProvider>,
    )

    await waitFor(() => expect(screen.getByTestId('cowork-row-12')).toBeInTheDocument())
    expect(screen.getByTestId('review-row-12')).toBeInTheDocument()
    expect(screen.getByTestId('cowork-head-12').textContent).toContain('Giáo viên 12')
    await waitFor(() =>
      expect(screen.getByTestId('cowork-cell-1-2').textContent).toBe('1'),
    )
    expect(screen.getByTestId('review-cell-1-4').textContent).toBe('1')
  })
})
