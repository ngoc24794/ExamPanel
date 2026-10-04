import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Award, Sparkles } from 'lucide-react'
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
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '@/components/ui/table'
import {
  useSubjects,
  useTeacherCompetencies,
  useReplaceTeacherCompetencies,
  useTeachers,
} from '@/lib/query/hooks'
import { toast } from 'sonner'
import type { Teacher, GradeScope, Competency } from '@/lib/api'

interface TeacherCompetenciesModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  teacher: Teacher | null
  schoolYearId: number
}

interface RoleConfig {
  enabled: boolean
  grade_scope: GradeScope
}

export function TeacherCompetenciesModal({
  open,
  onOpenChange,
  teacher,
  schoolYearId,
}: TeacherCompetenciesModalProps) {
  const { t } = useTranslation()
  const { data: subjects = [] } = useSubjects(schoolYearId)
  const { data: teachersWithGrades = [] } = useTeachers(schoolYearId)
  const { data: currentCompetencies = [], isLoading } = useTeacherCompetencies(
    teacher?.id ?? 0,
    schoolYearId,
  )

  const replaceMutation = useReplaceTeacherCompetencies(schoolYearId)

  // Local state for the current teacher's competencies:
  // subject_id -> { setter: { enabled, grade_scope }, reviewer: { enabled, grade_scope } }
  const [configs, setConfigs] = React.useState<
    Record<number, { setter: RoleConfig; reviewer: RoleConfig }>
  >({})

  React.useEffect(() => {
    if (!open || !teacher) return

    const initial: Record<number, { setter: RoleConfig; reviewer: RoleConfig }> = {}
    subjects.forEach((s) => {
      initial[s.id] = {
        setter: { enabled: false, grade_scope: 'taught' },
        reviewer: { enabled: false, grade_scope: 'taught' },
      }
    })

    currentCompetencies.forEach((c) => {
      if (!initial[c.subject_id]) {
        initial[c.subject_id] = {
          setter: { enabled: false, grade_scope: 'taught' },
          reviewer: { enabled: false, grade_scope: 'taught' },
        }
      }
      if (c.role === 'setter') {
        initial[c.subject_id].setter = {
          enabled: true,
          grade_scope: c.grade_scope,
        }
      } else if (c.role === 'reviewer') {
        initial[c.subject_id].reviewer = {
          enabled: true,
          grade_scope: c.grade_scope,
        }
      }
    })

    setConfigs(initial)
  }, [open, teacher, currentCompetencies, subjects])

  const handleToggle = (
    subjectId: number,
    role: 'setter' | 'reviewer',
    checked: boolean,
  ) => {
    setConfigs((prev) => ({
      ...prev,
      [subjectId]: {
        ...prev[subjectId],
        [role]: {
          ...prev[subjectId]?.[role],
          enabled: checked,
        },
      },
    }))
  }

  const handleScopeChange = (
    subjectId: number,
    role: 'setter' | 'reviewer',
    scope: GradeScope,
  ) => {
    setConfigs((prev) => ({
      ...prev,
      [subjectId]: {
        ...prev[subjectId],
        [role]: {
          ...prev[subjectId]?.[role],
          grade_scope: scope,
        },
      },
    }))
  }

  const handleSave = async () => {
    if (!teacher) return

    const competencies: Competency[] = []
    Object.entries(configs).forEach(([subIdStr, config]) => {
      const subject_id = parseInt(subIdStr, 10)
      if (config.setter.enabled) {
        competencies.push({
          teacher_id: teacher.id,
          subject_id,
          role: 'setter',
          grade_scope: config.setter.grade_scope,
        })
      }
      if (config.reviewer.enabled) {
        competencies.push({
          teacher_id: teacher.id,
          subject_id,
          role: 'reviewer',
          grade_scope: config.reviewer.grade_scope,
        })
      }
    })

    try {
      await replaceMutation.mutateAsync({
        teacher_id: teacher.id,
        school_year_id: schoolYearId,
        competencies,
      })
      toast.success(
        t('competencies.saveSuccess') || 'Cập nhật phân công chuyên môn thành công',
      )
      onOpenChange(false)
    } catch {
      // Error handled by mutation hook
    }
  }

  // Bulk action: assign taught grade scope for all subjects & roles to all teachers
  const handleBulkAssignTaught = async () => {
    if (teachersWithGrades.length === 0 || subjects.length === 0) return

    try {
      for (const twg of teachersWithGrades) {
        const competencies: Competency[] = []
        subjects.forEach((s) => {
          competencies.push({
            teacher_id: twg.teacher.id,
            subject_id: s.id,
            role: 'setter',
            grade_scope: 'taught',
          })
          competencies.push({
            teacher_id: twg.teacher.id,
            subject_id: s.id,
            role: 'reviewer',
            grade_scope: 'taught',
          })
        })

        await replaceMutation.mutateAsync({
          teacher_id: twg.teacher.id,
          school_year_id: schoolYearId,
          competencies,
        })
      }
      toast.success(
        t('competencies.bulkAssignSuccess') ||
          'Đã gán chuyên môn theo khối đang dạy cho toàn bộ giáo viên',
      )
      onOpenChange(false)
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col bg-card border-border">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Award className="h-5 w-5 text-primary" />
              <DialogTitle className="text-base font-semibold text-foreground">
                {t('competencies.modalTitle') || 'Phân công chuyên môn'}
                {teacher && (
                  <span className="text-primary ml-1.5 font-normal">
                    - {teacher.full_name}{' '}
                    {teacher.display_name ? `(${teacher.display_name})` : ''}
                  </span>
                )}
              </DialogTitle>
            </div>
          </div>
          <DialogDescription className="text-xs text-muted-foreground">
            {t('competencies.modalDesc') ||
              'Chỉ định các môn học và vai trò (ra đề / phản biện) giáo viên có thẩm quyền đảm nhận.'}
          </DialogDescription>
        </DialogHeader>

        {/* Bulk Action Button Bar */}
        <div className="flex items-center justify-between p-2.5 rounded bg-muted/30 border border-border text-xs">
          <span className="text-muted-foreground">Thao tác nhanh cho toàn trường:</span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleBulkAssignTaught}
            className="h-7 text-xs gap-1.5"
            data-testid="bulk-assign-competencies-btn"
          >
            <Sparkles className="h-3.5 w-3.5 text-amber-500" />
            {t('competencies.bulkAssignTaught') ||
              'Gán theo khối đang dạy cho tất cả giáo viên'}
          </Button>
        </div>

        <div className="flex-1 overflow-y-auto min-h-[240px] border rounded border-border">
          {isLoading ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              {t('common.loading') || 'Đang tải thông tin chuyên môn...'}
            </div>
          ) : subjects.length === 0 ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              {t('subjects.noSubjects') || 'Chưa có môn thi nào được cấu hình.'}
            </div>
          ) : (
            <Table className="text-xs">
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead className="w-40">Môn học</TableHead>
                  <TableHead>Vai trò Ra đề (Đề)</TableHead>
                  <TableHead>Vai trò Phản biện (PB)</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {subjects.map((s) => {
                  const cfg = configs[s.id] || {
                    setter: { enabled: false, grade_scope: 'taught' },
                    reviewer: { enabled: false, grade_scope: 'taught' },
                  }

                  return (
                    <TableRow key={s.id}>
                      <TableCell className="font-semibold text-foreground">
                        <div className="flex items-center gap-1.5">
                          <Badge
                            variant="outline"
                            className="font-bold text-[10px] uppercase"
                          >
                            {s.code}
                          </Badge>
                          <span>{s.name}</span>
                        </div>
                      </TableCell>

                      {/* Setter Config */}
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Checkbox
                            checked={cfg.setter.enabled}
                            onCheckedChange={(checked) =>
                              handleToggle(s.id, 'setter', Boolean(checked))
                            }
                            id={`setter-${s.id}`}
                          />
                          <label
                            htmlFor={`setter-${s.id}`}
                            className="text-xs font-medium cursor-pointer"
                          >
                            Đảm nhận
                          </label>

                          {cfg.setter.enabled && (
                            <Select
                              value={cfg.setter.grade_scope}
                              onValueChange={(val) =>
                                handleScopeChange(s.id, 'setter', val as GradeScope)
                              }
                            >
                              <SelectTrigger className="h-6 w-32 text-[11px] bg-background">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent className="bg-popover border-border text-xs">
                                <SelectItem value="taught">Khối đang dạy</SelectItem>
                                <SelectItem value="any">Mọi khối</SelectItem>
                              </SelectContent>
                            </Select>
                          )}
                        </div>
                      </TableCell>

                      {/* Reviewer Config */}
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Checkbox
                            checked={cfg.reviewer.enabled}
                            onCheckedChange={(checked) =>
                              handleToggle(s.id, 'reviewer', Boolean(checked))
                            }
                            id={`reviewer-${s.id}`}
                          />
                          <label
                            htmlFor={`reviewer-${s.id}`}
                            className="text-xs font-medium cursor-pointer"
                          >
                            Đảm nhận
                          </label>

                          {cfg.reviewer.enabled && (
                            <Select
                              value={cfg.reviewer.grade_scope}
                              onValueChange={(val) =>
                                handleScopeChange(s.id, 'reviewer', val as GradeScope)
                              }
                            >
                              <SelectTrigger className="h-6 w-32 text-[11px] bg-background">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent className="bg-popover border-border text-xs">
                                <SelectItem value="taught">Khối đang dạy</SelectItem>
                                <SelectItem value="any">Mọi khối</SelectItem>
                              </SelectContent>
                            </Select>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </div>

        <DialogFooter className="pt-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onOpenChange(false)}
            className="text-xs"
          >
            {t('common.cancel') || 'Hủy'}
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={handleSave}
            disabled={!teacher}
            className="text-xs bg-primary text-primary-foreground hover:bg-primary/90"
            data-testid="save-competencies-btn"
          >
            {t('common.save') || 'Lưu chuyên môn'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
