import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  AlertTriangle,
  Play,
  RotateCw,
  XCircle,
  ChevronDown,
  ChevronUp,
} from 'lucide-react'
import { useFeasibility } from '@/lib/query/hooks'
import { api, type OptimizeHandle, type Progress, type OptimizeOutcome } from '@/lib/api'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'

interface RunOptimizeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  schoolYearId: number
  onSuccess: (newPlanIds: number[]) => void
  onOpenFeasibility?: () => void
}

type EffortLevel = 'fast' | 'standard' | 'thorough'

export function RunOptimizeDialog({
  open,
  onOpenChange,
  schoolYearId,
  onSuccess,
  onOpenFeasibility,
}: RunOptimizeDialogProps) {
  const { t } = useTranslation()
  const { data: feasData } = useFeasibility(schoolYearId)
  const isFeasible = feasData?.report.is_feasible ?? false
  const blockingErrors = feasData?.report.errors ?? []

  const [numPlans, setNumPlans] = React.useState<number>(3)
  const [effort, setEffort] = React.useState<EffortLevel>('standard')
  const [showAdvanced, setShowAdvanced] = React.useState<boolean>(false)
  const [baseSeed, setBaseSeed] = React.useState<string>('42')

  const [isRunning, setIsRunning] = React.useState<boolean>(false)
  const [progress, setProgress] = React.useState<Progress | null>(null)
  const handleRef = React.useRef<OptimizeHandle | null>(null)

  React.useEffect(() => {
    return () => {
      if (handleRef.current) {
        void handleRef.current.cancel()
      }
    }
  }, [])

  const getRunsAndIterations = (level: EffortLevel) => {
    switch (level) {
      case 'fast':
        return { runs: 4, iterations: 50000 }
      case 'standard':
        return { runs: 8, iterations: 200000 }
      case 'thorough':
        return { runs: 16, iterations: 500000 }
    }
  }

  const handleStart = async () => {
    if (!isFeasible) return
    setIsRunning(true)
    setProgress(null)

    const { runs, iterations } = getRunsAndIterations(effort)
    const seed = baseSeed.trim() ? parseInt(baseSeed.trim(), 10) : undefined

    try {
      const handle = api.startOptimize(
        schoolYearId,
        {
          base_seed: isNaN(seed as number) ? 42 : seed,
          runs,
          budget: { type: 'Iterations', value: iterations },
          k: numPlans,
          diversity_threshold: 0.2,
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
      <DialogContent className="max-w-lg bg-card text-card-foreground border-border">
        <DialogHeader>
          <DialogTitle>{t('assignments.runTitle')}</DialogTitle>
          <DialogDescription>{t('assignments.runDesc')}</DialogDescription>
        </DialogHeader>

        {/* Preflight block if not feasible */}
        {!isFeasible && (
          <div className="rounded-md border border-destructive/50 bg-destructive/10 p-4 space-y-2">
            <div className="flex items-center gap-2 text-destructive font-medium">
              <AlertTriangle className="h-5 w-5" />
              <span>{t('assignments.preflightError')}</span>
            </div>
            <p className="text-sm text-muted-foreground">
              {blockingErrors.length} lỗi cấu hình khiến thuật toán không thể xếp lịch hợp lệ.
            </p>
            {onOpenFeasibility && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  onOpenChange(false)
                  onOpenFeasibility()
                }}
                className="mt-2 text-primary border-primary/30"
              >
                {t('assignments.preflightLink')}
              </Button>
            )}
          </div>
        )}

        {isFeasible && !isRunning && (
          <div className="space-y-4 py-2">
            {/* Number of plans */}
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
                  >
                    {k}
                  </Button>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">{t('assignments.plansCountDesc')}</p>
            </div>

            {/* Effort levels */}
            <div className="space-y-1.5">
              <label className="text-sm font-medium text-foreground">
                {t('assignments.effort')}
              </label>
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant={effort === 'fast' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setEffort('fast')}
                  className="flex-1 text-xs"
                >
                  {t('assignments.effortFast')}
                </Button>
                <Button
                  type="button"
                  variant={effort === 'standard' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setEffort('standard')}
                  className="flex-1 text-xs"
                >
                  {t('assignments.effortStandard')}
                </Button>
                <Button
                  type="button"
                  variant={effort === 'thorough' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setEffort('thorough')}
                  className="flex-1 text-xs"
                >
                  {t('assignments.effortThorough')}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {effort === 'fast' && t('assignments.effortFastDesc')}
                {effort === 'standard' && t('assignments.effortStandardDesc')}
                {effort === 'thorough' && t('assignments.effortThoroughDesc')}
              </p>
            </div>

            {/* Advanced Options */}
            <div className="pt-2 border-t border-border">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="flex items-center gap-1 text-xs text-muted-foreground px-0"
              >
                {showAdvanced ? <ChevronUp className="h-3.5 w-3.5" /> : <ChevronDown className="h-3.5 w-3.5" />}
                {t('assignments.advancedOptions')}
              </Button>
              {showAdvanced && (
                <div className="mt-2 space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    {t('assignments.baseSeed')}
                  </label>
                  <Input
                    type="number"
                    value={baseSeed}
                    onChange={(e) => setBaseSeed(e.target.value)}
                    placeholder="42"
                    className="h-8 text-xs max-w-xs"
                  />
                </div>
              )}
            </div>
          </div>
        )}

        {/* Running Progress Bar */}
        {isRunning && (
          <div className="space-y-4 py-4">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2 font-medium text-primary">
                <RotateCw className="h-4 w-4 animate-spin" />
                <span>{t('assignments.running')}</span>
              </div>
              <span className="text-xs text-muted-foreground">
                {t('assignments.elapsed', { ms: progress?.elapsed_ms ?? 0 })}
              </span>
            </div>

            <div className="w-full bg-muted rounded-full h-2.5 overflow-hidden">
              <div
                className="bg-primary h-2.5 rounded-full transition-all duration-300"
                style={{
                  width: `${Math.min(
                    100,
                    Math.round(
                      ((progress?.iteration ?? 0) /
                        getRunsAndIterations(effort).iterations) *
                        100,
                    ),
                  )}%`,
                }}
              />
            </div>

            <div className="grid grid-cols-2 gap-3 bg-muted/40 p-3 rounded-lg border border-border text-xs">
              <div>
                <span className="text-muted-foreground block">
                  {t('assignments.bestScore', { score: progress ? progress.best_score.toFixed(2) : '--' })}
                </span>
                <span className="font-semibold text-foreground text-sm">
                  {progress ? progress.best_score.toFixed(2) : '--'}
                </span>
              </div>
              <div>
                <span className="text-muted-foreground block">
                  {t('assignments.currentScore', { score: progress ? progress.current_score.toFixed(2) : '--' })}
                </span>
                <span className="font-semibold text-foreground text-sm">
                  {progress ? progress.current_score.toFixed(2) : '--'}
                </span>
              </div>
            </div>
          </div>
        )}

        <DialogFooter className="flex justify-between items-center gap-2 sm:justify-between">
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
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onOpenChange(false)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="button"
                size="sm"
                disabled={!isFeasible}
                onClick={handleStart}
                className="gap-1.5"
                data-testid="start-optimize-button"
              >
                <Play className="h-4 w-4" />
                {t('assignments.startRun')}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
