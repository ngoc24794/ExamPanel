import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import '@/i18n'
import { api, type OptimizeOutcome, type Progress } from '@/lib/api'
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

describe('RunOptimizeDialog progress (RA-012)', () => {
  beforeEach(async () => {
    localStorage.clear()
    vi.clearAllMocks()
    await api.seedDemo()
  })

  // Real shape: 8 runs executed in waves of 4; every event carries ONE run's counters.
  it('keeps the progress bar and best score monotonic across parallel waves', async () => {
    let emit: (p: Progress) => void = () => {}
    vi.spyOn(api, 'startOptimize').mockImplementation((_sy, _req, onProgress) => {
      emit = onProgress as (p: Progress) => void
      return { promise: new Promise<OptimizeOutcome>(() => {}), cancel: () => {} }
    })
    renderDialog()
    const start = await screen.findByTestId('start-optimize-button')
    await waitFor(() => expect(start).toBeEnabled())
    fireEvent.click(start)

    const budget = 200000 // 'standard' effort: 8 runs x 200000 iterations
    const events: Progress[] = []
    for (const wave of [0, 4]) {
      for (const frac of [0.25, 0.5, 1]) {
        for (let r = wave; r < wave + 4; r++) {
          events.push({
            run: r,
            iteration: Math.round(budget * frac),
            best_score: 180 - r * 2 - frac * 10 + (wave ? 8 : 0), // later waves may report worse bests
            current_score: 200,
            elapsed_ms: Math.round(frac * 800), // per-run clock restarts every wave
          })
        }
      }
    }

    let lastBar = -1
    let lastBest = Number.POSITIVE_INFINITY
    for (const ev of events) {
      act(() => emit(ev))
      const bar = await screen.findByRole('progressbar')
      const now = Number(bar.getAttribute('aria-valuenow'))
      expect(now).toBeGreaterThanOrEqual(lastBar)
      lastBar = now
      const bestText = screen.getByTestId('best-score-value')
      const best = Number(bestText.textContent)
      expect(best).toBeLessThanOrEqual(lastBest)
      lastBest = best
    }
    expect(lastBar).toBe(100)
  })
})
