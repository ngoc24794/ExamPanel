export interface AppInfo {
  name: string
  version: string
  identifier: string
  mode: 'tauri' | 'mock'
}

export type ThemeMode = 'light' | 'dark' | 'system'

export interface ExamPanelApi {
  /**
   * Ping backend core to check connection and obtain status string.
   */
  ping(): Promise<string>

  /**
   * Retrieves application metadata.
   */
  getAppInfo(): Promise<AppInfo>

  /**
   * Retrieves persisted theme setting.
   */
  getTheme(): Promise<ThemeMode | null>

  /**
   * Persists theme setting.
   */
  setTheme(theme: ThemeMode): Promise<void>
}
