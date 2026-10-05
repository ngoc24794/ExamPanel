import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Award,
  Search,
  Filter,
  Sparkles,
  RotateCcw,
  CheckCircle2,
  BookOpen,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { CampusChip } from '@/components/CampusChip'
import {
  useSchoolYears,
  useCampuses,
  useSubjects,
  useTeachers,
  useCompetencies,
  useSetCompetency,
  useDeleteCompetency,
  useReplaceTeacherCompetencies,
} from '@/lib/query/hooks'
import { toast } from 'sonner'
import type { GradeScope, Role, Competency } from '@/lib/api'

export const CompetenciesPage: React.FC = () => {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const schoolYearId = currentYear?.id

  const { data: campuses = [] } = useCampuses()
  const { data: subjects = [] } = useSubjects(schoolYearId)
  const { data: teachersWithGrades = [], isLoading } = useTeachers(schoolYearId)
  const { data: competencies = [] } = useCompetencies(schoolYearId)

  const setCompMutation = useSetCompetency(schoolYearId ?? 0)
  const deleteCompMutation = useDeleteCompetency(schoolYearId ?? 0)
  const replaceMutation = useReplaceTeacherCompetencies(schoolYearId ?? 0)

  // Filters
  const [searchTerm, setSearchTerm] = React.useState('')
  const [campusFilter, setCampusFilter] = React.useState('all')
  const [subjectFilter, setSubjectFilter] = React.useState<string>('all')

  const campusMap = React.useMemo(() => {
    const map = new Map<number, (typeof campuses)[0]>()
    for (const c of campuses) {
      map.set(c.id, c)
    }
    return map
  }, [campuses])

  // Map of (teacherId_subjectId_role) -> GradeScope
  const compMap = React.useMemo(() => {
    const map = new Map<string, GradeScope>()
    for (const c of competencies) {
      map.set(`${c.teacher_id}_${c.subject_id}_${c.role}`, c.grade_scope)
    }
    return map
  }, [competencies])

  // Filtered teachers
  const filteredTeachers = React.useMemo(() => {
    return teachersWithGrades.filter((tg) => {
      if (
        searchTerm.trim() &&
        !tg.teacher.full_name.toLowerCase().includes(searchTerm.toLowerCase().trim()) &&
        !tg.teacher.display_name
          ?.toLowerCase()
          .includes(searchTerm.toLowerCase().trim()) &&
        !tg.teacher.code?.toLowerCase().includes(searchTerm.toLowerCase().trim())
      ) {
        return false
      }
      if (campusFilter !== 'all' && tg.teacher.campus_id !== Number(campusFilter)) {
        return false
      }
      return true
    })
  }, [teachersWithGrades, searchTerm, campusFilter])

  // Filtered subjects
  const displayedSubjects = React.useMemo(() => {
    if (subjectFilter !== 'all') {
      const id = Number(subjectFilter)
      return subjects.filter((s) => s.id === id)
    }
    return subjects
  }, [subjects, subjectFilter])

  // Cycle toggle: off -> taught -> all -> off
  const handleCycleRole = async (teacherId: number, subjectId: number, role: Role) => {
    const currentScope = compMap.get(`${teacherId}_${subjectId}_${role}`)
    if (!currentScope) {
      // Off -> Taught
      await setCompMutation.mutateAsync({
        teacher_id: teacherId,
        subject_id: subjectId,
        role,
        grade_scope: 'taught',
      })
    } else if (currentScope === 'taught') {
      // Taught -> Any
      await setCompMutation.mutateAsync({
        teacher_id: teacherId,
        subject_id: subjectId,
        role,
        grade_scope: 'any',
      })
    } else {
      // Any -> Off
      await deleteCompMutation.mutateAsync({
        teacher_id: teacherId,
        subject_id: subjectId,
        role,
      })
    }
  }

  // Bulk actions
  const handleBulkAssignTaught = async () => {
    if (!schoolYearId) return
    try {
      for (const tg of teachersWithGrades) {
        const comps: Competency[] = []
        for (const sub of subjects) {
          comps.push({
            teacher_id: tg.teacher.id,
            subject_id: sub.id,
            role: 'setter',
            grade_scope: 'taught',
          })
          comps.push({
            teacher_id: tg.teacher.id,
            subject_id: sub.id,
            role: 'reviewer',
            grade_scope: 'taught',
          })
        }
        await replaceMutation.mutateAsync({
          school_year_id: schoolYearId,
          teacher_id: tg.teacher.id,
          competencies: comps,
        })
      }
      toast.success(t('competencies.bulkAssignSuccess'))
    } catch {
      toast.error(t('competencies.bulkError'))
    }
  }

  const handleBulkAssignAll = async () => {
    if (!schoolYearId) return
    try {
      for (const tg of teachersWithGrades) {
        const comps: Competency[] = []
        for (const sub of subjects) {
          comps.push({
            teacher_id: tg.teacher.id,
            subject_id: sub.id,
            role: 'setter',
            grade_scope: 'any',
          })
          comps.push({
            teacher_id: tg.teacher.id,
            subject_id: sub.id,
            role: 'reviewer',
            grade_scope: 'any',
          })
        }
        await replaceMutation.mutateAsync({
          school_year_id: schoolYearId,
          teacher_id: tg.teacher.id,
          competencies: comps,
        })
      }
      toast.success(t('competencies.bulkAssignAllSuccess'))
    } catch {
      toast.error(t('competencies.bulkError'))
    }
  }

  const handleBulkClear = async () => {
    if (!schoolYearId) return
    try {
      for (const tg of teachersWithGrades) {
        await replaceMutation.mutateAsync({
          school_year_id: schoolYearId,
          teacher_id: tg.teacher.id,
          competencies: [],
        })
      }
      toast.success(t('competencies.bulkClearSuccess'))
    } catch {
      toast.error(t('competencies.deleteError'))
    }
  }

  // Summary counts of eligible setters/reviewers per subject
  const eligibleStats = React.useMemo(() => {
    const stats: Record<number, { setters: number; reviewers: number }> = {}
    for (const sub of subjects) {
      let setters = 0
      let reviewers = 0
      for (const tg of teachersWithGrades) {
        if (!tg.teacher.active || tg.teacher.load_weight <= 0) continue
        const sScope = compMap.get(`${tg.teacher.id}_${sub.id}_setter`)
        if (sScope === 'any' || (sScope === 'taught' && tg.grade_ids.length > 0)) {
          setters++
        }
        const rScope = compMap.get(`${tg.teacher.id}_${sub.id}_reviewer`)
        if (rScope === 'any' || (rScope === 'taught' && tg.grade_ids.length > 0)) {
          reviewers++
        }
      }
      stats[sub.id] = { setters, reviewers }
    }
    return stats
  }, [subjects, teachersWithGrades, compMap])

  return (
    <div className="space-y-6" data-testid="competencies-page">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <Award className="h-6 w-6 text-primary" />
            {t('competencies.matrixTitle')}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t('competencies.matrixDesc')}
          </p>
        </div>

        {/* Bulk Action Buttons */}
        <div className="flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleBulkAssignTaught}
            className="h-8 text-xs gap-1.5"
            data-testid="bulk-assign-taught-btn"
          >
            <CheckCircle2 className="h-3.5 w-3.5 text-primary" />
            <span>{t('competencies.bulkAssignTaught')}</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleBulkAssignAll}
            className="h-8 text-xs gap-1.5"
            data-testid="bulk-assign-all-btn"
          >
            <Sparkles className="h-3.5 w-3.5 text-emerald-500" />
            <span>{t('competencies.bulkAssignAll')}</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleBulkClear}
            className="h-8 text-xs gap-1.5 text-destructive hover:bg-destructive/10"
            data-testid="bulk-clear-btn"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            <span>{t('competencies.bulkClear')}</span>
          </Button>
        </div>
      </div>

      {/* Filters Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder={t('competencies.searchTeacher')}
            className="pl-8 text-xs h-9 bg-card"
            data-testid="search-teacher-input"
          />
        </div>

        <div className="flex items-center gap-2">
          <Filter className="h-4 w-4 text-muted-foreground shrink-0" />
          <Select value={campusFilter} onValueChange={setCampusFilter}>
            <SelectTrigger
              className="w-[180px] text-xs h-9 bg-card"
              data-testid="filter-campus-select"
            >
              <SelectValue placeholder={t('competencies.filterCampus')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('teachers.allCampuses')}</SelectItem>
              {campuses.map((c) => (
                <SelectItem key={c.id} value={c.id.toString()}>
                  {c.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={subjectFilter} onValueChange={setSubjectFilter}>
            <SelectTrigger
              className="w-[180px] text-xs h-9 bg-card"
              data-testid="filter-subject-select"
            >
              <SelectValue placeholder={t('competencies.filterSubject')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('common.all')}</SelectItem>
              {subjects.map((s) => (
                <SelectItem key={s.id} value={s.id.toString()}>
                  {s.name} ({s.code})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Competencies Matrix Table */}
      <div className="rounded-lg border border-border bg-card shadow-sm overflow-x-auto">
        <Table>
          <TableHeader className="bg-muted/40">
            <TableRow>
              <TableHead className="w-[240px] font-semibold text-foreground">
                {t('teachers.fullName')}
              </TableHead>
              <TableHead className="w-[150px] font-semibold text-foreground">
                {t('teachers.campus')}
              </TableHead>
              {displayedSubjects.map((sub) => (
                <TableHead
                  key={sub.id}
                  className="text-center min-w-[200px] border-l border-border"
                >
                  <div className="font-semibold text-foreground">{sub.name}</div>
                  <div className="text-[11px] text-muted-foreground font-mono">
                    {sub.code} ({t('competencies.subjectComposition', { setters: sub.setters, reviewers: sub.reviewers })})
                  </div>
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>

          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell
                  colSpan={2 + displayedSubjects.length}
                  className="text-center p-8 text-muted-foreground"
                >
                  {t('app.loading')}
                </TableCell>
              </TableRow>
            ) : filteredTeachers.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={2 + displayedSubjects.length}
                  className="text-center p-8 text-muted-foreground"
                >
                  {t('teachers.empty')}
                </TableCell>
              </TableRow>
            ) : (
              filteredTeachers.map((tg) => {
                const campus = campusMap.get(tg.teacher.campus_id)
                return (
                  <TableRow
                    key={tg.teacher.id}
                    className={`hover:bg-muted/30 ${
                      !tg.teacher.active ? 'opacity-60 bg-muted/20' : ''
                    }`}
                  >
                    <TableCell className="font-medium">
                      <div className="flex items-center gap-1.5">
                        <span className="text-sm text-foreground font-semibold">
                          {tg.teacher.full_name}
                        </span>
                        {tg.teacher.display_name && (
                          <span className="text-xs text-muted-foreground font-normal">
                            ({tg.teacher.display_name})
                          </span>
                        )}
                        {tg.teacher.code && (
                          <Badge
                            variant="outline"
                            className="font-mono text-[10px] px-1 py-0 ml-1"
                          >
                            {tg.teacher.code}
                          </Badge>
                        )}
                      </div>
                    </TableCell>

                    <TableCell>
                      {campus && <CampusChip name={campus.name} color={campus.color} />}
                    </TableCell>

                    {/* Competency Toggles per Subject */}
                    {displayedSubjects.map((sub) => {
                      const setterScope = compMap.get(`${tg.teacher.id}_${sub.id}_setter`)
                      const reviewerScope = compMap.get(
                        `${tg.teacher.id}_${sub.id}_reviewer`,
                      )

                      return (
                        <TableCell
                          key={sub.id}
                          className="border-l border-border p-2.5 text-center"
                        >
                          <div className="flex items-center justify-center gap-2">
                            {/* Setter Toggle Chip */}
                            <button
                              type="button"
                              onClick={() =>
                                handleCycleRole(tg.teacher.id, sub.id, 'setter')
                              }
                              className={`px-2 py-1 rounded text-[11px] font-semibold transition-all border flex items-center gap-1 ${
                                setterScope === 'any'
                                  ? 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 border-emerald-500/40 shadow-xs'
                                  : setterScope === 'taught'
                                    ? 'bg-primary/20 text-primary border-primary/40 shadow-xs'
                                    : 'bg-muted/40 text-muted-foreground border-border hover:bg-muted'
                              }`}
                              title={`${t('common.roleSetter')}: ${
                                setterScope === 'any'
                                  ? t('competencies.stateAll')
                                  : setterScope === 'taught'
                                    ? t('competencies.stateTaught')
                                    : t('competencies.stateOff')
                              }`}
                              data-testid={`competency-toggle-${tg.teacher.id}-${sub.id}-setter`}
                            >
                              <span className="font-bold">{t('assignments.roleSetterShort')}:</span>
                              <span>
                                {setterScope === 'any'
                                  ? t('competencies.stateAll')
                                  : setterScope === 'taught'
                                    ? t('competencies.stateTaught')
                                    : t('competencies.stateOff')}
                              </span>
                            </button>

                            {/* Reviewer Toggle Chip */}
                            <button
                              type="button"
                              onClick={() =>
                                handleCycleRole(tg.teacher.id, sub.id, 'reviewer')
                              }
                              className={`px-2 py-1 rounded text-[11px] font-semibold transition-all border flex items-center gap-1 ${
                                reviewerScope === 'any'
                                  ? 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 border-emerald-500/40 shadow-xs'
                                  : reviewerScope === 'taught'
                                    ? 'bg-primary/20 text-primary border-primary/40 shadow-xs'
                                    : 'bg-muted/40 text-muted-foreground border-border hover:bg-muted'
                              }`}
                              title={`${t('common.roleReviewer')}: ${
                                reviewerScope === 'any'
                                  ? t('competencies.stateAll')
                                  : reviewerScope === 'taught'
                                    ? t('competencies.stateTaught')
                                    : t('competencies.stateOff')
                              }`}
                              data-testid={`competency-toggle-${tg.teacher.id}-${sub.id}-reviewer`}
                            >
                              <span className="font-bold">{t('assignments.roleReviewerShort')}:</span>
                              <span>
                                {reviewerScope === 'any'
                                  ? t('competencies.stateAll')
                                  : reviewerScope === 'taught'
                                    ? t('competencies.stateTaught')
                                    : t('competencies.stateOff')}
                              </span>
                            </button>
                          </div>
                        </TableCell>
                      )
                    })}
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
      </div>

      {/* Eligible-Count Footer */}
      <div
        className="p-3.5 rounded-lg border border-border bg-card shadow-xs flex flex-wrap items-center justify-between gap-4"
        data-testid="eligible-count-footer"
      >
        <div className="flex items-center gap-2">
          <BookOpen className="h-4 w-4 text-primary" />
          <span className="text-xs font-semibold text-foreground">
            {t('competencies.eligibleFooter')}:
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-4 text-xs">
          {subjects.map((sub) => {
            const stat = eligibleStats[sub.id] || { setters: 0, reviewers: 0 }
            return (
              <div
                key={sub.id}
                className="flex items-center gap-2 bg-muted/30 px-2.5 py-1 rounded border border-border/60"
              >
                <span className="font-semibold text-foreground">{sub.name}:</span>
                <span className="text-muted-foreground">
                  {stat.setters} {t('competencies.roleSetter').toLowerCase()}
                </span>
                <span className="text-border">|</span>
                <span className="text-muted-foreground">
                  {stat.reviewers} {t('competencies.roleReviewer').toLowerCase()}
                </span>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
