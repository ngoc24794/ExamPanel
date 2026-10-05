import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import '@/i18n'
import { api, type OptimizeOutcome } from '@/lib/api'
import { toast } from 'sonner'
import { RunOptimizeDialog } from './RunOptimizeDialog'

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), info: vi.fn(), error: vi.fn() },
}))

function renderDialog(onOpenChange = vi.fn(), onSuccess = vi.fn()) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  render(
    <QueryClientProvider client={client}>
      <RunOptimizeDialog
        open
        onOpenChange={onOpenChange}
        schoolYearId={1}
        onSuccess={onSuccess}
      />
    </QueryClientProvider>,
  )
  return { onOpenChange, onSuccess }
}

describe('RunOptimizeDialog cancel handling (RA-011)', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.clearAllMocks()
    await api.seedDemo()
  })

  it('cancelling a run saves nothing, shows ONE cancelled toast and keeps the dialog open', async () => {
    let rejectRun: (e: unknown) => void = () => {}
    const promise = new Promise<OptimizeOutcome>((_, reject) => {
      rejectRun = reject
    })
    // The real backend rejects with { code: 'cancelled' } once the flag is set.
    vi.spyOn(api, 'startOptimize').mockReturnValue({
      promise,
      cancel: () => {
        rejectRun({ code: 'cancelled', params: {} })
      },
    })
    const saveSpy = vi.spyOn(api, 'saveOptimizeResult')
    const { onOpenChange, onSuccess } = renderDialog()

    const start = await screen.findByTestId('start-optimize-button')
    await waitFor(() => expect(start).toBeEnabled())
    fireEvent.click(start)
    fireEvent.click(await screen.findByRole('button', { name: /hủy bỏ|cancel run/i }))

    await waitFor(() => expect(toast.info).toHaveBeenCalled())
    await new Promise((r) => setTimeout(r, 50))
    expect(toast.info).toHaveBeenCalledTimes(1)
    expect(toast.success).not.toHaveBeenCalled()
    expect(toast.error).not.toHaveBeenCalled()
    expect(saveSpy).not.toHaveBeenCalled()
    expect(onSuccess).not.toHaveBeenCalled()
    expect(onOpenChange).not.toHaveBeenCalledWith(false)
  })
})
