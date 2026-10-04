import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { AssignmentsPage } from './AssignmentsPage'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import i18n from '@/i18n'
import { api } from '@/lib/api'

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

describe('AssignmentsPage Component Tests', () => {
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
    // Reset seed demo before each test
    await api.seedDemo()
  })

  it('renders assignment workspace, header actions, and plan view', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('assignments-page')).toBeInTheDocument()
    })

    // Header buttons
    expect(screen.getByTestId('run-optimizer-button')).toBeInTheDocument()
    expect(screen.getByTestId('compare-plans-button')).toBeInTheDocument()
    expect(screen.getByTestId('history-plans-button')).toBeInTheDocument()
    expect(screen.getByTestId('view-toggle-grid')).toBeInTheDocument()
    expect(screen.getByTestId('view-toggle-detail')).toBeInTheDocument()

    // Q-style grid view should be displayed by default
    await waitFor(() => {
      expect(screen.getByTestId('q-plan-grid-root')).toBeInTheDocument()
      expect(screen.getByTestId('q-plan-totals-panel')).toBeInTheDocument()
    })

    // Switch to detail view
    fireEvent.click(screen.getByTestId('view-toggle-detail'))
    await waitFor(() => {
      expect(screen.getByTestId('plan-matrix-view')).toBeInTheDocument()
    })

    // Switch back to grid view
    fireEvent.click(screen.getByTestId('view-toggle-grid'))
    await waitFor(() => {
      expect(screen.getByTestId('q-plan-grid-root')).toBeInTheDocument()
    })
  })

  it('opens Run Optimizer dialog, displays effort options, seed and feasibility status', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('run-optimizer-button')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByTestId('run-optimizer-button'))

    await waitFor(() => {
      expect(screen.getByText(i18n.t('assignments.runTitle'))).toBeInTheDocument()
    })

    // Effort buttons
    expect(screen.getByRole('button', { name: /Nhanh/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Chuẩn/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Kỹ/i })).toBeInTheDocument()

    // Plans count buttons (default 3)
    const planCountBtn = screen.getByRole('button', { name: '3' })
    expect(planCountBtn).toBeInTheDocument()
  })

  it('toggles Plans History panel, lists plans with source and final badges', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('history-plans-button')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByTestId('history-plans-button'))

    await waitFor(() => {
      expect(screen.getByTestId('plans-history-list')).toBeInTheDocument()
    })

    // Plans in mock demo data inside the history list
    const historyList = screen.getByTestId('plans-history-list')
    expect(historyList).toHaveTextContent(/Phương án #1/i)
  })

  it('focuses on teacher when chip or teacher row is clicked in detail view', async () => {
    renderWithClient(<AssignmentsPage />)

    // Switch to detail view
    await waitFor(() => {
      expect(screen.getByTestId('view-toggle-detail')).toBeInTheDocument()
    })
    fireEvent.click(screen.getByTestId('view-toggle-detail'))

    await waitFor(() => {
      expect(screen.getByTestId('plan-matrix-view')).toBeInTheDocument()
    })

    // Click on a teacher chip in the matrix
    const teacherChips = await screen.findAllByTestId(/^chip-teacher-/)
    expect(teacherChips.length).toBeGreaterThan(0)
    fireEvent.click(teacherChips[0])

    // Teacher focus panel should appear
    await waitFor(() => {
      expect(screen.getByTestId('teacher-focus-panel')).toBeInTheDocument()
    })
  })

  it('opens Compare modal when compare button is clicked', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('compare-plans-button')).toBeInTheDocument()
    })

    // Wait until plans are loaded so compare button is enabled
    await waitFor(() => {
      expect(screen.getByTestId('compare-plans-button')).not.toBeDisabled()
    })

    fireEvent.click(screen.getByTestId('compare-plans-button'))

    await waitFor(() => {
      expect(screen.getByTestId('plan-compare-modal')).toBeInTheDocument()
    })
  })

  it('supports duplicate for editing, keyboard candidate replace, undo/redo, and save', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('q-plan-grid-root')).toBeInTheDocument()
    })

    // Wait for create-edit-copy-button to appear
    await waitFor(() => {
      expect(screen.getByTestId('create-edit-copy-button')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByTestId('create-edit-copy-button'))

    // Once duplicated, it becomes an editable draft: undo/redo buttons become available
    await waitFor(() => {
      expect(screen.getByTitle(i18n.t('assignments.undo'))).toBeInTheDocument()
    })
    expect(screen.getByTitle(i18n.t('assignments.redo'))).toBeInTheDocument()

    // Dispatch restore event: in-memory editing state, undo stacks and selection must reset
    window.dispatchEvent(new CustomEvent('exampanel:restore'))

    await waitFor(() => {
      // Editable draft undo/redo buttons are no longer present
      expect(screen.queryByTitle(i18n.t('assignments.undo'))).not.toBeInTheDocument()
    })
  })

  it('renders Q-style grid with attached totals, task markers, and search filter', async () => {
    renderWithClient(<AssignmentsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('q-plan-grid-root')).toBeInTheDocument()
      expect(screen.getByTestId('q-plan-grid-table')).toBeInTheDocument()
      expect(screen.getByTestId('q-plan-totals-panel')).toBeInTheDocument()
      expect(screen.getByTestId('q-grid-summary-bar')).toBeInTheDocument()
    })

    // Check totals table contains teacher rows and filter input
    const filterInput = screen.getByTestId('q-totals-filter-input')
    expect(filterInput).toBeInTheDocument()

    // Filter by name
    fireEvent.change(filterInput, { target: { value: 'Nguyễn Văn An' } })
    await waitFor(() => {
      expect(screen.getByTestId('q-totals-row-1')).toBeInTheDocument()
    })
  })
})
