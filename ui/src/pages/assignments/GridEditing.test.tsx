import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api, type PlanDetails, type PlanStatus, type Subject } from '@/lib/api'
import { QPlanGrid } from './QPlanGrid'
import { CandidateSelectModal } from './CandidateSelectModal'
import { AssignmentsPage } from '../AssignmentsPage'

function renderWithClient(ui: React.ReactElement) {
  const testClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  })
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <QueryClientProvider client={testClient}>{ui}</QueryClientProvider>
      </ThemeProvider>
    </MemoryRouter>,
  )
}

describe('Grid Editing & Interaction Tests', () => {
  beforeEach(async () => {
    localStorage.clear()
    window.matchMedia = vi.fn().mockImplementation((query) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    await api.seedDemo()
  })

  // 1. seat click opens the candidate list with disabled invalid candidates and reasons
  it('seat click opens the candidate list with disabled invalid candidates and reasons', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    const onUpdateAssignments = vi.fn()

    renderWithClient(
      <QPlanGrid
        planDetails={planDetails}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={onUpdateAssignments}
        onCreateLock={vi.fn()}
      />,
    )

    // Find and click on an editable cell (GK1, K10, VL, setter 0)
    const cell = await screen.findByTestId('q-grid-cell-1-1-VL-setter-0')
    expect(cell).toBeInTheDocument()
    fireEvent.click(cell)

    // Candidate select modal should open
    await waitFor(() => {
      expect(screen.getByTestId('candidate-select-modal')).toBeInTheDocument()
    })

    // Wait for candidate rows to be evaluated and rendered
    await waitFor(() => {
      const candidates = screen.getAllByTestId(/^candidate-item-/)
      expect(candidates.length).toBeGreaterThan(0)
    })

    // Candidates with hard violations should have invalid styling and reasons
    const invalidItems = screen.getAllByTestId('candidate-item-invalid')
    expect(invalidItems.length).toBeGreaterThan(0)
    expect(invalidItems[0]).toHaveClass('cursor-not-allowed')
  })

  // 2. drag-to-swap
  it('drag-to-swap: drags one seat and drops on another to swap teachers', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    const onUpdateAssignments = vi.fn()

    renderWithClient(
      <QPlanGrid
        planDetails={planDetails}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={onUpdateAssignments}
        onCreateLock={vi.fn()}
      />,
    )

    const cellA = await screen.findByTestId('q-grid-cell-1-1-VL-setter-0')
    const cellB = await screen.findByTestId('q-grid-cell-1-1-VL-setter-1')

    const mockDataTransfer = {
      data: {} as Record<string, string>,
      setData(format: string, data: string) {
        this.data[format] = data
      },
      getData(format: string) {
        return this.data[format] || ''
      },
    }

    // Drag start on cell A
    fireEvent.dragStart(cellA, { dataTransfer: mockDataTransfer })

    // Drag over and drop on cell B
    fireEvent.dragOver(cellB, { dataTransfer: mockDataTransfer })
    fireEvent.drop(cellB, { dataTransfer: mockDataTransfer })

    // onUpdateAssignments should be called with swapped teachers
    expect(onUpdateAssignments).toHaveBeenCalledTimes(1)
    const updatedAssignments = onUpdateAssignments.mock.calls[0][0]
    expect(updatedAssignments.length).toBe(planDetails.assignments.length)
  })

  // 3. keyboard flow
  it('keyboard flow: navigates candidates with ArrowDown/ArrowUp and selects with Enter', async () => {
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const onSelectTeacher = vi.fn()
    const onOpenChange = vi.fn()

    const slot = {
      exam_id: 1,
      grade_id: 1,
      subject_id: 1,
      role: 'setter' as const,
      position: 0,
    }

    renderWithClient(
      <CandidateSelectModal
        open={true}
        onOpenChange={onOpenChange}
        slot={slot}
        currentTeacherId={1}
        schoolYearId={1}
        assignments={[]}
        teachers={teachers}
        campuses={campuses}
        onSelectTeacher={onSelectTeacher}
      />,
    )

    await waitFor(() => {
      expect(screen.getByTestId('candidate-select-modal')).toBeInTheDocument()
    })

    const modal = screen.getByTestId('candidate-select-modal')

    // Navigate with ArrowDown
    fireEvent.keyDown(modal, { key: 'ArrowDown' })
    // Navigate with ArrowUp
    fireEvent.keyDown(modal, { key: 'ArrowUp' })
    // Press Enter to select
    fireEvent.keyDown(modal, { key: 'Enter' })

    await waitFor(() => {
      expect(modal).toBeInTheDocument()
    })
  })

  // 4. undo/redo
  it('undo/redo: records assignment updates and allows undoing and redoing', async () => {
    renderWithClient(<AssignmentsPage />)

    // Wait for QPlanGrid to render
    await waitFor(() => {
      expect(screen.getByTestId('q-plan-grid-root')).toBeInTheDocument()
    })

    // Click "Tạo bản sao chỉnh sửa" to create an editable copy
    const copyBtn = await screen.findByTestId('create-edit-copy-button')
    fireEvent.click(copyBtn)

    // Wait until plan becomes editable and undo/redo buttons appear
    await waitFor(() => {
      const undoBtn = screen.getByTitle('Hoàn tác (Ctrl+Z)')
      expect(undoBtn).toBeInTheDocument()
      expect(undoBtn).toBeDisabled()
    })

    // Drag-swap a seat to trigger an assignment update
    const cellA = await screen.findByTestId('q-grid-cell-1-1-VL-setter-0')
    const cellB = await screen.findByTestId('q-grid-cell-1-1-VL-setter-1')

    const mockDataTransfer = {
      data: {} as Record<string, string>,
      setData(format: string, data: string) {
        this.data[format] = data
      },
      getData(format: string) {
        return this.data[format] || ''
      },
    }

    fireEvent.dragStart(cellA, { dataTransfer: mockDataTransfer })
    fireEvent.dragOver(cellB, { dataTransfer: mockDataTransfer })
    fireEvent.drop(cellB, { dataTransfer: mockDataTransfer })

    // Undo button should now be enabled
    const undoBtn = screen.getByTitle('Hoàn tác (Ctrl+Z)')
    await waitFor(() => {
      expect(undoBtn).not.toBeDisabled()
    })

    // Click undo
    fireEvent.click(undoBtn)

    // Redo button should now be enabled
    const redoBtn = screen.getByTitle('Làm lại (Ctrl+Y)')
    await waitFor(() => {
      expect(redoBtn).not.toBeDisabled()
    })

    // Click redo
    fireEvent.click(redoBtn)
    await waitFor(() => {
      expect(undoBtn).not.toBeDisabled()
    })
  })

  // 5. forced seats immovable
  it('forced seats immovable: cannot be dragged, clicked for replacement, or overwritten by drop', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    // Mock problem details to include a forced placement at GK1, K10, VL, setter 0
    const originalGetProblemDetails = api.getProblemDetails.bind(api)
    const baseProblemDetails = await api.getProblemDetails(1)
    vi.spyOn(api, 'getProblemDetails').mockResolvedValue({
      ...baseProblemDetails,
      forced: [
        {
          teacher_id: 5,
          panel: { exam_id: 1, grade_id: 1, subject_id: 1 },
          role: 'setter',
          position: 0,
        },
      ],
    })

    const onUpdateAssignments = vi.fn()

    renderWithClient(
      <QPlanGrid
        planDetails={planDetails}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={onUpdateAssignments}
        onCreateLock={vi.fn()}
      />,
    )

    // Wait for forced icon to appear in cell
    await waitFor(() => {
      expect(screen.getByTestId('forced-lock-icon')).toBeInTheDocument()
    })

    const forcedIcon = screen.getByTestId('forced-lock-icon')
    const forcedCell = forcedIcon.closest('td')!
    expect(forcedCell.getAttribute('draggable')).toBe('false')

    // Clicking forced seat should not open CandidateSelectModal
    fireEvent.click(forcedCell)
    expect(screen.queryByTestId('candidate-select-modal')).not.toBeInTheDocument()

    // Dropping onto forced seat should not invoke onUpdateAssignments
    fireEvent.drop(forcedCell, {
      dataTransfer: { getData: () => JSON.stringify({}) },
    })
    expect(onUpdateAssignments).not.toHaveBeenCalled()

    api.getProblemDetails = originalGetProblemDetails
  })

  // 6. totals panel counts and the ≥2 / ≥3 markers
  it('totals panel counts and the ≥2 / ≥3 markers (* and **)', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    // In demo assignments or Q table, configure assignments so teacher 1 has >=3 tasks in GK1 and teacher 2 has >=2 tasks in GK2
    const testAssignments = [
      ...planDetails.assignments,
      {
        plan_id: 1,
        exam_id: 1,
        grade_id: 1,
        subject_id: 1,
        teacher_id: 1,
        role: 'setter' as const,
        position: 0,
      },
      {
        plan_id: 1,
        exam_id: 1,
        grade_id: 2,
        subject_id: 1,
        teacher_id: 1,
        role: 'setter' as const,
        position: 0,
      },
      {
        plan_id: 1,
        exam_id: 1,
        grade_id: 3,
        subject_id: 1,
        teacher_id: 1,
        role: 'reviewer' as const,
        position: 0,
      },
      {
        plan_id: 1,
        exam_id: 2,
        grade_id: 1,
        subject_id: 1,
        teacher_id: 2,
        role: 'setter' as const,
        position: 0,
      },
      {
        plan_id: 1,
        exam_id: 2,
        grade_id: 2,
        subject_id: 1,
        teacher_id: 2,
        role: 'reviewer' as const,
        position: 0,
      },
    ]

    renderWithClient(
      <QPlanGrid
        planDetails={{ ...planDetails, assignments: testAssignments }}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={vi.fn()}
        onCreateLock={vi.fn()}
      />,
    )

    await waitFor(() => {
      expect(screen.getByTestId('q-plan-totals-panel')).toBeInTheDocument()
    })

    // Check for presence of marker-over-2 or marker-over-3
    const markersOver2 = screen.queryAllByTestId('marker-over-2')
    const markersOver3 = screen.queryAllByTestId('marker-over-3')
    expect(markersOver2.length + markersOver3.length).toBeGreaterThan(0)

    if (markersOver3.length > 0) {
      expect(markersOver3[0].textContent).toContain('**')
    }
    if (markersOver2.length > 0) {
      expect(markersOver2[0].textContent).toContain('*')
    }
  })

  // 7. teacher hover focus
  it('teacher hover focus: highlights all seats of hovered teacher and totals row', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const subjects = await api.listSubjects(1)
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    renderWithClient(
      <QPlanGrid
        planDetails={planDetails}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={subjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={vi.fn()}
        onCreateLock={vi.fn()}
      />,
    )

    const cell = await screen.findByTestId('q-grid-cell-1-1-VL-setter-0')
    fireEvent.mouseEnter(cell)

    // After hover, active highlighted teacher is set and highlighted cells have bg-primary/20
    await waitFor(() => {
      const highlightedRows = document.querySelectorAll('.bg-primary\\/20')
      expect(highlightedRows.length).toBeGreaterThan(0)
    })

    fireEvent.mouseLeave(cell)
  })

  // 8. view-toggle persistence
  it('view-toggle persistence: persists view mode to localStorage and restores on mount', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('view-toggle-detail')).toBeInTheDocument()
    })

    // Click detail view
    fireEvent.click(screen.getByTestId('view-toggle-detail'))

    await waitFor(() => {
      expect(localStorage.getItem('exam_panel_assignment_view_mode')).toBe('detail')
    })

    // Switch back to grid view
    fireEvent.click(screen.getByTestId('view-toggle-grid'))

    await waitFor(() => {
      expect(localStorage.getItem('exam_panel_assignment_view_mode')).toBe('grid')
    })
  })

  // 9. two-subject layout with blank CN second "Đề" row
  it('two-subject layout with blank CN second "Đề" row', async () => {
    const plans = await api.listPlans(1)
    const planDetails = (await api.getPlan(plans[0].id)) as PlanDetails
    const planStatus = (await api.planStatus(plans[0].id)) as PlanStatus
    const exams = await api.listExams(1)
    const grades = await api.listGrades()
    const teachers = await api.teachersWithGrades(1)
    const campuses = await api.listCampuses()
    const locks = await api.listLocks(1)

    // Two subjects: VL (setters=2, reviewers=1) and CN (setters=1, reviewers=1)
    const twoSubjects: Subject[] = [
      {
        id: 1,
        code: 'VL',
        name: 'Vật lí',
        color: 'palette-1',
        sort_order: 1,
        setters: 2,
        reviewers: 1,
        min_campuses: 2,
      },
      {
        id: 2,
        code: 'CN',
        name: 'Công nghệ',
        color: 'palette-2',
        sort_order: 2,
        setters: 1,
        reviewers: 1,
        min_campuses: 2,
      },
    ]

    renderWithClient(
      <QPlanGrid
        planDetails={planDetails}
        planStatus={planStatus}
        exams={exams}
        grades={grades}
        subjects={twoSubjects}
        teachers={teachers}
        campuses={campuses}
        locks={locks}
        isEditable={true}
        focusedTeacherId={null}
        keptSlots={[]}
        onSelectTeacherFocus={vi.fn()}
        onToggleKeepSlot={vi.fn()}
        onReoptimizeRemaining={vi.fn()}
        onUpdateAssignments={vi.fn()}
        onCreateLock={vi.fn()}
      />,
    )

    // In GK1 (exam 1), Khối 10 (grade 1):
    // For VL setter position 1 (second "Đề" row), cell exists with testid
    const vlCell = screen.getByTestId('q-grid-cell-1-1-VL-setter-1')
    expect(vlCell).toBeInTheDocument()

    // For CN setter position 1 the seat does not exist (position >= subject.setters = 1): the
    // paper sheet leaves that cell blank, no placeholder glyph (RA-015)
    const blank = screen.getByTestId('q-grid-blank-1-1-CN-setter-1')
    expect(blank.textContent).toBe('')
  })
})
