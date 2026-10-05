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
import { createProgressAggregator, type AggregateProgress } from './optimizeProgress'
import { getRunsAndIterations, type EffortLevel } from './effortPresets'

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
  const [progress, setProgress] = React.useState<AggregateProgress | null>(null)
  const [elapsedMs, setElapsedMs] = React.useState(0)
  const [numPlans, setNumPlans] = React.useState(1)
  const [effort, setEffort] = React.useState<EffortLevel>('fast')
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
    setElapsedMs(0)

    const { runs, iterations } = getRunsAndIterations(effort)
    const aggregator = createProgressAggregator(runs, iterations)
    const startedAt = performance.now()

    try {
      const handle = api.reoptimizeFrom(
        {
          plan_id: planId,
          keep: keptSlots,
          request: {
            base_seed: 42,
            runs,
            budget: { type: 'Iterations', value: iterations },
            k: numPlans,
            diversity_threshold: 0.2,
          },
        },
        (p: Progress) => {
          setProgress(aggregator.update(p))
          setElapsedMs(Math.round(performance.now() - startedAt))
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
      // handleStart reports the 'cancelled' rejection once (RA-011).
      await handleRef.current.cancel()
    }
  }

  const percent = Math.round((progress?.fraction ?? 0) * 100)

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
            <span>{t('assignments.reoptimizeKeptNotice', { count: keptSlots.length })}</span>
          </div>

          {!isRunning && (
            <div className="space-y-3">
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-foreground">
                  {t('assignments.plansCount')} (k = {numPlans})
                </label>
                <div className="flex gap-2">
                  {[1, 2, 3, 4, 5].map((k) => (
                    <Button
                      key={k}
                      type="button"
                      variant={numPlans === k ? 'default' : 'outline'}
                      size="sm"
                      className="flex-1"
                      onClick={() => setNumPlans(k)}
                      data-testid={`reopt-k-${k}`}
                    >
                      {k}
                    </Button>
                  ))}
                </div>
              </div>
              <div className="space-y-1.5">
                <label className="text-sm font-medium text-foreground">
                  {t('assignments.effort')}
                </label>
                <div className="flex gap-2">
                  {(['fast', 'standard', 'thorough'] as const).map((level) => (
                    <Button
                      key={level}
                      type="button"
                      variant={effort === level ? 'default' : 'outline'}
                      size="sm"
                      className="flex-1 text-xs"
                      onClick={() => setEffort(level)}
                      data-testid={`reopt-effort-${level}`}
                    >
                      {t(
                        level === 'fast'
                          ? 'assignments.effortFast'
                          : level === 'standard'
                            ? 'assignments.effortStandard'
                            : 'assignments.effortThorough',
                      )}
                    </Button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {isRunning && (
            <div className="space-y-3 py-2">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span className="flex items-center gap-1.5 font-medium text-primary">
                  <RotateCw className="h-3.5 w-3.5 animate-spin" />
                  {t('assignments.reoptimizeRunning')}
                </span>
                <span>{t('assignments.elapsed', { ms: elapsedMs })}</span>
              </div>
              <div
                className="w-full bg-muted rounded-full h-2 overflow-hidden"
                role="progressbar"
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={percent}
              >
                <div
                  className="bg-primary h-2 rounded-full transition-all duration-300"
                  style={{ width: `${percent}%` }}
                />
              </div>
              <div className="text-xs text-muted-foreground text-center">
                {t('assignments.bestScore', {
                  score: progress ? progress.bestScore.toFixed(2) : '--',
                })}
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
                {t('assignments.reoptimizeStart')}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
