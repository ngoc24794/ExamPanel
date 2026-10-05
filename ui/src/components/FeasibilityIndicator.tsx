import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { AlertOctagon, AlertTriangle, CheckCircle2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useFeasibility } from '@/lib/query/hooks'
import { FeasibilitySheet } from './FeasibilitySheet'

interface FeasibilityIndicatorProps {
  schoolYearId: number | undefined
  schoolYearName?: string
}

export const FeasibilityIndicator: React.FC<FeasibilityIndicatorProps> = ({
  schoolYearId,
  schoolYearName,
}) => {
  const { t } = useTranslation()
  const { data: feasibility } = useFeasibility(schoolYearId)
  const [sheetOpen, setSheetOpen] = React.useState(false)

  const report = feasibility?.report
  const errors = report?.errors ?? []
  const warnings = report?.warnings ?? []
  const isFeasible = report?.is_feasible ?? true

  let statusColor =
    'text-emerald-500 border-emerald-500/30 bg-emerald-500/10 hover:bg-emerald-500/20'
  let StatusIcon = CheckCircle2
  let statusText = t('feasibility.statusFeasible')

  if (!isFeasible || errors.length > 0) {
    statusColor =
      'text-destructive border-destructive/30 bg-destructive/10 hover:bg-destructive/20'
    StatusIcon = AlertOctagon
    statusText = t('feasibility.statusErrors')
  } else if (warnings.length > 0) {
    statusColor =
      'text-amber-500 border-amber-500/30 bg-amber-500/10 hover:bg-amber-500/20'
    StatusIcon = AlertTriangle
    statusText = t('feasibility.statusWarnings')
  }

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        className={`h-8 gap-2 border px-2.5 text-xs font-medium transition-colors ${statusColor}`}
        onClick={() => setSheetOpen(true)}
        aria-label="Feasibility status"
        data-testid="feasibility-indicator"
      >
        <StatusIcon className="h-3.5 w-3.5 shrink-0" />
        <span className="hidden sm:inline">{statusText}</span>
        {errors.length > 0 ? (
          <Badge variant="destructive" className="h-4 px-1 text-[10px]">
            {errors.length}
          </Badge>
        ) : warnings.length > 0 ? (
          <Badge
            variant="secondary"
            className="h-4 px-1 text-[10px] bg-amber-500/20 text-amber-600 dark:text-amber-400"
          >
            {warnings.length}
          </Badge>
        ) : null}
      </Button>

      <FeasibilitySheet
        open={sheetOpen}
        onOpenChange={setSheetOpen}
        report={report}
        schoolYearId={schoolYearId}
        schoolYearName={schoolYearName}
      />
    </>
  )
}
