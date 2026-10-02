import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { ExamsPage } from './ExamsPage'
import { api } from '@/lib/api'

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <HashRouter>{ui}</HashRouter>
    </QueryClientProvider>,
  )
}

describe('ExamsPage', () => {
  beforeEach(async () => {
    await api.seedDemo()
  })

  it('renders exams and grades lists', async () => {
    renderWithProviders(<ExamsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('exams-page')).toBeInTheDocument()
      expect(screen.getByTestId('exam-row-GK1')).toBeInTheDocument()
      expect(screen.getByTestId('grade-row-10')).toBeInTheDocument()
    })
  })

  it('creates, reorders, and deletes an exam', async () => {
    const user = userEvent.setup()
    renderWithProviders(<ExamsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('exam-row-GK1')).toBeInTheDocument()
    })

    // Click Add Exam
    await user.click(screen.getByTestId('add-exam-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('exam-code-input')).toBeInTheDocument()
    })

    await user.type(screen.getByTestId('exam-code-input'), 'EXAM_TEST')
    await user.type(screen.getByTestId('exam-name-input'), 'Kỳ thi thử')
    await user.click(screen.getByTestId('exam-save-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('exam-row-EXAM_TEST')).toBeInTheDocument()
    })

    // Reorder: Move down GK1
    const moveDownBtn = screen.getByTestId('exam-move-down-0')
    await user.click(moveDownBtn)

    // Delete the new exam: Find delete button specifically for the newly created exam
    const allExams = await api.listExams(1)
    const testExam = allExams.find((e) => e.code === 'EXAM_TEST')
    expect(testExam).toBeDefined()

    const examDeleteBtn = screen.getByTestId(`exam-delete-${testExam!.id}`)
    await user.click(examDeleteBtn)

    await waitFor(() => {
      expect(screen.getByTestId('confirm-delete-exam-btn')).toBeInTheDocument()
    })

    await user.click(screen.getByTestId('confirm-delete-exam-btn'))

    await waitFor(() => {
      expect(screen.queryByTestId('exam-row-EXAM_TEST')).not.toBeInTheDocument()
    })
  })

  it('creates and edits a grade', async () => {
    const user = userEvent.setup()
    renderWithProviders(<ExamsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('grade-row-10')).toBeInTheDocument()
    })

    // Click Add Grade
    await user.click(screen.getByTestId('add-grade-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('grade-code-input')).toBeInTheDocument()
    })

    await user.type(screen.getByTestId('grade-code-input'), '9')
    await user.type(screen.getByTestId('grade-name-input'), 'Khối 9')
    await user.click(screen.getByTestId('grade-save-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('grade-row-9')).toBeInTheDocument()
    })
  })
})
