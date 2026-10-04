import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  type Assignment,
  type PlanDetails,
  type SlotRef,
  api,
  isTauriEnvironment,
} from '@/lib/api'
import { useNavigate } from 'react-router-dom'
import { getCampusDotColor } from '@/lib/theme/campus-colors'
import {
  useSchoolYears,
  usePlans,
  usePlanDetails,
  usePlanStatus,
  useCampuses,
  useGrades,
  useExams,
  useSubjects,
  useTeachers,
  useLocks,
  useFeasibility,
  useUpdatePlanAssignments,
  useCreateManualCopy,
  useCreateLock,
} from '@/lib/query/hooks'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  Play,
  History,
  GitCompare,
  Save,
  Undo2,
  Redo2,
  Copy,
  Star,
  AlertTriangle,
  FileSpreadsheet,
  Printer,
  RefreshCw,
} from 'lucide-react'
import { toast } from 'sonner'
import { RunOptimizeDialog } from './assignments/RunOptimizeDialog'
import { PlansHistoryList } from './assignments/PlansHistoryList'
import { PlanMatrixView } from './assignments/PlanMatrixView'
import { QPlanGrid } from './assignments/QPlanGrid'
import { TeacherFocusPanel } from './assignments/TeacherFocusPanel'
import { PlanCompareModal } from './assignments/PlanCompareModal'
import { ReoptimizeDialog } from './assignments/ReoptimizeDialog'
import { FeasibilitySheet } from '@/components/FeasibilitySheet'

export function AssignmentsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const isTauri = isTauriEnvironment()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const schoolYearId = currentYear?.id ?? 1

  // Master Data
  const { data: campuses = [] } = useCampuses()
  const { data: grades = [] } = useGrades()
  const { data: exams = [] } = useExams(schoolYearId)
  const { data: subjects = [] } = useSubjects(schoolYearId)
  const { data: teachers = [] } = useTeachers(schoolYearId)
  const { data: locks = [] } = useLocks(schoolYearId)
  const { data: feasibilityReport } = useFeasibility(schoolYearId)

  // Plans Data
  const { data: plans = [], refetch: refetchPlans } = usePlans(schoolYearId)
  const [selectedPlanId, setSelectedPlanId] = React.useState<number | null>(null)

  // Default to final plan or first plan, and sync if plans list changes or plan is deleted
  React.useEffect(() => {
    if (plans.length > 0) {
      if (selectedPlanId === null || !plans.some((p) => p.id === selectedPlanId)) {
        const finalPlan = plans.find((p) => p.is_final)
        setSelectedPlanId(finalPlan ? finalPlan.id : plans[0].id)
      }
    } else {
      setSelectedPlanId(null)
    }
  }, [plans, selectedPlanId])

  // Clear undo/redo/dirty state when selected plan changes
  React.useEffect(() => {
    setUndoStack([])
    setRedoStack([])
    setIsDirty(false)
    setKeptSlots([])
    setFocusedTeacherId(null)
  }, [selectedPlanId])

  // Listen to application-wide restore event to reset in-memory state
  React.useEffect(() => {
    const handleRestoreReset = () => {
      setSelectedPlanId(null)
      setCurrentAssignments([])
      setUndoStack([])
      setRedoStack([])
      setIsDirty(false)
      setKeptSlots([])
      setFocusedTeacherId(null)
    }

    window.addEventListener('exampanel:restore', handleRestoreReset)
    return () => {
      window.removeEventListener('exampanel:restore', handleRestoreReset)
    }
  }, [])

  const { data: loadedPlanDetails } = usePlanDetails(selectedPlanId)
  const { data: planStatus } = usePlanStatus(selectedPlanId)

  // Local editing state with undo/redo
  const [currentAssignments, setCurrentAssignments] = React.useState<Assignment[]>([])
  const [undoStack, setUndoStack] = React.useState<Assignment[][]>([])
  const [redoStack, setRedoStack] = React.useState<Assignment[][]>([])
  const [isDirty, setIsDirty] = React.useState(false)

  // Keep slots for B3 re-optimization
  const [keptSlots, setKeptSlots] = React.useState<SlotRef[]>([])

  // Focused teacher
  const [focusedTeacherId, setFocusedTeacherId] = React.useState<number | null>(null)

  // Modals & Panels
  const [showRunDialog, setShowRunDialog] = React.useState(false)
  const [showHistory, setShowHistory] = React.useState(false)
  const [showCompareModal, setShowCompareModal] = React.useState(false)
  const [showReoptimizeDialog, setShowReoptimizeDialog] = React.useState(false)
  const [showFeasibilitySheet, setShowFeasibilitySheet] = React.useState(false)

  // View mode toggle: 'grid' (Bảng tổ) | 'detail' (Chi tiết)
  const [viewMode, setViewMode] = React.useState<'grid' | 'detail'>(() => {
    return (localStorage.getItem('exam_panel_assignment_view_mode') as 'grid' | 'detail') || 'grid'
  })

  const handleViewModeChange = (mode: 'grid' | 'detail') => {
    setViewMode(mode)
    localStorage.setItem('exam_panel_assignment_view_mode', mode)
  }

  // Mutations
  const updateAssignmentsMutation = useUpdatePlanAssignments(schoolYearId)
  const duplicateMutation = useCreateManualCopy(schoolYearId)
  const createLockMutation = useCreateLock(schoolYearId)

  // Sync loaded plan details to local assignments
  React.useEffect(() => {
    if (loadedPlanDetails) {
      setCurrentAssignments(loadedPlanDetails.assignments)
      setUndoStack([])
      setRedoStack([])
      setIsDirty(false)
      setKeptSlots([])
    }
  }, [loadedPlanDetails])

  const activePlan = plans.find((p) => p.id === selectedPlanId)
  const isEditable = activePlan ? ['manual', 'duplicate'].includes(activePlan.source) && !activePlan.is_final : false

  // Apply assignment changes with undo/redo recording
  const handleUpdateAssignments = (newAssignments: Assignment[]) => {
    setUndoStack((prev) => [...prev, currentAssignments])
    setRedoStack([])
    setCurrentAssignments(newAssignments)
    setIsDirty(true)
  }

  const handleUndo = React.useCallback(() => {
    if (undoStack.length === 0) return
    const prev = undoStack[undoStack.length - 1]
    setUndoStack((s) => s.slice(0, s.length - 1))
    setRedoStack((s) => [...s, currentAssignments])
    setCurrentAssignments(prev)
    setIsDirty(true)
  }, [undoStack, currentAssignments])

  const handleRedo = React.useCallback(() => {
    if (redoStack.length === 0) return
    const next = redoStack[redoStack.length - 1]
    setRedoStack((s) => s.slice(0, s.length - 1))
    setUndoStack((s) => [...s, currentAssignments])
    setCurrentAssignments(next)
    setIsDirty(true)
  }, [redoStack, currentAssignments])

  // Keyboard shortcuts (Ctrl+Z, Ctrl+Y)
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
        if (e.shiftKey) {
          handleRedo()
        } else {
          handleUndo()
        }
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'y') {
        handleRedo()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleUndo, handleRedo])

  // Save changes
  const handleSave = async () => {
    if (!selectedPlanId) return
    await updateAssignmentsMutation.mutateAsync({
      id: selectedPlanId,
      assignments: currentAssignments,
    })
    setIsDirty(false)
  }

  const handleDiscard = () => {
    if (loadedPlanDetails) {
      setCurrentAssignments(loadedPlanDetails.assignments)
      setUndoStack([])
      setRedoStack([])
      setIsDirty(false)
    }
  }

  // Duplicate optimizer plan into an editable draft
  const handleCreateEditableCopy = async () => {
    if (!activePlan) return
    const copyName = `${activePlan.name} (Chỉnh sửa)`
    const newId = await duplicateMutation.mutateAsync({
      id: activePlan.id,
      name: copyName,
    })
    setSelectedPlanId(newId)
    toast.success(t('assignments.duplicateForEdit'))
  }

  const handleToggleKeepSlot = (slot: SlotRef) => {
    setKeptSlots((prev) => {
      const exists = prev.some(
        (s) =>
          s.exam_id === slot.exam_id &&
          s.grade_id === slot.grade_id &&
          s.subject_id === slot.subject_id &&
          s.role === slot.role &&
          s.position === slot.position,
      )
      if (exists) {
        return prev.filter(
          (s) =>
            !(
              s.exam_id === slot.exam_id &&
              s.grade_id === slot.grade_id &&
              s.subject_id === slot.subject_id &&
              s.role === slot.role &&
              s.position === slot.position
            ),
        )
      } else {
        return [...prev, slot]
      }
    })
  }

  const handleCreateLock = async (slot: SlotRef, teacherId: number, kind: 'pin' | 'forbid') => {
    await createLockMutation.mutateAsync({
      exam_id: slot.exam_id,
      grade_id: slot.grade_id,
      subject_id: slot.subject_id,
      teacher_id: teacherId,
      role: slot.role,
      kind,
    })
    toast.success(
      kind === 'pin' ? t('assignments.pinTeacherSlot') : t('assignments.forbidTeacherSlot'),
    )
  }

  const handleExportExcel = async () => {
    if (!activePlan) return
    if (!isTauri) {
      toast.info(t('export.exportMockDisabled'))
      return
    }
    try {
      const sanitizedYear = currentYear?.name.replace(/[^a-zA-Z0-9_-]/g, '_') || 'nam-hoc'
      const sanitizedPlan = activePlan.name.replace(/[^a-zA-Z0-9_-]/g, '_') || 'phuong-an'
      const defaultName = `phan-cong-${sanitizedYear}-${sanitizedPlan}.xlsx`
      const { save } = await import('@tauri-apps/plugin-dialog')
      const chosenPath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
      })
      if (!chosenPath) return
      await api.exportPlanExcel(activePlan.id, chosenPath)
      toast.success(t('export.exportSuccess'))
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi xuất Excel: ${msg}`)
    }
  }

  const activePlanDetails: PlanDetails | null = loadedPlanDetails
    ? {
        ...loadedPlanDetails,
        assignments: currentAssignments,
      }
    : null

  return (
    <div className="space-y-6" data-testid="assignments-page">
      {/* Top Workspace Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <span>{t('assignments.title')}</span>
            <Badge variant="outline" className="text-xs">
              {currentYear?.name}
            </Badge>
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            {t('assignments.description')}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {/* Run Optimizer Button */}
          <Button
            size="sm"
            onClick={() => setShowRunDialog(true)}
            className="gap-1.5"
            data-testid="run-optimizer-button"
          >
            <Play className="h-4 w-4" />
            {t('assignments.run')}
          </Button>

          {/* Compare Button */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowCompareModal(true)}
            disabled={plans.length < 2}
            className="gap-1.5"
            data-testid="compare-plans-button"
          >
            <GitCompare className="h-4 w-4" />
            {t('assignments.compare')}
          </Button>

          {/* Plans History Toggle */}
          <Button
            variant={showHistory ? 'default' : 'outline'}
            size="sm"
            onClick={() => setShowHistory(!showHistory)}
            className="gap-1.5"
            data-testid="history-plans-button"
          >
            <History className="h-4 w-4" />
            {t('assignments.history')} ({plans.length})
          </Button>

          {/* View Mode Toggle: Bảng tổ | Chi tiết */}
          <div className="flex items-center rounded-md border border-border p-0.5 bg-muted/40">
            <button
              type="button"
              onClick={() => handleViewModeChange('grid')}
              className={`px-2.5 py-1 rounded text-xs font-semibold transition-all ${
                viewMode === 'grid'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
              data-testid="view-toggle-grid"
            >
              {t('assignments.viewModeGrid') || 'Bảng tổ'}
            </button>
            <button
              type="button"
              onClick={() => handleViewModeChange('detail')}
              className={`px-2.5 py-1 rounded text-xs font-semibold transition-all ${
                viewMode === 'detail'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
              data-testid="view-toggle-detail"
            >
              {t('assignments.viewModeDetail') || 'Chi tiết'}
            </button>
          </div>
        </div>
      </div>

      {/* Plans History Drawer / List view */}
      {showHistory && (
        <div className="p-4 rounded-lg border border-border bg-card shadow-sm space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
              <History className="h-4 w-4 text-primary" />
              <span>{t('assignments.history')}</span>
            </h3>
            <Button variant="ghost" size="sm" onClick={() => setShowHistory(false)}>
              Đóng
            </Button>
          </div>
          <PlansHistoryList
            plans={plans}
            activePlanId={selectedPlanId}
            schoolYearId={schoolYearId}
            onSelectPlan={(id) => {
              setSelectedPlanId(id)
              setShowHistory(false)
            }}
          />
        </div>
      )}

      {/* Main Workspace Body */}
      {plans.length === 0 ? (
        <div className="p-12 text-center rounded-lg border border-dashed border-border bg-card space-y-4">
          <p className="text-sm text-muted-foreground">{t('assignments.noPlans')}</p>
          <Button onClick={() => setShowRunDialog(true)} className="gap-2">
            <Play className="h-4 w-4" />
            {t('assignments.startRun')}
          </Button>
        </div>
      ) : (
        <div className="space-y-4">
          {/* Active Plan Header & Controls */}
          {activePlan && (
              <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-lg bg-card border border-border">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="font-bold text-base text-foreground">
                    {activePlan.name}
                  </span>
                  {activePlan.is_final && (
                    <Badge variant="default" className="bg-emerald-600 hover:bg-emerald-700 text-white gap-1 text-xs">
                      <Star className="h-3 w-3 fill-current" />
                      {t('assignments.finalBadge')}
                    </Badge>
                  )}
                  {activePlan.is_stale && (
                    <Badge variant="destructive" className="gap-1 text-xs bg-amber-500/20 text-amber-600 dark:text-amber-400 border-amber-500/40">
                      <AlertTriangle className="h-3 w-3" />
                      {t('assignments.staleBadge')}
                    </Badge>
                  )}
                  <Badge variant="secondary" className="text-xs">
                    {activePlan.source === 'optimizer'
                      ? t('assignments.sourceOptimizer')
                      : activePlan.source === 'duplicate'
                      ? t('assignments.sourceDuplicate')
                      : t('assignments.sourceManual')}
                  </Badge>
                </div>

                {/* Edit Mode & Actions */}
                <div className="flex items-center gap-2">
                  {/* Optimizer plans: Show "Tạo bản chỉnh sửa" */}
                  {activePlan.source === 'optimizer' && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleCreateEditableCopy}
                      className="gap-1.5 text-xs"
                      data-testid="create-edit-copy-button"
                    >
                      <Copy className="h-3.5 w-3.5" />
                      {t('assignments.createEditCopy')}
                    </Button>
                  )}

                  {/* Manual/Duplicate plans: Show Edit controls */}
                  {isEditable && (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleUndo}
                        disabled={undoStack.length === 0}
                        title={t('assignments.undo')}
                        className="h-8 w-8 p-0"
                      >
                        <Undo2 className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleRedo}
                        disabled={redoStack.length === 0}
                        title={t('assignments.redo')}
                        className="h-8 w-8 p-0"
                      >
                        <Redo2 className="h-4 w-4" />
                      </Button>

                      {isDirty && (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={handleDiscard}
                            className="text-xs text-muted-foreground"
                          >
                            {t('assignments.discard')}
                          </Button>
                          <Button
                            size="sm"
                            onClick={handleSave}
                            className="gap-1.5 text-xs bg-emerald-600 hover:bg-emerald-700 text-white"
                            data-testid="save-plan-assignments-button"
                          >
                            <Save className="h-3.5 w-3.5" />
                            {t('assignments.save')}
                          </Button>
                        </>
                      )}
                    </>
                  )}

                  {/* Export Excel Button */}
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <span>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={handleExportExcel}
                            disabled={!isTauri}
                            className="gap-1.5 text-xs"
                            data-testid="export-excel-button"
                          >
                            <FileSpreadsheet className="h-3.5 w-3.5" />
                            {t('export.exportExcel')}
                          </Button>
                        </span>
                      </TooltipTrigger>
                      {!isTauri && (
                        <TooltipContent>
                          <p className="text-xs">{t('export.exportMockDisabled')}</p>
                        </TooltipContent>
                      )}
                    </Tooltip>
                  </TooltipProvider>

                  {/* Print Dropdown */}
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="gap-1.5 text-xs"
                        data-testid="print-menu-button"
                      >
                        <Printer className="h-3.5 w-3.5" />
                        {t('print.printBtn')}
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        data-testid="print-plan-item"
                        onClick={() => navigate(`/print/plan/${activePlan.id}`)}
                        className="gap-2 cursor-pointer text-xs"
                      >
                        <Printer className="h-3.5 w-3.5" />
                        {t('print.printPlan')}
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        data-testid="print-notices-item"
                        onClick={() => navigate(`/print/notices/${activePlan.id}`)}
                        className="gap-2 cursor-pointer text-xs"
                      >
                        <Printer className="h-3.5 w-3.5" />
                        {t('print.printNotices')}
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            )}

          {/* Conditional View: Q-Style Grid (Bảng tổ) vs Detailed Matrix (Chi tiết) */}
          {viewMode === 'grid' ? (
            activePlanDetails && (
              <QPlanGrid
                planDetails={activePlanDetails}
                planStatus={planStatus ?? null}
                exams={exams}
                grades={grades}
                subjects={subjects}
                teachers={teachers}
                campuses={campuses}
                locks={locks}
                isEditable={isEditable}
                focusedTeacherId={focusedTeacherId}
                keptSlots={keptSlots}
                onSelectTeacherFocus={setFocusedTeacherId}
                onToggleKeepSlot={handleToggleKeepSlot}
                onReoptimizeRemaining={() => setShowReoptimizeDialog(true)}
                onUpdateAssignments={handleUpdateAssignments}
                onCreateLock={handleCreateLock}
              />
            )
          ) : (
            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
              {/* Detailed Matrix View (3 columns on lg) */}
              <div className="lg:col-span-3 space-y-4">
                {activePlanDetails && (
                  <PlanMatrixView
                    planDetails={activePlanDetails}
                    planStatus={planStatus ?? null}
                    exams={exams}
                    grades={grades}
                    subjects={subjects}
                    teachers={teachers}
                    campuses={campuses}
                    locks={locks}
                    isEditable={isEditable}
                    focusedTeacherId={focusedTeacherId}
                    keptSlots={keptSlots}
                    onSelectTeacherFocus={setFocusedTeacherId}
                    onToggleKeepSlot={handleToggleKeepSlot}
                    onReoptimizeRemaining={() => setShowReoptimizeDialog(true)}
                    onUpdateAssignments={handleUpdateAssignments}
                    onCreateLock={handleCreateLock}
                  />
                )}
              </div>

              {/* Right Side Panel: Teacher Focus or Information (1 column on lg) */}
              <div className="space-y-4">
                {focusedTeacherId !== null ? (
                  <TeacherFocusPanel
                    teacherId={focusedTeacherId}
                    teachers={teachers}
                    campuses={campuses}
                    exams={exams}
                    grades={grades}
                    assignments={currentAssignments}
                    scoreReport={activePlanDetails?.score_report}
                    onClose={() => setFocusedTeacherId(null)}
                  />
                ) : (
                  <div className="p-4 rounded-lg border border-border bg-card shadow-sm space-y-3 text-xs">
                    <h4 className="font-semibold text-foreground text-sm flex items-center gap-1.5">
                      <span>Danh sách giáo viên</span>
                      <Badge variant="outline" className="text-xs">{teachers.length}</Badge>
                    </h4>
                    <p className="text-muted-foreground">
                      Nhấp vào tên giáo viên để xem chi tiết tải trọng và các vị trí được phân công trên ma trận.
                    </p>
                    <div className="max-h-[500px] overflow-y-auto space-y-1 pr-1">
                      {teachers.map((twg) => {
                        const tRec = twg.teacher
                        const campus = campuses.find((c) => c.id === tRec.campus_id)
                        const count = currentAssignments.filter(
                          (a) => a.teacher_id === tRec.id,
                        ).length

                        return (
                          <div
                            key={tRec.id}
                            onClick={() => setFocusedTeacherId(tRec.id)}
                            className="p-2 rounded border border-border bg-card hover:bg-accent/40 flex items-center justify-between cursor-pointer transition-colors"
                          >
                            <div className="flex items-center gap-1.5 min-w-0">
                              <div
                                className="w-2.5 h-2.5 rounded-full shrink-0"
                                style={{ backgroundColor: getCampusDotColor(campus?.color) }}
                              />
                              <span className="font-medium text-foreground truncate">
                                {tRec.full_name}
                              </span>
                            </div>
                            <Badge variant="secondary" className="text-[10px]">
                              {count} lượt
                            </Badge>
                          </div>
                        )
                      })}
                    </div>
                  </div>
                )}

                {/* Kept Slots Banner for B3 */}
                {keptSlots.length > 0 && (
                  <div className="p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-xs space-y-2">
                    <div className="flex items-center justify-between font-semibold text-amber-700 dark:text-amber-300">
                      <span>Đã chọn giữ: {keptSlots.length} ô</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-6 text-[10px] px-1 text-muted-foreground"
                        onClick={() => setKeptSlots([])}
                      >
                        Bỏ chọn tất cả
                      </Button>
                    </div>
                    <Button
                      size="sm"
                      onClick={() => setShowReoptimizeDialog(true)}
                      className="w-full text-xs gap-1.5 bg-amber-600 hover:bg-amber-700 text-white"
                      data-testid="reoptimize-kept-button"
                    >
                      <RefreshCw className="h-3.5 w-3.5" />
                      <span>{t('assignments.reoptimizeRest')}</span>
                    </Button>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Run Optimization Dialog (C1) */}
      <RunOptimizeDialog
        open={showRunDialog}
        onOpenChange={setShowRunDialog}
        schoolYearId={schoolYearId}
        onSuccess={(newIds) => {
          if (newIds.length > 0) {
            setSelectedPlanId(newIds[0])
            refetchPlans()
          }
        }}
        onOpenFeasibility={() => setShowFeasibilitySheet(true)}
      />

      {/* Plan Compare Modal (C5) */}
      <PlanCompareModal
        open={showCompareModal}
        onOpenChange={setShowCompareModal}
        plans={plans}
        initialPlanAId={selectedPlanId ?? undefined}
        exams={exams}
        grades={grades}
        teachers={teachers}
        campuses={campuses}
      />

      {/* Re-optimize Dialog (B3) */}
      {selectedPlanId && (
        <ReoptimizeDialog
          open={showReoptimizeDialog}
          onOpenChange={setShowReoptimizeDialog}
          planId={selectedPlanId}
          schoolYearId={schoolYearId}
          keptSlots={keptSlots}
          onSuccess={(newIds) => {
            if (newIds.length > 0) {
              setSelectedPlanId(newIds[0])
              refetchPlans()
            }
          }}
        />
      )}

      {/* Feasibility Sheet */}
      <FeasibilitySheet
        open={showFeasibilitySheet}
        onOpenChange={setShowFeasibilitySheet}
        report={feasibilityReport?.report}
        schoolYearId={schoolYearId}
        schoolYearName={currentYear?.name}
      />
    </div>
  )
}
