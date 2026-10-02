import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type Campus,
  type Exam,
  type Grade,
  type PlanDetails,
  type PlanSummary,
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

interface PlanCompareModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  plans: PlanSummary[]
  initialPlanAId?: number
  initialPlanBId?: number
  exams: Exam[]
  grades: Grade[]
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

  // Compute distance and difference map
  const comparison = React.useMemo(() => {
    if (!detailsA || !detailsB) return null

    const mapA = new Map<string, number>()
    detailsA.assignments.forEach((a) => {
      // Find role position
      const key = `${a.exam_id}_${a.grade_id}_${a.role}_${a.teacher_id}`
      mapA.set(key, a.teacher_id)
    })

    // Compare slot by slot (exam x grade x role x position)
    let distance = 0
    const diffSlots = new Set<string>()

    for (const exam of exams) {
      for (const grade of grades) {
        const assignA = detailsA.assignments.filter(
          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
        )
        const assignB = detailsB.assignments.filter(
          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
        )

        // Compare setters
        const settersA = assignA.filter((a) => a.role === 'setter').map((a) => a.teacher_id).sort()
        const settersB = assignB.filter((a) => a.role === 'setter').map((a) => a.teacher_id).sort()
        const reviewerA = assignA.find((a) => a.role === 'reviewer')?.teacher_id
        const reviewerB = assignB.find((a) => a.role === 'reviewer')?.teacher_id

        let panelDiff = false
        if (reviewerA !== reviewerB) {
          distance += 1
          panelDiff = true
        }
        for (let i = 0; i < Math.max(settersA.length, settersB.length); i++) {
          if (settersA[i] !== settersB[i]) {
            distance += 1
            panelDiff = true
          }
        }

        if (panelDiff) {
          diffSlots.add(`${exam.id}_${grade.id}`)
        }
      }
    }

    return { distance, diffSlots }
  }, [detailsA, detailsB, exams, grades])

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
          <DialogDescription>
            So sánh chi tiết phân công, điểm số và khác biệt giữa 2 phương án
          </DialogDescription>
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
                <SelectValue placeholder="Chọn phương án A" />
              </SelectTrigger>
              <SelectContent className="bg-card border-border">
                {plans.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()} className="text-xs">
                    {p.name} (Điểm: {p.score?.toFixed(1) ?? '--'})
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
                <SelectValue placeholder="Chọn phương án B" />
              </SelectTrigger>
              <SelectContent className="bg-card border-border">
                {plans.map((p) => (
                  <SelectItem key={p.id} value={p.id.toString()} className="text-xs">
                    {p.name} (Điểm: {p.score?.toFixed(1) ?? '--'})
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
                Điểm A: <strong>{detailsA.score_report?.total.toFixed(2) ?? '--'}</strong>
              </span>
              <span>
                Điểm B: <strong>{detailsB.score_report?.total.toFixed(2) ?? '--'}</strong>
              </span>
              <span>
                Chênh lệch (B - A):{' '}
                <strong
                  className={
                    (detailsB.score_report?.total ?? 0) < (detailsA.score_report?.total ?? 0)
                      ? 'text-emerald-600'
                      : 'text-amber-600'
                  }
                >
                  {((detailsB.score_report?.total ?? 0) - (detailsA.score_report?.total ?? 0)).toFixed(2)}
                </strong>
              </span>
            </div>
          </div>
        )}

        {/* Diff View Tabs */}
        {detailsA && detailsB && comparison && (
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-foreground">
              {t('assignments.diffHighlight')} ({comparison.diffSlots.size} ban đề có thay đổi)
            </h4>

            {/* Matrix comparison */}
            <div className="overflow-x-auto rounded border border-border">
              <table className="w-full border-collapse text-xs">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="p-2 text-left text-muted-foreground w-28">Kỳ / Khối</th>
                    {grades.map((g) => (
                      <th key={g.id} className="p-2 text-center text-foreground border-l border-border">
                        {g.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {exams.map((exam) => (
                    <tr key={exam.id} className="border-b border-border last:border-0">
                      <td className="p-2 font-medium bg-muted/10">{exam.name}</td>
                      {grades.map((grade) => {
                        const isDiff = comparison.diffSlots.has(`${exam.id}_${grade.id}`)
                        const assignA = detailsA.assignments.filter(
                          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
                        )
                        const assignB = detailsB.assignments.filter(
                          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
                        )

                        const getTeacherNames = (arr: typeof assignA) => {
                          const setters = arr
                            .filter((a) => a.role === 'setter')
                            .map((a) => teachers.find((t) => t.teacher.id === a.teacher_id)?.teacher.full_name)
                            .join(', ')
                          const reviewer = teachers.find(
                            (t) =>
                              t.teacher.id === arr.find((a) => a.role === 'reviewer')?.teacher_id,
                          )?.teacher.full_name
                          return { setters, reviewer }
                        }

                        const tA = getTeacherNames(assignA)
                        const tB = getTeacherNames(assignB)

                        return (
                          <td
                            key={grade.id}
                            className={`p-2 border-l border-border align-top ${
                              isDiff ? 'bg-amber-500/10' : ''
                            }`}
                          >
                            {isDiff ? (
                              <div className="space-y-1">
                                <div className="text-[10px] text-muted-foreground">
                                  <span className="font-semibold text-foreground">A:</span> {tA.setters} | PB: {tA.reviewer}
                                </div>
                                <div className="text-[10px] text-primary font-medium">
                                  <span className="font-semibold text-foreground">B:</span> {tB.setters} | PB: {tB.reviewer}
                                </div>
                              </div>
                            ) : (
                              <div className="text-[11px] text-muted-foreground">
                                {tA.setters} | {tA.reviewer}
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

            {/* Per-rule Score Differences */}
            {detailsA.score_report && detailsB.score_report && (
              <div className="space-y-2 pt-2 border-t border-border">
                <h4 className="text-sm font-semibold text-foreground">
                  {t('assignments.scoreDifference')}
                </h4>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
                  {detailsA.score_report.by_rule.map((rA) => {
                    const rB = detailsB.score_report?.by_rule.find((r) => r.rule === rA.rule)
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
