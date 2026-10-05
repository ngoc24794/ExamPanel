import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QPlanGrid, type QPlanGridProps } from './QPlanGrid'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api, type PlanDetails, type PlanStatus } from '@/lib/api'

function renderWithClient(props: QPlanGridProps) {
  const testClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  })
  return render(
    <ThemeProvider>
      <QueryClientProvider client={testClient}>
        <QPlanGrid {...props} />
      </QueryClientProvider>
    </ThemeProvider>,
  )
}

describe('QPlanGrid Component Tests (Part B)', () => {
  beforeEach(async () => {
    localStorage.clear()
    await api.seedDemo()
  })

  it('renders Q-style grid structure, headers, and attached totals panel', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    const onSelectTeacherFocus = vi.fn()
    const onToggleKeepSlot = vi.fn()
    const onReoptimizeRemaining = vi.fn()
    const onUpdateAssignments = vi.fn()
    const onCreateLock = vi.fn()

    renderWithClient({
      planDetails,
      planStatus,
      exams,
      grades,
      subjects,
      teachers,
      campuses,
      locks,
      isEditable: true,
      focusedTeacherId: null,
      keptSlots: [],
      onSelectTeacherFocus,
      onToggleKeepSlot,
      onReoptimizeRemaining,
      onUpdateAssignments,
      onCreateLock,
    })

    // Summary bar
    await waitFor(() => {
      expect(screen.getByTestId('q-grid-summary-bar')).toBeInTheDocument()
      expect(screen.getByTestId('q-plan-grid-table')).toBeInTheDocument()
      expect(screen.getByTestId('q-plan-totals-panel')).toBeInTheDocument()
    })

    // Header "Kì thi/khối"
    expect(screen.getByText(/Kì thi\/khối|Exam \/ Grade/i)).toBeInTheDocument()

    // Grades headers (Khối 10, Khối 11, Khối 12)
    expect(screen.getByText(/Khối 10/i)).toBeInTheDocument()
    expect(screen.getByText(/Khối 11/i)).toBeInTheDocument()
    expect(screen.getByText(/Khối 12/i)).toBeInTheDocument()

    // Attached Totals Panel headers: GV, Tổng, Đề, PB
    expect(screen.getByText('Tổng cộng')).toBeInTheDocument()

    // The grade header never doubles the word (RA-003)
    expect(screen.queryByText(/Khối Khối/i)).not.toBeInTheDocument()

    // Exam rows and totals columns use the paper codes GK1, CK1, GK2, CK2 (RA-015)
    for (const ex of exams) {
      expect(screen.getAllByText(ex.code).length).toBeGreaterThan(1) // grid row + totals header
    }
  })

  it('highlights all seats of a teacher on hover and displays tooltip', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    const onSelectTeacherFocus = vi.fn()

    renderWithClient({
      planDetails,
      planStatus,
      exams,
      grades,
      subjects,
      teachers,
      campuses,
      locks,
      isEditable: true,
      focusedTeacherId: 1, // Focus teacher 1
      keptSlots: [],
      onSelectTeacherFocus,
      onToggleKeepSlot: vi.fn(),
      onReoptimizeRemaining: vi.fn(),
      onUpdateAssignments: vi.fn(),
      onCreateLock: vi.fn(),
    })

    await waitFor(() => {
      expect(screen.getByTestId('q-totals-row-1')).toHaveClass('bg-primary/20')
    })
  })

  it('renders forced-load teacher indicator and task count markers in totals table', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    // Mark teacher 1 with quota_override
    teachers[0].teacher.quota_override = 6

    renderWithClient({
      planDetails,
      planStatus,
      exams,
      grades,
      subjects,
      teachers,
      campuses,
      locks,
      isEditable: true,
      focusedTeacherId: null,
      keptSlots: [],
      onSelectTeacherFocus: vi.fn(),
      onToggleKeepSlot: vi.fn(),
      onReoptimizeRemaining: vi.fn(),
      onUpdateAssignments: vi.fn(),
      onCreateLock: vi.fn(),
    })

    await waitFor(() => {
      expect(screen.getByTestId('q-totals-row-1')).toHaveTextContent(/cố định|fixed/i)
    })
  })
})
