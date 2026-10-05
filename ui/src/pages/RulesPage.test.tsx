import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, beforeEach, vi } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { RulesPage } from './RulesPage'
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

describe('RulesPage', () => {
  beforeEach(async () => {
    await api.seedDemo()
  })

  it('renders hard rules section with H3 and H4 toggles and limits', async () => {
    renderWithProviders(<RulesPage />)

    await waitFor(() => {
      expect(screen.getByTestId('rules-page')).toBeInTheDocument()
      expect(screen.getByTestId('hard-rules-section')).toBeInTheDocument()
      expect(screen.getByTestId('toggle-h3')).toBeInTheDocument()
      expect(screen.getByTestId('toggle-h4')).toBeInTheDocument()
      expect(screen.getByTestId('input-h4-max-tasks')).toBeInTheDocument()
      expect(screen.getByTestId('input-h4-max-setter')).toBeInTheDocument()
    })

    // Toggle H3 off -> should show warning box
    fireEvent.click(screen.getByTestId('toggle-h3'))
    await waitFor(() => {
      expect(screen.getByTestId('h3-warning-box')).toBeInTheDocument()
    })
  })

  it('navigates to soft rules tab, renders S1/S9/S10 and applies presets', async () => {
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)

    // Click Soft rules tab
    await user.click(screen.getByTestId('tab-soft-rules'))

    await waitFor(() => {
      expect(screen.getByTestId('soft-rules-section')).toBeInTheDocument()
      expect(screen.getByTestId('preset-workload-btn')).toBeInTheDocument()
      expect(screen.getByTestId('preset-balanced-btn')).toBeInTheDocument()
      expect(screen.getByTestId('preset-diversity-btn')).toBeInTheDocument()
      expect(screen.getByTestId('s1-mode-auto')).toBeInTheDocument()
      expect(screen.getByTestId('soft-rule-card-s9')).toBeInTheDocument()
      expect(screen.getByTestId('soft-rule-card-s10')).toBeInTheDocument()
    })

    // Click Preset "Ưu tiên công bằng khối lượng"
    await user.click(screen.getByTestId('preset-workload-btn'))

    // Save rules
    const saveBtn = screen.getByTestId('save-rules-btn')
    expect(saveBtn).not.toBeDisabled()
    await user.click(saveBtn)
  })

  it('renders quota preview and updates with H7 tolerance', async () => {
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)

    // Click Quotas tab
    await user.click(screen.getByTestId('tab-quotas'))

    await waitFor(() => {
      expect(screen.getByTestId('quotas-preview-section')).toBeInTheDocument()
      expect(screen.getByTestId('quota-preview-table')).toBeInTheDocument()
      expect(screen.getByTestId('quota-row-1')).toBeInTheDocument()
    })
  })

  it('creates a lock and displays in locks tab', async () => {
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)

    // Click Locks tab
    await user.click(screen.getByTestId('tab-locks'))

    await waitFor(() => {
      expect(screen.getByTestId('locks-section')).toBeInTheDocument()
      expect(screen.getByTestId('add-lock-btn')).toBeInTheDocument()
    })

    // Click Add Lock
    await user.click(screen.getByTestId('add-lock-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('lock-teacher-select')).toBeInTheDocument()
    })
  })

  // RA-017: S3 cannot be improved with a single campus
  it('says S3 is not applicable with a single campus, but not with several', async () => {
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)
    await user.click(screen.getByTestId('tab-soft-rules'))
    await screen.findByTestId('soft-rule-card-s3')
    await waitFor(() => expect(screen.queryByTestId('s3-not-applicable')).toBeNull())
  })

  it('shows the S3 not-applicable note when all teachers share one campus', async () => {
    const real = await api.teachersWithGrades(1)
    vi.spyOn(api, 'teachersWithGrades').mockResolvedValue(
      real.map((tg) => ({ ...tg, teacher: { ...tg.teacher, campus_id: 1 } })),
    )
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)
    await user.click(screen.getByTestId('tab-soft-rules'))
    expect(await screen.findByTestId('s3-not-applicable')).toBeInTheDocument()
  })
})
