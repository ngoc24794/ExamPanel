import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { describeViolation, useViolationLookup } from '@/lib/violations'
import {
  FileSpreadsheet,
  AlertTriangle,
  CheckCircle2,
  AlertCircle,
  FileCheck,
  FolderOpen,
  ArrowRight,
  ClipboardPaste,
  ArrowLeft,
} from 'lucide-react'
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
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '@/components/ui/table'
import { api, isTauriEnvironment } from '@/lib/api'
import type { PlanImportPreview } from '@/lib/api'
import { toast } from 'sonner'

interface ImportPlanModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  schoolYearId: number
  onSuccess: (newPlanId: number) => void
}

export const ImportPlanModal: React.FC<ImportPlanModalProps> = ({
  open,
  onOpenChange,
  schoolYearId,
  onSuccess,
}) => {
  const { t } = useTranslation()
  const lookup = useViolationLookup(schoolYearId)
  const isTauri = isTauriEnvironment()

  const [step, setStep] = React.useState<'input' | 'preview'>('input')
  const [sourceType, setSourceType] = React.useState<'excel' | 'tsv'>('excel')
  const [filePath, setFilePath] = React.useState('')
  const [tsvContent, setTsvContent] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const [applying, setApplying] = React.useState(false)
  const [preview, setPreview] = React.useState<PlanImportPreview | null>(null)
  const [previewTab, setPreviewTab] = React.useState<'totals' | 'issues' | 'assignments'>(
    'totals',
  )

  // Reset when dialog opens
  React.useEffect(() => {
    if (open) {
      setStep('input')
      setSourceType('excel')
      setFilePath('')
      setTsvContent('')
      setLoading(false)
      setApplying(false)
      setPreview(null)
      setPreviewTab('totals')
    }
  }, [open])

  // File browser
  const handleBrowseFile = async () => {
    if (!isTauri) {
      setFilePath('bang-phan-cong-mau.xlsx')
      return
    }
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog')
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
      })
      if (selected && typeof selected === 'string') {
        setFilePath(selected)
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(t('planImport.pickFileError', { message: msg }))
    }
  }

  // Run Preview
  const handleRunPreview = async () => {
    if (sourceType === 'excel' && !filePath.trim()) {
      toast.error(t('import.noFileSelected'))
      return
    }
    if (sourceType === 'tsv' && !tsvContent.trim()) {
      toast.error(t('planImport.pasteRequired'))
      return
    }

    setLoading(true)
    try {
      const result = await api.previewImportPlan(
        schoolYearId,
        sourceType === 'excel' ? filePath.trim() : undefined,
        sourceType === 'tsv' ? tsvContent.trim() : undefined,
      )
      setPreview(result)
      setStep('preview')
      if (result.errors.length > 0 || result.hard_violations.length > 0) {
        setPreviewTab('issues')
      } else {
        setPreviewTab('totals')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(t('planImport.readError', { message: msg }))
    } finally {
      setLoading(false)
    }
  }

  // Apply Plan
  const handleApply = async () => {
    if (!preview || !preview.can_apply) return
    setApplying(true)
    try {
      const planName = t('planImport.planName')
      const planId = await api.applyImportedPlan({
        school_year_id: schoolYearId,
        assignments: preview.assignments,
        plan_name: planName,
      })
      toast.success(t('planImport.importSuccess'))
      onOpenChange(false)
      onSuccess(planId)
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(t('planImport.importError', { error: msg }))
    } finally {
      setApplying(false)
    }
  }

  const issueCount =
    (preview?.errors.length || 0) +
    (preview?.warnings.length || 0) +
    (preview?.hard_violations.length || 0)

  const softScoreDisplay =
    preview?.score_report?.total !== undefined
      ? preview.score_report.total.toFixed(1)
      : '0.0'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-4xl max-h-[90vh] flex flex-col p-6 overflow-hidden"
        data-testid="import-plan-modal"
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-foreground">
            <FileSpreadsheet className="h-5 w-5 text-primary" />
            <span>{t('planImport.dialogTitle')}</span>
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            {t('planImport.dialogDesc')}
          </DialogDescription>
        </DialogHeader>

        {step === 'input' && (
          <div className="flex-1 overflow-y-auto space-y-4 py-2">
            {/* Source Type Toggle */}
            <div className="flex rounded-md border border-border p-1 bg-muted/40 gap-1">
              <button
                type="button"
                onClick={() => setSourceType('excel')}
                className={`flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded text-xs font-semibold transition-all ${
                  sourceType === 'excel'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                data-testid="tab-source-excel"
              >
                <FileSpreadsheet className="h-4 w-4" />
                <span>{t('planImport.tabExcel')}</span>
              </button>
              <button
                type="button"
                onClick={() => setSourceType('tsv')}
                className={`flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded text-xs font-semibold transition-all ${
                  sourceType === 'tsv'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                data-testid="tab-source-tsv"
              >
                <ClipboardPaste className="h-4 w-4" />
                <span>{t('planImport.tabTsv')}</span>
              </button>
            </div>

            {sourceType === 'excel' ? (
              <div className="space-y-2 pt-2">
                <label className="text-xs font-semibold text-foreground">
                  {t('planImport.excelBrowse')}
                </label>
                <div className="flex gap-2">
                  <Input
                    value={filePath}
                    onChange={(e) => setFilePath(e.target.value)}
                    placeholder={t('planImport.excelPlaceholder')}
                    className="font-mono text-xs"
                    data-testid="input-plan-file-path"
                  />
                  <Button
                    variant="outline"
                    onClick={handleBrowseFile}
                    className="gap-1.5 shrink-0"
                    data-testid="btn-browse-plan-file"
                  >
                    <FolderOpen className="h-4 w-4" />
                    {t('import.browse')}
                  </Button>
                </div>
                <p className="text-[11px] text-muted-foreground">
                  {t('planImport.fileHint')}
                </p>
              </div>
            ) : (
              <div className="space-y-2 pt-2">
                <label className="text-xs font-semibold text-foreground">
                  {t('planImport.tsvLabel')}
                </label>
                <textarea
                  value={tsvContent}
                  onChange={(e) => setTsvContent(e.target.value)}
                  placeholder={t('planImport.tsvPlaceholder')}
                  className="flex min-h-[220px] w-full rounded-md border border-input bg-background px-3 py-2 text-xs ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 font-mono"
                  data-testid="textarea-plan-tsv"
                />
                <p className="text-[11px] text-muted-foreground">
                  {t('planImport.pasteHint')}
                </p>
              </div>
            )}
          </div>
        )}

        {step === 'preview' && preview && (
          <div className="flex-1 overflow-y-auto space-y-4 py-2">
            {/* Status bar */}
            <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-lg border border-border bg-card">
              <div className="flex items-center gap-3">
                {preview.can_apply ? (
                  <Badge
                    variant="default"
                    className="bg-emerald-600 hover:bg-emerald-700 text-white gap-1 text-xs"
                  >
                    <CheckCircle2 className="h-3.5 w-3.5" />
                    {t('planImport.validStatus')}
                  </Badge>
                ) : (
                  <Badge variant="destructive" className="gap-1 text-xs">
                    <AlertTriangle className="h-3.5 w-3.5" />
                    {t('planImport.invalidStatus')}
                  </Badge>
                )}
                <span className="text-xs font-semibold text-foreground">
                  {t('planImport.summaryTotal', { count: preview.assignments.length })}
                </span>
              </div>

              <div className="text-xs text-muted-foreground">
                {t('planImport.scoreLabel', { score: softScoreDisplay })}
              </div>
            </div>

            {/* Preview Subtabs */}
            <div className="flex rounded-md border border-border p-1 bg-muted/40 gap-1">
              <button
                type="button"
                onClick={() => setPreviewTab('totals')}
                className={`flex-1 flex items-center justify-center gap-1.5 py-1 px-3 rounded text-xs font-semibold transition-all ${
                  previewTab === 'totals'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                data-testid="tab-preview-totals"
              >
                <FileCheck className="h-4 w-4" />
                <span>
                  {t('planImport.tabTotals', { count: preview.teacher_totals.length })}
                </span>
              </button>
              <button
                type="button"
                onClick={() => setPreviewTab('issues')}
                className={`flex-1 flex items-center justify-center gap-1.5 py-1 px-3 rounded text-xs font-semibold transition-all ${
                  previewTab === 'issues'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                data-testid="tab-preview-issues"
              >
                <AlertCircle className="h-4 w-4" />
                <span>{t('planImport.tabIssues', { count: issueCount })}</span>
              </button>
              <button
                type="button"
                onClick={() => setPreviewTab('assignments')}
                className={`flex-1 flex items-center justify-center gap-1.5 py-1 px-3 rounded text-xs font-semibold transition-all ${
                  previewTab === 'assignments'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                data-testid="tab-preview-assignments"
              >
                <FileSpreadsheet className="h-4 w-4" />
                <span>
                  {t('planImport.tabAssignments', {
                    count: preview.assignments.length,
                  })}
                </span>
              </button>
            </div>

            {/* Totals Table */}
            {previewTab === 'totals' && (
              <div className="border border-border rounded-lg overflow-hidden max-h-72 overflow-y-auto">
                <Table className="text-xs">
                  <TableHeader className="bg-muted/50 sticky top-0">
                    <TableRow>
                      <TableHead>{t('planImport.colTeacher')}</TableHead>
                      <TableHead>{t('planImport.colDisplayName')}</TableHead>
                      <TableHead className="text-center">
                        {t('planImport.colDetected')}
                      </TableHead>
                      <TableHead className="text-center">
                        {t('planImport.colFileTotal')}
                      </TableHead>
                      <TableHead className="text-center">
                        {t('planImport.colSetters')}
                      </TableHead>
                      <TableHead className="text-center">
                        {t('planImport.colReviewers')}
                      </TableHead>
                      <TableHead className="text-center">
                        {t('planImport.colStatus')}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.teacher_totals.map((tRow) => {
                      const isMatched =
                        tRow.file_total === null ||
                        tRow.file_total === undefined ||
                        Number(tRow.file_total) === Number(tRow.computed_total)
                      return (
                        <TableRow
                          key={tRow.teacher_id}
                          className={!isMatched ? 'bg-amber-500/10' : undefined}
                        >
                          <TableCell className="font-medium text-foreground">
                            {tRow.teacher_name}
                          </TableCell>
                          <TableCell className="text-muted-foreground">
                            {tRow.display_name || '-'}
                          </TableCell>
                          <TableCell className="text-center font-bold">
                            {Number(tRow.computed_total)}
                          </TableCell>
                          <TableCell className="text-center text-muted-foreground">
                            {tRow.file_total !== null && tRow.file_total !== undefined
                              ? Number(tRow.file_total)
                              : '-'}
                          </TableCell>
                          <TableCell className="text-center">
                            {Number(tRow.setter_count)}
                          </TableCell>
                          <TableCell className="text-center">
                            {Number(tRow.reviewer_count)}
                          </TableCell>
                          <TableCell className="text-center">
                            {isMatched ? (
                              <Badge
                                variant="outline"
                                className="text-[10px] text-emerald-600 border-emerald-500/30"
                              >
                                {t('planImport.statusMatched')}
                              </Badge>
                            ) : (
                              <Badge variant="destructive" className="text-[10px]">
                                {t('planImport.statusMismatch')}
                              </Badge>
                            )}
                          </TableCell>
                        </TableRow>
                      )
                    })}
                  </TableBody>
                </Table>
              </div>
            )}

            {/* Issues Tab */}
            {previewTab === 'issues' && (
              <div className="pt-1">
                {issueCount === 0 ? (
                  <div className="p-6 text-center text-xs text-muted-foreground border border-dashed border-border rounded-lg">
                    {t('planImport.noIssues')}
                  </div>
                ) : (
                  <div className="space-y-2 max-h-72 overflow-y-auto">
                    {preview.errors.map((err, i) => (
                      <div
                        key={`err-${i}`}
                        className="flex items-start gap-2 p-2.5 rounded bg-destructive/10 text-destructive text-xs border border-destructive/20"
                      >
                        <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                        <span>
                          {err.message ||
                            `${err.sheet} [${err.column}${err.row}]: ${err.code}`}
                        </span>
                      </div>
                    ))}
                    {preview.hard_violations.map((viol, i) => (
                      <div
                        key={`viol-${i}`}
                        className="flex items-start gap-2 p-2.5 rounded bg-destructive/10 text-destructive text-xs border border-destructive/20"
                      >
                        <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                        <span>{describeViolation(t, viol, lookup)}</span>
                      </div>
                    ))}
                    {preview.warnings.map((warn, i) => (
                      <div
                        key={`warn-${i}`}
                        className="flex items-start gap-2 p-2.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 text-xs border border-amber-500/20"
                      >
                        <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
                        <span>{warn}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Assignments List Tab */}
            {previewTab === 'assignments' && (
              <div className="border border-border rounded-lg overflow-hidden max-h-72 overflow-y-auto">
                <Table className="text-xs">
                  <TableHeader className="bg-muted/50 sticky top-0">
                    <TableRow>
                      <TableHead>{t('planImport.colExamId')}</TableHead>
                      <TableHead>{t('planImport.colGradeId')}</TableHead>
                      <TableHead>{t('planImport.colSubjectId')}</TableHead>
                      <TableHead>{t('planImport.colRole')}</TableHead>
                      <TableHead>{t('planImport.colPosition')}</TableHead>
                      <TableHead>{t('planImport.colTeacherId')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.assignments.map((asgn, i) => (
                      <TableRow key={i}>
                        <TableCell>{asgn.exam_id}</TableCell>
                        <TableCell>{asgn.grade_id}</TableCell>
                        <TableCell>{asgn.subject_id}</TableCell>
                        <TableCell>
                          <Badge
                            variant={asgn.role === 'setter' ? 'default' : 'secondary'}
                            className="text-[10px]"
                          >
                            {asgn.role === 'setter'
                              ? t('common.roleSetter')
                              : t('common.roleReviewer')}
                          </Badge>
                        </TableCell>
                        <TableCell>#{asgn.position}</TableCell>
                        <TableCell className="font-mono">{asgn.teacher_id}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </div>
        )}

        <DialogFooter className="flex items-center justify-between border-t border-border pt-4 mt-2">
          {step === 'preview' ? (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setStep('input')}
                disabled={applying}
                className="gap-1.5"
                data-testid="btn-plan-import-back"
              >
                <ArrowLeft className="h-4 w-4" />
                {t('planImport.backBtn')}
              </Button>
              <Button
                size="sm"
                onClick={handleApply}
                disabled={!preview?.can_apply || applying}
                className="gap-1.5 bg-emerald-600 hover:bg-emerald-700 text-white"
                data-testid="btn-plan-import-apply"
              >
                <FileCheck className="h-4 w-4" />
                {applying ? t('planImport.applying') : t('planImport.applyBtn')}
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onOpenChange(false)}
                className="text-xs text-muted-foreground"
              >
                {t('common.cancel')}
              </Button>
              <Button
                size="sm"
                onClick={handleRunPreview}
                disabled={loading}
                className="gap-1.5"
                data-testid="btn-plan-import-preview"
              >
                {loading ? t('planImport.reading') : t('planImport.previewBtn')}
                <ArrowRight className="h-4 w-4" />
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
