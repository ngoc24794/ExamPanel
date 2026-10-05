import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api, type Assignment, type EvaluationOutcome } from '@/lib/api'
import { toast } from 'sonner'
import { AssignmentsPage } from './AssignmentsPage'

vi.mock('sonner', () => ({ toast: { success: vi.fn(), info: vi.fn(), error: vi.fn() } }))

function renderPage() {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 60_000 },
      mutations: { retry: false },
    },
  })
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <QueryClientProvider client={client}>
          <AssignmentsPage />
        </QueryClientProvider>
      </ThemeProvider>
    </MemoryRouter>,
  )
}

const dataTransfer = () => {
  const data: Record<string, string> = {}
  return {
    setData: (k: string, v: string) => {
      data[k] = v
    },
    getData: (k: string) => data[k] ?? '',
  }
}

type Seat = Assignment & { subject_id: number; position: number }
const panelKey = (a: Assignment) => `${a.exam_id}-${a.grade_id}-${a.subject_id}`

async function seats(): Promise<{ list: Seat[]; testId: (a: Seat) => string }> {
  const plans = await api.listPlans(1)
  const details = await api.getPlan(plans[0].id)
  const subjects = await api.listSubjects(1)
  const counts = new Map<string, number>()
  const list = details.assignments.map((a) => {
    const subject_id = a.subject_id ?? 1
    const key = `${a.exam_id}-${a.grade_id}-${subject_id}-${a.role}`
    const position = a.position ?? counts.get(key) ?? 0
    counts.set(key, (counts.get(key) ?? 0) + 1)
    return { ...a, subject_id, position } as Seat
  })
  const testId = (a: Seat) => {
    const code = subjects.find((s) => s.id === a.subject_id)?.code
    return `q-grid-cell-${a.exam_id}-${a.grade_id}-${code}-${a.role}-${a.position}`
  }
  return { list, testId }
}

async function openEditableCopy() {
  renderPage()
  fireEvent.click(
    await screen.findByTestId('create-edit-copy-button', undefined, { timeout: 3000 }),
  )
  await screen.findByTitle(/Hoàn tác/, undefined, { timeout: 3000 })
}

function swap(cellA: HTMLElement, cellB: HTMLElement) {
  const dt = dataTransfer()
  fireEvent.dragStart(cellA, { dataTransfer: dt })
  fireEvent.dragOver(cellB, { dataTransfer: dt })
  fireEvent.drop(cellB, { dataTransfer: dt })
}

describe('AssignmentsPage live evaluation after edits (RA-020)', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.restoreAllMocks()
    vi.clearAllMocks()
    await api.seedDemo()
  })

  it('updates the header score and hard-violation count after a swap', async () => {
    const { list, testId } = await seats()
    const a = list.find((x) => x.role === 'setter')!
    const b = list.find(
      (x) =>
        x.role === 'setter' &&
        panelKey(x) !== panelKey(a) &&
        x.teacher_id !== a.teacher_id &&
        !list.some((y) => panelKey(y) === panelKey(a) && y.teacher_id === x.teacher_id) &&
        !list.some((y) => panelKey(y) === panelKey(x) && y.teacher_id === a.teacher_id),
    )!
    const baseline = await api.evaluateAssignments(1, [])
    const evaluated: EvaluationOutcome = {
      hard_violations: [{ rule: 'h4', code: 'max_tasks_per_exam_exceeded', params: {} }],
      score_report: { ...baseline.score_report, total: 999.5 },
    }
    const spy = vi.spyOn(api, 'evaluateAssignments').mockResolvedValue(evaluated)
    await openEditableCopy()

    swap(await screen.findByTestId(testId(a)), await screen.findByTestId(testId(b)))

    const bar = screen.getByTestId('q-grid-summary-bar')
    await waitFor(() => expect(within(bar).getByText(/999\.50/)).toBeInTheDocument())
    expect(spy).toHaveBeenCalled()
    expect(bar.textContent).not.toMatch(/Hợp lệ/)
    expect(bar.textContent).toMatch(/Vi phạm|1/)
  })

  it('rejects a swap that puts the same teacher twice in one panel', async () => {
    const { list, testId } = await seats()
    let pair: [Seat, Seat] | null = null
    for (const a of list) {
      const b = list.find(
        (x) =>
          panelKey(x) !== panelKey(a) &&
          x.teacher_id !== a.teacher_id &&
          list.some((y) => panelKey(y) === panelKey(x) && y.teacher_id === a.teacher_id),
      )
      if (b) {
        pair = [a, b]
        break
      }
    }
    expect(pair, 'demo plan must contain a swap that duplicates a teacher').not.toBeNull()
    const [a, b] = pair!
    await openEditableCopy()

    swap(await screen.findByTestId(testId(a)), await screen.findByTestId(testId(b)))

    await waitFor(() => expect(toast.error).toHaveBeenCalled())
    // nothing became dirty: no save button
    expect(screen.queryByTestId('save-plan-assignments-button')).not.toBeInTheDocument()
  })
})

describe('AssignmentsPage kept-seat banner (RA-026)', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.restoreAllMocks()
    await api.seedDemo()
  })

  it('shows the kept-seat banner in the default grid view, not only in the detail view', async () => {
    const user = userEvent.setup()
    renderPage()
    const grid = await screen.findByTestId('q-plan-grid-root', undefined, {
      timeout: 3000,
    })
    expect(screen.queryByTestId('kept-slots-banner')).not.toBeInTheDocument()

    const menuButtons = within(grid).getAllByTestId('q-cell-menu-btn')
    await user.click(menuButtons[0])
    await user.click(
      await screen.findByText(/Giữ ô này khi tối ưu lại|Keep Slot in Re-optimization/i),
    )

    const banner = await screen.findByTestId('kept-slots-banner')
    expect(banner.textContent).toMatch(/1/)
    expect(within(banner).getByTestId('reoptimize-kept-button')).toBeInTheDocument()
  })
})
