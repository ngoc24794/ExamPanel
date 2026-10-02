import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type PlanDetails,
  api,
} from '@/lib/api'
import {
  useSchoolYears,
  usePlans,
  useCampuses,
  useGrades,
  useExams,
  useTeachers,
} from '@/lib/query/hooks'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import {
  BarChart3,
  Users,
  Building2,
  Layers,
  ArrowUpDown,
  Loader2,
} from 'lucide-react'

export function StatisticsPage() {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const schoolYearId = currentYear?.id ?? 1

  const { data: campuses = [] } = useCampuses()
  const { data: grades = [] } = useGrades()
  const { data: exams = [] } = useExams(schoolYearId)
  const { data: teachers = [] } = useTeachers(schoolYearId)
  const { data: plans = [] } = usePlans(schoolYearId)

  const [selectedPlanId, setSelectedPlanId] = React.useState<number | null>(null)
  const [planDetails, setPlanDetails] = React.useState<PlanDetails | null>(null)
  const [isLoading, setIsLoading] = React.useState(false)

  // Default to final plan or latest plan
  React.useEffect(() => {
    if (plans.length > 0 && selectedPlanId === null) {
      const finalPlan = plans.find((p) => p.is_final)
      setSelectedPlanId(finalPlan ? finalPlan.id : plans[0].id)
    }
  }, [plans, selectedPlanId])

  // Load plan details when selectedPlanId changes
  React.useEffect(() => {
    if (!selectedPlanId) return
    setIsLoading(true)
    api
      .getPlan(selectedPlanId)
      .then((res) => {
        setPlanDetails(res)
        setIsLoading(false)
      })
      .catch(() => setIsLoading(false))
  }, [selectedPlanId])

  // Sort state for teacher table
  type SortKey = 'name' | 'campus' | 'count' | 'setter' | 'reviewer' | 'quota' | 'deviation'
  const [sortKey, setSortKey] = React.useState<SortKey>('count')
  const [sortAsc, setSortAsc] = React.useState<boolean>(false)

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortAsc(!sortAsc)
    } else {
      setSortKey(key)
      setSortAsc(false)
    }
  }

  // Teacher statistics derived from plan
  const teacherRows = React.useMemo(() => {
    if (!planDetails) return []
    const assignments = planDetails.assignments
    const scoreReport = planDetails.score_report

    return teachers.map((twg) => {
      const tRec = twg.teacher
      const campus = campuses.find((c) => c.id === tRec.campus_id)
      const tStats = scoreReport?.per_teacher.find((st) => st.teacher_id === tRec.id)

      const myAssign = assignments.filter((a) => a.teacher_id === tRec.id)
      const count = myAssign.length
      const setter = myAssign.filter((a) => a.role === 'setter').length
      const reviewer = myAssign.filter((a) => a.role === 'reviewer').length
      const quota = tStats?.quota ?? tRec.load_weight * 3.27
      const deviation = count - quota

      const myGradeIds = Array.from(new Set(myAssign.map((a) => a.grade_id)))
      const myExamIds = Array.from(new Set(myAssign.map((a) => a.exam_id)))

      const gradesStr = myGradeIds
        .map((gid) => grades.find((g) => g.id === gid)?.code ?? gid)
        .join(', ')

      const examsStr = myExamIds
        .map((eid) => exams.find((e) => e.id === eid)?.code ?? eid)
        .join(', ')

      return {
        id: tRec.id,
        name: tRec.full_name,
        campusName: campus?.name ?? '',
        campusColor: campus?.color ?? '#64748b',
        count,
        setter,
        reviewer,
        quota,
        deviation,
        gradesStr,
        examsStr,
      }
    })
  }, [planDetails, teachers, campuses, grades, exams])

  const sortedTeacherRows = React.useMemo(() => {
    return [...teacherRows].sort((a, b) => {
      let cmp = 0
      switch (sortKey) {
        case 'name':
          cmp = a.name.localeCompare(b.name)
          break
        case 'campus':
          cmp = a.campusName.localeCompare(b.campusName)
          break
        case 'count':
          cmp = a.count - b.count
          break
        case 'setter':
          cmp = a.setter - b.setter
          break
        case 'reviewer':
          cmp = a.reviewer - b.reviewer
          break
        case 'quota':
          cmp = a.quota - b.quota
          break
        case 'deviation':
          cmp = a.deviation - b.deviation
          break
      }
      return sortAsc ? cmp : -cmp
    })
  }, [teacherRows, sortKey, sortAsc])

  // Co-working matrix (teacher × teacher: count on same panel)
  const coWorkingMatrix = React.useMemo(() => {
    if (!planDetails) return null
    const assignments = planDetails.assignments

    // Panel key -> teacher ids in that panel
    const panelTeachers = new Map<string, number[]>()
    for (const a of assignments) {
      const key = `${a.exam_id}_${a.grade_id}`
      if (!panelTeachers.has(key)) panelTeachers.set(key, [])
      panelTeachers.get(key)!.push(a.teacher_id)
    }

    const matrix = new Map<string, number>()
    for (const [, tIds] of panelTeachers) {
      for (let i = 0; i < tIds.length; i++) {
        for (let j = 0; j < tIds.length; j++) {
          if (i !== j) {
            const key = `${tIds[i]}_${tIds[j]}`
            matrix.set(key, (matrix.get(key) || 0) + 1)
          }
        }
      }
    }
    return matrix
  }, [planDetails])

  // Review-relation matrix (reviewer → setter)
  const reviewRelationMatrix = React.useMemo(() => {
    if (!planDetails) return null
    const assignments = planDetails.assignments

    const matrix = new Map<string, number>()
    for (const exam of exams) {
      for (const grade of grades) {
        const panelAssign = assignments.filter(
          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
        )
        const reviewer = panelAssign.find((a) => a.role === 'reviewer')?.teacher_id
        const setters = panelAssign.filter((a) => a.role === 'setter').map((a) => a.teacher_id)

        if (reviewer) {
          for (const s of setters) {
            const key = `${reviewer}_${s}`
            matrix.set(key, (matrix.get(key) || 0) + 1)
          }
        }
      }
    }
    return matrix
  }, [planDetails, exams, grades])

  // Campus mix analysis
  const campusMixStats = React.useMemo(() => {
    if (!planDetails) return { mix1: 0, mix2: 0, mix3: 0, total: 0 }
    const assignments = planDetails.assignments

    let mix1 = 0
    let mix2 = 0
    let mix3 = 0
    let total = 0

    for (const exam of exams) {
      for (const grade of grades) {
        const panelAssign = assignments.filter(
          (a) => a.exam_id === exam.id && a.grade_id === grade.id,
        )
        if (panelAssign.length === 0) continue
        total += 1

        const campusIds = new Set(
          panelAssign
            .map((a) => teachers.find((t) => t.teacher.id === a.teacher_id)?.teacher.campus_id)
            .filter(Boolean),
        )

        if (campusIds.size === 1) mix1 += 1
        else if (campusIds.size === 2) mix2 += 1
        else if (campusIds.size >= 3) mix3 += 1
      }
    }

    return { mix1, mix2, mix3, total }
  }, [planDetails, exams, grades, teachers])

  return (
    <div className="space-y-6" data-testid="statistics-page">
      {/* Top Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <span>{t('statistics.title')}</span>
            <Badge variant="outline" className="text-xs">
              {currentYear?.name}
            </Badge>
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            {t('statistics.description')}
          </p>
        </div>

        {/* Plan Selector */}
        <div className="flex items-center gap-2 min-w-[280px]">
          <span className="text-xs text-muted-foreground whitespace-nowrap">
            {t('statistics.selectPlan')}:
          </span>
          <Select
            value={selectedPlanId?.toString() ?? ''}
            onValueChange={(val) => setSelectedPlanId(parseInt(val, 10))}
          >
            <SelectTrigger className="text-xs w-full bg-card border-border">
              <SelectValue placeholder="Chọn phương án" />
            </SelectTrigger>
            <SelectContent className="bg-card border-border">
              {plans.map((p) => (
                <SelectItem key={p.id} value={p.id.toString()} className="text-xs">
                  {p.name} {p.is_final ? '★ (Chính thức)' : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {isLoading && <Loader2 className="h-4 w-4 animate-spin text-muted-foreground shrink-0" />}
        </div>
      </div>

      {plans.length === 0 ? (
        <div className="p-8 text-center border rounded-lg bg-card text-muted-foreground border-dashed">
          Chưa có phương án nào để xem thống kê. Hãy chạy tối ưu trong mục Phân công.
        </div>
      ) : (
        <div className="space-y-6">
          {/* Workload Distribution Chart (Load Chart) */}
          <Card className="bg-card border-border shadow-sm">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-sm font-semibold flex items-center gap-2">
                    <BarChart3 className="h-4 w-4 text-primary" />
                    <span>{t('statistics.loadChartTitle')}</span>
                  </CardTitle>
                  <CardDescription className="text-xs">
                    So sánh số lượt phân công thực tế với chỉ tiêu phân bổ định mức
                  </CardDescription>
                </div>
                <div className="flex items-center gap-3 text-xs">
                  <div className="flex items-center gap-1.5">
                    <div className="w-3 h-3 rounded bg-primary" />
                    <span>{t('statistics.loadChartLegendActual')}</span>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <div className="w-3 h-3 rounded bg-muted-foreground/30 border border-muted-foreground/50" />
                    <span>{t('statistics.loadChartLegendQuota')}</span>
                  </div>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 pt-2">
                {teacherRows.map((row) => {
                  const maxVal = Math.max(
                    ...teacherRows.map((r) => Math.max(r.count, r.quota)),
                    6,
                  )
                  const actualPct = (row.count / maxVal) * 100
                  const quotaPct = (row.quota / maxVal) * 100

                  return (
                    <div key={row.id} className="grid grid-cols-12 items-center gap-2 text-xs">
                      <div className="col-span-3 truncate font-medium text-foreground flex items-center gap-1.5">
                        <div
                          className="w-2 h-2 rounded-full shrink-0"
                          style={{ backgroundColor: row.campusColor }}
                        />
                        <span className="truncate">{row.name}</span>
                      </div>

                      <div className="col-span-7 relative h-5 bg-muted/30 rounded overflow-hidden">
                        {/* Quota bar */}
                        <div
                          className="absolute top-0 bottom-0 left-0 bg-muted-foreground/20 border-r-2 border-muted-foreground"
                          style={{ width: `${quotaPct}%` }}
                        />
                        {/* Actual bar */}
                        <div
                          className="absolute top-1 bottom-1 left-0 bg-primary rounded-sm transition-all"
                          style={{ width: `${actualPct}%` }}
                        />
                      </div>

                      <div className="col-span-2 text-right text-[11px] font-mono">
                        <span className="font-bold text-foreground">{row.count}</span> / {row.quota.toFixed(1)}
                      </div>
                    </div>
                  )
                })}
              </div>
            </CardContent>
          </Card>

          {/* Per-Teacher Table */}
          <Card className="bg-card border-border shadow-sm">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <Users className="h-4 w-4 text-primary" />
                <span>{t('statistics.perTeacher')}</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto rounded border border-border">
                <table className="w-full border-collapse text-xs">
                  <thead>
                    <tr className="border-b border-border bg-muted/40 text-muted-foreground">
                      <th
                        className="p-2.5 text-left font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('name')}
                      >
                        <div className="flex items-center gap-1">
                          <span>{t('statistics.teacherName')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-left font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('campus')}
                      >
                        <div className="flex items-center gap-1">
                          <span>{t('statistics.campus')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-center font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('count')}
                      >
                        <div className="flex items-center justify-center gap-1">
                          <span>{t('statistics.count')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-center font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('setter')}
                      >
                        <div className="flex items-center justify-center gap-1">
                          <span>{t('statistics.setter')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-center font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('reviewer')}
                      >
                        <div className="flex items-center justify-center gap-1">
                          <span>{t('statistics.reviewer')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-center font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('quota')}
                      >
                        <div className="flex items-center justify-center gap-1">
                          <span>{t('statistics.quota')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th
                        className="p-2.5 text-center font-semibold cursor-pointer hover:text-foreground"
                        onClick={() => handleSort('deviation')}
                      >
                        <div className="flex items-center justify-center gap-1">
                          <span>{t('statistics.deviation')}</span>
                          <ArrowUpDown className="h-3 w-3" />
                        </div>
                      </th>
                      <th className="p-2.5 text-left font-semibold">
                        <span>{t('statistics.grades')}</span>
                      </th>
                      <th className="p-2.5 text-left font-semibold">
                        <span>{t('statistics.exams')}</span>
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {sortedTeacherRows.map((r) => {
                      const devAbs = Math.abs(r.deviation)
                      return (
                        <tr
                          key={r.id}
                          className="border-b border-border last:border-0 hover:bg-muted/10 transition-colors"
                        >
                          <td className="p-2.5 font-medium text-foreground">
                            <div className="flex items-center gap-1.5">
                              <div
                                className="w-2.5 h-2.5 rounded-full shrink-0"
                                style={{ backgroundColor: r.campusColor }}
                              />
                              <span>{r.name}</span>
                            </div>
                          </td>
                          <td className="p-2.5 text-muted-foreground">{r.campusName}</td>
                          <td className="p-2.5 text-center font-bold text-foreground">{r.count}</td>
                          <td className="p-2.5 text-center text-muted-foreground">{r.setter}</td>
                          <td className="p-2.5 text-center text-muted-foreground">{r.reviewer}</td>
                          <td className="p-2.5 text-center font-mono text-muted-foreground">
                            {r.quota.toFixed(2)}
                          </td>
                          <td className="p-2.5 text-center font-mono font-medium">
                            <span
                              className={
                                devAbs <= 0.5
                                  ? 'text-emerald-600 dark:text-emerald-400'
                                  : 'text-amber-600 dark:text-amber-400'
                              }
                            >
                              {r.deviation >= 0 ? `+${r.deviation.toFixed(2)}` : r.deviation.toFixed(2)}
                            </span>
                          </td>
                          <td className="p-2.5 text-muted-foreground">Khối {r.gradesStr || '--'}</td>
                          <td className="p-2.5 text-muted-foreground">{r.examsStr || '--'}</td>
                        </tr>
                      )
                    })}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          {/* Campus Mix Analysis */}
          <Card className="bg-card border-border shadow-sm">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <Building2 className="h-4 w-4 text-primary" />
                <span>{t('statistics.campusMixTitle')}</span>
              </CardTitle>
              <CardDescription className="text-xs">
                {t('statistics.campusMixDesc')} (Tổng số: {campusMixStats.total} ban đề)
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="p-4 rounded-lg border border-border bg-muted/20 space-y-1 text-center">
                  <span className="text-xs text-muted-foreground block">
                    {t('statistics.campusMix1')}
                  </span>
                  <span className="text-2xl font-bold text-destructive">
                    {campusMixStats.mix1}
                  </span>
                  <span className="text-[11px] text-muted-foreground block">
                    {campusMixStats.total > 0
                      ? `${Math.round((campusMixStats.mix1 / campusMixStats.total) * 100)}%`
                      : '0%'}
                  </span>
                </div>

                <div className="p-4 rounded-lg border border-border bg-muted/20 space-y-1 text-center">
                  <span className="text-xs text-muted-foreground block">
                    {t('statistics.campusMix2')}
                  </span>
                  <span className="text-2xl font-bold text-foreground">
                    {campusMixStats.mix2}
                  </span>
                  <span className="text-[11px] text-muted-foreground block">
                    {campusMixStats.total > 0
                      ? `${Math.round((campusMixStats.mix2 / campusMixStats.total) * 100)}%`
                      : '0%'}
                  </span>
                </div>

                <div className="p-4 rounded-lg border border-border bg-emerald-500/10 border-emerald-500/30 space-y-1 text-center">
                  <span className="text-xs text-emerald-700 dark:text-emerald-300 block font-medium">
                    {t('statistics.campusMix3')}
                  </span>
                  <span className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
                    {campusMixStats.mix3}
                  </span>
                  <span className="text-[11px] text-emerald-600/80 block">
                    {campusMixStats.total > 0
                      ? `${Math.round((campusMixStats.mix3 / campusMixStats.total) * 100)}%`
                      : '0%'}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Co-working & Review Relation Heatmaps */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Co-Working Heatmap */}
            <Card className="bg-card border-border shadow-sm">
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-semibold flex items-center gap-2">
                  <Users className="h-4 w-4 text-primary" />
                  <span>{t('statistics.coWorkingTitle')}</span>
                </CardTitle>
                <CardDescription className="text-xs">
                  {t('statistics.coWorkingDesc')}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="border-collapse text-[10px]">
                    <thead>
                      <tr>
                        <th className="p-1"></th>
                        {teachers.slice(0, 11).map((tW) => (
                          <th key={tW.teacher.id} className="p-1 font-normal text-muted-foreground w-6 text-center truncate">
                            {tW.teacher.id}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {teachers.slice(0, 11).map((tA) => (
                        <tr key={tA.teacher.id}>
                          <td className="p-1 font-medium text-muted-foreground text-right pr-2 truncate max-w-[80px]">
                            {tA.teacher.full_name}
                          </td>
                          {teachers.slice(0, 11).map((tB) => {
                            if (tA.teacher.id === tB.teacher.id) {
                              return <td key={tB.teacher.id} className="p-1 bg-muted/40 text-center">-</td>
                            }
                            const count = coWorkingMatrix?.get(`${tA.teacher.id}_${tB.teacher.id}`) || 0
                            return (
                              <td
                                key={tB.teacher.id}
                                className={`p-1 text-center font-mono ${
                                  count > 2
                                    ? 'bg-primary text-primary-foreground font-bold'
                                    : count > 0
                                    ? 'bg-primary/30 text-foreground'
                                    : 'bg-card text-muted-foreground/40'
                                }`}
                              >
                                {count}
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

            {/* Review-relation Heatmap */}
            <Card className="bg-card border-border shadow-sm">
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-semibold flex items-center gap-2">
                  <Layers className="h-4 w-4 text-primary" />
                  <span>{t('statistics.reviewRelationTitle')}</span>
                </CardTitle>
                <CardDescription className="text-xs">
                  {t('statistics.reviewRelationDesc')}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="border-collapse text-[10px]">
                    <thead>
                      <tr>
                        <th className="p-1"></th>
                        {teachers.slice(0, 11).map((tW) => (
                          <th key={tW.teacher.id} className="p-1 font-normal text-muted-foreground w-6 text-center truncate">
                            {tW.teacher.id}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {teachers.slice(0, 11).map((tA) => (
                        <tr key={tA.teacher.id}>
                          <td className="p-1 font-medium text-muted-foreground text-right pr-2 truncate max-w-[80px]">
                            {tA.teacher.full_name}
                          </td>
                          {teachers.slice(0, 11).map((tB) => {
                            if (tA.teacher.id === tB.teacher.id) {
                              return <td key={tB.teacher.id} className="p-1 bg-muted/40 text-center">-</td>
                            }
                            const count = reviewRelationMatrix?.get(`${tA.teacher.id}_${tB.teacher.id}`) || 0
                            return (
                              <td
                                key={tB.teacher.id}
                                className={`p-1 text-center font-mono ${
                                  count > 1
                                    ? 'bg-amber-500 text-white font-bold'
                                    : count > 0
                                    ? 'bg-amber-500/30 text-foreground'
                                    : 'bg-card text-muted-foreground/40'
                                }`}
                              >
                                {count}
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
          </div>
        </div>
      )}
    </div>
  )
}
