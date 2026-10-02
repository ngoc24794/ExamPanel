import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Play,
  Square,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Database,
  Search,
  Sparkles,
  Loader2,
} from 'lucide-react'
import {
  api,
  type Exam,
  type Grade,
  type Teacher,
  type RankedPlan,
  type Progress,
  type FeasibilityReportWithQuotas,
  type AppError,
  formatAppError,
} from '@/lib/api'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

export function DevPage() {
  const { t } = useTranslation()

  const [schoolYearId] = React.useState<number>(1)
  const [teachers, setTeachers] = React.useState<Map<number, Teacher>>(new Map())
  const [exams, setExams] = React.useState<Exam[]>([])
  const [grades, setGrades] = React.useState<Grade[]>([])

  const [feasibility, setFeasibility] =
    React.useState<FeasibilityReportWithQuotas | null>(null)
  const [checkingFeasibility, setCheckingFeasibility] = React.useState<boolean>(false)

  const [optimizing, setOptimizing] = React.useState<boolean>(false)
  const [progress, setProgress] = React.useState<Progress | null>(null)
  const [bestPlan, setBestPlan] = React.useState<RankedPlan | null>(null)
  const [statusMessage, setStatusMessage] = React.useState<string | null>(null)
  const [errorMessage, setErrorMessage] = React.useState<string | null>(null)

  const cancelRef = React.useRef<(() => void) | null>(null)

  const loadBaseData = React.useCallback(async () => {
    try {
      const [tList, eList, gList] = await Promise.all([
        api.listTeachers(),
        api.listExams(schoolYearId),
        api.listGrades(),
      ])
      const map = new Map<number, Teacher>()
      for (const teacher of tList) {
        map.set(teacher.id, teacher)
      }
      setTeachers(map)
      setExams(eList.sort((a: Exam, b: Exam) => a.sort_order - b.sort_order))
      setGrades(gList.sort((a: Grade, b: Grade) => a.sort_order - b.sort_order))

      // Check if existing plans exist
      const plans = await api.listPlans(schoolYearId)
      if (plans.length > 0 && !bestPlan) {
        const details = await api.getPlan(plans[0].id)
        if (details.score_report) {
          setBestPlan({
            rank: details.plan.rank ?? 1,
            seed: details.plan.seed,
            assignments: details.assignments,
            report: details.score_report,
          })
        }
      }
    } catch (err) {
      console.error('Failed to load base data', err)
    }
  }, [schoolYearId, bestPlan])

  React.useEffect(() => {
    loadBaseData()
  }, [loadBaseData])

  const handleSeedDemo = async () => {
    setStatusMessage(null)
    setErrorMessage(null)
    try {
      await api.seedDemo()
      setStatusMessage(t('dev.seedSuccess'))
      await loadBaseData()
    } catch (err) {
      setErrorMessage(formatAppError(err, t))
    }
  }

  const handleCheckFeasibility = async () => {
    setCheckingFeasibility(true)
    setStatusMessage(null)
    setErrorMessage(null)
    try {
      const res = await api.checkFeasibility(schoolYearId)
      setFeasibility(res)
    } catch (err) {
      setErrorMessage(formatAppError(err, t))
    } finally {
      setCheckingFeasibility(false)
    }
  }

  const handleStartOptimize = async () => {
    setOptimizing(true)
    setProgress(null)
    setStatusMessage(null)
    setErrorMessage(null)

    try {
      const handle = api.startOptimize(
        schoolYearId,
        {
          base_seed: 42,
          runs: 4,
          budget: { type: 'Iterations', value: 50000 },
          k: 3,
          diversity_threshold: 0.2,
        },
        (p: Progress) => {
          setProgress(p)
        },
      )

      cancelRef.current = handle.cancel
      const outcome = await handle.promise
      if (outcome.plans.length > 0) {
        setBestPlan(outcome.plans[0])
        await api.saveOptimizeResult(schoolYearId, outcome)
      }
    } catch (err) {
      const appErr = err as AppError
      if (appErr?.code === 'cancelled') {
        setStatusMessage(t('errors.cancelled'))
      } else {
        setErrorMessage(formatAppError(err, t))
      }
    } finally {
      setOptimizing(false)
      cancelRef.current = null
    }
  }

  const handleCancelOptimize = () => {
    if (cancelRef.current) {
      cancelRef.current()
    } else {
      api.cancelOptimize().catch(console.error)
    }
  }

  return (
    <div className="max-w-6xl mx-auto space-y-6 pb-12">
      {/* Header */}
      <div>
        <h3 className="text-2xl font-bold tracking-tight">{t('dev.title')}</h3>
        <p className="text-sm text-muted-foreground mt-1">{t('dev.subtitle')}</p>
      </div>

      {/* Status Notifications */}
      {statusMessage && (
        <div className="p-3 rounded-lg border border-emerald-500/20 bg-emerald-500/10 text-emerald-600 flex items-center gap-2 text-sm font-medium">
          <CheckCircle2 className="h-4 w-4 flex-shrink-0" />
          <span>{statusMessage}</span>
        </div>
      )}

      {errorMessage && (
        <div className="p-3 rounded-lg border border-destructive/20 bg-destructive/10 text-destructive flex items-center gap-2 text-sm font-medium">
          <XCircle className="h-4 w-4 flex-shrink-0" />
          <span>{errorMessage}</span>
        </div>
      )}

      {/* Control Actions Bar */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-primary" />
            <span>Developer Actions</span>
          </CardTitle>
          <CardDescription>
            Control solver execution, test feasibility verification, and seed demo
            records.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap items-center gap-3">
            <Button
              variant="outline"
              onClick={handleSeedDemo}
              disabled={optimizing}
              className="gap-2"
            >
              <Database className="h-4 w-4" />
              <span>{t('dev.seedDemo')}</span>
            </Button>

            <Button
              variant="outline"
              onClick={handleCheckFeasibility}
              disabled={optimizing || checkingFeasibility}
              className="gap-2"
            >
              {checkingFeasibility ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Search className="h-4 w-4" />
              )}
              <span>{t('dev.checkFeasibility')}</span>
            </Button>

            {!optimizing ? (
              <Button variant="default" onClick={handleStartOptimize} className="gap-2">
                <Play className="h-4 w-4" />
                <span>{t('dev.startOptimize')}</span>
              </Button>
            ) : (
              <Button
                variant="destructive"
                onClick={handleCancelOptimize}
                className="gap-2"
              >
                <Square className="h-4 w-4" />
                <span>{t('dev.cancelOptimize')}</span>
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Feasibility Report Output */}
      {feasibility && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">{t('dev.checkFeasibility')}</CardTitle>
              {feasibility.report.is_feasible ? (
                <Badge
                  variant="outline"
                  className="bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                >
                  <CheckCircle2 className="h-3 w-3 mr-1" />
                  {t('dev.feasibilityFeasible')}
                </Badge>
              ) : (
                <Badge
                  variant="outline"
                  className="bg-destructive/10 text-destructive border-destructive/20"
                >
                  <AlertTriangle className="h-3 w-3 mr-1" />
                  {t('dev.feasibilityInfeasible')}
                </Badge>
              )}
            </div>
            <CardDescription>
              {t('dev.errors', { count: feasibility.report.errors.length })} •{' '}
              {t('dev.warnings', { count: feasibility.report.warnings.length })}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {feasibility.report.errors.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-destructive uppercase tracking-wider">
                  {t('dev.errors', { count: feasibility.report.errors.length })}
                </h4>
                <div className="space-y-1">
                  {feasibility.report.errors.map((err, idx) => (
                    <div
                      key={idx}
                      className="p-2 rounded bg-destructive/10 border border-destructive/20 text-xs text-destructive"
                    >
                      {t(`diagnostics.${err.code}`, {
                        ...err.params,
                        defaultValue: err.code,
                      })}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {feasibility.report.warnings.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  {t('dev.warnings', { count: feasibility.report.warnings.length })}
                </h4>
                <div className="space-y-1">
                  {feasibility.report.warnings.map((warn, idx) => (
                    <div
                      key={idx}
                      className="p-2 rounded bg-muted border text-xs text-muted-foreground"
                    >
                      {t(`diagnostics.${warn.code}`, {
                        ...warn.params,
                        defaultValue: warn.code,
                      })}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Optimization Progress */}
      {(optimizing || progress) && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base flex items-center gap-2">
                {optimizing && <Loader2 className="h-4 w-4 animate-spin text-primary" />}
                <span>{optimizing ? t('dev.optimizing') : 'Optimization Completed'}</span>
              </CardTitle>
              {progress && (
                <div className="flex items-center gap-2 font-mono text-xs">
                  <Badge variant="outline">
                    {t('dev.run')}: {progress.run}
                  </Badge>
                  <Badge variant="outline">
                    {t('dev.iteration')}: {progress.iteration.toLocaleString()}
                  </Badge>
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Visual Progress Bar */}
            <div className="h-2 w-full overflow-hidden rounded-full bg-secondary">
              <div
                className={`h-full bg-primary transition-all duration-300 ${
                  optimizing ? 'animate-pulse' : ''
                }`}
                style={{
                  width: `${Math.min(
                    100,
                    Math.max(5, ((progress?.iteration ?? 0) / 50000) * 100),
                  )}%`,
                }}
              />
            </div>

            {progress && (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-center">
                <div className="p-3 rounded-lg bg-card border">
                  <div className="text-xs text-muted-foreground font-medium">
                    {t('dev.bestScore')}
                  </div>
                  <div className="text-xl font-bold font-mono text-primary mt-1">
                    {progress.best_score.toFixed(2)}
                  </div>
                </div>
                <div className="p-3 rounded-lg bg-card border">
                  <div className="text-xs text-muted-foreground font-medium">
                    {t('dev.currentScore')}
                  </div>
                  <div className="text-xl font-bold font-mono mt-1">
                    {progress.current_score.toFixed(2)}
                  </div>
                </div>
                <div className="p-3 rounded-lg bg-card border">
                  <div className="text-xs text-muted-foreground font-medium">
                    {t('dev.iteration')}
                  </div>
                  <div className="text-xl font-bold font-mono mt-1">
                    {progress.iteration.toLocaleString()}
                  </div>
                </div>
                <div className="p-3 rounded-lg bg-card border">
                  <div className="text-xs text-muted-foreground font-medium">
                    Elapsed Time
                  </div>
                  <div className="text-xl font-bold font-mono mt-1">
                    {(progress.elapsed_ms / 1000).toFixed(1)}s
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Best Plan 4 x 3 Matrix */}
      {bestPlan ? (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="text-base">{t('dev.planTableTitle')}</CardTitle>
                <CardDescription>
                  Seed: {bestPlan.seed} • Rank: #{bestPlan.rank} • Total Penalty:{' '}
                  <span className="font-mono font-bold text-foreground">
                    {bestPlan.report.total.toFixed(2)}
                  </span>
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto border rounded-lg">
              <table className="w-full text-xs border-collapse">
                <thead>
                  <tr className="bg-muted/50 border-b">
                    <th className="p-3 text-left font-semibold text-muted-foreground w-32 border-r">
                      {t('dev.exam')} \ {t('dev.grade')}
                    </th>
                    {grades.map((grade) => (
                      <th
                        key={grade.id}
                        className="p-3 text-left font-semibold text-muted-foreground border-r last:border-r-0"
                      >
                        {grade.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y">
                  {exams.map((exam) => (
                    <tr key={exam.id} className="hover:bg-muted/20 transition-colors">
                      <td className="p-3 font-semibold bg-muted/30 border-r text-foreground">
                        {exam.name}
                      </td>
                      {grades.map((grade) => {
                        const cellAssignments = bestPlan.assignments.filter(
                          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
                        )
                        const setters = cellAssignments.filter((a) => a.role === 'setter')
                        const reviewer = cellAssignments.find(
                          (a) => a.role === 'reviewer',
                        )

                        return (
                          <td
                            key={grade.id}
                            className="p-3 border-r last:border-r-0 align-top space-y-1.5"
                          >
                            <div className="space-y-1">
                              <div className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider">
                                {t('dev.setters')}
                              </div>
                              <div className="space-y-0.5">
                                {setters.map((s) => (
                                  <div
                                    key={s.teacher_id}
                                    className="font-medium text-foreground bg-muted/60 px-1.5 py-0.5 rounded text-[11px]"
                                  >
                                    {teachers.get(s.teacher_id)?.full_name ??
                                      `Teacher #${s.teacher_id}`}
                                  </div>
                                ))}
                              </div>
                            </div>

                            {reviewer && (
                              <div className="pt-1 border-t space-y-1">
                                <div className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider">
                                  {t('dev.reviewer')}
                                </div>
                                <div className="font-medium text-primary bg-primary/10 px-1.5 py-0.5 rounded text-[11px]">
                                  {teachers.get(reviewer.teacher_id)?.full_name ??
                                    `Teacher #${reviewer.teacher_id}`}
                                </div>
                              </div>
                            )}
                          </td>
                        )
                      })}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="py-8 text-center text-muted-foreground text-sm">
            {t('dev.noPlan')}
          </CardContent>
        </Card>
      )}

      {/* Score Breakdown & Lower Bounds */}
      {bestPlan && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t('dev.scoreBreakdownTitle')}</CardTitle>
            <CardDescription>
              Objective penalty breakdown alongside mathematically provable lower bounds.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto border rounded-lg">
              <table className="w-full text-xs border-collapse">
                <thead>
                  <tr className="bg-muted/50 border-b text-left">
                    <th className="p-3 font-semibold text-muted-foreground">
                      {t('dev.rule')}
                    </th>
                    <th className="p-3 font-semibold text-muted-foreground text-right">
                      {t('dev.weight')}
                    </th>
                    <th className="p-3 font-semibold text-muted-foreground text-right">
                      {t('dev.units')}
                    </th>
                    <th className="p-3 font-semibold text-muted-foreground text-right">
                      {t('dev.penalty')}
                    </th>
                    <th className="p-3 font-semibold text-muted-foreground text-right">
                      {t('dev.lowerBound')}
                    </th>
                    <th className="p-3 font-semibold text-muted-foreground text-center">
                      {t('dev.lowerBoundStatus')}
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y">
                  {bestPlan.report.by_rule.map((r) => {
                    const isOptimal = Math.abs(r.units - r.lower_bound) <= 1e-4

                    return (
                      <tr key={r.rule} className="hover:bg-muted/20">
                        <td className="p-3 font-mono font-medium text-foreground">
                          {r.rule}
                        </td>
                        <td className="p-3 font-mono text-right text-muted-foreground">
                          {r.weight.toFixed(1)}
                        </td>
                        <td className="p-3 font-mono text-right text-foreground font-semibold">
                          {r.units.toFixed(2)}
                        </td>
                        <td className="p-3 font-mono text-right text-foreground font-semibold">
                          {r.penalty.toFixed(2)}
                        </td>
                        <td className="p-3 font-mono text-right text-muted-foreground">
                          {r.lower_bound.toFixed(2)}
                        </td>
                        <td className="p-3 text-center">
                          {isOptimal ? (
                            <Badge
                              variant="outline"
                              className="border-emerald-500/30 text-emerald-600 bg-emerald-500/10 text-[10px]"
                            >
                              {t('dev.cannotImprove')}
                            </Badge>
                          ) : (
                            <Badge
                              variant="secondary"
                              className="text-[10px] text-muted-foreground"
                            >
                              {t('dev.canImprove')}
                            </Badge>
                          )}
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
                <tfoot>
                  <tr className="bg-muted/40 border-t font-semibold">
                    <td colSpan={3} className="p-3 text-foreground font-bold">
                      {t('dev.total')}
                    </td>
                    <td className="p-3 font-mono text-right font-bold text-primary text-sm">
                      {bestPlan.report.total.toFixed(2)}
                    </td>
                    <td colSpan={2} />
                  </tr>
                </tfoot>
              </table>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
