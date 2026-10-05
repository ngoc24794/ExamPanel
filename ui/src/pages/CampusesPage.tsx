import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Plus, Edit2, Trash2, Building2, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from '@/components/ui/alert-dialog'
import { Skeleton } from '@/components/ui/skeleton'
import { CampusChip } from '@/components/CampusChip'
import {
  useCampuses,
  useTeachers,
  useSchoolYears,
  useCreateCampus,
  useUpdateCampus,
  useDeleteCampus,
} from '@/lib/query/hooks'
import {
  CAMPUS_PALETTE_OPTIONS,
  normalizeCampusColorKey,
} from '@/lib/theme/campus-colors'
import { type Campus } from '@/lib/api'

const campusSchema = z.object({
  code: z.string().min(1, 'campuses.errCodeRequired'),
  name: z.string().min(1, 'campuses.errNameRequired'),
  color: z.string().min(1, 'campuses.errColorRequired'),
})

type CampusFormValues = z.infer<typeof campusSchema>

export const CampusesPage: React.FC = () => {
  const { t, i18n } = useTranslation()
  const { data: campuses = [], isLoading: loadingCampuses } = useCampuses()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const { data: teachersWithGrades = [] } = useTeachers(currentYear?.id)

  const createMutation = useCreateCampus()
  const updateMutation = useUpdateCampus()
  const deleteMutation = useDeleteCampus()

  const [dialogOpen, setDialogOpen] = React.useState(false)
  const [editingCampus, setEditingCampus] = React.useState<Campus | null>(null)
  const [deleteCampusId, setDeleteCampusId] = React.useState<number | null>(null)
  const [inUseError, setInUseError] = React.useState<string | null>(null)

  const {
    register,
    handleSubmit,
    setValue,
    watch,
    reset,
    formState: { errors },
  } = useForm<CampusFormValues>({
    resolver: zodResolver(campusSchema),
    defaultValues: {
      code: '',
      name: '',
      color: 'blue',
    },
  })

  const selectedColor = watch('color')

  const openCreateDialog = () => {
    setEditingCampus(null)
    reset({
      code: '',
      name: '',
      color: 'blue',
    })
    setDialogOpen(true)
  }

  const openEditDialog = (campus: Campus) => {
    setEditingCampus(campus)
    reset({
      code: campus.code,
      name: campus.name,
      color: normalizeCampusColorKey(campus.color),
    })
    setDialogOpen(true)
  }

  const onSubmit = async (values: CampusFormValues) => {
    try {
      if (editingCampus) {
        await updateMutation.mutateAsync({
          ...editingCampus,
          code: values.code.trim(),
          name: values.name.trim(),
          color: values.color,
        })
      } else {
        await createMutation.mutateAsync({
          code: values.code.trim(),
          name: values.name.trim(),
          color: values.color,
        })
      }
      setDialogOpen(false)
    } catch {
      // handled
    }
  }

  const handleDelete = async () => {
    if (!deleteCampusId) return
    setInUseError(null)
    try {
      await deleteMutation.mutateAsync(deleteCampusId)
      setDeleteCampusId(null)
    } catch (err: unknown) {
      const errorObj = err as { code?: string }
      if (errorObj?.code === 'campus_in_use') {
        setInUseError(t('campuses.campusInUseHint'))
      }
    }
  }

  // Active teacher count per campus
  const activeTeachersPerCampus = React.useMemo(() => {
    const counts = new Map<number, number>()
    for (const tg of teachersWithGrades) {
      if (tg.teacher.active) {
        counts.set(tg.teacher.campus_id, (counts.get(tg.teacher.campus_id) || 0) + 1)
      }
    }
    return counts
  }, [teachersWithGrades])

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <Building2 className="h-6 w-6 text-primary" />
            {t('campuses.title')}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t('campuses.description')}
          </p>
        </div>
        <Button onClick={openCreateDialog} size="sm" className="gap-2">
          <Plus className="h-4 w-4" />
          {t('campuses.createCampus')}
        </Button>
      </div>

      {loadingCampuses ? (
        <div className="space-y-3">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-14 w-full" />
          <Skeleton className="h-14 w-full" />
          <Skeleton className="h-14 w-full" />
        </div>
      ) : campuses.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-12 border border-dashed rounded-lg bg-card text-center">
          <Building2 className="h-12 w-12 text-muted-foreground/50 mb-3" />
          <p className="text-sm font-medium text-foreground">{t('campuses.empty')}</p>
          <Button
            onClick={openCreateDialog}
            variant="outline"
            size="sm"
            className="mt-4 gap-2"
          >
            <Plus className="h-4 w-4" />
            {t('campuses.createCampus')}
          </Button>
        </div>
      ) : (
        <div className="rounded-md border bg-card shadow-sm overflow-hidden">
          <Table>
            <TableHeader className="bg-muted/40">
              <TableRow>
                <TableHead className="w-[140px]">{t('campuses.color')}</TableHead>
                <TableHead className="w-[120px]">{t('campuses.code')}</TableHead>
                <TableHead>{t('campuses.name')}</TableHead>
                <TableHead className="w-[180px] text-right">
                  {t('campuses.activeTeachers')}
                </TableHead>
                <TableHead className="w-[100px] text-right">
                  {t('teachers.actions')}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {campuses.map((campus) => {
                const count = activeTeachersPerCampus.get(campus.id) || 0
                return (
                  <TableRow key={campus.id} className="hover:bg-muted/50">
                    <TableCell>
                      <CampusChip name={campus.name} color={campus.color} />
                    </TableCell>
                    <TableCell className="font-mono text-xs font-semibold text-foreground">
                      {campus.code}
                    </TableCell>
                    <TableCell className="font-medium text-foreground">
                      {campus.name}
                    </TableCell>
                    <TableCell className="text-right text-xs text-muted-foreground">
                      <span className="inline-flex items-center px-2 py-0.5 rounded-full bg-muted font-medium text-foreground">
                        {t('campuses.teachersCount', { count })}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-foreground"
                          onClick={() => openEditDialog(campus)}
                          aria-label={t('common.edit')}
                        >
                          <Edit2 className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-destructive"
                          onClick={() => {
                            setDeleteCampusId(campus.id)
                            setInUseError(null)
                          }}
                          aria-label={t('common.delete')}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </div>
      )}

      {/* Create / Edit Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={(e) => e.preventDefault()}>
            <DialogHeader>
              <DialogTitle>
                {editingCampus ? t('campuses.editCampus') : t('campuses.createCampus')}
              </DialogTitle>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('campuses.code')}
                </label>
                <Input
                  {...register('code')}
                  placeholder={t('campuses.codePlaceholder')}
                  autoFocus
                />
                {errors.code && (
                  <p className="text-xs text-destructive">{t(errors.code.message ?? '')}</p>
                )}
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('campuses.name')}
                </label>
                <Input
                  {...register('name')}
                  placeholder={t('campuses.namePlaceholder')}
                />
                {errors.name && (
                  <p className="text-xs text-destructive">{t(errors.name.message ?? '')}</p>
                )}
              </div>

              <div className="space-y-2">
                <label className="text-xs font-medium text-foreground">
                  {t('campuses.selectColor')}
                </label>
                <div className="grid grid-cols-4 gap-2">
                  {CAMPUS_PALETTE_OPTIONS.map((opt) => {
                    const isSelected = selectedColor === opt.key
                    const label = i18n.language === 'en' ? opt.labelEn : opt.labelVi
                    return (
                      <button
                        type="button"
                        key={opt.key}
                        onClick={() => setValue('color', opt.key)}
                        className={`flex items-center gap-2 p-2 rounded-md border text-xs text-left transition-all ${
                          isSelected
                            ? 'border-primary ring-2 ring-primary/20 bg-primary/5 font-semibold text-foreground'
                            : 'border-border hover:bg-muted/50 text-muted-foreground'
                        }`}
                      >
                        <span
                          className="w-3.5 h-3.5 rounded-full shrink-0 border border-black/10"
                          style={{ backgroundColor: opt.previewColor }}
                        />
                        <span className="truncate">{label}</span>
                      </button>
                    )
                  })}
                </div>
              </div>
            </div>

            <DialogFooter className="gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setDialogOpen(false)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="button"
                size="sm"
                onClick={handleSubmit(onSubmit)}
                disabled={createMutation.isPending || updateMutation.isPending}
              >
                {t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Alert Dialog */}
      <AlertDialog
        open={deleteCampusId !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteCampusId(null)
            setInUseError(null)
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('campuses.deleteConfirm')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('campuses.deleteConfirmDescription')}
            </AlertDialogDescription>
          </AlertDialogHeader>

          {inUseError && (
            <div className="flex items-start gap-2 p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20 my-2">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <span>{inUseError}</span>
            </div>
          )}

          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleteMutation.isPending}>
              {t('common.cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault()
                handleDelete()
              }}
              disabled={deleteMutation.isPending}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
