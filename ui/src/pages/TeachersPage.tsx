import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import {
  Plus,
  Edit2,
  Trash2,
  Users,
  Search,
  Filter,
  ArrowUpDown,
  AlertCircle,
  CheckCircle,
  Download,
  FileSpreadsheet,
  Award,
  Calculator,
} from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { api, type TeacherWithGrades, type Teacher } from '@/lib/api'
import { ImportWizardModal } from './teachers/ImportWizardModal'
import { QuotaPreviewModal } from './teachers/QuotaPreviewModal'
import { TeacherCompetenciesModal } from './teachers/TeacherCompetenciesModal'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
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
import { Checkbox } from '@/components/ui/checkbox'
import { Skeleton } from '@/components/ui/skeleton'
import { CampusChip } from '@/components/CampusChip'
import { CoveragePanel } from '@/components/CoveragePanel'
import {
  useSchoolYears,
  useCampuses,
  useGrades,
  useTeachers,
  useFeasibility,
  useCreateTeacher,
  useUpdateTeacher,
  useDeleteTeacher,
  useDeactivateTeacher,
  useToggleTeacherGrade,
} from '@/lib/query/hooks'

const teacherFormSchema = z.object({
  fullName: z.string().min(1, 'Họ tên không được để trống'),
  campusId: z.number().min(1, 'Vui lòng chọn phân hiệu'),
  loadWeight: z.number().min(0).max(1),
  active: z.boolean(),
  note: z.string().optional(),
  code: z.string().optional(),
  displayName: z.string().optional(),
  quotaOverride: z.number().min(0).optional().nullable(),
  maxTasksPerExamOverride: z.number().min(0).optional().nullable(),
})

type TeacherFormValues = z.infer<typeof teacherFormSchema>

export const TeachersPage: React.FC = () => {
  const { t } = useTranslation()

  // Queries
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const schoolYearId = currentYear?.id

  const { data: campuses = [] } = useCampuses()
  const { data: grades = [] } = useGrades()
  const { data: teachersWithGrades = [], isLoading: loadingTeachers } =
    useTeachers(schoolYearId)
  const { data: feasibility } = useFeasibility(schoolYearId)

  // Mutations
  const createTeacherMutation = useCreateTeacher(schoolYearId ?? 0)
  const updateTeacherMutation = useUpdateTeacher(schoolYearId ?? 0)
  const deleteTeacherMutation = useDeleteTeacher(schoolYearId ?? 0)
  const deactivateTeacherMutation = useDeactivateTeacher(schoolYearId ?? 0)
  const toggleGradeMutation = useToggleTeacherGrade(schoolYearId ?? 0)

  // Filters & Search
  const [searchTerm, setSearchTerm] = React.useState('')
  const [campusFilter, setCampusFilter] = React.useState<string>('all')
  const [gradeFilter, setGradeFilter] = React.useState<string>('all')
  const [activeFilter, setActiveFilter] = React.useState<string>('all')
  const [sortField, setSortField] = React.useState<'name' | 'campus' | 'load' | 'active'>(
    'name',
  )
  const [sortAsc, setSortAsc] = React.useState(true)

  // Dialog State
  const queryClient = useQueryClient()
  const [importWizardOpen, setImportWizardOpen] = React.useState(false)
  const [dialogOpen, setDialogOpen] = React.useState(false)
  const [editingTeacher, setEditingTeacher] = React.useState<Teacher | null>(null)
  const [selectedGradeIds, setSelectedGradeIds] = React.useState<number[]>([])
  const [competenciesTeacher, setCompetenciesTeacher] = React.useState<Teacher | null>(null)
  const [quotaPreviewOpen, setQuotaPreviewOpen] = React.useState(false)

  const handleDownloadTemplate = async () => {
    try {
      let targetPath = 'mau-nhap-du-lieu.xlsx'
      try {
        const { save } = await import('@tauri-apps/plugin-dialog')
        const chosen = await save({
          defaultPath: 'mau-nhap-du-lieu.xlsx',
          filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
        })
        if (!chosen) return
        targetPath = chosen
      } catch {
        // Fallback for tests/browser
      }
      await api.generateImportTemplate(targetPath)
      toast.success(t('import.downloadTemplate') + ' thành công!')
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi tạo tệp mẫu: ${msg}`)
    }
  }

  // Delete State
  const [deleteTeacherId, setDeleteTeacherId] = React.useState<number | null>(null)
  const [inUseError, setInUseError] = React.useState<boolean>(false)

  const {
    register,
    handleSubmit,
    setValue,
    watch,
    reset,
    formState: { errors },
  } = useForm<TeacherFormValues>({
    resolver: zodResolver(teacherFormSchema),
    defaultValues: {
      fullName: '',
      campusId: campuses[0]?.id || 1,
      loadWeight: 1.0,
      active: true,
      note: '',
    },
  })

  const formCampusId = watch('campusId')
  const formLoadWeight = watch('loadWeight')
  const formActive = watch('active')

  const openCreateDialog = () => {
    setEditingTeacher(null)
    setSelectedGradeIds([])
    reset({
      fullName: '',
      campusId: campuses[0]?.id || 1,
      loadWeight: 1.0,
      active: true,
      note: '',
      code: '',
      displayName: '',
      quotaOverride: undefined,
      maxTasksPerExamOverride: undefined,
    })
    setDialogOpen(true)
  }

  const openEditDialog = (tg: TeacherWithGrades) => {
    setEditingTeacher(tg.teacher)
    setSelectedGradeIds([...tg.grade_ids])
    reset({
      fullName: tg.teacher.full_name,
      campusId: tg.teacher.campus_id,
      loadWeight: tg.teacher.load_weight,
      active: tg.teacher.active,
      note: tg.teacher.note || '',
      code: tg.teacher.code || '',
      displayName: tg.teacher.display_name || '',
      quotaOverride: tg.teacher.quota_override ?? undefined,
      maxTasksPerExamOverride: tg.teacher.max_tasks_per_exam_override ?? undefined,
    })
    setDialogOpen(true)
  }

  const onSubmit = async (values: TeacherFormValues) => {
    if (editingTeacher) {
      await updateTeacherMutation.mutateAsync({
        teacher: {
          ...editingTeacher,
          full_name: values.fullName.trim(),
          campus_id: values.campusId,
          load_weight: values.loadWeight,
          active: values.active,
          note: values.note?.trim() || null,
          code: values.code?.trim() || undefined,
          display_name: values.displayName?.trim() || undefined,
          quota_override: values.quotaOverride ?? undefined,
          max_tasks_per_exam_override: values.maxTasksPerExamOverride ?? undefined,
        },
        gradeIds: selectedGradeIds,
      })
    } else {
      await createTeacherMutation.mutateAsync({
        input: {
          full_name: values.fullName.trim(),
          campus_id: values.campusId,
          load_weight: values.loadWeight,
          active: values.active,
          note: values.note?.trim() || null,
          code: values.code?.trim() || undefined,
          display_name: values.displayName?.trim() || undefined,
          quota_override: values.quotaOverride ?? undefined,
          max_tasks_per_exam_override: values.maxTasksPerExamOverride ?? undefined,
        },
        gradeIds: selectedGradeIds,
      })
    }
    setDialogOpen(false)
  }

  const handleDelete = async () => {
    if (!deleteTeacherId) return
    setInUseError(false)
    try {
      await deleteTeacherMutation.mutateAsync(deleteTeacherId)
      setDeleteTeacherId(null)
    } catch (err: unknown) {
      const errorObj = err as { code?: string }
      if (errorObj?.code === 'teacher_in_use') {
        setInUseError(true)
      }
    }
  }

  const handleDeactivateInstead = async () => {
    if (!deleteTeacherId) return
    await deactivateTeacherMutation.mutateAsync(deleteTeacherId)
    setDeleteTeacherId(null)
    setInUseError(false)
  }

  // Inline grade toggle handler
  const handleToggleGrade = (tg: TeacherWithGrades, gradeId: number) => {
    const currentGrades = tg.grade_ids
    const nextGrades = currentGrades.includes(gradeId)
      ? currentGrades.filter((id) => id !== gradeId)
      : [...currentGrades, gradeId]

    toggleGradeMutation.mutate({
      teacherId: tg.teacher.id,
      gradeIds: nextGrades,
    })
  }

  // Inline active toggle handler
  const handleToggleActive = (teacher: Teacher, active: boolean) => {
    updateTeacherMutation.mutate({
      teacher: { ...teacher, active },
    })
  }

  // Inline load weight handler
  const handleLoadWeightChange = (teacher: Teacher, weight: number) => {
    updateTeacherMutation.mutate({
      teacher: { ...teacher, load_weight: weight },
    })
  }

  // Campus map lookup
  const campusMap = React.useMemo(() => {
    const map = new Map<number, (typeof campuses)[0]>()
    for (const c of campuses) {
      map.set(c.id, c)
    }
    return map
  }, [campuses])

  // Filtered & Sorted list
  const filteredTeachers = React.useMemo(() => {
    return teachersWithGrades
      .filter((tg) => {
        // Name search
        if (
          searchTerm.trim() &&
          !tg.teacher.full_name.toLowerCase().includes(searchTerm.toLowerCase().trim())
        ) {
          return false
        }
        // Campus filter
        if (campusFilter !== 'all' && tg.teacher.campus_id !== Number(campusFilter)) {
          return false
        }
        // Grade filter
        if (gradeFilter !== 'all' && !tg.grade_ids.includes(Number(gradeFilter))) {
          return false
        }
        // Active filter
        if (activeFilter === 'active' && !tg.teacher.active) {
          return false
        }
        if (activeFilter === 'inactive' && tg.teacher.active) {
          return false
        }
        return true
      })
      .sort((a, b) => {
        let cmp = 0
        if (sortField === 'name') {
          cmp = a.teacher.full_name.localeCompare(b.teacher.full_name, 'vi')
        } else if (sortField === 'campus') {
          const cA = campusMap.get(a.teacher.campus_id)?.name || ''
          const cB = campusMap.get(b.teacher.campus_id)?.name || ''
          cmp = cA.localeCompare(cB, 'vi')
        } else if (sortField === 'load') {
          cmp = a.teacher.load_weight - b.teacher.load_weight
        } else if (sortField === 'active') {
          cmp = (a.teacher.active ? 1 : 0) - (b.teacher.active ? 1 : 0)
        }
        return sortAsc ? cmp : -cmp
      })
  }, [
    teachersWithGrades,
    searchTerm,
    campusFilter,
    gradeFilter,
    activeFilter,
    sortField,
    sortAsc,
    campusMap,
  ])

  const toggleSort = (field: typeof sortField) => {
    if (sortField === field) {
      setSortAsc(!sortAsc)
    } else {
      setSortField(field)
      setSortAsc(true)
    }
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <Users className="h-6 w-6 text-primary" />
            {t('teachers.title')}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t('teachers.description')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            data-testid="quota-preview-btn"
            variant="outline"
            size="sm"
            onClick={() => setQuotaPreviewOpen(true)}
            className="gap-2"
          >
            <Calculator className="h-4 w-4 text-primary" />
            {t('quota.previewBtn') || 'Xem chỉ tiêu'}
          </Button>
          <Button
            data-testid="competencies-btn"
            variant="outline"
            size="sm"
            onClick={() => {
              if (teachersWithGrades.length > 0) {
                setCompetenciesTeacher(teachersWithGrades[0].teacher)
              }
            }}
            className="gap-2"
          >
            <Award className="h-4 w-4 text-primary" />
            {t('competencies.title') || 'Chuyên môn'}
          </Button>
          <Button
            data-testid="download-template-btn"
            variant="outline"
            size="sm"
            onClick={handleDownloadTemplate}
            className="gap-2"
          >
            <Download className="h-4 w-4" />
            {t('import.downloadTemplate')}
          </Button>
          <Button
            data-testid="import-excel-btn"
            variant="outline"
            size="sm"
            onClick={() => setImportWizardOpen(true)}
            className="gap-2"
          >
            <FileSpreadsheet className="h-4 w-4" />
            {t('import.importExcel')}
          </Button>
          <Button
            data-testid="create-teacher-btn"
            onClick={openCreateDialog}
            size="sm"
            className="gap-2"
          >
            <Plus className="h-4 w-4" />
            {t('teachers.createTeacher')}
          </Button>
        </div>
      </div>

      {/* Coverage Panel Above Table */}
      <CoveragePanel
        campuses={campuses}
        grades={grades}
        teachersWithGrades={teachersWithGrades}
        feasibility={feasibility}
      />

      {/* Filters and Search Bar */}
      <div className="flex flex-col md:flex-row items-stretch md:items-center gap-3 bg-card p-3 rounded-lg border border-border shadow-sm">
        <div className="relative flex-1">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder={t('teachers.searchPlaceholder')}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-8 text-xs h-9"
          />
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {/* Campus Filter */}
          <Select value={campusFilter} onValueChange={setCampusFilter}>
            <SelectTrigger className="w-[140px] text-xs h-9">
              <Filter className="h-3.5 w-3.5 mr-1 text-muted-foreground" />
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

          {/* Grade Filter */}
          <Select value={gradeFilter} onValueChange={setGradeFilter}>
            <SelectTrigger className="w-[120px] text-xs h-9">
              <SelectValue placeholder={t('teachers.filterGrade')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('teachers.allGrades')}</SelectItem>
              {grades.map((g) => (
                <SelectItem key={g.id} value={g.id.toString()}>
                  {g.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* Active Filter */}
          <Select value={activeFilter} onValueChange={setActiveFilter}>
            <SelectTrigger className="w-[130px] text-xs h-9">
              <SelectValue placeholder={t('teachers.filterActive')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('teachers.allStatuses')}</SelectItem>
              <SelectItem value="active">{t('teachers.activeOnly')}</SelectItem>
              <SelectItem value="inactive">{t('teachers.inactiveOnly')}</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Teachers Table */}
      {loadingTeachers ? (
        <div className="space-y-3">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-14 w-full" />
          <Skeleton className="h-14 w-full" />
          <Skeleton className="h-14 w-full" />
        </div>
      ) : filteredTeachers.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-12 border border-dashed rounded-lg bg-card text-center">
          <Users className="h-12 w-12 text-muted-foreground/50 mb-3" />
          <p className="text-sm font-medium text-foreground">{t('teachers.empty')}</p>
        </div>
      ) : (
        <div className="rounded-md border bg-card shadow-sm overflow-hidden">
          <Table>
            <TableHeader className="bg-muted/40">
              <TableRow>
                <TableHead className="w-[100px]">{t('teachers.code')}</TableHead>
                <TableHead
                  className="cursor-pointer hover:text-foreground w-[200px]"
                  onClick={() => toggleSort('name')}
                >
                  <div className="flex items-center gap-1.5">
                    <span>{t('teachers.fullName')}</span>
                    <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                </TableHead>
                <TableHead
                  className="cursor-pointer hover:text-foreground w-[160px]"
                  onClick={() => toggleSort('campus')}
                >
                  <div className="flex items-center gap-1.5">
                    <span>{t('teachers.campus')}</span>
                    <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                </TableHead>
                <TableHead className="w-[180px]">{t('teachers.gradesTaught')}</TableHead>
                <TableHead
                  className="cursor-pointer hover:text-foreground w-[140px]"
                  onClick={() => toggleSort('load')}
                >
                  <div className="flex items-center gap-1.5">
                    <span>{t('teachers.loadWeight')}</span>
                    <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                </TableHead>
                <TableHead
                  className="cursor-pointer hover:text-foreground w-[100px] text-center"
                  onClick={() => toggleSort('active')}
                >
                  <div className="flex items-center justify-center gap-1.5">
                    <span>{t('teachers.active')}</span>
                    <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                </TableHead>
                <TableHead>{t('teachers.note')}</TableHead>
                <TableHead className="w-[90px] text-right">
                  {t('teachers.actions')}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredTeachers.map((tg) => {
                const campus = campusMap.get(tg.teacher.campus_id)
                return (
                  <TableRow
                    key={tg.teacher.id}
                    className={`hover:bg-muted/50 ${
                      !tg.teacher.active ? 'opacity-60 bg-muted/20' : ''
                    }`}
                  >
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {tg.teacher.code ? (
                        <Badge variant="outline" className="font-mono text-xs">
                          {tg.teacher.code}
                        </Badge>
                      ) : (
                        '—'
                      )}
                    </TableCell>
                    <TableCell className="font-semibold text-foreground">
                      {tg.teacher.full_name}
                    </TableCell>
                    <TableCell>
                      {campus && <CampusChip name={campus.name} color={campus.color} />}
                    </TableCell>

                    {/* Inline Grades Toggle Chips */}
                    <TableCell>
                      <div className="flex items-center gap-1.5">
                        {grades.map((grade) => {
                          const isTaught = tg.grade_ids.includes(grade.id)
                          return (
                            <button
                              key={grade.id}
                              type="button"
                              onClick={() => handleToggleGrade(tg, grade.id)}
                              className={`px-2 py-0.5 rounded text-[11px] font-semibold transition-all border ${
                                isTaught
                                  ? 'bg-primary text-primary-foreground border-primary shadow-xs'
                                  : 'bg-muted/50 text-muted-foreground border-border hover:bg-muted'
                              }`}
                              title={`${grade.name}: ${
                                isTaught ? 'Đang phân công' : 'Chưa phân công'
                              }`}
                            >
                              {grade.code}
                            </button>
                          )
                        })}
                      </div>
                    </TableCell>

                    {/* Load Weight Inline Select */}
                    <TableCell>
                      <Select
                        value={tg.teacher.load_weight.toString()}
                        onValueChange={(val) =>
                          handleLoadWeightChange(tg.teacher, Number(val))
                        }
                      >
                        <SelectTrigger className="h-7 text-xs w-[110px] bg-background">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="1">1.0</SelectItem>
                          <SelectItem value="0.75">0.75</SelectItem>
                          <SelectItem value="0.5">0.5</SelectItem>
                          <SelectItem value="0.25">0.25</SelectItem>
                          <SelectItem value="0">
                            {t('teachers.loadWeightExempt')}
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </TableCell>

                    {/* Active Switch */}
                    <TableCell className="text-center">
                      <div className="flex justify-center">
                        <Switch
                          checked={tg.teacher.active}
                          onCheckedChange={(checked) =>
                            handleToggleActive(tg.teacher, checked)
                          }
                          aria-label={t('teachers.active')}
                        />
                      </div>
                    </TableCell>

                    <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate">
                      {tg.teacher.note || '—'}
                    </TableCell>

                    {/* Actions */}
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-foreground"
                          onClick={() => setCompetenciesTeacher(tg.teacher)}
                          title={t('competencies.editTitle') || 'Phân công chuyên môn'}
                          data-testid={`teacher-competencies-btn-${tg.teacher.id}`}
                        >
                          <Award className="h-3.5 w-3.5 text-primary" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-foreground"
                          onClick={() => openEditDialog(tg)}
                          aria-label={t('common.edit')}
                        >
                          <Edit2 className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-destructive"
                          onClick={() => {
                            setDeleteTeacherId(tg.teacher.id)
                            setInUseError(false)
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
          <form onSubmit={handleSubmit(onSubmit)}>
            <DialogHeader>
              <DialogTitle>
                {editingTeacher ? t('teachers.editTeacher') : t('teachers.createTeacher')}
              </DialogTitle>
            </DialogHeader>

            <div className="space-y-4 py-4">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div className="sm:col-span-1 space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    {t('teachers.code')}
                  </label>
                  <Input
                    {...register('code')}
                    placeholder={t('teachers.codePlaceholder')}
                    className="font-mono text-xs"
                    data-testid="teacher-code-input"
                  />
                </div>
                <div className="sm:col-span-2 space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    {t('teachers.fullName')}
                  </label>
                  <Input
                    {...register('fullName')}
                    placeholder={t('teachers.fullNamePlaceholder')}
                    autoFocus
                    data-testid="teacher-name-input"
                  />
                  {errors.fullName && (
                    <p className="text-xs text-destructive">{errors.fullName.message}</p>
                  )}
                </div>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('teachers.campus')}
                </label>
                <Select
                  value={formCampusId.toString()}
                  onValueChange={(val) => setValue('campusId', Number(val))}
                >
                  <SelectTrigger className="w-full text-xs">
                    <SelectValue placeholder={t('teachers.selectCampus')} />
                  </SelectTrigger>
                  <SelectContent>
                    {campuses.map((c) => (
                      <SelectItem key={c.id} value={c.id.toString()}>
                        {c.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Grades Taught Multi-Check */}
              <div className="space-y-2">
                <label className="text-xs font-medium text-foreground">
                  {t('teachers.gradesTaught')} (Năm học {currentYear?.name})
                </label>
                <div className="flex items-center gap-4">
                  {grades.map((grade) => {
                    const isChecked = selectedGradeIds.includes(grade.id)
                    return (
                      <div key={grade.id} className="flex items-center space-x-2">
                        <Checkbox
                          id={`grade-check-${grade.id}`}
                          checked={isChecked}
                          onCheckedChange={(checked) => {
                            if (checked) {
                              setSelectedGradeIds([...selectedGradeIds, grade.id])
                            } else {
                              setSelectedGradeIds(
                                selectedGradeIds.filter((id) => id !== grade.id),
                              )
                            }
                          }}
                        />
                        <label
                          htmlFor={`grade-check-${grade.id}`}
                          className="text-xs font-medium text-foreground cursor-pointer"
                        >
                          {grade.name}
                        </label>
                      </div>
                    )
                  })}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    {t('teachers.loadWeight')}
                  </label>
                  <Select
                    value={formLoadWeight.toString()}
                    onValueChange={(val) => setValue('loadWeight', Number(val))}
                  >
                    <SelectTrigger className="w-full text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1.0</SelectItem>
                      <SelectItem value="0.75">0.75</SelectItem>
                      <SelectItem value="0.5">0.5</SelectItem>
                      <SelectItem value="0.25">0.25</SelectItem>
                      <SelectItem value="0">{t('teachers.loadWeightExempt')}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    {t('teachers.active')}
                  </label>
                  <div className="pt-2">
                    <Switch
                      checked={formActive}
                      onCheckedChange={(checked) => setValue('active', checked)}
                    />
                  </div>
                </div>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('teachers.note')}
                </label>
                <Input
                  {...register('note')}
                  placeholder={t('teachers.notePlaceholder')}
                />
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
                type="submit"
                size="sm"
                onClick={handleSubmit(onSubmit)}
                disabled={
                  createTeacherMutation.isPending || updateTeacherMutation.isPending
                }
              >
                {t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete / Deactivate AlertDialog */}
      <AlertDialog
        open={deleteTeacherId !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTeacherId(null)
            setInUseError(false)
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('teachers.deleteConfirm')}</AlertDialogTitle>
            <AlertDialogDescription>
              {inUseError
                ? t('teachers.teacherInUseHint')
                : t('campuses.deleteConfirmDescription')}
            </AlertDialogDescription>
          </AlertDialogHeader>

          {inUseError && (
            <div className="flex items-start gap-2 p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-400 border border-amber-500/20 my-2">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <span>{t('teachers.teacherInUseHint')}</span>
            </div>
          )}

          <AlertDialogFooter className="gap-2">
            <AlertDialogCancel disabled={deleteTeacherMutation.isPending}>
              {t('common.cancel')}
            </AlertDialogCancel>

            {inUseError ? (
              <Button
                variant="default"
                size="sm"
                onClick={handleDeactivateInstead}
                disabled={deactivateTeacherMutation.isPending}
                className="gap-1.5"
              >
                <CheckCircle className="h-4 w-4" />
                {t('teachers.deactivateInstead')}
              </Button>
            ) : (
              <AlertDialogAction
                onClick={(e) => {
                  e.preventDefault()
                  handleDelete()
                }}
                disabled={deleteTeacherMutation.isPending}
                className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              >
                {t('common.delete')}
              </AlertDialogAction>
            )}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <ImportWizardModal
        open={importWizardOpen}
        onOpenChange={setImportWizardOpen}
        schoolYearId={schoolYearId ?? 0}
        onSuccess={() => {
          void queryClient.invalidateQueries({ queryKey: ['teachers'] })
          void queryClient.invalidateQueries({ queryKey: ['campuses'] })
          void queryClient.invalidateQueries({ queryKey: ['feasibility'] })
          void queryClient.invalidateQueries({ queryKey: ['unavailability'] })
        }}
      />

      <QuotaPreviewModal
        open={quotaPreviewOpen}
        onOpenChange={setQuotaPreviewOpen}
        schoolYearId={schoolYearId ?? 0}
      />

      <TeacherCompetenciesModal
        open={competenciesTeacher !== null}
        onOpenChange={(open) => !open && setCompetenciesTeacher(null)}
        teacher={competenciesTeacher}
        schoolYearId={schoolYearId ?? 0}
      />
    </div>
  )
}
