import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import '@/i18n'
import { api, type OptimizeOutcome } from '@/lib/api'
import { toast } from 'sonner'
import { ReoptimizeDialog } from './ReoptimizeDialog'

vi.mock('sonner', () => ({ toast: { success: vi.fn(), info: vi.fn(), error: vi.fn() } }))

const kept = [
  { exam_id: 1, grade_id: 1, subject_id: 1, role: 'setter' as const, position: 0 },
]

describe('ReoptimizeDialog (RA-026 UI, RA-011)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.restoreAllMocks()
  })

  it('sends the chosen number of plans and effort', async () => {
    const spy = vi.spyOn(api, 'reoptimizeFrom').mockReturnValue({
      promise: new Promise<OptimizeOutcome>(() => {}),
      cancel: () => {},
    })
    render(
      <ReoptimizeDialog
        open
        onOpenChange={() => {}}
        planId={1}
        schoolYearId={1}
        keptSlots={kept}
        onSuccess={() => {}}
      />,
    )
    fireEvent.click(screen.getByTestId('reopt-k-3'))
    fireEvent.click(screen.getByTestId('reopt-effort-standard'))
    fireEvent.click(
      screen.getByRole('button', { name: /Bắt đầu tối ưu lại|Start re-optimizing/ }),
    )
    await waitFor(() => expect(spy).toHaveBeenCalled())
    const req = spy.mock.calls[0][0]
    expect(req.request.k).toBe(3)
    expect(req.request.runs).toBe(8)
    expect(req.request.budget).toEqual({ type: 'Iterations', value: 200000 })
    expect(req.keep).toEqual(kept)
  })

  it('shows a single cancelled toast when the run is cancelled', async () => {
    let reject: (e: unknown) => void = () => {}
    vi.spyOn(api, 'reoptimizeFrom').mockReturnValue({
      promise: new Promise<OptimizeOutcome>((_, r) => {
        reject = r
      }),
      cancel: () => reject({ code: 'cancelled', params: {} }),
    })
    const save = vi.spyOn(api, 'saveOptimizeResult')
    render(
      <ReoptimizeDialog
        open
        onOpenChange={() => {}}
        planId={1}
        schoolYearId={1}
        keptSlots={kept}
        onSuccess={() => {}}
      />,
    )
    fireEvent.click(
      screen.getByRole('button', { name: /Bắt đầu tối ưu lại|Start re-optimizing/ }),
    )
    fireEvent.click(await screen.findByRole('button', { name: /Hủy bỏ|Cancel run/i }))
    await waitFor(() => expect(toast.info).toHaveBeenCalledTimes(1))
    expect(save).not.toHaveBeenCalled()
  })
})
