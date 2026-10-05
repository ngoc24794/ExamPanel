import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { BookOpen, Plus, Pencil, Trash2 } from 'lucide-react'
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
import {
  useSchoolYears,
  useSubjects,
  useCreateSubject,
  useUpdateSubject,
  useDeleteSubject,
} from '@/lib/query/hooks'
import { getErrorMessage } from '@/lib/query/query-client'
import { toast } from 'sonner'
import type { Subject } from '@/lib/api'
import { getCampusDotColor } from '@/lib/theme/campus-colors'

export const SubjectsPage: React.FC = () => {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const schoolYearId = currentYear?.id ?? 1

  const { data: subjects = [], isLoading } = useSubjects(schoolYearId)
  const createMutation = useCreateSubject()
  const updateMutation = useUpdateSubject(schoolYearId)
  const deleteMutation = useDeleteSubject(schoolYearId)

  const [dialogOpen, setDialogOpen] = React.useState(false)
  const [editingSubject, setEditingSubject] = React.useState<Subject | null>(null)
  const [deleteSubjectId, setDeleteSubjectId] = React.useState<number | null>(null)

  // Form states
  const [code, setCode] = React.useState('')
  const [name, setName] = React.useState('')
  const [color, setColor] = React.useState('palette-1')
  const [setters, setSetters] = React.useState(2)
  const [reviewers, setReviewers] = React.useState(1)
  const [minCampuses, setMinCampuses] = React.useState(2)
  const [formError, setFormError] = React.useState<string | null>(null)

  const openCreateDialog = () => {
    setEditingSubject(null)
    setCode('')
    setName('')
    setColor(`palette-${(subjects.length % 8) + 1}`)
    setSetters(2)
    setReviewers(1)
    setMinCampuses(2)
    setFormError(null)
    setDialogOpen(true)
  }

  const openEditDialog = (subj: Subject) => {
    setEditingSubject(subj)
    setCode(subj.code)
    setName(subj.name)
    setColor(subj.color)
    setSetters(subj.setters)
    setReviewers(subj.reviewers)
    setMinCampuses(subj.min_campuses)
    setFormError(null)
    setDialogOpen(true)
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()
    setFormError(null)

    const trimmedCode = code.trim().toUpperCase()
    const trimmedName = name.trim()

    if (!trimmedCode) {
      setFormError(
        t('subjects.validationCodeRequired') || 'Mã môn thi không được để trống',
      )
      return
    }
    if (!trimmedName) {
      setFormError(
        t('subjects.validationNameRequired') || 'Tên môn thi không được để trống',
      )
      return
    }
    if (setters < 1 || setters > 4) {
      setFormError(
        t('subjects.validationSettersRange') || 'Số người ra đề phải từ 1 đến 4',
      )
      return
    }
    if (reviewers < 0 || reviewers > 2) {
      setFormError(
        t('subjects.validationReviewersRange') || 'Số người phản biện phải từ 0 đến 2',
      )
      return
    }
    if (minCampuses < 0 || minCampuses > 3) {
      setFormError(
        t('subjects.validationMinCampusesRange') || 'Số phân hiệu tối thiểu từ 0 đến 3',
      )
      return
    }

    try {
      if (editingSubject) {
        await updateMutation.mutateAsync({
          id: editingSubject.id,
          code: trimmedCode,
          name: trimmedName,
          color,
          sort_order: editingSubject.sort_order,
          setters,
          reviewers,
          min_campuses: minCampuses,
        })
        toast.success(t('subjects.updateSuccess') || 'Cập nhật môn thi thành công')
      } else {
        await createMutation.mutateAsync({
          school_year_id: schoolYearId,
          code: trimmedCode,
          name: trimmedName,
          color,
          sort_order: subjects.length + 1,
          setters,
          reviewers,
          min_campuses: minCampuses,
        })
        toast.success(t('subjects.createSuccess') || 'Tạo môn thi mới thành công')
      }
      setDialogOpen(false)
    } catch (err) {
      setFormError(getErrorMessage(err))
    }
  }

  const handleDelete = async () => {
    if (!deleteSubjectId) return
    try {
      await deleteMutation.mutateAsync(deleteSubjectId)
      setDeleteSubjectId(null)
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  return (
    <div className="space-y-6" data-testid="subjects-page">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <BookOpen className="h-5 w-5 text-primary" />
            <span>{t('subjects.title') || 'Danh mục Môn thi'}</span>
            {currentYear && (
              <Badge variant="outline" className="text-xs">
                {currentYear.name}
              </Badge>
            )}
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            {t('subjects.description') ||
              'Quản lý các môn thi, định mức ban đề (ra đề / phản biện) và yêu cầu đa dạng phân hiệu.'}
          </p>
        </div>
        <Button
          onClick={openCreateDialog}
          size="sm"
          className="gap-1.5 text-xs bg-primary hover:bg-primary/90 text-primary-foreground"
          data-testid="create-subject-btn"
        >
          <Plus className="h-4 w-4" />
          {t('subjects.createBtn') || 'Thêm môn thi'}
        </Button>
      </div>

      {isLoading ? (
        <div className="p-8 text-center text-sm text-muted-foreground">
          {t('common.loading') || 'Đang tải dữ liệu...'}
        </div>
      ) : subjects.length === 0 ? (
        <Card className="p-8 text-center border-dashed">
          <BookOpen className="h-10 w-10 mx-auto text-muted-foreground mb-3 opacity-50" />
          <p className="text-sm font-medium text-foreground">
            {t('subjects.noSubjects') || 'Chưa có môn thi nào'}
          </p>
          <p className="text-xs text-muted-foreground mt-1 mb-4">
            {t('subjects.noSubjectsHelp') ||
              'Tạo môn thi đầu tiên (ví dụ: VL - Vật lí, CN - Công nghệ).'}
          </p>
          <Button size="sm" onClick={openCreateDialog} className="text-xs">
            {t('subjects.createBtn') || 'Thêm môn thi'}
          </Button>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {subjects.map((subj) => (
            <Card
              key={subj.id}
              className="border border-border bg-card shadow-sm hover:border-primary/40 transition-colors"
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div
                      className="w-3.5 h-3.5 rounded-full"
                      style={{ backgroundColor: getCampusDotColor(subj.color) }}
                    />
                    <Badge variant="outline" className="font-bold text-xs uppercase">
                      {subj.code}
                    </Badge>
                  </div>
                  <div className="flex items-center gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-foreground"
                      onClick={() => openEditDialog(subj)}
                      title={t('common.edit') || 'Sửa'}
                      data-testid={`edit-subject-${subj.id}`}
                    >
                      <Pencil className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 p-0 text-destructive/70 hover:text-destructive"
                      onClick={() => setDeleteSubjectId(subj.id)}
                      title={t('common.delete') || 'Xóa'}
                      data-testid={`delete-subject-${subj.id}`}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
                <CardTitle className="text-base font-semibold text-foreground mt-2">
                  {subj.name}
                </CardTitle>
                <CardDescription className="text-xs text-muted-foreground">
                  Thứ tự sắp xếp: {subj.sort_order}
                </CardDescription>
              </CardHeader>
              <CardContent className="pt-0 text-xs space-y-2 border-t border-border/60 mt-2">
                <div className="flex justify-between items-center pt-2">
                  <span className="text-muted-foreground">
                    {t('subjects.composition') || 'Cơ cấu ban đề'}:
                  </span>
                  <span className="font-medium text-foreground">
                    {subj.setters} {t('assignments.setter') || 'Đề'} + {subj.reviewers}{' '}
                    {t('assignments.reviewer') || 'PB'}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">
                    {t('subjects.minCampuses') || 'Đa dạng phân hiệu tối thiểu'}:
                  </span>
                  <span className="font-medium text-foreground">
                    {subj.min_campuses > 0
                      ? t('subjects.minCampusesValue', { count: subj.min_campuses }) ||
                        `${subj.min_campuses} phân hiệu`
                      : t('subjects.minCampusesNone') || 'Không ràng buộc'}
                  </span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Create / Edit Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-md bg-card border-border">
          <DialogHeader>
            <DialogTitle className="text-base font-semibold text-foreground">
              {editingSubject
                ? t('subjects.editTitle') || 'Chỉnh sửa môn thi'
                : t('subjects.createTitle') || 'Thêm môn thi mới'}
            </DialogTitle>
          </DialogHeader>

          <form onSubmit={handleSave} className="space-y-4">
            {formError && (
              <div className="p-2.5 rounded text-xs bg-destructive/10 text-destructive border border-destructive/20">
                {formError}
              </div>
            )}

            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-foreground block mb-1">
                  {t('subjects.codeLabel') || 'Mã môn thi'}{' '}
                  <span className="text-destructive">*</span>
                </label>
                <Input
                  value={code}
                  onChange={(e) => setCode(e.target.value.toUpperCase())}
                  placeholder="VL, CN..."
                  className="h-8 text-xs font-mono"
                  data-testid="subject-code-input"
                  required
                />
              </div>

              <div>
                <label className="text-xs font-medium text-foreground block mb-1">
                  {t('subjects.nameLabel') || 'Tên môn thi'}{' '}
                  <span className="text-destructive">*</span>
                </label>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Vật lí, Công nghệ..."
                  className="h-8 text-xs"
                  data-testid="subject-name-input"
                  required
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-medium text-foreground block mb-1">
                    {t('subjects.settersLabel') || 'Số người ra đề'} (1..4)
                  </label>
                  <Input
                    type="number"
                    min={1}
                    max={4}
                    value={setters}
                    onChange={(e) => setSetters(parseInt(e.target.value, 10) || 1)}
                    className="h-8 text-xs"
                    data-testid="subject-setters-input"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-foreground block mb-1">
                    {t('subjects.reviewersLabel') || 'Số người phản biện'} (0..2)
                  </label>
                  <Input
                    type="number"
                    min={0}
                    max={2}
                    value={reviewers}
                    onChange={(e) => setReviewers(parseInt(e.target.value, 10) || 0)}
                    className="h-8 text-xs"
                    data-testid="subject-reviewers-input"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-medium text-foreground block mb-1">
                    {t('subjects.minCampusesLabel') || 'Phân hiệu tối thiểu'} (0..3)
                  </label>
                  <Input
                    type="number"
                    min={0}
                    max={3}
                    value={minCampuses}
                    onChange={(e) => setMinCampuses(parseInt(e.target.value, 10) || 0)}
                    className="h-8 text-xs"
                    data-testid="subject-min-campuses-input"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-foreground block mb-1">
                    {t('subjects.colorLabel') || 'Màu sắc'}
                  </label>
                  <div className="flex items-center gap-1.5 mt-1">
                    {[1, 2, 3, 4, 5, 6, 7, 8].map((i) => {
                      const pal = `palette-${i}`
                      return (
                        <button
                          key={i}
                          type="button"
                          className={`w-6 h-6 rounded-full border-2 transition-transform ${
                            color === pal
                              ? 'scale-110 border-primary ring-2 ring-primary/30'
                              : 'border-transparent'
                          }`}
                          style={{ backgroundColor: getCampusDotColor(pal) }}
                          onClick={() => setColor(pal)}
                        />
                      )
                    })}
                  </div>
                </div>
              </div>
            </div>

            <DialogFooter className="pt-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setDialogOpen(false)}
                className="text-xs"
              >
                {t('common.cancel') || 'Hủy'}
              </Button>
              <Button
                type="submit"
                size="sm"
                className="text-xs bg-primary text-primary-foreground hover:bg-primary/90"
                data-testid="save-subject-btn"
              >
                {t('common.save') || 'Lưu'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <AlertDialog
        open={deleteSubjectId !== null}
        onOpenChange={(open) => !open && setDeleteSubjectId(null)}
      >
        <AlertDialogContent className="bg-card border-border">
          <AlertDialogHeader>
            <AlertDialogTitle className="text-base text-foreground">
              {t('subjects.deleteConfirmTitle') || 'Xác nhận xóa môn thi?'}
            </AlertDialogTitle>
            <AlertDialogDescription className="text-xs text-muted-foreground">
              {t('subjects.deleteConfirmDesc') ||
                'Hành động này sẽ xóa môn thi khỏi cấu hình. Nếu môn thi đã có dữ liệu phân công, việc xóa sẽ bị từ chối.'}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel className="text-xs">
              {t('common.cancel') || 'Hủy'}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="text-xs bg-destructive text-destructive-foreground hover:bg-destructive/90"
              data-testid="confirm-delete-subject-btn"
            >
              {t('common.delete') || 'Xóa'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
