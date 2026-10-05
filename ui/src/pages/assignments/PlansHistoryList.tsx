import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { type PlanSummary } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import {
  Star,
  AlertTriangle,
  MoreVertical,
  Edit2,
  Copy,
  Trash2,
  CheckCircle,
  FolderOpen,
} from 'lucide-react'
import {
  useRenamePlan,
  useDeletePlan,
  useMarkFinal,
  useCreateManualCopy,
} from '@/lib/query/hooks'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'

interface PlansHistoryListProps {
  plans: PlanSummary[]
  activePlanId: number | null
  schoolYearId: number
  onSelectPlan: (planId: number) => void
}

export function PlansHistoryList({
  plans,
  activePlanId,
  schoolYearId,
  onSelectPlan,
}: PlansHistoryListProps) {
  const { t } = useTranslation()
  const renameMutation = useRenamePlan(schoolYearId)
  const deleteMutation = useDeletePlan(schoolYearId)
  const markFinalMutation = useMarkFinal(schoolYearId)
  const duplicateMutation = useCreateManualCopy(schoolYearId)

  const [renamePlan, setRenamePlan] = React.useState<PlanSummary | null>(null)
  const [newName, setNewName] = React.useState('')
  const [deleteConfirmId, setDeleteConfirmId] = React.useState<number | null>(null)

  const handleOpenRename = (plan: PlanSummary) => {
    setRenamePlan(plan)
    setNewName(plan.name)
  }

  const handleSaveRename = async () => {
    if (!renamePlan || !newName.trim()) return
    await renameMutation.mutateAsync({ id: renamePlan.id, newName: newName.trim() })
    setRenamePlan(null)
  }

  const handleDuplicate = async (plan: PlanSummary) => {
    const copyName = `${plan.name} (Bản sao)`
    const newId = await duplicateMutation.mutateAsync({ id: plan.id, name: copyName })
    onSelectPlan(newId)
  }

  const handleDelete = async (id: number) => {
    await deleteMutation.mutateAsync(id)
    setDeleteConfirmId(null)
    if (activePlanId === id) {
      const remaining = plans.filter((p) => p.id !== id)
      if (remaining.length > 0) {
        onSelectPlan(remaining[0].id)
      }
    }
  }

  const handleMarkFinal = async (plan: PlanSummary) => {
    try {
      await markFinalMutation.mutateAsync(plan.id)
    } catch (err: unknown) {
      const errorObj = err as { code?: string; params?: Record<string, string> }
      if (errorObj?.code === 'plan_stale') {
        toast.error(t('assignments.cannotMarkFinalStale'))
      } else if (errorObj?.code === 'plan_invalid') {
        toast.error(
          t('assignments.cannotMarkFinalInvalid', {
            count: Number(errorObj.params?.count ?? errorObj.params?.violations ?? 1),
          }),
        )
      } else {
        toast.error(getErrorMessage(err))
      }
    }
  }

  const getSourceLabel = (source: string) => {
    switch (source) {
      case 'optimizer':
        return t('assignments.sourceOptimizer')
      case 'manual':
        return t('assignments.sourceManual')
      case 'duplicate':
        return t('assignments.sourceDuplicate')
      default:
        return source
    }
  }

  if (plans.length === 0) {
    return (
      <div className="p-8 text-center border rounded-lg bg-card text-muted-foreground border-dashed">
        <p>{t('assignments.noPlans')}</p>
      </div>
    )
  }

  return (
    <div className="space-y-3" data-testid="plans-history-list">
      {plans.map((plan) => {
        const isActive = plan.id === activePlanId
        return (
          <div
            key={plan.id}
            data-testid={`plan-item-${plan.id}`}
            onClick={() => onSelectPlan(plan.id)}
            className={`p-4 rounded-lg border transition-all cursor-pointer flex flex-col md:flex-row md:items-center justify-between gap-3 ${
              isActive
                ? 'bg-accent/40 border-primary ring-1 ring-primary'
                : 'bg-card hover:bg-accent/20 border-border'
            }`}
          >
            <div className="space-y-1.5 flex-1 min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-semibold text-foreground truncate">
                  {plan.name}
                </span>
                {plan.is_final && (
                  <Badge
                    variant="default"
                    className="bg-emerald-600 hover:bg-emerald-700 text-white gap-1 text-xs"
                  >
                    <Star className="h-3 w-3 fill-current" />
                    {t('assignments.finalBadge')}
                  </Badge>
                )}
                {plan.is_stale && (
                  <Badge
                    variant="destructive"
                    className="gap-1 text-xs bg-amber-500/20 text-amber-600 dark:text-amber-400 border-amber-500/40"
                  >
                    <AlertTriangle className="h-3 w-3" />
                    {t('assignments.staleBadge')}
                  </Badge>
                )}
                {plan.rank !== undefined && plan.rank !== null && (
                  <Badge variant="outline" className="text-xs">
                    {t('assignments.rank', { rank: plan.rank })}
                  </Badge>
                )}
                <Badge variant="secondary" className="text-xs">
                  {getSourceLabel(plan.source)}
                </Badge>
              </div>

              <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
                <span>
                  {t('assignments.score')}:{' '}
                  <strong className="text-foreground">
                    {plan.score !== undefined && plan.score !== null
                      ? plan.score.toFixed(2)
                      : '--'}
                  </strong>
                </span>
                <span>{new Date(plan.created_at).toLocaleString()}</span>
              </div>
            </div>

            <div
              className="flex items-center gap-1.5 self-end md:self-center"
              onClick={(e) => e.stopPropagation()}
            >
              <Button
                variant={isActive ? 'default' : 'outline'}
                size="sm"
                onClick={() => onSelectPlan(plan.id)}
                className="gap-1 text-xs"
              >
                <FolderOpen className="h-3.5 w-3.5" />
                {t('assignments.openPlan')}
              </Button>

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0"
                    data-testid={`plan-menu-${plan.id}`}
                  >
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48 bg-card border-border">
                  <DropdownMenuItem
                    onClick={() => handleMarkFinal(plan)}
                    className="gap-2"
                  >
                    <CheckCircle className="h-4 w-4 text-emerald-600" />
                    <span>{t('assignments.markFinal')}</span>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => handleOpenRename(plan)}
                    className="gap-2"
                  >
                    <Edit2 className="h-4 w-4" />
                    <span>{t('assignments.rename')}</span>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => handleDuplicate(plan)}
                    className="gap-2"
                  >
                    <Copy className="h-4 w-4" />
                    <span>{t('assignments.duplicateForEdit')}</span>
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => setDeleteConfirmId(plan.id)}
                    className="gap-2 text-destructive focus:text-destructive"
                  >
                    <Trash2 className="h-4 w-4" />
                    <span>{t('assignments.deletePlan')}</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        )
      })}

      {/* Rename Dialog */}
      {renamePlan && (
        <Dialog open={true} onOpenChange={() => setRenamePlan(null)}>
          <DialogContent className="max-w-md bg-card border-border">
            <DialogHeader>
              <DialogTitle>{t('assignments.rename')}</DialogTitle>
            </DialogHeader>
            <div className="py-2">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                autoFocus
                placeholder={t('assignments.planName')}
              />
            </div>
            <DialogFooter>
              <Button variant="outline" size="sm" onClick={() => setRenamePlan(null)}>
                {t('common.cancel')}
              </Button>
              <Button size="sm" onClick={handleSaveRename}>
                {t('common.save')}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}

      {/* Delete Confirmation Dialog */}
      {deleteConfirmId !== null && (
        <Dialog open={true} onOpenChange={() => setDeleteConfirmId(null)}>
          <DialogContent className="max-w-md bg-card border-border">
            <DialogHeader>
              <DialogTitle>{t('assignments.deletePlan')}</DialogTitle>
            </DialogHeader>
            <p className="text-sm text-muted-foreground">
              Bạn có chắc chắn muốn xóa phương án này? Hành động này không thể hoàn tác.
            </p>
            <DialogFooter>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDeleteConfirmId(null)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => handleDelete(deleteConfirmId)}
              >
                {t('common.delete')}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </div>
  )
}
