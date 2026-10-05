import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  CalendarX,
  Check,
  CheckCheck,
  Filter,
  MessageSquare,
  Search,
  Users,
} from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { CampusChip } from '@/components/CampusChip'
import {
  useCampuses,
  useExams,
  useGrades,
  useSchoolYears,
  useTeachers,
  useUnavailabilities,
  useSetUnavailability,
  useDeleteUnavailability,
  useFeasibility,
} from '@/lib/query/hooks'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'
import type { Exam, TeacherWithGrades, Unavailability } from '@/lib/api'

export const UnavailabilityPage: React.FC = () => {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]

  const { data: campuses = [] } = useCampuses()
  const { data: grades = [] } = useGrades()
  const { data: exams = [] } = useExams(currentYear?.id)
  const { data: teachersWithGrades = [] } = useTeachers(currentYear?.id)
  const { data: unavailabilities = [] } = useUnavailabilities(currentYear?.id)
  const { data: feasibility } = useFeasibility(currentYear?.id)

  const setUnavailMutation = useSetUnavailability(currentYear?.id ?? 0)
  const deleteUnavailMutation = useDeleteUnavailability(currentYear?.id ?? 0)

  // Filters
  const [search, setSearch] = React.useState('')
  const [selectedCampusId, setSelectedCampusId] = React.useState<string>('all')
  const [selectedGradeId, setSelectedGradeId] = React.useState<string>('all')

  // Reason Dialog
  const [reasonDialogOpen, setReasonDialogOpen] = React.useState(false)
  const [activeCell, setActiveCell] = React.useState<{
    teacher: TeacherWithGrades['teacher']
    exam: Exam
    currentReason?: string
  } | null>(null)
  const [reasonInput, setReasonInput] = React.useState('')

  const sortedExams = [...exams].sort((a, b) => a.sort_order - b.sort_order)
  const sortedGrades = [...grades].sort((a, b) => a.sort_order - b.sort_order)

  // Semester exam splits by sort order
  const midPoint = Math.ceil(sortedExams.length / 2)
  const semester1Exams = sortedExams.slice(0, midPoint)
  const semester2Exams = sortedExams.slice(midPoint)

  const activeTeachers = teachersWithGrades.filter(
    (twg) => twg.teacher.active && twg.teacher.load_weight > 0,
  )

  const filteredTeachers = activeTeachers.filter(({ teacher, grade_ids }) => {
    if (search && !teacher.full_name.toLowerCase().includes(search.toLowerCase())) {
      return false
    }
    if (selectedCampusId !== 'all' && teacher.campus_id !== Number(selectedCampusId)) {
      return false
    }
    if (selectedGradeId !== 'all' && !grade_ids.includes(Number(selectedGradeId))) {
      return false
    }
    return true
  })

  // Quick lookup map: "teacherId_examId" -> Unavailability
  const unavailMap = React.useMemo(() => {
    const map = new Map<string, Unavailability>()
    for (const u of unavailabilities) {
      map.set(`${u.teacher_id}_${u.exam_id}`, u)
    }
    return map
  }, [unavailabilities])

  // Feasibility diagnostic lookup for tight/insufficient pools
  const diagnosticPoolMap = React.useMemo(() => {
    const map = new Map<string, 'tight' | 'insufficient'>()
    const report = feasibility?.report
    if (!report) return map

    for (const err of report.errors) {
      if (
        err.panel &&
        (err.code.includes('insufficient_') || err.code.includes('unfillable'))
      ) {
        map.set(`${err.panel.exam_id}_${err.panel.grade_id}`, 'insufficient')
      }
    }
    for (const warn of report.warnings) {
      if (warn.panel && warn.code.includes('tight_')) {
        map.set(`${warn.panel.exam_id}_${warn.panel.grade_id}`, 'tight')
      }
    }
    return map
  }, [feasibility])

  const handleToggleCell = async (teacherId: number, examId: number) => {
    const key = `${teacherId}_${examId}`
    const existing = unavailMap.get(key)
    try {
      if (existing) {
        await deleteUnavailMutation.mutateAsync({ teacherId, examId })
      } else {
        await setUnavailMutation.mutateAsync({
          teacher_id: teacherId,
          exam_id: examId,
          reason: null,
        })
      }
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  const handleOpenReasonDialog = (
    e: React.MouseEvent,
    teacher: TeacherWithGrades['teacher'],
    exam: Exam,
  ) => {
    e.stopPropagation()
    const existing = unavailMap.get(`${teacher.id}_${exam.id}`)
    setActiveCell({
      teacher,
      exam,
      currentReason: existing?.reason || '',
    })
    setReasonInput(existing?.reason || '')
    setReasonDialogOpen(true)
  }

  const handleSaveReason = async () => {
    if (!activeCell) return
    try {
      await setUnavailMutation.mutateAsync({
        teacher_id: activeCell.teacher.id,
        exam_id: activeCell.exam.id,
        reason: reasonInput.trim() || null,
      })
      setReasonDialogOpen(false)
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  // Bulk Row Actions
  const handleBulkAbsentSemester = async (teacherId: number, examsToMark: Exam[]) => {
    try {
      for (const exam of examsToMark) {
        if (!unavailMap.has(`${teacherId}_${exam.id}`)) {
          await setUnavailMutation.mutateAsync({
            teacher_id: teacherId,
            exam_id: exam.id,
            reason: null,
          })
        }
      }
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  const handleBulkPresentAll = async (teacherId: number) => {
    try {
      for (const exam of sortedExams) {
        if (unavailMap.has(`${teacherId}_${exam.id}`)) {
          await deleteUnavailMutation.mutateAsync({ teacherId, examId: exam.id })
        }
      }
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  return (
    <div className="space-y-6" data-testid="unavailability-page">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {t('unavailability.title')}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          {t('unavailability.description')}
        </p>
      </div>

      {/* Filter Toolbar */}
      <Card className="bg-card border-border">
        <CardContent className="p-4 flex flex-wrap items-center gap-3">
          <div className="relative flex-1 min-w-[200px]">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder={t('teachers.searchPlaceholder')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8 text-xs bg-background"
              data-testid="search-unavailability-input"
            />
          </div>

          <Select value={selectedCampusId} onValueChange={setSelectedCampusId}>
            <SelectTrigger className="w-[180px] text-xs bg-background">
              <Filter className="h-3.5 w-3.5 mr-1.5 text-muted-foreground" />
              <SelectValue placeholder={t('teachers.filterCampus')} />
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

          <Select value={selectedGradeId} onValueChange={setSelectedGradeId}>
            <SelectTrigger className="w-[150px] text-xs bg-background">
              <Filter className="h-3.5 w-3.5 mr-1.5 text-muted-foreground" />
              <SelectValue placeholder={t('teachers.filterGrade')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('teachers.allGrades')}</SelectItem>
              {sortedGrades.map((g) => (
                <SelectItem key={g.id} value={g.id.toString()}>
                  {g.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      {/* Grid Table */}
      <Card className="bg-card border-border overflow-hidden">
        <div className="overflow-x-auto">
          <table
            className="w-full border-collapse text-left text-xs"
            data-testid="unavailability-grid"
          >
            <thead>
              <tr className="border-b border-border bg-muted/40 font-semibold text-muted-foreground">
                <th className="p-3 w-56">{t('teachers.fullName')}</th>
                <th className="p-3 w-32">{t('teachers.campus')}</th>
                <th className="p-3 w-28 text-center">{t('teachers.actions')}</th>
                {sortedExams.map((exam) => (
                  <th
                    key={exam.id}
                    className="p-3 text-center border-l border-border min-w-[120px]"
                    data-testid={`exam-col-${exam.code}`}
                  >
                    <div className="font-bold text-foreground">{exam.code}</div>
                    <div className="text-[11px] font-normal text-muted-foreground truncate">
                      {exam.name}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filteredTeachers.map(({ teacher }) => {
                const campus = campuses.find((c) => c.id === teacher.campus_id)
                return (
                  <tr
                    key={teacher.id}
                    className="hover:bg-muted/20 transition-colors"
                    data-testid={`teacher-row-${teacher.id}`}
                  >
                    <td className="p-3 font-medium text-foreground">
                      {teacher.full_name}
                    </td>
                    <td className="p-3">
                      {campus && <CampusChip name={campus.name} color={campus.color} />}
                    </td>
                    <td className="p-3 text-center">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-7 text-[11px] text-muted-foreground hover:text-foreground px-2"
                            data-testid={`bulk-actions-${teacher.id}`}
                          >
                            <span>{t('common.actions')}</span>
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="text-xs">
                          <DropdownMenuItem
                            onClick={() =>
                              handleBulkAbsentSemester(teacher.id, semester1Exams)
                            }
                            data-testid={`bulk-absent-sem1-${teacher.id}`}
                          >
                            <CalendarX className="h-3.5 w-3.5 mr-2 text-destructive" />
                            <span>{t('unavailability.absentSemester1')}</span>
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() =>
                              handleBulkAbsentSemester(teacher.id, semester2Exams)
                            }
                            data-testid={`bulk-absent-sem2-${teacher.id}`}
                          >
                            <CalendarX className="h-3.5 w-3.5 mr-2 text-destructive" />
                            <span>{t('unavailability.absentSemester2')}</span>
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() => handleBulkPresentAll(teacher.id)}
                            data-testid={`bulk-present-all-${teacher.id}`}
                          >
                            <CheckCheck className="h-3.5 w-3.5 mr-2 text-emerald-500" />
                            <span>{t('unavailability.presentAll')}</span>
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </td>

                    {sortedExams.map((exam) => {
                      const unavail = unavailMap.get(`${teacher.id}_${exam.id}`)
                      const isAbsent = !!unavail

                      return (
                        <td
                          key={exam.id}
                          className="p-2 border-l border-border text-center select-none"
                        >
                          <div className="flex items-center justify-center gap-1">
                            <button
                              type="button"
                              onClick={() => handleToggleCell(teacher.id, exam.id)}
                              className={`h-8 w-20 rounded-md font-medium text-xs flex items-center justify-center gap-1 transition-all focus:outline-none focus:ring-2 focus:ring-ring ${
                                isAbsent
                                  ? 'bg-destructive/15 text-destructive border border-destructive/30 font-semibold'
                                  : 'bg-muted/40 text-muted-foreground hover:bg-muted hover:text-foreground'
                              }`}
                              aria-label={`${teacher.full_name} ${exam.code} ${
                                isAbsent
                                  ? t('unavailability.absent')
                                  : t('unavailability.present')
                              }`}
                              data-testid={`unavail-cell-${teacher.id}-${exam.id}`}
                            >
                              {isAbsent ? (
                                <span>{t('unavailability.absent')}</span>
                              ) : (
                                <Check className="h-3.5 w-3.5 text-muted-foreground/50" />
                              )}
                            </button>

                            {isAbsent && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-7 w-7 text-muted-foreground hover:text-foreground"
                                onClick={(e) => handleOpenReasonDialog(e, teacher, exam)}
                                title={unavail.reason || t('unavailability.reason')}
                                data-testid={`reason-btn-${teacher.id}-${exam.id}`}
                              >
                                <MessageSquare
                                  className={`h-3 w-3 ${
                                    unavail.reason ? 'text-primary fill-primary/20' : ''
                                  }`}
                                />
                              </Button>
                            )}
                          </div>
                        </td>
                      )
                    })}
                  </tr>
                )
              })}
            </tbody>

            {/* Footer Summary per Exam x Grade */}
            <tfoot>
              <tr className="border-t-2 border-border bg-muted/50 font-semibold">
                <td colSpan={3} className="p-3 text-foreground font-semibold">
                  <div className="flex items-center gap-2">
                    <Users className="h-4 w-4 text-primary" />
                    <span>{t('unavailability.totalAvailable')}</span>
                  </div>
                </td>
                {sortedExams.map((exam) => (
                  <td
                    key={exam.id}
                    className="p-3 border-l border-border align-top text-xs space-y-1"
                    data-testid={`footer-exam-${exam.code}`}
                  >
                    {sortedGrades.map((grade) => {
                      const qualified = activeTeachers.filter((twg) =>
                        twg.grade_ids.includes(grade.id),
                      )
                      const availableCount = qualified.filter(
                        (twg) => !unavailMap.has(`${twg.teacher.id}_${exam.id}`),
                      ).length

                      const poolStatus = diagnosticPoolMap.get(`${exam.id}_${grade.id}`)
                      const isInsufficient =
                        availableCount < 3 || poolStatus === 'insufficient'
                      const isTight =
                        !isInsufficient &&
                        (availableCount === 3 || poolStatus === 'tight')

                      return (
                        <div
                          key={grade.id}
                          className={`flex items-center justify-between px-2 py-0.5 rounded text-[11px] ${
                            isInsufficient
                              ? 'bg-destructive/15 text-destructive font-bold'
                              : isTight
                                ? 'bg-amber-500/15 text-amber-600 dark:text-amber-400 font-semibold'
                                : 'text-muted-foreground'
                          }`}
                          title={
                            isInsufficient
                              ? t('unavailability.insufficientPool')
                              : isTight
                                ? t('unavailability.tightPool')
                                : undefined
                          }
                          data-testid={`pool-count-${exam.id}-${grade.id}`}
                        >
                          <span>{grade.name}:</span>
                          <span>
                            {t('unavailability.availableTeachersCount', {
                              count: availableCount,
                            })}
                          </span>
                        </div>
                      )
                    })}
                  </td>
                ))}
              </tr>
            </tfoot>
          </table>
        </div>
      </Card>

      {/* Reason Dialog */}
      <Dialog open={reasonDialogOpen} onOpenChange={setReasonDialogOpen}>
        <DialogContent className="sm:max-w-md bg-card text-foreground border-border">
          <DialogHeader>
            <DialogTitle>{t('unavailability.reason')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 pt-2">
            <div className="text-xs text-muted-foreground">
              {activeCell?.teacher.full_name} — {activeCell?.exam.name}
            </div>
            <Input
              value={reasonInput}
              onChange={(e) => setReasonInput(e.target.value)}
              placeholder={t('unavailability.reasonPlaceholder')}
              data-testid="unavail-reason-input"
            />
            <DialogFooter className="pt-2">
              <Button variant="outline" onClick={() => setReasonDialogOpen(false)}>
                {t('common.cancel')}
              </Button>
              <Button onClick={handleSaveReason} data-testid="save-reason-btn">
                {t('common.save')}
              </Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
