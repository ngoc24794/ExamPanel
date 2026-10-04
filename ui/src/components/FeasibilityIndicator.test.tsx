import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { FeasibilityIndicator } from './FeasibilityIndicator'
import { api, type ProblemDetails } from '@/lib/api'

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <HashRouter>{ui}</HashRouter>
    </QueryClientProvider>,
  )
}

describe('FeasibilityIndicator & FeasibilitySheet', () => {
  beforeEach(async () => {
    await api.seedDemo()
  })

  it('renders green indicator when problem is feasible', async () => {
    renderWithProviders(
      <FeasibilityIndicator schoolYearId={1} schoolYearName="2026-2027" />,
    )

    await waitFor(() => {
      const indicator = screen.getByTestId('feasibility-indicator')
      expect(indicator).toBeInTheDocument()
      expect(indicator).toHaveTextContent(/Khả thi|Feasible/i)
    })
  })

  it('opens sheet on click, lists diagnostics, and handles "Go to fix" navigation', async () => {
    const user = userEvent.setup()

    // Mock checkFeasibility to return an error diagnostic
    const originalCheckFeasibility = api.checkFeasibility.bind(api)
    vi.spyOn(api, 'checkFeasibility').mockResolvedValueOnce({
      report: {
        is_feasible: false,
        errors: [
          {
            rule: 'h2',
            code: 'insufficient_panel_teachers',
            panel: { exam_id: 1, grade_id: 1, subject_id: 1 },
            params: { exam: 'GK1', grade: '10', count: '2' },
          },
        ],
        warnings: [],
        quotas: [],
      },
      quotas: [],
    })

    renderWithProviders(
      <FeasibilityIndicator schoolYearId={1} schoolYearName="2026-2027" />,
    )

    // Should display red status
    await waitFor(() => {
      const indicator = screen.getByTestId('feasibility-indicator')
      expect(indicator).toHaveTextContent(/Không khả thi|Infeasible/i)
    })

    // Click indicator to open sheet
    const indicator = screen.getByTestId('feasibility-indicator')
    await user.click(indicator)

    await waitFor(() => {
      expect(screen.getByTestId('feasibility-sheet')).toBeInTheDocument()
      expect(screen.getByTestId('feasibility-error-item')).toBeInTheDocument()
    })

    // Click "Go to fix"
    const fixBtn = screen.getByTestId('go-to-fix-btn')
    await user.click(fixBtn)

    // Sheet should close and navigate
    await waitFor(() => {
      expect(screen.queryByTestId('feasibility-sheet')).not.toBeInTheDocument()
    })

    api.checkFeasibility = originalCheckFeasibility
  })

  it('renders forced-placement info group and warning diagnostics like h3_reviewer_pool_reduced', async () => {
    const user = userEvent.setup()

    const originalGetProblemDetails = api.getProblemDetails.bind(api)
    vi.spyOn(api, 'getProblemDetails').mockResolvedValueOnce({
      problem: {},
      forced: [
        {
          teacher_id: 1,
          role: 'setter',
          position: 0,
          panel: { exam_id: 1, grade_id: 1, subject_id: 1 },
        },
      ],
    } as unknown as ProblemDetails)

    const originalCheckFeasibility = api.checkFeasibility.bind(api)
    vi.spyOn(api, 'checkFeasibility').mockResolvedValueOnce({
      report: {
        is_feasible: true,
        errors: [],
        warnings: [
          {
            rule: 'h3',
            code: 'h3_reviewer_pool_reduced',
            panel: { exam_id: 1, grade_id: 1, subject_id: 2 },
            params: { count: '1' },
          },
        ],
        quotas: [],
      },
      quotas: [],
    })

    renderWithProviders(
      <FeasibilityIndicator schoolYearId={1} schoolYearName="2026-2027" />,
    )

    const indicator = screen.getByTestId('feasibility-indicator')
    await user.click(indicator)

    await waitFor(() => {
      expect(screen.getByTestId('feasibility-sheet')).toBeInTheDocument()
      expect(screen.getByTestId('forced-placements-info-group')).toBeInTheDocument()
      expect(screen.getByText(/h3_reviewer_pool_reduced|chỉ còn|eligible/i)).toBeInTheDocument()
    })

    api.getProblemDetails = originalGetProblemDetails
    api.checkFeasibility = originalCheckFeasibility
  })
})
