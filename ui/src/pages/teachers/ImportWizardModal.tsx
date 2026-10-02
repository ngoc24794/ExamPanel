import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  FileSpreadsheet,
  AlertTriangle,
  CheckCircle2,
  AlertCircle,
  FileCheck,
  FolderOpen,
  ArrowRight,
  Info,
} from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
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
import { api } from '@/lib/api'
import type { ImportPreviewResult, ImportRowStatus } from '@/lib/api'
import { toast } from 'sonner'

interface ImportWizardModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  schoolYearId: number
  onSuccess: () => void
}

export const ImportWizardModal: React.FC<ImportWizardModalProps> = ({
  open,
  onOpenChange,
  schoolYearId,
  onSuccess,
}) => {
  const { t } = useTranslation()

  const [step, setStep] = React.useState<'select' | 'preview'>('select')
  const [filePath, setFilePath] = React.useState('')
  const [mode, setMode] = React.useState<'upsert' | 'sync'>('upsert')
  const [loading, setLoading] = React.useState(false)
  const [preview, setPreview] = React.useState<ImportPreviewResult | null>(null)
  const [activeTab, setActiveTab] = React.useState<'teachers' | 'campuses' | 'unavailabilities' | 'deactivated'>('teachers')
  const [statusFilter, setStatusFilter] = React.useState<ImportRowStatus | 'all'>('all')

  // Reset when dialog opens
  React.useEffect(() => {
    if (open) {
      setStep('select')
      setFilePath('')
      setMode('upsert')
      setPreview(null)
      setStatusFilter('all')
      setActiveTab('teachers')
    }
  }, [open])

  // Browse file
  const handleBrowseFile = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
      })
      if (selected && typeof selected === 'string') {
        setFilePath(selected)
      }
    } catch {
      // In web/test mock mode without tauri plugin, default to sample path
      setFilePath('mau-nhap-du-lieu.xlsx')
    }
  }

  // Load preview
  const handleRunPreview = async () => {
    if (!filePath.trim()) {
      toast.error(t('import.noFileSelected'))
      return
    }
    setLoading(true)
    try {
      const result = await api.previewImport(schoolYearId, filePath.trim(), mode)
      setPreview(result)
      setStep('preview')
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi đọc tệp Excel: ${msg}`)
    } finally {
      setLoading(false)
    }
  }

  // Apply import
  const handleApply = async () => {
    if (!preview || !preview.can_apply) return
    setLoading(true)
    try {
      const result = await api.applyImport(schoolYearId, preview)
      toast.success(t('import.applySuccess'), {
        description: t('import.appliedCounts', {
          teachersCreated: result.teachers_created,
          teachersUpdated: result.teachers_updated,
          teachersDeactivated: result.teachers_deactivated,
        }),
      })
      onSuccess()
      onOpenChange(false)
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi áp dụng nhập dữ liệu: ${msg}`)
    } finally {
      setLoading(false)
    }
  }

  const renderStatusBadge = (status: ImportRowStatus) => {
    switch (status) {
      case 'new':
        return <Badge className="bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border-emerald-500/20">{t('import.statusNew')}</Badge>
      case 'update':
        return <Badge className="bg-blue-500/15 text-blue-700 dark:text-blue-400 border-blue-500/20">{t('import.statusUpdate')}</Badge>
      case 'unchanged':
        return <Badge variant="outline" className="text-muted-foreground">{t('import.statusUnchanged')}</Badge>
      case 'error':
        return <Badge variant="destructive">{t('import.statusError')}</Badge>
      case 'skipped':
        return <Badge variant="secondary">{t('import.statusSkipped')}</Badge>
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] flex flex-col p-6 overflow-hidden">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-xl font-bold">
            <FileSpreadsheet className="h-5 w-5 text-primary" />
            {t('import.wizardTitle')}
          </DialogTitle>
        </DialogHeader>

        {step === 'select' && (
          <div className="space-y-6 py-4 flex-1 overflow-y-auto">
            {/* File Selection */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                {t('import.selectFile')}
              </label>
              <div className="flex gap-2">
                <Input
                  data-testid="import-file-input"
                  placeholder={t('import.filePathPlaceholder')}
                  value={filePath}
                  onChange={(e) => setFilePath(e.target.value)}
                  className="flex-1 text-xs"
                />
                <Button
                  data-testid="import-browse-btn"
                  type="button"
                  variant="outline"
                  onClick={handleBrowseFile}
                  className="gap-2 shrink-0"
                >
                  <FolderOpen className="h-4 w-4" />
                  {t('import.browse')}
                </Button>
              </div>
            </div>

            {/* Mode Selection */}
            <div className="space-y-3">
              <label className="text-sm font-medium text-foreground">
                {t('import.modeTitle')}
              </label>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div
                  data-testid="mode-upsert-card"
                  onClick={() => setMode('upsert')}
                  className={`p-4 rounded-lg border-2 cursor-pointer transition-all ${
                    mode === 'upsert'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-border/80'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-semibold text-sm text-foreground">
                      {t('import.modeUpsert')}
                    </span>
                    {mode === 'upsert' && (
                      <CheckCircle2 className="h-4 w-4 text-primary" />
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground leading-relaxed">
                    {t('import.modeUpsertDesc')}
                  </p>
                </div>

                <div
                  data-testid="mode-sync-card"
                  onClick={() => setMode('sync')}
                  className={`p-4 rounded-lg border-2 cursor-pointer transition-all ${
                    mode === 'sync'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-border/80'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-semibold text-sm text-foreground">
                      {t('import.modeSync')}
                    </span>
                    {mode === 'sync' && (
                      <CheckCircle2 className="h-4 w-4 text-primary" />
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground leading-relaxed">
                    {t('import.modeSyncDesc')}
                  </p>
                </div>
              </div>
            </div>

            {/* Note */}
            <div className="flex items-start gap-3 p-3 rounded-md bg-muted text-muted-foreground text-xs">
              <Info className="h-4 w-4 shrink-0 mt-0.5 text-primary" />
              <span>{t('import.noteAutoBackup')}</span>
            </div>
          </div>
        )}

        {step === 'preview' && preview && (
          <div className="flex-1 flex flex-col space-y-4 overflow-hidden py-2">
            {/* Feasibility Alert */}
            {preview.feasibility_report && !preview.feasibility_report.report.is_feasible && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <span>
                  {t('import.feasibilityWarning')} (Có {preview.feasibility_report.report.errors.length} lỗi bắt buộc)
                </span>
              </div>
            )}

            {/* Error Banner */}
            {!preview.can_apply && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                <AlertTriangle className="h-4 w-4 shrink-0" />
                <span>
                  {t('import.errorsFound', {
                    count:
                      preview.campuses_summary.error_count +
                      preview.teachers_summary.error_count +
                      preview.unavailabilities_summary.error_count,
                  })}
                </span>
              </div>
            )}

            {/* Tabs & Filters */}
            <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border pb-2">
              <div className="flex gap-2">
                <Button
                  data-testid="tab-teachers"
                  size="sm"
                  variant={activeTab === 'teachers' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('teachers')
                    setStatusFilter('all')
                  }}
                  className="h-8 text-xs"
                >
                  {t('import.tabTeachers')} ({preview.teachers.length})
                </Button>
                <Button
                  data-testid="tab-campuses"
                  size="sm"
                  variant={activeTab === 'campuses' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('campuses')
                    setStatusFilter('all')
                  }}
                  className="h-8 text-xs"
                >
                  {t('import.tabCampuses')} ({preview.campuses.length})
                </Button>
                <Button
                  data-testid="tab-unavailabilities"
                  size="sm"
                  variant={activeTab === 'unavailabilities' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('unavailabilities')
                    setStatusFilter('all')
                  }}
                  className="h-8 text-xs"
                >
                  {t('import.tabUnavailabilities')} ({preview.unavailabilities.length})
                </Button>
                {preview.mode === 'sync' && (
                  <Button
                    data-testid="tab-deactivated"
                    size="sm"
                    variant={activeTab === 'deactivated' ? 'default' : 'outline'}
                    onClick={() => {
                      setActiveTab('deactivated')
                      setStatusFilter('all')
                    }}
                    className="h-8 text-xs"
                  >
                    {t('import.tabDeactivated')} ({preview.deactivated_teachers.length})
                  </Button>
                )}
              </div>

              {activeTab !== 'deactivated' && (
                <div className="flex items-center gap-1">
                  {(['all', 'new', 'update', 'unchanged', 'error'] as const).map((st) => (
                    <Button
                      key={st}
                      data-testid={`filter-${st}`}
                      size="sm"
                      variant={statusFilter === st ? 'secondary' : 'ghost'}
                      onClick={() => setStatusFilter(st)}
                      className="h-7 px-2 text-xs"
                    >
                      {st === 'all' ? t('import.statusAll') : renderStatusBadge(st)}
                    </Button>
                  ))}
                </div>
              )}
            </div>

            {/* Preview Table Container */}
            <div className="flex-1 overflow-auto border border-border rounded-md">
              {activeTab === 'teachers' && (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-12">{t('import.rowIndex')}</TableHead>
                      <TableHead className="w-24">{t('import.status')}</TableHead>
                      <TableHead>{t('import.code')}</TableHead>
                      <TableHead>{t('import.fullName')}</TableHead>
                      <TableHead>{t('import.campus')}</TableHead>
                      <TableHead>{t('import.grades')}</TableHead>
                      <TableHead className="w-20">{t('import.loadWeight')}</TableHead>
                      <TableHead className="w-24">{t('import.active')}</TableHead>
                      <TableHead>{t('import.note')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.teachers
                      .filter((row) => statusFilter === 'all' || row.status === statusFilter)
                      .map((row) => {
                        const hasErr = row.errors.length > 0
                        return (
                          <TableRow
                            key={row.row_index}
                            className={hasErr ? 'bg-destructive/5' : undefined}
                          >
                            <TableCell className="font-mono text-xs">{row.row_index}</TableCell>
                            <TableCell>{renderStatusBadge(row.status)}</TableCell>
                            <TableCell className="font-mono text-xs">{row.code || '-'}</TableCell>
                            <TableCell className="font-medium text-xs">
                              {row.full_name}
                              {hasErr && (
                                <div className="text-destructive text-[11px] mt-0.5">
                                  {row.errors.map((e) => e.message).join('; ')}
                                </div>
                              )}
                            </TableCell>
                            <TableCell className="text-xs">{row.campus_code}</TableCell>
                            <TableCell className="text-xs">{row.grades_str}</TableCell>
                            <TableCell className="text-xs">{row.load_weight}</TableCell>
                            <TableCell className="text-xs">
                              {row.active ? t('import.statusNew') : t('import.statusSkipped')}
                            </TableCell>
                            <TableCell className="text-xs text-muted-foreground">
                              {row.note || '-'}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                  </TableBody>
                </Table>
              )}

              {activeTab === 'campuses' && (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-12">{t('import.rowIndex')}</TableHead>
                      <TableHead className="w-24">{t('import.status')}</TableHead>
                      <TableHead>{t('import.code')}</TableHead>
                      <TableHead>{t('import.name')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.campuses
                      .filter((row) => statusFilter === 'all' || row.status === statusFilter)
                      .map((row) => {
                        const hasErr = row.errors.length > 0
                        return (
                          <TableRow
                            key={row.row_index}
                            className={hasErr ? 'bg-destructive/5' : undefined}
                          >
                            <TableCell className="font-mono text-xs">{row.row_index}</TableCell>
                            <TableCell>{renderStatusBadge(row.status)}</TableCell>
                            <TableCell className="font-mono text-xs">{row.code}</TableCell>
                            <TableCell className="text-xs">
                              {row.name}
                              {hasErr && (
                                <div className="text-destructive text-[11px] mt-0.5">
                                  {row.errors.map((e) => e.message).join('; ')}
                                </div>
                              )}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                  </TableBody>
                </Table>
              )}

              {activeTab === 'unavailabilities' && (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-12">{t('import.rowIndex')}</TableHead>
                      <TableHead className="w-24">{t('import.status')}</TableHead>
                      <TableHead>{t('import.teacherRef')}</TableHead>
                      <TableHead>{t('import.exam')}</TableHead>
                      <TableHead>{t('import.reason')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.unavailabilities
                      .filter((row) => statusFilter === 'all' || row.status === statusFilter)
                      .map((row) => {
                        const hasErr = row.errors.length > 0
                        return (
                          <TableRow
                            key={row.row_index}
                            className={hasErr ? 'bg-destructive/5' : undefined}
                          >
                            <TableCell className="font-mono text-xs">{row.row_index}</TableCell>
                            <TableCell>{renderStatusBadge(row.status)}</TableCell>
                            <TableCell className="text-xs font-medium">
                              {row.teacher_ref}
                              {hasErr && (
                                <div className="text-destructive text-[11px] mt-0.5">
                                  {row.errors.map((e) => e.message).join('; ')}
                                </div>
                              )}
                            </TableCell>
                            <TableCell className="text-xs">{row.exam_code}</TableCell>
                            <TableCell className="text-xs text-muted-foreground">
                              {row.reason || '-'}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                  </TableBody>
                </Table>
              )}

              {activeTab === 'deactivated' && (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-24">{t('import.code')}</TableHead>
                      <TableHead>{t('import.fullName')}</TableHead>
                      <TableHead>{t('import.campus')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.deactivated_teachers.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} className="text-center py-4 text-xs text-muted-foreground">
                          Không có giáo viên nào bị ngưng hoạt động.
                        </TableCell>
                      </TableRow>
                    ) : (
                      preview.deactivated_teachers.map((t) => (
                        <TableRow key={t.id.toString()}>
                          <TableCell className="font-mono text-xs">{t.code || '-'}</TableCell>
                          <TableCell className="font-medium text-xs text-destructive">
                            {t.full_name}
                          </TableCell>
                          <TableCell className="text-xs">{t.campus_name}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              )}
            </div>
          </div>
        )}

        <DialogFooter className="flex items-center justify-between sm:justify-between border-t border-border pt-4">
          {step === 'preview' ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setStep('select')}
              disabled={loading}
            >
              Quay lại chọn tệp
            </Button>
          ) : (
            <div />
          )}

          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Hủy
            </Button>
            {step === 'select' ? (
              <Button
                data-testid="import-run-preview-btn"
                size="sm"
                onClick={handleRunPreview}
                disabled={loading || !filePath.trim()}
                className="gap-2"
              >
                {t('import.btnPreview')}
                <ArrowRight className="h-4 w-4" />
              </Button>
            ) : (
              <Button
                data-testid="import-apply-btn"
                size="sm"
                onClick={handleApply}
                disabled={loading || !preview?.can_apply}
                className="gap-2"
              >
                <FileCheck className="h-4 w-4" />
                {loading ? t('import.applying') : t('import.btnApply')}
              </Button>
            )}
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
