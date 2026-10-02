import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type Assignment,
  type CandidateEval,
  type Campus,
  type SlotRef,
  type TeacherWithGrades,
  api,
} from '@/lib/api'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Search, AlertCircle, ArrowDown, ArrowUp } from 'lucide-react'

interface CandidateSelectModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  slot: SlotRef
  currentTeacherId: number | null
  schoolYearId: number
  assignments: Assignment[]
  teachers: TeacherWithGrades[]
  campuses: Campus[]
  onSelectTeacher: (teacherId: number) => void
}

export function CandidateSelectModal({
  open,
  onOpenChange,
  slot,
  currentTeacherId,
  schoolYearId,
  assignments,
  teachers,
  campuses,
  onSelectTeacher,
}: CandidateSelectModalProps) {
  const { t } = useTranslation()
  const [search, setSearch] = React.useState('')
  const [candidates, setCandidates] = React.useState<CandidateEval[]>([])
  const [isLoading, setIsLoading] = React.useState(false)
  const [selectedIndex, setSelectedIndex] = React.useState(0)

  // Evaluate candidates when modal opens or slot changes
  React.useEffect(() => {
    if (!open) return
    let active = true
    setIsLoading(true)

    api
      .evaluateCandidates(schoolYearId, assignments, slot)
      .then((res) => {
        if (active) {
          setCandidates(res)
          setIsLoading(false)
        }
      })
      .catch(() => {
        if (active) setIsLoading(false)
      })

    return () => {
      active = false
    }
  }, [open, schoolYearId, assignments, slot])

  // Filter candidates by search term
  const filteredCandidates = React.useMemo(() => {
    return candidates.filter((c) => {
      const teacher = teachers.find((t) => t.teacher.id === c.teacher_id)?.teacher
      if (!teacher) return false
      if (!search.trim()) return true
      return teacher.full_name.toLowerCase().includes(search.trim().toLowerCase())
    })
  }, [candidates, teachers, search])

  // Keyboard navigation (Arrow keys + Enter)
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (filteredCandidates.length === 0) return

    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setSelectedIndex((prev) => (prev + 1) % filteredCandidates.length)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setSelectedIndex((prev) =>
        prev - 1 < 0 ? filteredCandidates.length - 1 : prev - 1,
      )
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const selected = filteredCandidates[selectedIndex]
      if (selected && selected.hard_violations.length === 0) {
        onSelectTeacher(selected.teacher_id)
        onOpenChange(false)
      }
    }
  }

  const roleLabel = slot.role === 'setter' ? t('assignments.setter') : t('assignments.reviewer')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-lg bg-card text-card-foreground border-border"
        data-testid="candidate-select-modal"
        onKeyDown={handleKeyDown}
      >
        <DialogHeader>
          <DialogTitle>{t('assignments.replaceModalTitle')}</DialogTitle>
          <DialogDescription>
            {t('assignments.replaceModalDesc')} ({roleLabel} #{slot.position + 1})
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3 py-2">
          {/* Search box */}
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => {
                setSearch(e.target.value)
                setSelectedIndex(0)
              }}
              placeholder={t('common.search')}
              className="pl-8 text-sm"
              autoFocus
            />
          </div>

          {/* Candidates list */}
          <div className="max-h-72 overflow-y-auto space-y-1.5 pr-1">
            {isLoading && (
              <div className="p-4 text-center text-xs text-muted-foreground">
                {t('app.loading')}
              </div>
            )}

            {!isLoading && filteredCandidates.length === 0 && (
              <div className="p-4 text-center text-xs text-muted-foreground">
                Không tìm thấy giáo viên phù hợp
              </div>
            )}

            {!isLoading &&
              filteredCandidates.map((cand, idx) => {
                const teacherRec = teachers.find((t) => t.teacher.id === cand.teacher_id)
                if (!teacherRec) return null
                const teacher = teacherRec.teacher
                const campus = campuses.find((c) => c.id === teacher.campus_id)
                const isCurrent = teacher.id === currentTeacherId
                const isInvalid = cand.hard_violations.length > 0
                const isFocused = idx === selectedIndex

                return (
                  <div
                    key={cand.teacher_id}
                    data-testid={isInvalid ? 'candidate-item-invalid' : 'candidate-item-valid'}
                    onClick={() => {
                      if (!isInvalid) {
                        onSelectTeacher(cand.teacher_id)
                        onOpenChange(false)
                      }
                    }}
                    onMouseEnter={() => setSelectedIndex(idx)}
                    className={`p-2.5 rounded-lg border transition-all flex items-center justify-between text-xs cursor-pointer ${
                      isInvalid
                        ? 'opacity-60 bg-muted/20 border-border cursor-not-allowed'
                        : isFocused
                        ? 'bg-accent/70 border-primary ring-1 ring-primary'
                        : 'bg-card hover:bg-accent/30 border-border'
                    }`}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <div
                        className="w-3 h-3 rounded-full shrink-0"
                        style={{ backgroundColor: campus?.color ?? '#64748b' }}
                      />
                      <div className="min-w-0">
                        <div className="flex items-center gap-1.5">
                          <span className="font-semibold text-foreground truncate">
                            {teacher.full_name}
                          </span>
                          {isCurrent && (
                            <Badge variant="outline" className="text-[10px] px-1 py-0">
                              Hiện tại
                            </Badge>
                          )}
                          <span className="text-[11px] text-muted-foreground">
                            ({campus?.name})
                          </span>
                        </div>

                        {/* Violation reasons if invalid */}
                        {isInvalid && (
                          <div className="flex items-center gap-1 text-[11px] text-destructive mt-0.5">
                            <AlertCircle className="h-3 w-3 shrink-0" />
                            <span>
                              {cand.hard_violations
                                .map((v) => `${v.rule.toUpperCase()}: ${v.code}`)
                                .join(', ')}
                            </span>
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="flex items-center gap-2 shrink-0">
                      {/* Delta score display */}
                      {!isInvalid && (
                        <div className="text-right">
                          <span
                            className={`font-semibold text-xs flex items-center gap-0.5 justify-end ${
                              cand.delta_score < 0
                                ? 'text-emerald-600 dark:text-emerald-400'
                                : cand.delta_score > 0
                                ? 'text-amber-600 dark:text-amber-400'
                                : 'text-muted-foreground'
                            }`}
                          >
                            {cand.delta_score < 0 ? (
                              <>
                                <ArrowDown className="h-3 w-3" />
                                {cand.delta_score.toFixed(1)}
                              </>
                            ) : cand.delta_score > 0 ? (
                              <>
                                <ArrowUp className="h-3 w-3" />
                                +{cand.delta_score.toFixed(1)}
                              </>
                            ) : (
                              '±0.0'
                            )}
                          </span>
                          <span className="text-[10px] text-muted-foreground block">
                            Tổng: {cand.new_total.toFixed(1)}
                          </span>
                        </div>
                      )}

                      {!isInvalid && (
                        <Button
                          size="sm"
                          variant={isFocused ? 'default' : 'outline'}
                          className="h-7 text-xs px-2.5"
                          onClick={(e) => {
                            e.stopPropagation()
                            onSelectTeacher(cand.teacher_id)
                            onOpenChange(false)
                          }}
                        >
                          {t('assignments.applyCandidate')}
                        </Button>
                      )}
                    </div>
                  </div>
                )
              })}
          </div>
        </div>

        <DialogFooter className="flex justify-between items-center text-xs text-muted-foreground">
          <span>Gợi ý: Dùng phím ↑ ↓ để chọn, Enter để áp dụng</span>
          <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
