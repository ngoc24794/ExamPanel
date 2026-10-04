import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type Assignment,
  type Campus,
  type Exam,
  type Grade,
  type Lock,
  type PlanDetails,
  type PlanStatus,
  type Role,
  type SlotRef,
  type Subject,
  type TeacherWithGrades,
} from '@/lib/api'
import { useProblemDetails } from '@/lib/query/hooks'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Lock as LockIcon,
  Pin,
  Ban,
  BookmarkCheck,
  RefreshCw,
  ArrowRightLeft,
  MoreVertical,
  CheckCircle2,
  AlertOctagon,
  AlertTriangle,
  ArrowUpDown,
  Search,
} from 'lucide-react'
import { CandidateSelectModal } from './CandidateSelectModal'

export interface QPlanGridProps {
  planDetails: PlanDetails
  planStatus: PlanStatus | null
  exams: Exam[]
  grades: Grade[]
  subjects: Subject[]
  teachers: TeacherWithGrades[]
  campuses: Campus[]
  locks: Lock[]
  isEditable: boolean
  focusedTeacherId: number | null
  keptSlots: SlotRef[]
  onSelectTeacherFocus: (id: number | null) => void
  onToggleKeepSlot: (slot: SlotRef) => void
  onReoptimizeRemaining: () => void
  onUpdateAssignments: (assignments: Assignment[]) => void
  onCreateLock: (slot: SlotRef, teacherId: number, kind: 'pin' | 'forbid') => void
}

export const QPlanGrid: React.FC<QPlanGridProps> = ({
  planDetails,
  planStatus,
  exams,
  grades,
  subjects,
  teachers,
  campuses,
  locks,
  isEditable,
  focusedTeacherId,
  keptSlots,
  onSelectTeacherFocus,
  onToggleKeepSlot,
  onReoptimizeRemaining,
  onUpdateAssignments,
  onCreateLock,
}) => {
  const { t } = useTranslation()
  const schoolYearId = exams[0]?.school_year_id ?? 1
  const { data: problemDetails } = useProblemDetails(schoolYearId)

  const [hoveredTeacherId, setHoveredTeacherId] = React.useState<number | null>(null)
  const [totalsSearch, setTotalsSearch] = React.useState('')
  const [totalsSortField, setTotalsSortField] = React.useState<'name' | 'total'>('total')
  const [totalsSortAsc, setTotalsSortAsc] = React.useState(false)

  // Candidate replacement modal state
  const [selectedSlotForReplace, setSelectedSlotForReplace] = React.useState<{
    slot: SlotRef
    currentTeacherId: number | null
  } | null>(null)

  // Drag and drop state
  const [draggedSlot, setDraggedSlot] = React.useState<{
    slot: SlotRef
    teacherId: number
  } | null>(null)

  const assignments = planDetails.assignments
  const scoreReport = planDetails.score_report

  // Forced placements lookup
  const forcedPlacements = React.useMemo(() => {
    return problemDetails?.forced ?? []
  }, [problemDetails])

  const forcedTeacherIds = React.useMemo(() => {
    return new Set(forcedPlacements.map((fp) => fp.teacher_id))
  }, [forcedPlacements])

  const isSlotForced = React.useCallback(
    (examId: number, gradeId: number, subjectId: number, role: Role, position: number) => {
      return forcedPlacements.some(
        (fp) =>
          fp.panel.exam_id === examId &&
          fp.panel.grade_id === gradeId &&
          fp.panel.subject_id === subjectId &&
          fp.role === role &&
          fp.position === position,
      )
    },
    [forcedPlacements],
  )

  const isSlotLocked = React.useCallback(
    (examId: number, gradeId: number, subjectId: number, teacherId: number) => {
      return locks.some(
        (l) =>
          l.exam_id === examId &&
          l.grade_id === gradeId &&
          (l.subject_id === subjectId || (!l.subject_id && subjectId === 1)) &&
          l.teacher_id === teacherId &&
          l.kind === 'pin',
      )
    },
    [locks],
  )

  const isSlotKept = React.useCallback(
    (examId: number, gradeId: number, subjectId: number, role: Role, position: number) => {
      return keptSlots.some(
        (s) =>
          s.exam_id === examId &&
          s.grade_id === gradeId &&
          s.subject_id === subjectId &&
          s.role === role &&
          s.position === position,
      )
    },
    [keptSlots],
  )

  // Effective subjects
  const defaultSubject: Subject = {
    id: 1,
    code: 'VL',
    name: 'Vật lí',
    color: 'palette-1',
    sort_order: 1,
    setters: 2,
    reviewers: 1,
    min_campuses: 2,
  }
  const effectiveSubjects = subjects.length > 0 ? subjects : [defaultSubject]
  const maxSetters = Math.max(...effectiveSubjects.map((s) => s.setters || 1), 1)
  const maxReviewers = Math.max(...effectiveSubjects.map((s) => s.reviewers || 1), 1)

  const teacherMap = React.useMemo(() => {
    const map = new Map<number, (typeof teachers)[0]['teacher']>()
    for (const twg of teachers) {
      map.set(twg.teacher.id, twg.teacher)
    }
    return map
  }, [teachers])

  // Teacher statistics across assignments
  const teacherStats = React.useMemo(() => {
    const stats = new Map<
      number,
      {
        total: number
        setters: number
        reviewers: number
        perExam: Map<number, number>
        examNames: Set<string>
      }
    >()

    for (const twg of teachers) {
      stats.set(twg.teacher.id, {
        total: 0,
        setters: 0,
        reviewers: 0,
        perExam: new Map(),
        examNames: new Set(),
      })
    }

    for (const a of assignments) {
      let st = stats.get(a.teacher_id)
      if (!st) {
        st = {
          total: 0,
          setters: 0,
          reviewers: 0,
          perExam: new Map(),
          examNames: new Set(),
        }
        stats.set(a.teacher_id, st)
      }
      st.total += 1
      if (a.role === 'setter') st.setters += 1
      else st.reviewers += 1

      const countInExam = st.perExam.get(a.exam_id) ?? 0
      st.perExam.set(a.exam_id, countInExam + 1)

      const ex = exams.find((e) => e.id === a.exam_id)
      if (ex) st.examNames.add(ex.name)
    }

    return stats
  }, [teachers, assignments, exams])

  // Quota map
  const teacherQuotas = React.useMemo(() => {
    const map = new Map<number, number>()
    if (scoreReport?.per_teacher) {
      for (const pt of scoreReport.per_teacher) {
        map.set(pt.teacher_id, pt.quota)
      }
    }
    return map
  }, [scoreReport])

  // Replacement & swap actions
  const handleApplyReplacement = (slot: SlotRef, newTeacherId: number) => {
    let replaced = false
    const nextAssignments = assignments.map((a) => {
      if (
        a.exam_id === slot.exam_id &&
        a.grade_id === slot.grade_id &&
        a.subject_id === slot.subject_id &&
        a.role === slot.role &&
        a.position === slot.position
      ) {
        replaced = true
        return { ...a, teacher_id: newTeacherId }
      }
      return a
    })

    if (!replaced) {
      nextAssignments.push({
        plan_id: planDetails.plan.id,
        exam_id: slot.exam_id,
        grade_id: slot.grade_id,
        subject_id: slot.subject_id,
        teacher_id: newTeacherId,
        role: slot.role,
        position: slot.position,
      })
    }

    onUpdateAssignments(nextAssignments)
  }

  const handleSwapSlots = (slotA: SlotRef, slotB: SlotRef) => {
    const aItem = assignments.find(
      (a) =>
        a.exam_id === slotA.exam_id &&
        a.grade_id === slotA.grade_id &&
        a.subject_id === slotA.subject_id &&
        a.role === slotA.role &&
        a.position === slotA.position,
    )
    const bItem = assignments.find(
      (a) =>
        a.exam_id === slotB.exam_id &&
        a.grade_id === slotB.grade_id &&
        a.subject_id === slotB.subject_id &&
        a.role === slotB.role &&
        a.position === slotB.position,
    )

    if (!aItem || !bItem) return
    const tA = aItem.teacher_id
    const tB = bItem.teacher_id

    const nextAssignments = assignments.map((a) => {
      if (
        a.exam_id === slotA.exam_id &&
        a.grade_id === slotA.grade_id &&
        a.subject_id === slotA.subject_id &&
        a.role === slotA.role &&
        a.position === slotA.position
      ) {
        return { ...a, teacher_id: tB }
      }
      if (
        a.exam_id === slotB.exam_id &&
        a.grade_id === slotB.grade_id &&
        a.subject_id === slotB.subject_id &&
        a.role === slotB.role &&
        a.position === slotB.position
      ) {
        return { ...a, teacher_id: tA }
      }
      return a
    })

    onUpdateAssignments(nextAssignments)
  }

  // Filtered and sorted teachers for totals panel
  const displayedTeachers = React.useMemo(() => {
    return teachers
      .filter((twg) => {
        if (!totalsSearch.trim()) return true
        const q = totalsSearch.toLowerCase().trim()
        const fn = twg.teacher.full_name.toLowerCase()
        const dn = (twg.teacher.display_name || '').toLowerCase()
        return fn.includes(q) || dn.includes(q)
      })
      .sort((a, b) => {
        if (totalsSortField === 'name') {
          const nA = a.teacher.display_name || a.teacher.full_name
          const nB = b.teacher.display_name || b.teacher.full_name
          return totalsSortAsc ? nA.localeCompare(nB, 'vi') : nB.localeCompare(nA, 'vi')
        } else {
          const tA = teacherStats.get(a.teacher.id)?.total ?? 0
          const tB = teacherStats.get(b.teacher.id)?.total ?? 0
          return totalsSortAsc ? tA - tB : tB - tA
        }
      })
  }, [teachers, totalsSearch, totalsSortField, totalsSortAsc, teacherStats])

  // Summary lower bound lookup
  const ruleBreakdown = React.useMemo(() => {
    if (!scoreReport) return []
    return scoreReport.by_rule || []
  }, [scoreReport])

  const activeHighlightedTeacherId = focusedTeacherId ?? hoveredTeacherId

  return (
    <div className="space-y-4" data-testid="q-plan-grid-root">
      {/* Summary KPI Bar */}
      <div
        className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-lg border border-border bg-card shadow-sm text-xs"
        data-testid="q-grid-summary-bar"
      >
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex items-center gap-1.5 font-bold text-sm text-foreground">
            <span>{t('assignments.summaryTotalScore', { score: scoreReport ? scoreReport.total.toFixed(2) : '0' })}</span>
          </div>

          <div className="flex items-center gap-1.5">
            {(!planStatus || planStatus.hard_violations_now.length === 0) ? (
              <span className="flex items-center gap-1 text-emerald-600 dark:text-emerald-400 font-medium">
                <CheckCircle2 className="h-4 w-4" />
                {t('assignments.hardValid')}
              </span>
            ) : (
              <span className="flex items-center gap-1 text-rose-600 dark:text-rose-400 font-medium">
                <AlertOctagon className="h-4 w-4" />
                {t('assignments.hardInvalid', { count: planStatus.hard_violations_now.length })}
              </span>
            )}
          </div>
        </div>

        {/* Soft rules penalty breakdown with lower bounds */}
        <div className="flex flex-wrap items-center gap-2">
          {ruleBreakdown.map((rb) => {
            const isAtLowerBound =
              rb.lower_bound !== undefined &&
              rb.lower_bound !== null &&
              Math.abs(rb.units - rb.lower_bound) < 1e-4

            return (
              <TooltipProvider key={rb.rule}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span
                      className={`px-2 py-0.5 rounded border text-[11px] font-mono cursor-help flex items-center gap-1 ${
                        rb.units === 0
                          ? 'border-border text-muted-foreground bg-muted/20'
                          : isAtLowerBound
                          ? 'border-emerald-500/30 text-emerald-600 dark:text-emerald-400 bg-emerald-500/10'
                          : 'border-amber-500/30 text-amber-600 dark:text-amber-400 bg-amber-500/10'
                      }`}
                    >
                      <span className="font-bold uppercase">{rb.rule}</span>: {rb.units}
                      {isAtLowerBound && rb.units > 0 && (
                        <span className="text-[9px] font-sans">({t('assignments.atLowerBound')})</span>
                      )}
                    </span>
                  </TooltipTrigger>
                  <TooltipContent className="text-xs space-y-1">
                    <p className="font-semibold uppercase">{rb.rule.toUpperCase()}</p>
                    <p>Số đơn vị phạt: {rb.units}</p>
                    <p>Trọng số: {rb.weight}</p>
                    <p>Điểm phạt: {rb.penalty.toFixed(2)}</p>
                    {rb.lower_bound !== undefined && (
                      <p className="font-medium text-emerald-600 dark:text-emerald-400">
                        {t('assignments.lowerBoundLabel', { bound: rb.lower_bound })}
                      </p>
                    )}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )
          })}
        </div>
      </div>

      {/* Main Grid + Attached Totals Panel Layout */}
      <div className="flex flex-col xl:flex-row items-start gap-4 w-full">
        {/* Left: Q-Style Assignment Grid */}
        <div className="flex-1 w-full overflow-x-auto rounded-lg border border-border bg-card shadow-sm">
          <table
            className="w-full border-collapse text-xs select-none"
            data-testid="q-plan-grid-table"
          >
            <thead>
              {/* Row 1: Corner header + Grade Group Headers */}
              <tr className="bg-muted/60 border-b border-border text-foreground font-semibold text-center">
                <th
                  rowSpan={2}
                  colSpan={2}
                  className="p-2 border-r border-border text-center font-bold text-xs uppercase bg-muted/80 w-36"
                >
                  {t('assignments.examGradeHeader') || 'Kì thi/khối'}
                </th>
                {grades.map((grade) => (
                  <th
                    key={grade.id}
                    colSpan={effectiveSubjects.length}
                    className="p-2 border-r border-border text-center font-bold text-xs bg-muted/50"
                  >
                    Khối {grade.name}
                  </th>
                ))}
              </tr>

              {/* Row 2: Subject sub-columns under each Grade */}
              <tr className="bg-muted/40 border-b border-border text-foreground text-center">
                {grades.flatMap((grade) =>
                  effectiveSubjects.map((sub) => (
                    <th
                      key={`${grade.id}-${sub.id}`}
                      className="px-2 py-1 border-r border-border font-bold text-[11px] text-center"
                      style={{
                        backgroundColor: sub.color === 'palette-1' ? 'rgba(59, 130, 246, 0.08)' : 'rgba(16, 185, 129, 0.08)',
                      }}
                    >
                      <span className="font-mono">{sub.code}</span>
                    </th>
                  )),
                )}
              </tr>
            </thead>

            <tbody>
              {exams.map((exam, examIdx) => {
                const examBg =
                  examIdx % 2 === 0 ? 'bg-card' : 'bg-muted/20'

                // Build row definitions for this exam:
                // Setters: maxSetters rows labelled "Đề"
                // Reviewers: maxReviewers rows labelled "P.Biện"
                const rowDefs: Array<{
                  role: Role
                  position: number
                  label: string
                  isFirst: boolean
                }> = []

                for (let p = 0; p < maxSetters; p++) {
                  rowDefs.push({
                    role: 'setter',
                    position: p,
                    label: t('assignments.roleSetterShort') || 'Đề',
                    isFirst: p === 0,
                  })
                }
                for (let p = 0; p < maxReviewers; p++) {
                  rowDefs.push({
                    role: 'reviewer',
                    position: p,
                    label: t('assignments.roleReviewerShort') || 'P.Biện',
                    isFirst: p === 0 && maxSetters === 0,
                  })
                }

                const totalExamRows = rowDefs.length

                return rowDefs.map((rowDef, rIdx) => (
                  <tr
                    key={`${exam.id}-${rowDef.role}-${rowDef.position}`}
                    className={`border-b border-border/80 hover:bg-muted/30 transition-colors ${examBg}`}
                  >
                    {/* Exam Name Merged Cell */}
                    {rIdx === 0 && (
                      <td
                        rowSpan={totalExamRows}
                        className="p-2 border-r border-border text-center font-bold text-xs bg-muted/40 align-middle w-20"
                      >
                        <div className="font-bold text-foreground">{exam.name}</div>
                      </td>
                    )}

                    {/* Role Label Cell */}
                    <td className="px-2 py-1 border-r border-border font-semibold text-muted-foreground text-[11px] text-center w-16 bg-muted/20">
                      {rowDef.label}
                    </td>

                    {/* Matrix Cells per Grade x Subject */}
                    {grades.flatMap((grade) =>
                      effectiveSubjects.map((subject) => {
                        const isSlotValid =
                          rowDef.role === 'setter'
                            ? rowDef.position < subject.setters
                            : rowDef.position < subject.reviewers

                        if (!isSlotValid) {
                          return (
                            <td
                              key={`${exam.id}-${grade.id}-${subject.id}-${rowDef.role}-${rowDef.position}`}
                              className="border-r border-border bg-muted/10 p-1 text-center text-muted-foreground/30 font-mono text-[11px]"
                            >
                              —
                            </td>
                          )
                        }

                        const slotRef: SlotRef = {
                          exam_id: exam.id,
                          grade_id: grade.id,
                          subject_id: subject.id,
                          role: rowDef.role,
                          position: rowDef.position,
                        }

                        const assignment = assignments.find(
                          (a) =>
                            a.exam_id === exam.id &&
                            a.grade_id === grade.id &&
                            a.subject_id === subject.id &&
                            a.role === rowDef.role &&
                            a.position === rowDef.position,
                        )

                        const teacher = assignment
                          ? teacherMap.get(assignment.teacher_id)
                          : null

                        const isForced = isSlotForced(
                          exam.id,
                          grade.id,
                          subject.id,
                          rowDef.role,
                          rowDef.position,
                        )
                        const isPinned = teacher
                          ? isSlotLocked(exam.id, grade.id, subject.id, teacher.id)
                          : false
                        const isKept = isSlotKept(
                          exam.id,
                          grade.id,
                          subject.id,
                          rowDef.role,
                          rowDef.position,
                        )

                        const isHighlighted =
                          teacher &&
                          activeHighlightedTeacherId !== null &&
                          teacher.id === activeHighlightedTeacherId

                        // Relevant soft violations for this panel
                        const panelViolations = scoreReport?.violations.filter(
                          (v) =>
                            v.panel &&
                            v.panel.exam_id === exam.id &&
                            v.panel.grade_id === grade.id &&
                            v.panel.subject_id === subject.id,
                        ) || []

                        const teacherStat = teacher ? teacherStats.get(teacher.id) : null
                        const teacherQuota = teacher ? teacherQuotas.get(teacher.id) ?? teacher.load_weight * 4 : 0

                        return (
                          <td
                            key={`${exam.id}-${grade.id}-${subject.id}-${rowDef.role}-${rowDef.position}`}
                            className={`border-r border-border p-1 text-center relative group transition-all cursor-pointer ${
                              isHighlighted
                                ? 'bg-primary/20 ring-2 ring-primary ring-inset font-bold text-foreground'
                                : 'hover:bg-accent/40'
                            }`}
                            onMouseEnter={() => {
                              if (teacher) setHoveredTeacherId(teacher.id)
                            }}
                            onMouseLeave={() => setHoveredTeacherId(null)}
                            onClick={() => {
                              if (teacher) onSelectTeacherFocus(teacher.id)
                              if (isEditable && !isForced) {
                                setSelectedSlotForReplace({
                                  slot: slotRef,
                                  currentTeacherId: teacher?.id ?? null,
                                })
                              }
                            }}
                            draggable={isEditable && !isForced && !!teacher}
                            onDragStart={(e) => {
                              if (!isEditable || isForced || !teacher) return
                              setDraggedSlot({ slot: slotRef, teacherId: teacher.id })
                              e.dataTransfer.setData('text/plain', JSON.stringify(slotRef))
                            }}
                            onDragOver={(e) => {
                              if (isEditable && !isForced) {
                                e.preventDefault()
                              }
                            }}
                            onDrop={(e) => {
                              if (!isEditable || isForced) return
                              e.preventDefault()
                              if (draggedSlot) {
                                handleSwapSlots(draggedSlot.slot, slotRef)
                                setDraggedSlot(null)
                              }
                            }}
                            data-testid={`q-grid-cell-${exam.id}-${grade.id}-${subject.code}-${rowDef.role}-${rowDef.position}`}
                          >
                            <TooltipProvider>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <div className="flex items-center justify-between gap-1 w-full min-h-[26px] px-1">
                                    <span className="truncate text-xs font-medium">
                                      {teacher ? (
                                        teacher.display_name || teacher.full_name
                                      ) : (
                                        <span className="text-muted-foreground/60 italic text-[11px]">
                                          Trống
                                        </span>
                                      )}
                                    </span>

                                    {/* Badges / Indicators */}
                                    <div className="flex items-center gap-0.5 shrink-0">
                                      {isForced && (
                                        <LockIcon
                                          className="h-3 w-3 text-amber-500 shrink-0"
                                          data-testid="forced-lock-icon"
                                        />
                                      )}
                                      {isPinned && !isForced && (
                                        <Pin
                                          className="h-3 w-3 text-primary shrink-0"
                                          data-testid="pin-slot-icon"
                                        />
                                      )}
                                      {isKept && (
                                        <BookmarkCheck className="h-3 w-3 text-emerald-500 shrink-0" />
                                      )}
                                      {panelViolations.length > 0 && (
                                        <AlertTriangle
                                          className="h-3 w-3 text-amber-500 shrink-0"
                                          data-testid="soft-violation-icon"
                                        />
                                      )}

                                      {/* Context Dropdown Menu */}
                                      <DropdownMenu>
                                        <DropdownMenuTrigger
                                          asChild
                                          onClick={(e) => e.stopPropagation()}
                                        >
                                          <button
                                            type="button"
                                            className="h-4 w-4 p-0 opacity-0 group-hover:opacity-100 transition-opacity hover:text-foreground inline-flex items-center justify-center"
                                            data-testid="q-cell-menu-btn"
                                          >
                                            <MoreVertical className="h-3 w-3 text-muted-foreground" />
                                          </button>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent
                                          align="end"
                                          className="w-52 bg-card border-border text-xs"
                                        >
                                          {isEditable && !isForced && (
                                            <>
                                              <DropdownMenuItem
                                                onClick={() =>
                                                  setSelectedSlotForReplace({
                                                    slot: slotRef,
                                                    currentTeacherId: teacher?.id ?? null,
                                                  })
                                                }
                                                className="gap-2"
                                              >
                                                <ArrowRightLeft className="h-3.5 w-3.5" />
                                                <span>{t('assignments.replaceModalTitle')}</span>
                                              </DropdownMenuItem>
                                              <DropdownMenuSeparator />
                                            </>
                                          )}

                                          {teacher && !isForced && (
                                            <>
                                              <DropdownMenuItem
                                                onClick={() =>
                                                  onCreateLock(slotRef, teacher.id, 'pin')
                                                }
                                                className="gap-2"
                                              >
                                                <Pin className="h-3.5 w-3.5 text-primary" />
                                                <span>{t('assignments.pinTeacherSlot')}</span>
                                              </DropdownMenuItem>
                                              <DropdownMenuItem
                                                onClick={() =>
                                                  onCreateLock(slotRef, teacher.id, 'forbid')
                                                }
                                                className="gap-2 text-destructive"
                                              >
                                                <Ban className="h-3.5 w-3.5" />
                                                <span>{t('assignments.forbidTeacherSlot')}</span>
                                              </DropdownMenuItem>
                                              <DropdownMenuSeparator />
                                            </>
                                          )}

                                          <DropdownMenuItem
                                            onClick={() => onToggleKeepSlot(slotRef)}
                                            className="gap-2"
                                          >
                                            <BookmarkCheck className="h-3.5 w-3.5 text-amber-500" />
                                            <span>
                                              {isKept
                                                ? t('assignments.unkeepSlot')
                                                : t('assignments.keepSlotReoptimize')}
                                            </span>
                                          </DropdownMenuItem>

                                          <DropdownMenuItem
                                            onClick={onReoptimizeRemaining}
                                            className="gap-2"
                                          >
                                            <RefreshCw className="h-3.5 w-3.5 text-emerald-600" />
                                            <span>{t('assignments.reoptimizeRest')}</span>
                                          </DropdownMenuItem>
                                        </DropdownMenuContent>
                                      </DropdownMenu>
                                    </div>
                                  </div>
                                </TooltipTrigger>
                                <TooltipContent className="text-xs space-y-1">
                                  {teacher ? (
                                    <>
                                      <p className="font-bold">
                                        {teacher.full_name}
                                        {teacher.display_name ? ` (${teacher.display_name})` : ''}
                                      </p>
                                      <p className="text-muted-foreground">
                                        {t('assignments.teacherQuotaCount', {
                                          count: teacherStat?.total ?? 0,
                                          quota: teacherQuota,
                                        })}
                                      </p>
                                      <p>
                                        {t('assignments.teacherRoles', {
                                          setter: teacherStat?.setters ?? 0,
                                          reviewer: teacherStat?.reviewers ?? 0,
                                        })}
                                      </p>
                                      <p>
                                        {t('assignments.teacherExams', {
                                          exams: Array.from(teacherStat?.examNames ?? []).join(', ') || 'Chưa có',
                                        })}
                                      </p>
                                      {isForced && (
                                        <p className="font-semibold text-amber-600 dark:text-amber-400">
                                          {t('assignments.forcedSeatLock')}
                                        </p>
                                      )}
                                    </>
                                  ) : (
                                    <p className="italic">Chưa có giáo viên phân công</p>
                                  )}
                                </TooltipContent>
                              </Tooltip>
                            </TooltipProvider>
                          </td>
                        )
                      }),
                    )}
                  </tr>
                ))
              })}
            </tbody>
          </table>
        </div>

        {/* Right: Attached Totals Panel */}
        <div
          className="w-full xl:w-96 rounded-lg border border-border bg-card shadow-sm p-3 space-y-3 shrink-0"
          data-testid="q-plan-totals-panel"
        >
          <div className="flex items-center justify-between gap-2 border-b border-border pb-2">
            <span className="font-bold text-xs uppercase text-foreground">
              {t('assignments.totalsHeader') || 'Bảng tổng hợp lượt'}
            </span>
            <div className="flex items-center gap-1.5">
              <Button
                variant="ghost"
                size="sm"
                className="h-6 px-1.5 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
                onClick={() => {
                  if (totalsSortField === 'total') {
                    setTotalsSortAsc(!totalsSortAsc)
                  } else {
                    setTotalsSortField('total')
                    setTotalsSortAsc(false)
                  }
                }}
              >
                <ArrowUpDown className="h-3 w-3" />
                <span>{totalsSortField === 'total' ? (totalsSortAsc ? 'Tăng' : 'Giảm') : 'Tổng'}</span>
              </Button>
            </div>
          </div>

          {/* Search filter input */}
          <div className="relative">
            <Search className="h-3.5 w-3.5 text-muted-foreground absolute left-2.5 top-2" />
            <Input
              value={totalsSearch}
              onChange={(e) => setTotalsSearch(e.target.value)}
              placeholder={t('assignments.teacherSearchPlaceholder') || 'Lọc giáo viên...'}
              className="h-7 text-xs pl-8 bg-background"
              data-testid="q-totals-filter-input"
            />
          </div>

          {/* Totals Table */}
          <div className="overflow-x-auto max-h-[520px] overflow-y-auto">
            <table className="w-full border-collapse text-xs select-none">
              <thead className="bg-muted/50 sticky top-0 border-b border-border text-[11px] font-bold">
                <tr>
                  <th className="p-1.5 text-left font-semibold">{t('assignments.totalsTeacher') || 'GV'}</th>
                  <th className="p-1.5 text-center font-bold text-primary">{t('assignments.totalsTotal') || 'Tổng'}</th>
                  <th className="p-1.5 text-center text-muted-foreground">{t('assignments.totalsSetter') || 'Đề'}</th>
                  <th className="p-1.5 text-center text-muted-foreground">{t('assignments.totalsReviewer') || 'PB'}</th>
                  {exams.map((ex) => (
                    <th key={ex.id} className="p-1.5 text-center font-mono text-[10px]">
                      {ex.name}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {displayedTeachers.map((twg) => {
                  const tRec = twg.teacher
                  const st = teacherStats.get(tRec.id)
                  const isForcedTeacher =
                    tRec.quota_override !== null && tRec.quota_override !== undefined
                      ? true
                      : forcedTeacherIds.has(tRec.id)

                  const isHighlighted =
                    activeHighlightedTeacherId !== null &&
                    activeHighlightedTeacherId === tRec.id

                  return (
                    <tr
                      key={tRec.id}
                      className={`border-b border-border/60 hover:bg-muted/40 cursor-pointer transition-colors ${
                        isHighlighted ? 'bg-primary/20 font-bold' : ''
                      }`}
                      onMouseEnter={() => setHoveredTeacherId(tRec.id)}
                      onMouseLeave={() => setHoveredTeacherId(null)}
                      onClick={() => onSelectTeacherFocus(tRec.id)}
                      data-testid={`q-totals-row-${tRec.id}`}
                    >
                      {/* Name + Fixed label */}
                      <td className="p-1.5">
                        <div className="flex items-center gap-1.5 truncate">
                          <span className="truncate font-medium text-foreground">
                            {tRec.display_name || tRec.full_name}
                          </span>
                          {isForcedTeacher && (
                            <Badge
                              variant="outline"
                              className="text-[9px] px-1 py-0 border-amber-500/30 text-amber-600 dark:text-amber-400 bg-amber-500/10 shrink-0 font-normal"
                            >
                              {t('assignments.totalsFixed') || 'cố định'}
                            </Badge>
                          )}
                        </div>
                      </td>

                      {/* Total */}
                      <td className="p-1.5 text-center font-bold text-foreground">
                        {st?.total ?? 0}
                      </td>

                      {/* Setters */}
                      <td className="p-1.5 text-center text-muted-foreground">
                        {st?.setters ?? 0}
                      </td>

                      {/* Reviewers */}
                      <td className="p-1.5 text-center text-muted-foreground">
                        {st?.reviewers ?? 0}
                      </td>

                      {/* Per-exam tasks count with badge marker */}
                      {exams.map((ex) => {
                        const count = st?.perExam.get(ex.id) ?? 0
                        const isOver2 = count >= 2
                        const isOver3 = count >= 3

                        return (
                          <td
                            key={ex.id}
                            className="p-1 text-center font-mono text-[11px]"
                          >
                            {count === 0 ? (
                              <span className="text-muted-foreground/30">0</span>
                            ) : isOver3 ? (
                              <span
                                className="inline-block px-1.5 py-0.2 rounded font-black text-rose-600 dark:text-rose-400 bg-rose-500/20 border border-rose-500/30"
                                title={t('assignments.taskCountOver3')}
                                data-testid="marker-over-3"
                              >
                                {count}**
                              </span>
                            ) : isOver2 ? (
                              <span
                                className="inline-block px-1 py-0.2 rounded font-bold text-amber-600 dark:text-amber-400 bg-amber-500/10 border border-amber-500/20"
                                title={t('assignments.taskCountOver2')}
                                data-testid="marker-over-2"
                              >
                                {count}*
                              </span>
                            ) : (
                              <span>{count}</span>
                            )}
                          </td>
                        )
                      })}
                    </tr>
                  )
                })}
              </tbody>
              {/* Total Row */}
              <tfoot className="bg-muted/80 sticky bottom-0 border-t border-border font-bold text-[11px]">
                <tr>
                  <td className="p-1.5 text-foreground">Tổng cộng</td>
                  <td className="p-1.5 text-center text-primary font-bold">
                    {Array.from(teacherStats.values()).reduce((sum, s) => sum + s.total, 0)}
                  </td>
                  <td className="p-1.5 text-center text-muted-foreground">
                    {Array.from(teacherStats.values()).reduce((sum, s) => sum + s.setters, 0)}
                  </td>
                  <td className="p-1.5 text-center text-muted-foreground">
                    {Array.from(teacherStats.values()).reduce((sum, s) => sum + s.reviewers, 0)}
                  </td>
                  {exams.map((ex) => {
                    const examTotal = Array.from(teacherStats.values()).reduce(
                      (sum, s) => sum + (s.perExam.get(ex.id) ?? 0),
                      0,
                    )
                    return (
                      <td key={ex.id} className="p-1 text-center font-mono">
                        {examTotal}
                      </td>
                    )
                  })}
                </tr>
              </tfoot>
            </table>
          </div>
        </div>
      </div>

      {/* Candidate Selection Modal */}
      {selectedSlotForReplace && (
        <CandidateSelectModal
          open={!!selectedSlotForReplace}
          onOpenChange={(open) => {
            if (!open) setSelectedSlotForReplace(null)
          }}
          slot={selectedSlotForReplace.slot}
          currentTeacherId={selectedSlotForReplace.currentTeacherId}
          schoolYearId={schoolYearId}
          assignments={assignments}
          teachers={teachers}
          campuses={campuses}
          onSelectTeacher={(newTeacherId) => {
            handleApplyReplacement(selectedSlotForReplace.slot, newTeacherId)
            setSelectedSlotForReplace(null)
          }}
        />
      )}
    </div>
  )
}
