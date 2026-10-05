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
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  AlertTriangle,
  Pin,
  Ban,
  CheckCircle2,
  BookmarkCheck,
  RefreshCw,
  MoreVertical,
  ShieldAlert,
  ArrowRightLeft,
  Sparkles,
  Info,
  FileEdit,
  Eye,
} from 'lucide-react'
import { getCampusDotColor } from '@/lib/theme/campus-colors'
import { CandidateSelectModal } from './CandidateSelectModal'

interface PlanMatrixViewProps {
  planDetails: PlanDetails
  planStatus: PlanStatus | null
  exams: Exam[]
  grades: Grade[]
  subjects?: Subject[]
  teachers: TeacherWithGrades[]
  campuses: Campus[]
  locks: Lock[]
  isEditable: boolean
  focusedTeacherId: number | null
  keptSlots: SlotRef[]
  onSelectTeacherFocus: (teacherId: number | null) => void
  onToggleKeepSlot: (slot: SlotRef) => void
  onReoptimizeRemaining: () => void
  onUpdateAssignments: (assignments: Assignment[]) => void
  onCreateLock: (slot: SlotRef, teacherId: number, kind: 'pin' | 'forbid') => void
}

export function PlanMatrixView({
  planDetails,
  planStatus,
  exams,
  grades,
  subjects = [],
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
}: PlanMatrixViewProps) {
  const { t } = useTranslation()
  const assignments = planDetails.assignments
  const scoreReport = planDetails.score_report

  const defaultSubject: Subject = {
    id: 1,
    code: 'CHUNG',
    name: 'Chung',
    color: 'blue',
    sort_order: 1,
    setters: 2,
    reviewers: 1,
    min_campuses: 2,
  }
  const effectiveSubjects = subjects.length > 0 ? subjects : [defaultSubject]
  const [selectedSubjectFilter, setSelectedSubjectFilter] = React.useState<
    number | 'all'
  >('all')
  const displayedSubjects =
    selectedSubjectFilter === 'all'
      ? effectiveSubjects
      : effectiveSubjects.filter((s) => s.id === selectedSubjectFilter)

  const [selectedSlotForReplace, setSelectedSlotForReplace] = React.useState<{
    slot: SlotRef
    currentTeacherId: number | null
  } | null>(null)

  // Drag and drop state
  const [draggedSlot, setDraggedSlot] = React.useState<{
    slot: SlotRef
    teacherId: number
  } | null>(null)

  const isSlotKept = (
    examId: number,
    gradeId: number,
    subjectId: number,
    role: Role,
    position: number,
  ) => {
    return keptSlots.some(
      (s) =>
        s.exam_id === examId &&
        s.grade_id === gradeId &&
        (s.subject_id === subjectId || (!s.subject_id && subjectId === 1)) &&
        s.role === role &&
        s.position === position,
    )
  }

  const isSlotLocked = (
    examId: number,
    gradeId: number,
    subjectId: number,
    teacherId: number,
  ) => {
    return locks.some(
      (l) =>
        l.exam_id === examId &&
        l.grade_id === gradeId &&
        (l.subject_id === subjectId || (!l.subject_id && subjectId === 1)) &&
        l.teacher_id === teacherId &&
        l.kind === 'pin',
    )
  }

  // Find assignments for a specific panel (exam × grade × subject)
  const getPanelAssignments = (examId: number, gradeId: number, subjectId: number) => {
    const panelAssignments = assignments.filter(
      (a) =>
        a.exam_id === examId &&
        a.grade_id === gradeId &&
        (a.subject_id === subjectId || (!a.subject_id && subjectId === 1)),
    )
    const setters = panelAssignments.filter((a) => a.role === 'setter')
    const reviewers = panelAssignments.filter((a) => a.role === 'reviewer')
    return { setters, reviewers }
  }

  // Soft violations relevant to a specific panel (S3, S4, S5, S9, S10)
  const getPanelViolations = (examId: number, gradeId: number, subjectId: number) => {
    if (!scoreReport) return []
    return scoreReport.violations.filter(
      (v) =>
        v.panel &&
        v.panel.exam_id === examId &&
        v.panel.grade_id === gradeId &&
        v.panel.subject_id === subjectId &&
        ['s3', 's4', 's5', 's9', 's10'].includes(v.rule),
    )
  }

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

  return (
    <div className="space-y-4" data-testid="plan-matrix-view">
      {/* Data Changed Warning Banner */}
      {planStatus?.data_changed && (
        <div className="rounded-lg border border-amber-500/50 bg-amber-500/10 p-4 text-amber-700 dark:text-amber-300 space-y-2">
          <div className="flex items-center gap-2 font-medium">
            <AlertTriangle className="h-5 w-5 text-amber-500" />
            <span>{t('assignments.dataChangedWarning')}</span>
          </div>
          {planStatus.hard_violations_now.length > 0 ? (
            <div className="text-xs space-y-1 pl-7">
              <p className="font-semibold text-destructive">
                {t('assignments.staleCurrentViolations')} (
                {planStatus.hard_violations_now.length})
              </p>
              <ul className="list-disc pl-4 space-y-0.5">
                {planStatus.hard_violations_now.map((v, i) => (
                  <li key={i}>
                    <strong className="uppercase">{v.rule}</strong>: {v.code} (
                    {Object.entries(v.params)
                      .map(([k, val]) => `${k}=${val}`)
                      .join(', ')}
                    )
                  </li>
                ))}
              </ul>
            </div>
          ) : (
            <p className="text-xs pl-7 text-muted-foreground">
              {t('assignments.dataChangedNoViolations')}
            </p>
          )}
        </div>
      )}

      {/* Rules Changed Info Banner */}
      {!planStatus?.data_changed && planStatus?.rules_changed && (
        <div className="rounded-lg border border-blue-500/40 bg-blue-500/10 p-3 text-blue-700 dark:text-blue-300 flex items-center gap-2 text-xs">
          <Info className="h-4 w-4 text-blue-500 shrink-0" />
          <span>{t('assignments.rulesChangedInfo')}</span>
        </div>
      )}

      {/* Summary Score Bar */}
      {scoreReport && (
        <div className="rounded-lg border border-border bg-card p-4 shadow-sm flex flex-wrap items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            <div>
              <span className="text-xs text-muted-foreground block">
                {t('assignments.summaryTotalScore', {
                  score: scoreReport.total.toFixed(2),
                })}
              </span>
              <span className="text-2xl font-bold text-foreground">
                {scoreReport.total.toFixed(2)}
              </span>
            </div>

            <div className="h-8 w-px bg-border hidden sm:block" />

            {/* Hard validity */}
            <div>
              {planStatus && planStatus.hard_violations_now.length > 0 ? (
                <Badge variant="destructive" className="gap-1">
                  <ShieldAlert className="h-3.5 w-3.5" />
                  {t('assignments.hardInvalid', {
                    count: planStatus.hard_violations_now.length,
                  })}
                </Badge>
              ) : (
                <Badge
                  variant="default"
                  className="bg-emerald-600 hover:bg-emerald-700 text-white gap-1"
                >
                  <CheckCircle2 className="h-3.5 w-3.5" />
                  {t('assignments.hardValid')}
                </Badge>
              )}
            </div>
          </div>

          {/* Rule Breakdown with Lower Bounds */}
          <div className="flex flex-wrap items-center gap-2 text-xs">
            {scoreReport.by_rule.map((r) => {
              const atBound = Math.abs(r.units - r.lower_bound) <= 1e-4
              return (
                <TooltipProvider key={r.rule}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div
                        className={`px-2.5 py-1 rounded border flex items-center gap-1.5 cursor-help ${
                          atBound
                            ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-700 dark:text-emerald-300'
                            : 'bg-muted/50 border-border text-foreground'
                        }`}
                      >
                        <span className="font-semibold uppercase">{r.rule}</span>
                        <span>{r.penalty.toFixed(1)}</span>
                        {atBound && <Sparkles className="h-3 w-3 text-emerald-500" />}
                      </div>
                    </TooltipTrigger>
                    <TooltipContent className="text-xs space-y-1 bg-popover text-popover-foreground border-border">
                      <p className="font-bold uppercase">Tiêu chí {r.rule}</p>
                      <p>
                        Hệ số: {r.weight} | Đơn vị vi phạm: {r.units}
                      </p>
                      <p>Điểm phạt: {r.penalty.toFixed(2)}</p>
                      <p>
                        {t('assignments.lowerBoundLabel', {
                          bound: r.lower_bound.toFixed(2),
                        })}
                      </p>
                      {atBound && (
                        <p className="text-emerald-500 font-semibold">
                          {t('assignments.atLowerBound')}
                        </p>
                      )}
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )
            })}
          </div>
        </div>
      )}

      {/* Subject Filter Tabs */}
      {effectiveSubjects.length > 1 && (
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-muted-foreground">
            {t('subjects.subject') || 'Môn học'}:
          </span>
          <div className="flex items-center gap-1 bg-muted/40 p-1 rounded-lg border border-border">
            <Button
              variant={selectedSubjectFilter === 'all' ? 'default' : 'ghost'}
              size="sm"
              className="h-7 text-xs px-2.5"
              onClick={() => setSelectedSubjectFilter('all')}
            >
              {t('common.all') || 'Tất cả'} ({effectiveSubjects.length})
            </Button>
            {effectiveSubjects.map((s) => (
              <Button
                key={s.id}
                variant={selectedSubjectFilter === s.id ? 'default' : 'ghost'}
                size="sm"
                className="h-7 text-xs px-2.5"
                onClick={() => setSelectedSubjectFilter(s.id)}
              >
                {s.code} - {s.name}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Matrix Table: Exams (rows) × Grades (columns) */}
      <div className="overflow-x-auto rounded-lg border border-border bg-card shadow-sm">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border bg-muted/30">
              <th className="p-3 text-left font-semibold text-muted-foreground w-40">
                Kỳ thi / Khối
              </th>
              {grades.map((grade) => (
                <th
                  key={grade.id}
                  className="p-3 text-center font-semibold text-foreground border-l border-border"
                >
                  {grade.name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {exams.map((exam) => (
              <tr
                key={exam.id}
                className="border-b border-border last:border-0 hover:bg-muted/10 transition-colors"
              >
                <td className="p-3 font-medium text-foreground bg-muted/10">
                  <div className="font-semibold">{exam.name}</div>
                  <span className="text-xs text-muted-foreground">{exam.code}</span>
                </td>

                {grades.map((grade) => {
                  return (
                    <td
                      key={grade.id}
                      className="p-2.5 border-l border-border align-top min-w-[220px]"
                    >
                      <div className="space-y-3">
                        {displayedSubjects.map((sub) => {
                          const { setters, reviewers } = getPanelAssignments(
                            exam.id,
                            grade.id,
                            sub.id,
                          )
                          const panelViolations = getPanelViolations(
                            exam.id,
                            grade.id,
                            sub.id,
                          )
                          const numSetters = sub.setters || 2
                          const numReviewers = sub.reviewers || 1

                          return (
                            <div
                              key={sub.id}
                              className="space-y-1.5 p-1.5 rounded bg-muted/20 border border-border/50"
                            >
                              {displayedSubjects.length > 1 && (
                                <div className="flex items-center justify-between text-[11px] font-semibold text-muted-foreground pb-0.5 border-b border-border/40">
                                  <span>
                                    {sub.code} ({sub.name})
                                  </span>
                                  <span className="text-[10px] font-normal">
                                    {sub.setters}+{sub.reviewers}
                                  </span>
                                </div>
                              )}

                              {/* Soft violation badges for this panel */}
                              {panelViolations.length > 0 && (
                                <div className="flex flex-wrap gap-1 mb-1">
                                  {panelViolations.map((v, i) => (
                                    <TooltipProvider key={i}>
                                      <Tooltip>
                                        <TooltipTrigger asChild>
                                          <Badge
                                            variant="destructive"
                                            className="text-[10px] px-1 py-0 h-4 bg-amber-500/20 text-amber-600 dark:text-amber-400 border-amber-500/40 cursor-help"
                                          >
                                            {v.rule.toUpperCase()}
                                          </Badge>
                                        </TooltipTrigger>
                                        <TooltipContent className="text-xs bg-popover text-popover-foreground border-border">
                                          <p className="font-bold">
                                            {v.rule.toUpperCase()}: {v.code}
                                          </p>
                                          <p>
                                            {Object.entries(v.params)
                                              .map(([k, val]) => `${k}: ${val}`)
                                              .join(', ')}
                                          </p>
                                        </TooltipContent>
                                      </Tooltip>
                                    </TooltipProvider>
                                  ))}
                                </div>
                              )}

                              {/* Setters */}
                              <div className="space-y-1">
                                {Array.from({ length: numSetters }).map((_, pos) => {
                                  const setterAssignment = setters[pos]
                                  const teacherId = setterAssignment?.teacher_id ?? null
                                  const teacherRec = teacherId
                                    ? teachers.find((t) => t.teacher.id === teacherId)
                                    : null
                                  const campus = teacherRec
                                    ? campuses.find(
                                        (c) => c.id === teacherRec.teacher.campus_id,
                                      )
                                    : null

                                  const slotRef: SlotRef = {
                                    exam_id: exam.id,
                                    grade_id: grade.id,
                                    subject_id: sub.id,
                                    role: 'setter',
                                    position: pos,
                                  }
                                  const isKept = isSlotKept(
                                    exam.id,
                                    grade.id,
                                    sub.id,
                                    'setter',
                                    pos,
                                  )
                                  const isLocked = teacherId
                                    ? isSlotLocked(exam.id, grade.id, sub.id, teacherId)
                                    : false
                                  const isFocused = teacherId === focusedTeacherId

                                  return (
                                    <SlotChip
                                      key={`setter-${pos}`}
                                      slot={slotRef}
                                      teacher={teacherRec?.teacher ?? null}
                                      campus={campus ?? null}
                                      roleLabel={t('assignments.setter')}
                                      isKept={isKept}
                                      isLocked={isLocked}
                                      isFocused={isFocused}
                                      isEditable={isEditable}
                                      onFocus={() =>
                                        teacherId &&
                                        onSelectTeacherFocus(
                                          focusedTeacherId === teacherId
                                            ? null
                                            : teacherId,
                                        )
                                      }
                                      onReplace={() =>
                                        isEditable &&
                                        setSelectedSlotForReplace({
                                          slot: slotRef,
                                          currentTeacherId: teacherId,
                                        })
                                      }
                                      onToggleKeep={() => onToggleKeepSlot(slotRef)}
                                      onPin={() =>
                                        teacherId &&
                                        onCreateLock(slotRef, teacherId, 'pin')
                                      }
                                      onForbid={() =>
                                        teacherId &&
                                        onCreateLock(slotRef, teacherId, 'forbid')
                                      }
                                      onReoptimize={onReoptimizeRemaining}
                                      onDragStart={() =>
                                        teacherId &&
                                        setDraggedSlot({ slot: slotRef, teacherId })
                                      }
                                      onDrop={() => {
                                        if (draggedSlot && isEditable) {
                                          handleSwapSlots(draggedSlot.slot, slotRef)
                                          setDraggedSlot(null)
                                        }
                                      }}
                                    />
                                  )
                                })}
                              </div>

                              {/* Reviewers */}
                              {numReviewers > 0 && (
                                <div className="pt-1 border-t border-border/50 space-y-1">
                                  {Array.from({ length: numReviewers }).map((_, pos) => {
                                    const revAssignment = reviewers[pos]
                                    const teacherId = revAssignment?.teacher_id ?? null
                                    const teacherRec = teacherId
                                      ? teachers.find((t) => t.teacher.id === teacherId)
                                      : null
                                    const campus = teacherRec
                                      ? campuses.find(
                                          (c) => c.id === teacherRec.teacher.campus_id,
                                        )
                                      : null

                                    const slotRef: SlotRef = {
                                      exam_id: exam.id,
                                      grade_id: grade.id,
                                      subject_id: sub.id,
                                      role: 'reviewer',
                                      position: pos,
                                    }
                                    const isKept = isSlotKept(
                                      exam.id,
                                      grade.id,
                                      sub.id,
                                      'reviewer',
                                      pos,
                                    )
                                    const isLocked = teacherId
                                      ? isSlotLocked(exam.id, grade.id, sub.id, teacherId)
                                      : false
                                    const isFocused = teacherId === focusedTeacherId

                                    return (
                                      <SlotChip
                                        key={`reviewer-${pos}`}
                                        slot={slotRef}
                                        teacher={teacherRec?.teacher ?? null}
                                        campus={campus ?? null}
                                        roleLabel={t('assignments.reviewer')}
                                        isReviewer
                                        isKept={isKept}
                                        isLocked={isLocked}
                                        isFocused={isFocused}
                                        isEditable={isEditable}
                                        onFocus={() =>
                                          teacherId &&
                                          onSelectTeacherFocus(
                                            focusedTeacherId === teacherId
                                              ? null
                                              : teacherId,
                                          )
                                        }
                                        onReplace={() =>
                                          isEditable &&
                                          setSelectedSlotForReplace({
                                            slot: slotRef,
                                            currentTeacherId: teacherId,
                                          })
                                        }
                                        onToggleKeep={() => onToggleKeepSlot(slotRef)}
                                        onPin={() =>
                                          teacherId &&
                                          onCreateLock(slotRef, teacherId, 'pin')
                                        }
                                        onForbid={() =>
                                          teacherId &&
                                          onCreateLock(slotRef, teacherId, 'forbid')
                                        }
                                        onReoptimize={onReoptimizeRemaining}
                                        onDragStart={() =>
                                          teacherId &&
                                          setDraggedSlot({ slot: slotRef, teacherId })
                                        }
                                        onDrop={() => {
                                          if (draggedSlot && isEditable) {
                                            handleSwapSlots(draggedSlot.slot, slotRef)
                                            setDraggedSlot(null)
                                          }
                                        }}
                                      />
                                    )
                                  })}
                                </div>
                              )}
                            </div>
                          )
                        })}
                      </div>
                    </td>
                  )
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Keyboard / Candidate Replacement Dialog */}
      {selectedSlotForReplace && (
        <CandidateSelectModal
          open={true}
          onOpenChange={(open) => !open && setSelectedSlotForReplace(null)}
          slot={selectedSlotForReplace.slot}
          currentTeacherId={selectedSlotForReplace.currentTeacherId}
          schoolYearId={planDetails.plan.school_year_id}
          assignments={assignments}
          teachers={teachers}
          campuses={campuses}
          onSelectTeacher={(teacherId) => {
            handleApplyReplacement(selectedSlotForReplace.slot, teacherId)
            setSelectedSlotForReplace(null)
          }}
        />
      )}
    </div>
  )
}

// -----------------------------------------------------------------------------
// Individual Slot Chip Component
// -----------------------------------------------------------------------------

interface SlotChipProps {
  slot: SlotRef
  teacher: TeacherWithGrades['teacher'] | null
  campus: Campus | null
  roleLabel: string
  isReviewer?: boolean
  isKept: boolean
  isLocked: boolean
  isFocused: boolean
  isEditable: boolean
  onFocus: () => void
  onReplace: () => void
  onToggleKeep: () => void
  onPin: () => void
  onForbid: () => void
  onReoptimize: () => void
  onDragStart: () => void
  onDrop: () => void
}

function SlotChip({
  slot: _slot,
  teacher,
  campus,
  roleLabel: _roleLabel,
  isReviewer = false,
  isKept,
  isLocked,
  isFocused,
  isEditable,
  onFocus,
  onReplace,
  onToggleKeep,
  onPin,
  onForbid,
  onReoptimize,
  onDragStart,
  onDrop,
}: SlotChipProps) {
  const { t } = useTranslation()

  return (
    <div
      data-testid={teacher ? `chip-teacher-${teacher.id}` : 'chip-slot-empty'}
      draggable={isEditable && !!teacher}
      onDragStart={onDragStart}
      onDragOver={(e) => {
        if (isEditable) {
          e.preventDefault()
        }
      }}
      onDrop={(e) => {
        if (isEditable) {
          e.preventDefault()
          onDrop()
        }
      }}
      onClick={onFocus}
      className={`group relative flex items-center justify-between p-1.5 rounded border transition-all text-xs select-none ${
        isFocused
          ? 'bg-primary/20 border-primary ring-2 ring-primary ring-offset-1'
          : isKept
            ? 'bg-amber-500/10 border-amber-500/50'
            : 'bg-card hover:bg-accent/40 border-border'
      } ${isEditable ? 'cursor-pointer' : 'cursor-default'}`}
    >
      <div className="flex items-center gap-1.5 min-w-0">
        <div
          className="w-2.5 h-2.5 rounded-full shrink-0"
          style={{ backgroundColor: getCampusDotColor(campus?.color) }}
        />
        <span
          className={`inline-flex items-center gap-0.5 px-1 py-0.2 rounded text-[10px] font-semibold border shrink-0 ${
            isReviewer
              ? 'bg-primary/10 text-primary border-primary/20'
              : 'bg-muted text-muted-foreground border-border'
          }`}
        >
          {isReviewer ? (
            <Eye className="h-2.5 w-2.5" />
          ) : (
            <FileEdit className="h-2.5 w-2.5" />
          )}
          {isReviewer ? 'PB' : 'Đề'}
        </span>
        <span
          className={`font-medium truncate ${
            isReviewer ? 'text-primary font-semibold' : 'text-foreground'
          }`}
          title={
            teacher
              ? teacher.display_name
                ? `${teacher.full_name} (${teacher.display_name})`
                : teacher.full_name
              : undefined
          }
        >
          {teacher ? (
            teacher.display_name || teacher.full_name
          ) : (
            <span className="text-muted-foreground italic">Trống</span>
          )}
        </span>
      </div>

      <div className="flex items-center gap-1 shrink-0 ml-1">
        {/* Badges for Pin, Kept */}
        {isLocked && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Pin className="h-3 w-3 text-primary fill-current" />
              </TooltipTrigger>
              <TooltipContent className="text-xs">
                {t('assignments.pinned')}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}

        {isKept && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <BookmarkCheck className="h-3 w-3 text-amber-500" />
              </TooltipTrigger>
              <TooltipContent className="text-xs">
                {t('assignments.keptForReoptimize')}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}

        {/* Slot context dropdown menu for editing and actions */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
            <Button
              variant="ghost"
              size="sm"
              data-testid="slot-menu-btn"
              className="h-5 w-5 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <MoreVertical className="h-3 w-3 text-muted-foreground" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-52 bg-card border-border text-xs">
            {isEditable && (
              <>
                <DropdownMenuItem onClick={onReplace} className="gap-2">
                  <ArrowRightLeft className="h-3.5 w-3.5" />
                  <span>{t('assignments.replaceModalTitle')}</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
              </>
            )}

            {teacher && (
              <>
                <DropdownMenuItem onClick={onPin} className="gap-2">
                  <Pin className="h-3.5 w-3.5 text-primary" />
                  <span>{t('assignments.pinTeacherSlot')}</span>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={onForbid} className="gap-2 text-destructive">
                  <Ban className="h-3.5 w-3.5" />
                  <span>{t('assignments.forbidTeacherSlot')}</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
              </>
            )}

            <DropdownMenuItem onClick={onToggleKeep} className="gap-2">
              <BookmarkCheck className="h-3.5 w-3.5 text-amber-500" />
              <span>
                {isKept
                  ? t('assignments.unkeepSlot')
                  : t('assignments.keepSlotReoptimize')}
              </span>
            </DropdownMenuItem>

            <DropdownMenuItem onClick={onReoptimize} className="gap-2">
              <RefreshCw className="h-3.5 w-3.5 text-emerald-600" />
              <span>{t('assignments.reoptimizeRest')}</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
