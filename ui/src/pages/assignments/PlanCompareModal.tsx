import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type Campus,
  type Exam,
  type Grade,
  type PlanDetails,
  type PlanSummary,
  type Subject,
  type TeacherWithGrades,
  api,
} from '@/lib/api'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { GitCompare } from 'lucide-react'
import { comparePlans, panelKey, panelMembers } from './planCompare'

interface PlanCompareModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  plans: PlanSummary[]
  initialPlanAId?: number
  initialPlanBId?: number
  exams: Exam[]
  grades: Grade[]
  subjects: Subject[]
  teachers: TeacherWithGrades[]
  campuses?: Campus[]
}

export function PlanCompareModal({
  open,
  onOpenChange,
  plans,
  initialPlanAId,
  initialPlanBId,
  exams,
  grades,
  subjects,
  teachers,
}: PlanCompareModalProps) {
  const { t } = useTranslation()

  const [planAId, setPlanAId] = React.useState<number>(
    initialPlanAId ?? (plans[0]?.id || 0),
  )
  const [planBId, setPlanBId] = React.useState<number>(
    initialPlanBId ?? (plans[1]?.id || plans[0]?.id || 0),
  )

  const [detailsA, setDetailsA] = React.useState<PlanDetails | null>(null)
  const [detailsB, setDetailsB] = React.useState<PlanDetails | null>(null)

  React.useEffect(() => {
    if (!open || !planAId || !planBId) return
    Promise.all([api.getPlan(planAId), api.getPlan(planBId)])
      .then(([a, b]) => {
        setDetailsA(a)
        setDetailsB(b)
      })
      .catch(() => {})
  }, [open, planAId, planBId])

  // Compute distance and difference map per (exam, grade, subject) panel (RA-013).
  const comparison = React.useMemo(() => {
    if (!detailsA || !detailsB) return null
    return comparePlans(detailsA.assignments, detailsB.assignments, exams, grades, subjects)
  }, [detailsA, detailsB, exams, grades, subjects])

  const teacherName = (id: number | undefined) =>
    teachers.find((tw) => tw.teacher.id === id)?.teacher.full_name ?? ''

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-4xl max-h-[90vh] overflow-y-auto bg-card text-card-foreground border-border"
        data-testid="plan-compare-modal"
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <GitCompare className="h-5 w-5 text-primary" />
            <span>{t('assignments.compareTitle')}</span>
          </DialogTitle>
          <DialogDescription>{t('assignments.compareDescription')}</DialogDescription>
        </DialogHeader>

        {/* Plan Selectors */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 py-2 border-b border-border">
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              {t('assignments.comparePlanA')}
            </label>
            <Select
              value={planAId.toString()}
              onValueChange={(val) => setPlanAId(parseInt(val, 10))}
            >
              <SelectTrigger className="w-full text-xs">
                <SelectValue placeholder={t('assignments.comparePickA')} />
              </SelectTrigger>
              <SelectContent className="bg-card border-border">
                {plans.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()} className="text-xs">
                    {p.name} ({t('assignments.compareScoreShort', { score: p.score?.toFixed(1) ?? '--' })})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              {t('assignments.comparePlanB')}
            </label>
            <Select
              value={planBId.toString()}
              onValueChange={(val) => setPlanBId(parseInt(val, 10))}
            >
              <SelectTrigger className="w-full text-xs">
                <SelectValue placeholder={t('assignments.comparePickB')} />
              </SelectTrigger>
              <SelectContent className="bg-card border-border">
                {plans.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()} className="text-xs">
                    {p.name} ({t('assignments.compareScoreShort', { score: p.score?.toFixed(1) ?? '--' })})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Comparison Summary Banner */}
        {comparison && detailsA && detailsB && (
          <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-lg bg-muted/40 border border-border text-xs">
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="text-xs font-bold">
                {t('assignments.distance', { distance: comparison.distance })}
              </Badge>
              {comparison.distance === 0 && (
                <span className="text-emerald-600 font-medium">
                  {t('assignments.identicalPlans')}
                </span>
              )}
            </div>

            <div className="flex items-center gap-4">
              <span>
                {t('assignments.compareScoreA')}:{' '}
                <strong>{detailsA.score_report?.total.toFixed(2) ?? '--'}</strong>
              </span>
              <span>
                {t('assignments.compareScoreB')}:{' '}
                <strong>{detailsB.score_report?.total.toFixed(2) ?? '--'}</strong>
              </span>
              <span>
                {t('assignments.compareScoreDelta')}:{' '}
                <strong
                  className={
                    (detailsB.score_report?.total ?? 0) <
                    (detailsA.score_report?.total ?? 0)
                      ? 'text-emerald-600'
                      : 'text-amber-600'
                  }
                >
                  {(
                    (detailsB.score_report?.total ?? 0) -
                    (detailsA.score_report?.total ?? 0)
                  ).toFixed(2)}
                </strong>
              </span>
            </div>
          </div>
        )}

        {/* Diff View Tabs */}
        {detailsA && detailsB && comparison && (
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-foreground">
              {t('assignments.diffHighlight')} (
              {t('assignments.comparePanelsChanged', { count: comparison.diffPanels.size })})
            </h4>

            {/* Matrix comparison: one block per subject panel */}
            <div className="overflow-x-auto rounded border border-border">
              <table className="w-full border-collapse text-xs">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="p-2 text-left text-muted-foreground w-28">
                      {t('assignments.compareExamGrade')}
                    </th>
                    {grades.map((g) => (
                      <th
                        key={g.id}
                        className="p-2 text-center text-foreground border-l border-border"
                      >
                        {g.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {exams.map((exam) => (
                    <tr key={exam.id} className="border-b border-border last:border-0">
                      <td className="p-2 font-medium bg-muted/10">{exam.name}</td>
                      {grades.map((grade) => (
                        <td
                          key={grade.id}
                          className="p-2 border-l border-border align-top space-y-2"
                        >
                          {subjects.map((subject) => {
                            const isDiff = comparison.diffPanels.has(
                              panelKey(exam.id, grade.id, subject.id),
                            )
                            const mA = panelMembers(
                              detailsA.assignments,
                              exam.id,
                              grade.id,
                              subject.id,
                            )
                            const mB = panelMembers(
                              detailsB.assignments,
                              exam.id,
                              grade.id,
                              subject.id,
                            )
                            const describe = (m: typeof mA) =>
                              `${m.setters.map(teacherName).join(', ')} | ${t('assignments.compareReviewerShort')}: ${m.reviewers
                                .map(teacherName)
                                .join(', ')}`
                            return (
                              <div
                                key={subject.id}
                                className={`rounded px-1.5 py-1 ${isDiff ? 'bg-amber-500/10' : ''}`}
                              >
                                <div className="text-[10px] font-semibold text-foreground">
                                  {subject.code}
                                </div>
                                {isDiff ? (
                                  <div className="space-y-0.5">
                                    <div
                                      className="text-[10px] text-muted-foreground"
                                      data-testid={`compare-cell-${exam.id}-${grade.id}-${subject.code}-A`}
                                    >
                                      <span className="font-semibold text-foreground">
                                        A:
                                      </span>{' '}
                                      {describe(mA)}
                                    </div>
                                    <div
                                      className="text-[10px] text-primary font-medium"
                                      data-testid={`compare-cell-${exam.id}-${grade.id}-${subject.code}-B`}
                                    >
                                      <span className="font-semibold text-foreground">
                                        B:
                                      </span>{' '}
                                      {describe(mB)}
                                    </div>
                                  </div>
                                ) : (
                                  <div
                                    className="text-[11px] text-muted-foreground"
                                    data-testid={`compare-cell-${exam.id}-${grade.id}-${subject.code}-same`}
                                  >
                                    {describe(mA)}
                                  </div>
                                )}
                              </div>
                            )
                          })}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Per-rule Score Differences */}
            {detailsA.score_report && detailsB.score_report && (
              <div className="space-y-2 pt-2 border-t border-border">
                <h4 className="text-sm font-semibold text-foreground">
                  {t('assignments.scoreDifference')}
                </h4>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
                  {detailsA.score_report.by_rule.map((rA) => {
                    const rB = detailsB.score_report?.by_rule.find(
                      (r) => r.rule === rA.rule,
                    )
                    const delta = (rB?.penalty ?? 0) - rA.penalty
                    return (
                      <div
                        key={rA.rule}
                        className="p-2 rounded border border-border bg-card flex justify-between items-center"
                      >
                        <span className="font-semibold uppercase">{rA.rule}</span>
                        <div className="text-right">
                          <span className="text-muted-foreground">
                            {rA.penalty.toFixed(1)} → {rB?.penalty.toFixed(1) ?? '--'}
                          </span>
                          <span
                            className={`block font-semibold ${
                              delta < 0
                                ? 'text-emerald-600'
                                : delta > 0
                                  ? 'text-amber-600'
                                  : 'text-muted-foreground'
                            }`}
                          >
                            {delta > 0 ? `+${delta.toFixed(1)}` : delta.toFixed(1)}
                          </span>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
