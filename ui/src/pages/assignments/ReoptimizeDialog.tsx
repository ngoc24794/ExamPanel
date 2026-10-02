import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type OptimizeHandle,
  type OptimizeOutcome,
  type Progress,
  type SlotRef,
  api,
} from '@/lib/api'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { RefreshCw, RotateCw, XCircle, Play, BookmarkCheck } from 'lucide-react'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'

interface ReoptimizeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  planId: number
  schoolYearId: number
  keptSlots: SlotRef[]
  onSuccess: (newPlanIds: number[]) => void
}

export function ReoptimizeDialog({
  open,
  onOpenChange,
  planId,
  schoolYearId,
  keptSlots,
  onSuccess,
}: ReoptimizeDialogProps) {
  const { t } = useTranslation()
  const [isRunning, setIsRunning] = React.useState(false)
  const [progress, setProgress] = React.useState<Progress | null>(null)
  const handleRef = React.useRef<OptimizeHandle | null>(null)

  React.useEffect(() => {
    return () => {
      if (handleRef.current) {
        void handleRef.current.cancel()
      }
    }
  }, [])

  const handleStart = async () => {
    setIsRunning(true)
    setProgress(null)

    try {
      const handle = api.reoptimizeFrom(
        {
          plan_id: planId,
          keep: keptSlots,
          request: {
            base_seed: 42,
            runs: 4,
            budget: { type: 'Iterations', value: 50000 },
            k: 1,
            diversity_threshold: 0.2,
          },
        },
        (p) => {
          setProgress(p)
        },
      )
      handleRef.current = handle
      const outcome: OptimizeOutcome = await handle.promise
      const savedIds = await api.saveOptimizeResult(schoolYearId, outcome)
      toast.success(t('assignments.runSuccess', { count: savedIds.length }))
      setIsRunning(false)
      onOpenChange(false)
      onSuccess(savedIds)
    } catch (err: unknown) {
      const errorObj = err as { code?: string }
      if (errorObj?.code === 'cancelled') {
        toast.info(t('assignments.cancelled'))
      } else {
        toast.error(getErrorMessage(err))
      }
      setIsRunning(false)
    } finally {
      handleRef.current = null
    }
  }

  const handleCancel = async () => {
    if (handleRef.current) {
      await handleRef.current.cancel()
      setIsRunning(false)
      toast.info(t('assignments.cancelled'))
    }
  }

  return (
    <Dialog open={open} onOpenChange={isRunning ? undefined : onOpenChange}>
      <DialogContent className="max-w-md bg-card border-border">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <RefreshCw className="h-5 w-5 text-primary" />
            <span>{t('assignments.reoptimizeTitle')}</span>
          </DialogTitle>
          <DialogDescription>
            {t('assignments.reoptimizeDesc', { count: keptSlots.length })}
          </DialogDescription>
        </DialogHeader>

        <div className="py-3 space-y-3">
          <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/30 text-xs">
            <BookmarkCheck className="h-4 w-4 text-amber-500 shrink-0" />
            <span>
              <strong>{keptSlots.length}</strong> vị trí được giữ cố định, thuật toán sẽ sắp xếp lại các vị trí còn lại để cải thiện điểm số.
            </span>
          </div>

          {isRunning && (
            <div className="space-y-3 py-2">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span className="flex items-center gap-1.5 font-medium text-primary">
                  <RotateCw className="h-3.5 w-3.5 animate-spin" />
                  Đang tối ưu lại...
                </span>
                <span>{progress?.elapsed_ms ?? 0}ms</span>
              </div>
              <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
                <div
                  className="bg-primary h-2 rounded-full transition-all duration-300"
                  style={{
                    width: `${Math.min(
                      100,
                      Math.round(((progress?.iteration ?? 0) / 50000) * 100),
                    )}%`,
                  }}
                />
              </div>
              <div className="text-xs text-muted-foreground text-center">
                Điểm tốt nhất: <strong>{progress?.best_score.toFixed(2) ?? '--'}</strong>
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="flex justify-between items-center sm:justify-between">
          {isRunning ? (
            <Button
              type="button"
              variant="destructive"
              size="sm"
              onClick={handleCancel}
              className="gap-1.5"
            >
              <XCircle className="h-4 w-4" />
              {t('assignments.cancelRun')}
            </Button>
          ) : (
            <>
              <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
                {t('common.cancel')}
              </Button>
              <Button size="sm" onClick={handleStart} className="gap-1.5">
                <Play className="h-4 w-4" />
                Bắt đầu tối ưu lại
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
