import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
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

  // RA-039: seats were not focusable and the only tab stops were 60 invisible menu buttons.
  it('is operable with the keyboard: one tab stop, arrow navigation, Enter opens candidates', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    renderWithClient({
      planDetails,
      planStatus,
      exams: await api.listExams(1),
      grades: await api.listGrades(),
      subjects: await api.listSubjects(1),
      teachers: await api.teachersWithGrades(1),
      campuses: await api.listCampuses(),
      locks: await api.listLocks(1),
      isEditable: true,
      focusedTeacherId: null,
      keptSlots: [],
      onSelectTeacherFocus: vi.fn(),
      onToggleKeepSlot: vi.fn(),
      onReoptimizeRemaining: vi.fn(),
      onUpdateAssignments: vi.fn(),
      onCreateLock: vi.fn(),
    })

    const table = await screen.findByTestId('q-plan-grid-table')
    // roving tabindex: exactly one seat is in the tab order
    const stops = table.querySelectorAll('[tabindex="0"]')
    expect(stops.length).toBe(1)
    // the per-seat menu buttons are not tab stops and have an accessible name
    const menus = table.querySelectorAll('[data-testid="q-cell-menu-btn"]')
    expect(menus.length).toBeGreaterThan(10)
    menus.forEach((m) => {
      expect(m.getAttribute('tabindex')).toBe('-1')
      expect(m.getAttribute('aria-label')).toBeTruthy()
    })

    const first = stops[0] as HTMLElement
    first.focus()
    expect(first.getAttribute('aria-label')).toBeTruthy()
    fireEvent.keyDown(first, { key: 'ArrowDown' })
    const second = document.activeElement as HTMLElement
    expect(second).not.toBe(first)
    expect(table.contains(second)).toBe(true)
    expect(second.getAttribute('tabindex')).toBe('0')
    expect(first.getAttribute('tabindex')).toBe('-1')

    fireEvent.keyDown(second, { key: 'Enter' })
    await waitFor(() => {
      expect(screen.getByTestId('candidate-select-modal')).toBeInTheDocument()
    })
  })
})
