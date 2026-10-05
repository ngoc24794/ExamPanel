import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { api } from '@/lib/api'
import { toast } from 'sonner'
import { ExamsPage } from './ExamsPage'

vi.mock('sonner', () => ({ toast: { success: vi.fn(), info: vi.fn(), error: vi.fn() } }))

// Like the app: failed mutations are reported by the global QueryClient handler.
import { queryClient as appQueryClient } from '@/lib/query/query-client'

function renderPage() {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false, onError: appQueryClient.getDefaultOptions().mutations?.onError },
    },
  })
  return render(
    <QueryClientProvider client={client}>
      <HashRouter>
        <ExamsPage />
      </HashRouter>
    </QueryClientProvider>,
  )
}

async function openCreateDialog(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByTestId('exam-row-GK1')
  await user.click(screen.getByTestId('add-exam-btn'))
  await user.type(await screen.findByTestId('exam-code-input'), 'DBL')
  await user.type(screen.getByTestId('exam-name-input'), 'Kỳ thi đúp')
}

describe('Exam dialog submit (RA-041)', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.restoreAllMocks()
    vi.clearAllMocks()
    await api.seedDemo()
  })

  it('a double click on Save sends ONE request and ONE success toast', async () => {
    const user = userEvent.setup()
    const create = vi.spyOn(api, 'createExam')
    renderPage()
    await openCreateDialog(user)

    const save = screen.getByTestId('exam-save-btn')
    fireEvent.click(save)
    fireEvent.click(save) // second click lands before React re-renders the disabled state

    await waitFor(() => expect(toast.success).toHaveBeenCalled())
    await new Promise((r) => setTimeout(r, 100))
    expect(create).toHaveBeenCalledTimes(1)
    expect(toast.success).toHaveBeenCalledTimes(1)
    expect(toast.error).not.toHaveBeenCalled()
  })

  it('a failing save raises ONE error toast (not one from the global handler and one from the page)', async () => {
    const user = userEvent.setup()
    vi.spyOn(api, 'createExam').mockRejectedValue({ code: 'duplicate_entry', params: {} })
    renderPage()
    await openCreateDialog(user)

    await user.click(screen.getByTestId('exam-save-btn'))
    await waitFor(() => expect(toast.error).toHaveBeenCalled())
    await new Promise((r) => setTimeout(r, 100))
    expect(toast.error).toHaveBeenCalledTimes(1)
  })
})
