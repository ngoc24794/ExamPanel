import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Building2,
  Check,
  Copy,
  FolderOpen,
  Globe,
  HardDrive,
  Info,
  Laptop,
  Moon,
  Palette,
  Save,
  Sun,
} from 'lucide-react'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { BackupSection } from './settings/BackupSection'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useTheme } from '@/lib/theme'
import { useAppInfo, useSettings } from '@/lib/query/hooks'
import { api } from '@/lib/api'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'

export const SettingsPage: React.FC = () => {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const { data: appInfo } = useAppInfo()
  const { data: settings, refetch: refetchSettings } = useSettings()

  const [copied, setCopied] = React.useState(false)
  const [schoolName, setSchoolName] = React.useState('')
  const [departmentName, setDepartmentName] = React.useState('')
  const [signerTitle, setSignerTitle] = React.useState('')
  const [signerName, setSignerName] = React.useState('')
  const [placeName, setPlaceName] = React.useState('')
  const [savingOrg, setSavingOrg] = React.useState(false)

  React.useEffect(() => {
    if (settings) {
      setSchoolName(settings.school_name || '')
      setDepartmentName(settings.department_name || '')
      setSignerTitle(settings.signer_title || '')
      setSignerName(settings.signer_name || '')
      setPlaceName(settings.place_name || '')
    }
  }, [settings])

  const handleSaveOrgInfo = async (e: React.FormEvent) => {
    e.preventDefault()
    setSavingOrg(true)
    try {
      await api.setSetting('school_name', schoolName.trim())
      await api.setSetting('department_name', departmentName.trim())
      await api.setSetting('signer_title', signerTitle.trim())
      await api.setSetting('signer_name', signerName.trim())
      await api.setSetting('place_name', placeName.trim())
      await refetchSettings()
      toast.success(t('settings.saveOrgInfoSuccess'))
    } catch (err) {
      toast.error(getErrorMessage(err))
    } finally {
      setSavingOrg(false)
    }
  }

  const handleCopyPath = () => {
    if (!appInfo?.data_dir) return
    navigator.clipboard.writeText(appInfo.data_dir).then(() => {
      setCopied(true)
      toast.success(t('settings.copySuccess'))
      setTimeout(() => setCopied(false), 2000)
    })
  }

  const handleOpenFolder = async () => {
    try {
      await api.openDataFolder()
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  const isMock = appInfo?.mode === 'mock'

  return (
    <div className="space-y-8 max-w-4xl" data-testid="settings-page">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {t('settings.title')}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">{t('settings.description')}</p>
      </div>

      <div className="space-y-6">
        {/* Section 1: Appearance & Language */}
        <Card className="bg-card border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
              <Palette className="h-4 w-4 text-primary" />
              <span>{t('settings.appearanceSection')}</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-6 pt-2">
            {/* Theme Toggle Buttons */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-4 border-b border-border">
              <div>
                <label className="text-xs font-semibold text-foreground">
                  {t('settings.themeLabel')}
                </label>
                <p className="text-[11px] text-muted-foreground mt-0.5">
                  Tùy chỉnh giao diện hiển thị sáng, tối hoặc theo hệ điều hành.
                </p>
              </div>
              <div className="flex items-center gap-1.5 p-1 bg-muted/50 rounded-lg border border-border">
                <Button
                  variant={theme === 'light' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setTheme('light')}
                  className="h-7 px-2.5 text-xs gap-1.5"
                  data-testid="theme-light-btn"
                >
                  <Sun className="h-3.5 w-3.5" />
                  <span>{t('theme.light')}</span>
                </Button>
                <Button
                  variant={theme === 'dark' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setTheme('dark')}
                  className="h-7 px-2.5 text-xs gap-1.5"
                  data-testid="theme-dark-btn"
                >
                  <Moon className="h-3.5 w-3.5" />
                  <span>{t('theme.dark')}</span>
                </Button>
                <Button
                  variant={theme === 'system' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setTheme('system')}
                  className="h-7 px-2.5 text-xs gap-1.5"
                  data-testid="theme-system-btn"
                >
                  <Laptop className="h-3.5 w-3.5" />
                  <span>{t('theme.system')}</span>
                </Button>
              </div>
            </div>

            {/* Language Switcher */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <label className="text-xs font-semibold text-foreground">
                  {t('settings.languageLabel')}
                </label>
                <p className="text-[11px] text-muted-foreground mt-0.5">
                  Chọn ngôn ngữ giao diện (Tiếng Việt mặc định hoặc English).
                </p>
              </div>
              <div className="flex items-center gap-1.5 p-1 bg-muted/50 rounded-lg border border-border">
                <Button
                  variant={i18n.language === 'vi' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => i18n.changeLanguage('vi')}
                  className="h-7 px-3 text-xs gap-1.5"
                  data-testid="lang-vi-btn"
                >
                  <Globe className="h-3.5 w-3.5" />
                  <span>{t('language.vi')}</span>
                </Button>
                <Button
                  variant={i18n.language === 'en' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => i18n.changeLanguage('en')}
                  className="h-7 px-3 text-xs gap-1.5"
                  data-testid="lang-en-btn"
                >
                  <Globe className="h-3.5 w-3.5" />
                  <span>{t('language.en')}</span>
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Section 2: Organization Info */}
        <Card className="bg-card border-border" data-testid="org-info-card">
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
              <Building2 className="h-4 w-4 text-primary" />
              <span>{t('settings.organizationSection')}</span>
            </CardTitle>
            <p className="text-xs text-muted-foreground mt-0.5">
              {t('settings.organizationDesc')}
            </p>
          </CardHeader>
          <CardContent className="pt-2">
            <form onSubmit={handleSaveOrgInfo} className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-1.5 sm:col-span-2">
                  <label className="text-xs font-semibold text-foreground">
                    {t('settings.schoolName')}
                  </label>
                  <Input
                    value={schoolName}
                    onChange={(e) => setSchoolName(e.target.value)}
                    placeholder={t('settings.schoolNamePlaceholder')}
                    data-testid="org-school-name-input"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    {t('settings.departmentName')}
                  </label>
                  <Input
                    value={departmentName}
                    onChange={(e) => setDepartmentName(e.target.value)}
                    placeholder={t('settings.departmentNamePlaceholder')}
                    data-testid="org-department-name-input"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    {t('settings.placeName')}
                  </label>
                  <Input
                    value={placeName}
                    onChange={(e) => setPlaceName(e.target.value)}
                    placeholder={t('settings.placeNamePlaceholder')}
                    data-testid="org-place-name-input"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    {t('settings.signerTitle')}
                  </label>
                  <Input
                    value={signerTitle}
                    onChange={(e) => setSignerTitle(e.target.value)}
                    placeholder={t('settings.signerTitlePlaceholder')}
                    data-testid="org-signer-title-input"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    {t('settings.signerName')}
                  </label>
                  <Input
                    value={signerName}
                    onChange={(e) => setSignerName(e.target.value)}
                    placeholder={t('settings.signerNamePlaceholder')}
                    data-testid="org-signer-name-input"
                  />
                </div>
              </div>

              <div className="flex justify-end pt-2">
                <Button
                  type="submit"
                  size="sm"
                  disabled={savingOrg}
                  className="gap-1.5 text-xs"
                  data-testid="save-org-info-btn"
                >
                  <Save className="h-3.5 w-3.5" />
                  <span>{savingOrg ? '...' : t('settings.saveOrgInfo')}</span>
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        {/* Section 3: Data Storage */}
        <Card className="bg-card border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
              <HardDrive className="h-4 w-4 text-primary" />
              <span>{t('settings.storageSection')}</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 pt-2">
            <div className="flex items-center justify-between pb-3 border-b border-border">
              <span className="text-xs font-semibold text-foreground">
                {t('settings.storageMode')}
              </span>
              <Badge
                variant="outline"
                className="font-mono text-xs"
                data-testid="storage-mode-badge"
              >
                {appInfo?.is_portable
                  ? t('settings.modePortable')
                  : t('settings.modeAppData')}
              </Badge>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                {t('settings.dataPath')}
              </label>
              <div className="flex items-center gap-2">
                <div
                  className="flex-1 p-2.5 rounded-md border border-border bg-muted/40 font-mono text-xs text-foreground truncate select-all"
                  data-testid="data-path-display"
                >
                  {appInfo?.data_dir || '...'}
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCopyPath}
                  className="h-9 gap-1.5 px-3 text-xs shrink-0"
                  data-testid="copy-path-btn"
                >
                  {copied ? (
                    <Check className="h-3.5 w-3.5 text-emerald-500" />
                  ) : (
                    <Copy className="h-3.5 w-3.5" />
                  )}
                  <span>{t('settings.copyPath')}</span>
                </Button>

                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span>
                        <Button
                          variant="secondary"
                          size="sm"
                          disabled={isMock}
                          onClick={handleOpenFolder}
                          className="h-9 gap-1.5 px-3 text-xs shrink-0"
                          data-testid="open-folder-btn"
                        >
                          <FolderOpen className="h-3.5 w-3.5" />
                          <span>{t('settings.openFolder')}</span>
                        </Button>
                      </span>
                    </TooltipTrigger>
                    {isMock && (
                      <TooltipContent className="text-xs">
                        {t('settings.openFolderDisabledMock')}
                      </TooltipContent>
                    )}
                  </Tooltip>
                </TooltipProvider>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Section: Backup & Restore */}
        <BackupSection />

        {/* Section 3: About ExamPanel */}
        <Card className="bg-card border-border" data-testid="about-section">
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
              <Info className="h-4 w-4 text-primary" />
              <span>{t('settings.aboutSection')}</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 pt-2 text-xs">
            <div className="flex items-center justify-between pb-2 border-b border-border">
              <span className="text-muted-foreground">{t('settings.version')}</span>
              <span className="font-mono font-semibold text-foreground" data-testid="about-version">
                v{appInfo?.version || '0.1.0'}
              </span>
            </div>
            <div className="flex items-center justify-between pb-2 border-b border-border">
              <span className="text-muted-foreground">{t('settings.commitHash')}</span>
              <span className="font-mono text-foreground" data-testid="about-commit">
                {appInfo?.commit_hash || '0c24c8e'}
              </span>
            </div>
            <div className="flex items-center justify-between pb-2 border-b border-border">
              <span className="text-muted-foreground">{t('settings.buildDate')}</span>
              <span className="font-mono text-foreground" data-testid="about-build-date">
                {appInfo?.build_date || '2026-10-02'}
              </span>
            </div>
            <div className="flex items-center justify-between pb-2 border-b border-border">
              <span className="text-muted-foreground">{t('settings.storageMode')}</span>
              <Badge variant={appInfo?.is_portable ? 'default' : 'secondary'} className="text-[10px]">
                {appInfo?.is_portable ? t('settings.portableMode') : t('settings.installedMode')}
              </Badge>
            </div>
            <div className="flex flex-col gap-1 pb-2 border-b border-border">
              <span className="text-muted-foreground">{t('settings.dataLocation')}</span>
              <span className="font-mono text-[11px] text-foreground break-all bg-muted/50 p-2 rounded border border-border">
                {appInfo?.data_dir || '/data'}
              </span>
            </div>
            <p className="text-muted-foreground pt-1">{t('settings.releaseNotes')}</p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
