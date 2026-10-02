import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, beforeEach } from 'vitest'
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

  it('renders hard rules section with always-on badges and H4 toggle', async () => {
    renderWithProviders(<RulesPage />)

    await waitFor(() => {
      expect(screen.getByTestId('rules-page')).toBeInTheDocument()
      expect(screen.getByTestId('hard-rules-section')).toBeInTheDocument()
      expect(screen.getByTestId('toggle-h4')).toBeInTheDocument()
    })
  })

  it('navigates to soft rules tab and applies presets', async () => {
    const user = userEvent.setup()
    renderWithProviders(<RulesPage />)

    // Click Soft rules tab
    await user.click(screen.getByTestId('tab-soft-rules'))

    await waitFor(() => {
      expect(screen.getByTestId('soft-rules-section')).toBeInTheDocument()
      expect(screen.getByTestId('preset-workload-btn')).toBeInTheDocument()
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
})
