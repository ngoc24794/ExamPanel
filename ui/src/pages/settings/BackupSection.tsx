import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Database,
  Download,
  Upload,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
  Clock,
} from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
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
import { api, isTauriEnvironment } from '@/lib/api'
import type { BackupFileInfo, BackupValidationSummary } from '@/lib/api'
import { toast } from 'sonner'
import { useQueryClient } from '@tanstack/react-query'

export const BackupSection: React.FC = () => {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const isTauri = isTauriEnvironment()

  const [backups, setBackups] = React.useState<BackupFileInfo[]>([])
  const [loadingList, setLoadingList] = React.useState(false)
  const [backingUp, setBackingUp] = React.useState(false)

  // Restore dialog state
  const [restoreDialogOpen, setRestoreDialogOpen] = React.useState(false)
  const [selectedBackupPath, setSelectedBackupPath] = React.useState<string | null>(null)
  const [validationSummary, setValidationSummary] = React.useState<BackupValidationSummary | null>(null)
  const [validating, setValidating] = React.useState(false)
  const [restoring, setRestoring] = React.useState(false)

  const loadBackups = React.useCallback(async () => {
    setLoadingList(true)
    try {
      const list = await api.listBackups()
      setBackups(list)
    } catch {
      // In web/mock mode ignore or use empty
    } finally {
      setLoadingList(false)
    }
  }, [])

  React.useEffect(() => {
    void loadBackups()
  }, [loadBackups])

  // Backup now
  const handleBackupNow = async () => {
    setBackingUp(true)
    try {
      const now = new Date()
      const pad = (n: number) => n.toString().padStart(2, '0')
      const timestamp = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}`
      const defaultName = `exampanel-backup-${timestamp}.db`

      let targetPath = defaultName
      if (isTauri) {
        const { save } = await import('@tauri-apps/plugin-dialog')
        const chosen = await save({
          defaultPath: defaultName,
          filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }],
        })
        if (!chosen) {
          setBackingUp(false)
          return
        }
        targetPath = chosen
      }

      await api.backupDatabase(targetPath)
      toast.success(t('backup.backupSuccess'))
      void loadBackups()
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi sao lưu: ${msg}`)
    } finally {
      setBackingUp(false)
    }
  }

  // Choose file to restore
  const handleChooseRestoreFile = async () => {
    try {
      let chosenPath = 'exampanel-backup-test.db'
      if (isTauri) {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const selected = await open({
          multiple: false,
          filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }],
        })
        if (!selected || typeof selected !== 'string') return
        chosenPath = selected
      }
      await initiateRestore(chosenPath)
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi chọn tệp: ${msg}`)
    }
  }

  // Initiate restore with validation
  const initiateRestore = async (path: string) => {
    setSelectedBackupPath(path)
    setValidating(true)
    setRestoreDialogOpen(true)
    try {
      const summary = await api.validateBackup(path)
      setValidationSummary(summary)
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      setValidationSummary({
        valid: false,
        user_version: 0,
        school_years_count: 0,
        teachers_count: 0,
        plans_count: 0,
        error: msg,
      })
    } finally {
      setValidating(false)
    }
  }

  // Execute restore
  const handleConfirmRestore = async () => {
    if (!selectedBackupPath || !validationSummary?.valid) return
    setRestoring(true)
    try {
      await api.restoreDatabase(selectedBackupPath)
      toast.success(t('backup.restoreSuccess'))
      setRestoreDialogOpen(false)
      // Invalidate queries to refresh app state
      await queryClient.invalidateQueries()
      void loadBackups()
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      toast.error(`Lỗi phục hồi: ${msg}`)
    } finally {
      setRestoring(false)
    }
  }

  const formatSize = (bytes: bigint | number) => {
    const b = Number(bytes)
    if (b < 1024) return `${b} B`
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`
    return `${(b / (1024 * 1024)).toFixed(2)} MB`
  }

  return (
    <Card className="bg-card border-border">
      <CardHeader className="pb-3">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
          <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
            <Database className="h-4 w-4 text-primary" />
            <span>{t('backup.title')}</span>
          </CardTitle>
          <div className="flex items-center gap-2">
            <Button
              data-testid="backup-now-btn"
              variant="outline"
              size="sm"
              onClick={handleBackupNow}
              disabled={backingUp}
              className="gap-1.5 text-xs h-8"
            >
              <Download className="h-3.5 w-3.5" />
              <span>{backingUp ? 'Đang sao lưu...' : t('backup.backupNow')}</span>
            </Button>
            <Button
              data-testid="restore-file-btn"
              variant="outline"
              size="sm"
              onClick={handleChooseRestoreFile}
              className="gap-1.5 text-xs h-8"
            >
              <Upload className="h-3.5 w-3.5" />
              <span>{t('backup.restoreFromFile')}</span>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-2 text-xs">
        <p className="text-muted-foreground">{t('backup.description')}</p>

        {/* Automatic Backups List */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <h4 className="font-semibold text-foreground flex items-center gap-1.5">
              <Clock className="h-3.5 w-3.5 text-muted-foreground" />
              <span>{t('backup.autoBackups')}</span>
            </h4>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void loadBackups()}
              disabled={loadingList}
              className="h-7 w-7 p-0"
              title="Làm mới"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${loadingList ? 'animate-spin' : ''}`} />
            </Button>
          </div>

          <div className="border border-border rounded-md overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-1/2">Tệp sao lưu</TableHead>
                  <TableHead className="w-24">Dung lượng</TableHead>
                  <TableHead className="w-36">Thời gian</TableHead>
                  <TableHead className="w-24 text-right">Thao tác</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {backups.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-4 text-muted-foreground">
                      {t('backup.noAutoBackups')}
                    </TableCell>
                  </TableRow>
                ) : (
                  backups.map((b) => (
                    <TableRow key={b.filename}>
                      <TableCell className="font-mono text-xs font-medium">
                        {b.filename}
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {formatSize(b.size_bytes)}
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {b.modified_at}
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => void initiateRestore(b.path)}
                          className="h-7 px-2 text-xs text-primary hover:text-primary"
                        >
                          Phục hồi
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </div>

        {/* Validation and Restore Dialog */}
        <Dialog open={restoreDialogOpen} onOpenChange={setRestoreDialogOpen}>
          <DialogContent className="max-w-md">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2 text-base font-bold">
                <Database className="h-4 w-4 text-primary" />
                {t('backup.restoreConfirmTitle')}
              </DialogTitle>
            </DialogHeader>

            <div className="space-y-4 py-2 text-xs">
              <p className="text-muted-foreground leading-relaxed">
                {t('backup.restoreConfirmDesc')}
              </p>

              {validating ? (
                <div className="p-4 text-center text-muted-foreground">
                  Đang kiểm tra tính toàn vẹn của tệp sao lưu...
                </div>
              ) : validationSummary ? (
                <div className="space-y-3">
                  {validationSummary.valid ? (
                    <div className="flex items-center gap-2 p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-400">
                      <CheckCircle2 className="h-4 w-4 shrink-0" />
                      <span>{t('backup.integrityOk')}</span>
                    </div>
                  ) : (
                    <div className="flex items-start gap-2 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive">
                      <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                      <div>
                        <span className="font-semibold">{t('backup.integrityFail')}</span>
                        {validationSummary.error && (
                          <p className="text-[11px] mt-1">{validationSummary.error}</p>
                        )}
                      </div>
                    </div>
                  )}

                  {validationSummary.valid && (
                    <div className="grid grid-cols-2 gap-2 p-3 rounded-md bg-muted/40 border border-border">
                      <div>
                        <span className="text-muted-foreground">Năm học:</span>{' '}
                        <span className="font-semibold">{validationSummary.school_years_count}</span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Giáo viên:</span>{' '}
                        <span className="font-semibold">{validationSummary.teachers_count}</span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Phương án:</span>{' '}
                        <span className="font-semibold">{validationSummary.plans_count}</span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Phiên bản:</span>{' '}
                        <span className="font-semibold font-mono">v{validationSummary.user_version}</span>
                      </div>
                    </div>
                  )}
                </div>
              ) : null}
            </div>

            <DialogFooter className="gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setRestoreDialogOpen(false)}
                disabled={restoring}
              >
                Hủy
              </Button>
              <Button
                data-testid="confirm-restore-btn"
                variant="destructive"
                size="sm"
                onClick={handleConfirmRestore}
                disabled={restoring || validating || !validationSummary?.valid}
              >
                {restoring ? 'Đang phục hồi...' : 'Xác nhận phục hồi'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </CardContent>
    </Card>
  )
}
