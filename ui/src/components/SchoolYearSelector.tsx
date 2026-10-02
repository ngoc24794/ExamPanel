import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Calendar, Plus, Check } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuLabel,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import {
  useSchoolYears,
  useSetCurrentSchoolYear,
  useCreateSchoolYear,
} from '@/lib/query/hooks'

export const SchoolYearSelector: React.FC = () => {
  const { t } = useTranslation()
  const { data: schoolYears = [] } = useSchoolYears()
  const setCurrentMutation = useSetCurrentSchoolYear()
  const createMutation = useCreateSchoolYear()

  const [dialogOpen, setDialogOpen] = React.useState(false)
  const [newYearName, setNewYearName] = React.useState('')
  const [copyFromId, setCopyFromId] = React.useState<string>('none')
  const [setAsCurrent, setSetAsCurrent] = React.useState(true)

  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newYearName.trim()) return

    await createMutation.mutateAsync({
      name: newYearName.trim(),
      is_current: setAsCurrent,
      copy_grades_from: copyFromId !== 'none' ? Number(copyFromId) : undefined,
    })

    setDialogOpen(false)
    setNewYearName('')
    setCopyFromId('none')
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="h-8 gap-2 border-border/80 bg-background text-xs font-medium text-foreground hover:bg-muted"
          >
            <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
            <span>{currentYear ? currentYear.name : t('app.schoolYearDefault')}</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel className="text-xs text-muted-foreground">
            {t('schoolYear.selector')}
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          {schoolYears.map((year) => (
            <DropdownMenuItem
              key={year.id}
              onClick={() => setCurrentMutation.mutate(year.id)}
              className="flex items-center justify-between text-xs cursor-pointer"
            >
              <span>{year.name}</span>
              {year.id === currentYear?.id && (
                <Check className="h-3.5 w-3.5 text-primary" />
              )}
            </DropdownMenuItem>
          ))}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => setDialogOpen(true)}
            className="gap-2 text-xs text-primary cursor-pointer font-medium"
          >
            <Plus className="h-3.5 w-3.5" />
            {t('schoolYear.newYear')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <form onSubmit={handleCreate}>
            <DialogHeader>
              <DialogTitle>{t('schoolYear.newYear')}</DialogTitle>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('schoolYear.yearName')}
                </label>
                <Input
                  required
                  placeholder={t('schoolYear.yearNamePlaceholder')}
                  value={newYearName}
                  onChange={(e) => setNewYearName(e.target.value)}
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  {t('schoolYear.copyGradesFrom')}
                </label>
                <Select value={copyFromId} onValueChange={setCopyFromId}>
                  <SelectTrigger className="w-full text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">{t('schoolYear.noCopy')}</SelectItem>
                    {schoolYears.map((y) => (
                      <SelectItem key={y.id} value={y.id.toString()}>
                        {y.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-center space-x-2 pt-1">
                <Checkbox
                  id="set-as-current"
                  checked={setAsCurrent}
                  onCheckedChange={(c) => setSetAsCurrent(Boolean(c))}
                />
                <label
                  htmlFor="set-as-current"
                  className="text-xs font-medium text-foreground cursor-pointer"
                >
                  {t('schoolYear.setAsCurrent')}
                </label>
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
                disabled={!newYearName.trim() || createMutation.isPending}
              >
                {t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
