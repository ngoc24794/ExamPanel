import { useTranslation } from 'react-i18next'
import {
  type Assignment,
  type Campus,
  type Exam,
  type Grade,
  type ScoreReport,
  type TeacherWithGrades,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { X, AlertTriangle, Layers, Calendar } from 'lucide-react'

interface TeacherFocusPanelProps {
  teacherId: number
  teachers: TeacherWithGrades[]
  campuses: Campus[]
  exams: Exam[]
  grades: Grade[]
  assignments: Assignment[]
  scoreReport?: ScoreReport | null
  onClose: () => void
}

export function TeacherFocusPanel({
  teacherId,
  teachers,
  campuses,
  exams,
  grades,
  assignments,
  scoreReport,
  onClose,
}: TeacherFocusPanelProps) {
  const { t } = useTranslation()
  const teacherRec = teachers.find((t) => t.teacher.id === teacherId)
  if (!teacherRec) return null

  const teacher = teacherRec.teacher
  const campus = campuses.find((c) => c.id === teacher.campus_id)
  const teacherStats = scoreReport?.per_teacher.find((st) => st.teacher_id === teacherId)

  // Assigned slots
  const myAssignments = assignments.filter((a) => a.teacher_id === teacherId)
  const setterCount = myAssignments.filter((a) => a.role === 'setter').length
  const reviewerCount = myAssignments.filter((a) => a.role === 'reviewer').length
  const totalCount = myAssignments.length

  const assignedExamIds = Array.from(new Set(myAssignments.map((a) => a.exam_id)))
  const assignedGradeIds = Array.from(new Set(myAssignments.map((a) => a.grade_id)))

  const examNames = assignedExamIds
    .map((eid) => exams.find((e) => e.id === eid)?.name ?? eid)
    .join(', ')

  const gradeNames = assignedGradeIds
    .map((gid) => grades.find((g) => g.id === gid)?.name ?? gid)
    .join(', ')

  // Soft violations involving this teacher
  const myViolations = (scoreReport?.violations ?? []).filter((v) =>
    v.teachers.includes(teacherId),
  )

  const quota = teacherStats?.quota ? teacherStats.quota.toFixed(2) : '--'

  return (
    <div
      className="rounded-lg border border-border bg-card p-4 space-y-4 shadow-sm"
      data-testid="teacher-focus-panel"
    >
      <div className="flex items-center justify-between border-b border-border pb-3">
        <div className="flex items-center gap-2">
          <div
            className="w-3.5 h-3.5 rounded-full"
            style={{ backgroundColor: campus?.color ?? '#64748b' }}
          />
          <div>
            <h4 className="font-semibold text-foreground text-sm flex items-center gap-1.5">
              <span>{teacher.full_name}</span>
              <Badge variant="outline" className="text-xs font-normal">
                {campus?.name}
              </Badge>
            </h4>
            <span className="text-xs text-muted-foreground">
              {t('assignments.focusLoadWeight', { weight: teacher.load_weight })}
            </span>
          </div>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose} className="h-7 w-7 p-0">
          <X className="h-4 w-4" />
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-3 text-xs">
        <div className="bg-muted/40 p-2.5 rounded border border-border">
          <span className="text-muted-foreground block">
            {t('assignments.teacherQuotaCount', { count: totalCount, quota })}
          </span>
          <span className="text-base font-bold text-foreground">{totalCount}</span>
        </div>
        <div className="bg-muted/40 p-2.5 rounded border border-border">
          <span className="text-muted-foreground block">{t('assignments.focusRoles')}</span>
          <span className="text-xs font-medium text-foreground">
            {t('assignments.focusRoleCounts', { setters: setterCount, reviewers: reviewerCount })}
          </span>
        </div>
      </div>

      <div className="space-y-2 text-xs">
        <div className="flex items-start gap-1.5">
          <Calendar className="h-4 w-4 text-muted-foreground shrink-0 mt-0.5" />
          <div>
            <strong className="text-foreground">{t('assignments.focusExams')}:</strong>{' '}
            <span className="text-muted-foreground">{examNames || t('assignments.focusUnassigned')}</span>
          </div>
        </div>

        <div className="flex items-start gap-1.5">
          <Layers className="h-4 w-4 text-muted-foreground shrink-0 mt-0.5" />
          <div>
            <strong className="text-foreground">{t('assignments.focusGrades')}:</strong>{' '}
            <span className="text-muted-foreground">
              {gradeNames || t('assignments.focusUnassigned')}
            </span>
          </div>
        </div>
      </div>

      {myViolations.length > 0 && (
        <div className="pt-2 border-t border-border space-y-2">
          <div className="flex items-center gap-1 text-xs font-semibold text-amber-600 dark:text-amber-400">
            <AlertTriangle className="h-3.5 w-3.5" />
            <span>{t('assignments.focusSoftViolations', { count: myViolations.length })}</span>
          </div>
          <div className="space-y-1.5">
            {myViolations.map((v, i) => (
              <div
                key={i}
                className="text-xs p-2 rounded bg-amber-500/10 border border-amber-500/20 text-muted-foreground"
              >
                <div className="font-medium text-foreground uppercase">
                  {v.rule} - {v.code}
                </div>
                <div>
                  {Object.entries(v.params)
                    .map(([k, val]) => `${k}: ${val}`)
                    .join(', ')}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
