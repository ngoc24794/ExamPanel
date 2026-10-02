import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { CheckCircle2, AlertTriangle, AlertCircle, ShieldAlert } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { CampusChip } from '@/components/CampusChip'
import {
  type Campus,
  type Grade,
  type TeacherWithGrades,
  type FeasibilityReportWithQuotas,
} from '@/lib/api'

interface CoveragePanelProps {
  campuses: Campus[]
  grades: Grade[]
  teachersWithGrades: TeacherWithGrades[]
  feasibility?: FeasibilityReportWithQuotas
}

export const CoveragePanel: React.FC<CoveragePanelProps> = ({
  campuses,
  grades,
  teachersWithGrades,
  feasibility,
}) => {
  const { t } = useTranslation()

  // Matrix of active teachers per [gradeId][campusId]
  const matrix = React.useMemo(() => {
    const map = new Map<number, Map<number, number>>()
    for (const g of grades) {
      map.set(g.id, new Map<number, number>())
    }

    for (const tg of teachersWithGrades) {
      if (!tg.teacher.active) continue
      const cId = tg.teacher.campus_id
      for (const gId of tg.grade_ids) {
        const gradeMap = map.get(gId)
        if (gradeMap) {
          gradeMap.set(cId, (gradeMap.get(cId) || 0) + 1)
        }
      }
    }
    return map
  }, [grades, teachersWithGrades])

  // Get status and diagnostics per grade from feasibility report
  const getGradeStatus = (grade: Grade) => {
    if (!feasibility?.report) {
      return { status: 'ok', message: t('teachers.coverageStatusOk') }
    }

    const gradeCodeStr = grade.code.toString()

    // Check errors
    const err = feasibility.report.errors.find((e) => e.params?.grade === gradeCodeStr)
    if (err) {
      const msg = t(`diagnostics.${err.code}`, err.params)
      return { status: 'insufficient', message: msg }
    }

    // Check warnings
    const warn = feasibility.report.warnings.find((w) => w.params?.grade === gradeCodeStr)
    if (warn) {
      const msg = t(`diagnostics.${warn.code}`, warn.params)
      return { status: 'tight', message: msg }
    }

    return { status: 'ok', message: t('teachers.coverageStatusOk') }
  }

  if (campuses.length === 0 || grades.length === 0) {
    return null
  }

  return (
    <TooltipProvider>
      <Card className="border-border bg-card shadow-sm overflow-hidden">
        <div className="px-4 py-2.5 bg-muted/40 border-b border-border flex items-center justify-between">
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            {t('teachers.coverageTitle')}
          </span>
          {feasibility?.report && !feasibility.report.is_feasible && (
            <span className="inline-flex items-center gap-1.5 text-xs font-medium text-destructive">
              <ShieldAlert className="h-3.5 w-3.5" />
              {t('dev.feasibilityInfeasible')}
            </span>
          )}
        </div>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-xs text-foreground">
              <thead>
                <tr className="border-b border-border/80 bg-muted/20">
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground w-24">
                    {t('dev.grade')}
                  </th>
                  {campuses.map((campus) => (
                    <th
                      key={campus.id}
                      className="py-2 px-3 text-center font-medium min-w-[90px]"
                    >
                      <CampusChip name={campus.code} color={campus.color} />
                    </th>
                  ))}
                  <th className="py-2 px-3 text-center font-semibold text-foreground w-20">
                    {t('teachers.coverageTotal')}
                  </th>
                  <th className="py-2 px-3 text-right font-medium text-muted-foreground w-36">
                    {t('common.status')}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/60">
                {grades.map((grade) => {
                  const gradeMap = matrix.get(grade.id)
                  let rowTotal = 0
                  campuses.forEach((c) => {
                    rowTotal += gradeMap?.get(c.id) || 0
                  })

                  const { status, message } = getGradeStatus(grade)

                  return (
                    <tr key={grade.id} className="hover:bg-muted/30">
                      <td className="py-2 px-3 font-semibold text-foreground">
                        {grade.name}
                      </td>
                      {campuses.map((campus) => {
                        const count = gradeMap?.get(campus.id) || 0
                        return (
                          <td
                            key={campus.id}
                            className={`py-2 px-3 text-center tabular-nums ${
                              count === 0
                                ? 'text-muted-foreground/40 font-normal'
                                : 'font-semibold text-foreground'
                            }`}
                          >
                            {count}
                          </td>
                        )
                      })}
                      <td className="py-2 px-3 text-center font-bold tabular-nums text-foreground">
                        {rowTotal}
                      </td>
                      <td className="py-2 px-3 text-right">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <span className="inline-flex items-center cursor-help">
                              {status === 'ok' && (
                                <Badge
                                  variant="secondary"
                                  className="bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20 gap-1 text-[11px] font-medium"
                                >
                                  <CheckCircle2 className="h-3 w-3" />
                                  {t('teachers.coverageStatusOk')}
                                </Badge>
                              )}
                              {status === 'tight' && (
                                <Badge
                                  variant="secondary"
                                  className="bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20 gap-1 text-[11px] font-medium"
                                >
                                  <AlertTriangle className="h-3 w-3" />
                                  {t('teachers.coverageStatusTight')}
                                </Badge>
                              )}
                              {status === 'insufficient' && (
                                <Badge
                                  variant="destructive"
                                  className="gap-1 text-[11px] font-medium"
                                >
                                  <AlertCircle className="h-3 w-3" />
                                  {t('teachers.coverageStatusInsufficient')}
                                </Badge>
                              )}
                            </span>
                          </TooltipTrigger>
                          <TooltipContent side="left" className="max-w-xs text-xs">
                            {message}
                          </TooltipContent>
                        </Tooltip>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </TooltipProvider>
  )
}
