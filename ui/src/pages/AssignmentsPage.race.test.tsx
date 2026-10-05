import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api } from '@/lib/api'
import { AssignmentsPage } from './AssignmentsPage'

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

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))

describe('AssignmentsPage with a real-shaped (asynchronous) backend', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.restoreAllMocks()
    await api.seedDemo()
  })

  // RA-018: the real backend answers list_plans some milliseconds after the mutation; the page
  // selected the new copy before the list contained it and then fell back to the first plan.
  it('selects the new editable copy and shows the edit toolbar', async () => {
    const real = api.listPlans.bind(api)
    vi.spyOn(api, 'listPlans').mockImplementation(async (id: number) => {
      await sleep(120)
      return real(id)
    })
    renderPage()

    const copyBtn = await screen.findByTestId('create-edit-copy-button', undefined, {
      timeout: 3000,
    })
    fireEvent.click(copyBtn)

    await waitFor(
      () => expect(screen.getAllByText(/\(Chỉnh sửa\)/).length).toBeGreaterThan(0),
      { timeout: 3000 },
    )
    // Stays selected after the delayed list refresh has settled.
    await sleep(500)
    expect(screen.getAllByText(/\(Chỉnh sửa\)/).length).toBeGreaterThan(0)
    expect(screen.queryByTestId('create-edit-copy-button')).not.toBeInTheDocument()
  })
})
