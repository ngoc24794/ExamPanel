import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowDown, ArrowUp, Calendar, Layers, Pencil, Plus, Trash2 } from 'lucide-react'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { useQueryClient } from '@tanstack/react-query'
import {
  useExams,
  useGrades,
  useSchoolYears,
  useCreateExam,
  useUpdateExam,
  useDeleteExam,
  useReorderExams,
  useCreateGrade,
  useUpdateGrade,
  useDeleteGrade,
  queryKeys,
} from '@/lib/query/hooks'
import { reportError } from '@/lib/query/query-client'
import { useSingleFlight } from '@/lib/useSingleFlight'
import { api } from '@/lib/api'
import { toast } from 'sonner'
import type { Exam, Grade } from '@/lib/api'

export const ExamsPage: React.FC = () => {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]

  const { data: exams = [] } = useExams(currentYear?.id)
  const { data: grades = [] } = useGrades()

  const createExamMutation = useCreateExam(currentYear?.id ?? 0)
  const updateExamMutation = useUpdateExam(currentYear?.id ?? 0)
  const deleteExamMutation = useDeleteExam(currentYear?.id ?? 0)
  const reorderExamsMutation = useReorderExams(currentYear?.id ?? 0)

  const createGradeMutation = useCreateGrade()
  const updateGradeMutation = useUpdateGrade()
  const deleteGradeMutation = useDeleteGrade()

  // Exam dialog state
  const [examDialogOpen, setExamDialogOpen] = React.useState(false)
  const [editingExam, setEditingExam] = React.useState<Exam | null>(null)
  const [examCode, setExamCode] = React.useState('')
  const [examName, setExamName] = React.useState('')

  // Delete Exam state
  const [deletingExam, setDeletingExam] = React.useState<Exam | null>(null)

  // Grade dialog state
  const [gradeDialogOpen, setGradeDialogOpen] = React.useState(false)
  const [editingGrade, setEditingGrade] = React.useState<Grade | null>(null)
  const [gradeCode, setGradeCode] = React.useState<number | ''>('')
  const [gradeName, setGradeName] = React.useState('')

  // Delete Grade state
  const [deletingGrade, setDeletingGrade] = React.useState<Grade | null>(null)

  const sortedExams = [...exams].sort((a, b) => a.sort_order - b.sort_order)
  const sortedGrades = [...grades].sort((a, b) => a.sort_order - b.sort_order)

  // Exam Handlers
  const handleOpenCreateExam = () => {
    setEditingExam(null)
    setExamCode('')
    setExamName('')
    setExamDialogOpen(true)
  }

  const handleOpenEditExam = (exam: Exam) => {
    setEditingExam(exam)
    setExamCode(exam.code)
    setExamName(exam.name)
    setExamDialogOpen(true)
  }

  const saveExamOnce = useSingleFlight()
  const saveGradeOnce = useSingleFlight()

  const saveExam = async () => {
    let activeYear = currentYear
    if (!activeYear) {
      try {
        const created = await api.createSchoolYear({
          name: t('app.schoolYearDefault'),
          is_current: true,
        })
        activeYear = created
        await qc.invalidateQueries({ queryKey: queryKeys.schoolYears })
      } catch (err) {
        reportError(err)
        return
      }
    }

    try {
      if (editingExam) {
        await updateExamMutation.mutateAsync({
          ...editingExam,
          name: examName.trim(),
        })
        toast.success(t('exams.updateSuccess'))
      } else {
        const nextOrder =
          exams.length > 0 ? Math.max(...exams.map((x) => x.sort_order)) + 1 : 1
        await createExamMutation.mutateAsync({
          school_year_id: activeYear.id,
          code: examCode.trim().toUpperCase(),
          name: examName.trim(),
          sort_order: nextOrder,
        })
        toast.success(t('exams.createSuccess'))
      }
      setExamDialogOpen(false)
    } catch (err) {
      reportError(err)
    }
  }
  const handleSaveExam = (e: React.FormEvent) => {
    e.preventDefault()
    return saveExamOnce(saveExam)
  }

  const handleConfirmDeleteExam = async () => {
    if (!deletingExam) return
    try {
      await deleteExamMutation.mutateAsync(deletingExam.id)
      setDeletingExam(null)
      toast.success(t('exams.deleteExam'))
    } catch (err) {
      reportError(err)
    }
  }

  const handleMoveExam = async (index: number, direction: 'up' | 'down') => {
    const targetIndex = direction === 'up' ? index - 1 : index + 1
    if (targetIndex < 0 || targetIndex >= sortedExams.length) return

    const newOrder = [...sortedExams]
    const temp = newOrder[index]
    newOrder[index] = newOrder[targetIndex]
    newOrder[targetIndex] = temp

    try {
      await reorderExamsMutation.mutateAsync(newOrder.map((e) => e.id))
    } catch (err) {
      reportError(err)
    }
  }

  // Grade Handlers
  const handleOpenCreateGrade = () => {
    setEditingGrade(null)
    setGradeCode('')
    setGradeName('')
    setGradeDialogOpen(true)
  }

  const handleOpenEditGrade = (grade: Grade) => {
    setEditingGrade(grade)
    setGradeCode(grade.code)
    setGradeName(grade.name)
    setGradeDialogOpen(true)
  }

  const saveGrade = async () => {
    if (gradeCode === '') return

    try {
      if (editingGrade) {
        await updateGradeMutation.mutateAsync({
          ...editingGrade,
          name: gradeName.trim(),
        })
        toast.success(t('exams.gradeUpdateSuccess'))
      } else {
        const nextOrder =
          grades.length > 0 ? Math.max(...grades.map((g) => g.sort_order)) + 1 : 1
        await createGradeMutation.mutateAsync({
          code: Number(gradeCode),
          name: gradeName.trim() || t('rules.gradeFallback', { id: gradeCode }),
          sort_order: nextOrder,
        })
        toast.success(t('exams.gradeCreateSuccess'))
      }
      setGradeDialogOpen(false)
    } catch (err) {
      reportError(err)
    }
  }
  const handleSaveGrade = (e: React.FormEvent) => {
    e.preventDefault()
    return saveGradeOnce(saveGrade)
  }

  const handleConfirmDeleteGrade = async () => {
    if (!deletingGrade) return
    try {
      await deleteGradeMutation.mutateAsync(deletingGrade.id)
      setDeletingGrade(null)
      toast.success(t('exams.deleteGrade'))
    } catch (err) {
      reportError(err)
    }
  }

  return (
    <div className="space-y-8" data-testid="exams-page">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {t('exams.title')}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">{t('exams.description')}</p>
      </div>

      <div className="grid gap-8 lg:grid-cols-2">
        {/* Section 1: Exams (Current Year) */}
        <Card className="bg-card border-border flex flex-col">
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <div className="space-y-1">
              <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
                <Calendar className="h-4 w-4 text-primary" />
                <span>{t('exams.examsSection')}</span>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground">
                {currentYear
                  ? t('common.schoolYearName', { name: currentYear.name })
                  : ''}
              </CardDescription>
            </div>
            <Button
              size="sm"
              onClick={handleOpenCreateExam}
              className="gap-1.5 h-8 text-xs"
              data-testid="add-exam-btn"
            >
              <Plus className="h-3.5 w-3.5" />
              <span>{t('exams.createExam')}</span>
            </Button>
          </CardHeader>
          <CardContent className="flex-1 space-y-3">
            {sortedExams.length === 0 ? (
              <div className="p-8 text-center text-sm text-muted-foreground">
                {t('exams.emptyExams')}
              </div>
            ) : (
              <div className="divide-y divide-border border rounded-md">
                {sortedExams.map((exam, idx) => (
                  <div
                    key={exam.id}
                    className="flex items-center justify-between p-3 hover:bg-muted/30 transition-colors gap-3"
                    data-testid={`exam-row-${exam.code}`}
                  >
                    <div className="flex items-center gap-3">
                      <Badge
                        variant="outline"
                        className="font-mono text-xs w-16 justify-center"
                      >
                        {exam.code}
                      </Badge>
                      <span className="text-sm font-medium text-foreground">
                        {exam.name}
                      </span>
                    </div>

                    <div className="flex items-center gap-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground hover:text-foreground"
                        disabled={idx === 0}
                        onClick={() => handleMoveExam(idx, 'up')}
                        aria-label={t('exams.reorderUp')}
                        title={t('exams.reorderUp')}
                        data-testid={`exam-move-up-${idx}`}
                      >
                        <ArrowUp className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground hover:text-foreground"
                        disabled={idx === sortedExams.length - 1}
                        onClick={() => handleMoveExam(idx, 'down')}
                        aria-label={t('exams.reorderDown')}
                        title={t('exams.reorderDown')}
                        data-testid={`exam-move-down-${idx}`}
                      >
                        <ArrowDown className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground hover:text-foreground ml-1"
                        onClick={() => handleOpenEditExam(exam)}
                        aria-label={t('exams.editExam')}
                        title={t('exams.editExam')}
                        data-testid={`exam-edit-${exam.id}`}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-destructive hover:bg-destructive/10"
                        onClick={() => setDeletingExam(exam)}
                        aria-label={t('exams.deleteExam')}
                        title={t('exams.deleteExam')}
                        data-testid={`exam-delete-${exam.id}`}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Section 2: Grades (Global) */}
        <Card className="bg-card border-border flex flex-col">
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <div className="space-y-1">
              <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
                <Layers className="h-4 w-4 text-primary" />
                <span>{t('exams.gradesSection')}</span>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground">
                {t('exams.description')}
              </CardDescription>
            </div>
            <Button
              size="sm"
              onClick={handleOpenCreateGrade}
              className="gap-1.5 h-8 text-xs"
              data-testid="add-grade-btn"
            >
              <Plus className="h-3.5 w-3.5" />
              <span>{t('exams.createGrade')}</span>
            </Button>
          </CardHeader>
          <CardContent className="flex-1 space-y-3">
            {sortedGrades.length === 0 ? (
              <div className="p-8 text-center text-sm text-muted-foreground">
                {t('exams.emptyGrades')}
              </div>
            ) : (
              <div className="divide-y divide-border border rounded-md">
                {sortedGrades.map((grade) => (
                  <div
                    key={grade.id}
                    className="flex items-center justify-between p-3 hover:bg-muted/30 transition-colors gap-3"
                    data-testid={`grade-row-${grade.code}`}
                  >
                    <div className="flex items-center gap-3">
                      <Badge
                        variant="outline"
                        className="font-mono text-xs w-16 justify-center"
                      >
                        {grade.code}
                      </Badge>
                      <span className="text-sm font-medium text-foreground">
                        {grade.name}
                      </span>
                    </div>

                    <div className="flex items-center gap-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground hover:text-foreground"
                        onClick={() => handleOpenEditGrade(grade)}
                        aria-label={t('exams.editGrade')}
                        title={t('exams.editGrade')}
                        data-testid={`grade-edit-${grade.id}`}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-destructive hover:bg-destructive/10"
                        onClick={() => setDeletingGrade(grade)}
                        aria-label={t('exams.deleteGrade')}
                        title={t('exams.deleteGrade')}
                        data-testid={`grade-delete-${grade.id}`}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Add / Edit Exam Dialog */}
      <Dialog open={examDialogOpen} onOpenChange={setExamDialogOpen}>
        <DialogContent className="sm:max-w-md bg-card text-foreground border-border">
          <DialogHeader>
            <DialogTitle>
              {editingExam ? t('exams.editExam') : t('exams.createExam')}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveExam} className="space-y-4 pt-2">
            {!editingExam && (
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('exams.examCode')}
                </label>
                <Input
                  value={examCode}
                  onChange={(e) => setExamCode(e.target.value)}
                  placeholder={t('exams.codePlaceholder')}
                  required
                  data-testid="exam-code-input"
                />
              </div>
            )}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                {t('exams.examName')}
              </label>
              <Input
                value={examName}
                onChange={(e) => setExamName(e.target.value)}
                placeholder={t('exams.namePlaceholder')}
                required
                data-testid="exam-name-input"
              />
            </div>
            <DialogFooter className="pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setExamDialogOpen(false)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="submit"
                data-testid="exam-save-btn"
                disabled={createExamMutation.isPending || updateExamMutation.isPending}
              >
                {createExamMutation.isPending || updateExamMutation.isPending
                  ? t('common.saving')
                  : t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Exam Confirmation Dialog */}
      <AlertDialog
        open={!!deletingExam}
        onOpenChange={(open) => !open && setDeletingExam(null)}
      >
        <AlertDialogContent className="bg-card text-foreground border-border">
          <AlertDialogHeader>
            <AlertDialogTitle>{t('exams.deleteExamConfirm')}</AlertDialogTitle>
            <AlertDialogDescription className="text-xs text-muted-foreground">
              {t('exams.deleteExamDesc')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDeleteExam}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              data-testid="confirm-delete-exam-btn"
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Add / Edit Grade Dialog */}
      <Dialog open={gradeDialogOpen} onOpenChange={setGradeDialogOpen}>
        <DialogContent className="sm:max-w-md bg-card text-foreground border-border">
          <DialogHeader>
            <DialogTitle>
              {editingGrade ? t('exams.editGrade') : t('exams.createGrade')}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveGrade} className="space-y-4 pt-2">
            {!editingGrade && (
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('exams.gradeCode')}
                </label>
                <Input
                  type="number"
                  value={gradeCode}
                  onChange={(e) =>
                    setGradeCode(e.target.value === '' ? '' : Number(e.target.value))
                  }
                  placeholder={t('exams.gradeCodePlaceholder')}
                  required
                  data-testid="grade-code-input"
                />
              </div>
            )}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                {t('exams.gradeName')}
              </label>
              <Input
                value={gradeName}
                onChange={(e) => setGradeName(e.target.value)}
                placeholder={t('exams.gradeNamePlaceholder')}
                required
                data-testid="grade-name-input"
              />
            </div>
            <DialogFooter className="pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setGradeDialogOpen(false)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="submit"
                data-testid="grade-save-btn"
                disabled={createGradeMutation.isPending || updateGradeMutation.isPending}
              >
                {createGradeMutation.isPending || updateGradeMutation.isPending
                  ? t('common.saving')
                  : t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Grade Confirmation Dialog */}
      <AlertDialog
        open={!!deletingGrade}
        onOpenChange={(open) => !open && setDeletingGrade(null)}
      >
        <AlertDialogContent className="bg-card text-foreground border-border">
          <AlertDialogHeader>
            <AlertDialogTitle>{t('exams.deleteGradeConfirm')}</AlertDialogTitle>
            <AlertDialogDescription className="text-xs text-muted-foreground">
              {t('exams.deleteGradeDesc')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDeleteGrade}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              data-testid="confirm-delete-grade-btn"
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
