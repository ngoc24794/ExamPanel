import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { UnavailabilityPage } from './UnavailabilityPage'
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

describe('UnavailabilityPage', () => {
  beforeEach(async () => {
    await api.seedDemo()
  })

  it('renders unavailability grid and teacher rows', async () => {
    renderWithProviders(<UnavailabilityPage />)

    await waitFor(() => {
      expect(screen.getByTestId('unavailability-page')).toBeInTheDocument()
      expect(screen.getByTestId('unavailability-grid')).toBeInTheDocument()
      expect(screen.getByTestId('exam-col-GK1')).toBeInTheDocument()
    })
  })

  it('toggles absence on click and allows adding reason', async () => {
    const user = userEvent.setup()
    renderWithProviders(<UnavailabilityPage />)

    // Wait for cell to render
    const cellBtn = await screen.findByTestId('unavail-cell-1-1')
    expect(cellBtn).toBeInTheDocument()

    // Toggle to absent
    await user.click(cellBtn)

    await waitFor(() => {
      expect(cellBtn).toHaveTextContent(/Vắng|Unavailable/i)
      expect(screen.getByTestId('reason-btn-1-1')).toBeInTheDocument()
    })

    // Click reason button
    await user.click(screen.getByTestId('reason-btn-1-1'))

    await waitFor(() => {
      expect(screen.getByTestId('unavail-reason-input')).toBeInTheDocument()
    })

    await user.type(screen.getByTestId('unavail-reason-input'), 'Bận công tác chuyên môn')
    await user.click(screen.getByTestId('save-reason-btn'))

    await waitFor(() => {
      expect(screen.queryByTestId('unavail-reason-input')).not.toBeInTheDocument()
    })
  })

  it('performs bulk actions: vắng cả học kỳ 1 and có mặt tất cả', async () => {
    const user = userEvent.setup()
    renderWithProviders(<UnavailabilityPage />)

    const bulkBtn = await screen.findByTestId('bulk-actions-1')
    await user.click(bulkBtn)

    const sem1Item = await screen.findByTestId('bulk-absent-sem1-1')
    await user.click(sem1Item)

    // Teacher 1 should be absent in exams 1 and 2
    await waitFor(() => {
      expect(screen.getByTestId('unavail-cell-1-1')).toHaveTextContent(
        /Vắng|Unavailable/i,
      )
      expect(screen.getByTestId('unavail-cell-1-2')).toHaveTextContent(
        /Vắng|Unavailable/i,
      )
    })

    // Reset with "Có mặt tất cả"
    await user.click(screen.getByTestId('bulk-actions-1'))
    const presentAllItem = await screen.findByTestId('bulk-present-all-1')
    await user.click(presentAllItem)

    await waitFor(() => {
      expect(screen.getByTestId('unavail-cell-1-1')).not.toHaveTextContent(
        /Vắng|Unavailable/i,
      )
    })
  })
})
