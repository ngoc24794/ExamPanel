import { useTranslation } from 'react-i18next'
import { Calculator, ShieldAlert, Sparkles } from 'lucide-react'
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
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '@/components/ui/table'
import {
  usePreviewQuotas,
  useProblemDetails,
  useTeachers,
  useSchoolYears,
} from '@/lib/query/hooks'
import type { QuotaPreviewItem } from '@/lib/api'

interface QuotaPreviewModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  schoolYearId: number
}

export function QuotaPreviewModal({
  open,
  onOpenChange,
  schoolYearId,
}: QuotaPreviewModalProps) {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.id === schoolYearId)

  const { data: quotaItems = [], isLoading, error } = usePreviewQuotas(schoolYearId)
  const { data: problemDetails } = useProblemDetails(schoolYearId)
  const { data: teachersWithGrades = [] } = useTeachers(schoolYearId)

  const forcedMap = new Map<number, number>()
  if (problemDetails?.forced) {
    for (const p of problemDetails.forced) {
      forcedMap.set(p.teacher_id, (forcedMap.get(p.teacher_id) ?? 0) + 1)
    }
  }

  const teacherMap = new Map(
    teachersWithGrades.map((twg) => [twg.teacher.id, twg.teacher]),
  )

  const totalQuota = quotaItems.reduce(
    (acc: number, it: QuotaPreviewItem) => acc + it.quota,
    0,
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Calculator className="h-5 w-5 text-primary" />
            <DialogTitle className="text-base font-semibold text-foreground">
              {t('quota.previewTitle')}
            </DialogTitle>
            {currentYear && (
              <Badge variant="outline" className="text-xs">
                {currentYear.name}
              </Badge>
            )}
          </div>
          <DialogDescription className="text-xs text-muted-foreground">
            {t('quota.previewDesc')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto min-h-[300px] border rounded-md border-border">
          {isLoading ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              {t('teachers.quotaLoading')}
            </div>
          ) : error ? (
            <div className="p-4 text-xs text-destructive bg-destructive/10 flex items-center gap-2">
              <ShieldAlert className="h-4 w-4 shrink-0" />
              <span>{(error as Error).message}</span>
            </div>
          ) : quotaItems.length === 0 ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              {t('quota.empty')}
            </div>
          ) : (
            <div>
              <div className="p-3 bg-muted/30 border-b border-border flex items-center justify-between text-xs">
                <div>
                  <span className="text-muted-foreground">{t('teachers.quotaTotalTeachers')}: </span>
                  <span className="font-bold text-foreground">{quotaItems.length}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">{t('teachers.quotaTotalQuota')}: </span>
                  <span className="font-bold text-foreground">
                    {totalQuota.toFixed(2)}
                  </span>
                </div>
              </div>

              <Table className="text-xs">
                <TableHeader>
                  <TableRow className="hover:bg-transparent">
                    <TableHead className="w-12 text-center">#</TableHead>
                    <TableHead>{t('teachers.quotaColTeacher')}</TableHead>
                    <TableHead className="text-center">{t('teachers.quotaColWeight')}</TableHead>
                    <TableHead className="text-center">{t('teachers.quotaColAvailable')}</TableHead>
                    <TableHead className="text-center">{t('teachers.quotaColForced')}</TableHead>
                    <TableHead className="text-right">{t('teachers.quotaColTarget')}</TableHead>
                    <TableHead className="text-center">
                      {t('teachers.quotaColRange')}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {quotaItems.map((item: QuotaPreviewItem, idx: number) => {
                    const forcedCount = forcedMap.get(item.teacher_id) ?? 0
                    const isForced = forcedCount > 0
                    const teacherRec = teacherMap.get(item.teacher_id)

                    return (
                      <TableRow
                        key={item.teacher_id}
                        className={isForced ? 'bg-amber-500/5 hover:bg-amber-500/10' : ''}
                      >
                        <TableCell className="text-center text-muted-foreground">
                          {idx + 1}
                        </TableCell>
                        <TableCell>
                          <div className="font-medium text-foreground">
                            {item.teacher_name}
                          </div>
                          {teacherRec?.display_name && (
                            <div className="text-[11px] text-muted-foreground">
                              {teacherRec.display_name}
                            </div>
                          )}
                        </TableCell>
                        <TableCell className="text-center">
                          {item.load_weight.toFixed(2)}
                        </TableCell>
                        <TableCell className="text-center">
                          {item.available_exams}
                        </TableCell>
                        <TableCell className="text-center">
                          {isForced ? (
                            <Badge
                              variant="outline"
                              className="bg-amber-500/15 border-amber-500/40 text-amber-600 dark:text-amber-400 font-semibold gap-1 text-[11px]"
                            >
                              <Sparkles className="h-3 w-3" />
                              {t('teachers.quotaForcedBadge', { count: forcedCount })}
                            </Badge>
                          ) : (
                            <span className="text-muted-foreground">0</span>
                          )}
                        </TableCell>
                        <TableCell className="text-right font-mono font-semibold text-primary">
                          {item.quota.toFixed(2)}
                        </TableCell>
                        <TableCell className="text-center font-mono">
                          <span className="px-2 py-0.5 rounded bg-muted/60 text-foreground">
                            [{item.lo}, {item.hi}]
                          </span>
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </div>
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
            {t('common.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
