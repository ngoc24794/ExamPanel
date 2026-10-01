import type { AppInfo, ExamPanelApi, ThemeMode } from './types'

const THEME_STORAGE_KEY = 'exampanel_theme_mode'

export class MockExamPanelApi implements ExamPanelApi {
  async ping(): Promise<string> {
    return 'pong from Mock Engine (browser mode)'
  }

  async getAppInfo(): Promise<AppInfo> {
    return {
      name: 'ExamPanel',
      version: '0.1.0',
      identifier: 'vn.exampanel.app',
      mode: 'mock',
    }
  }

  async getTheme(): Promise<ThemeMode | null> {
    const saved = localStorage.getItem(THEME_STORAGE_KEY)
    if (saved === 'light' || saved === 'dark' || saved === 'system') {
      return saved
    }
    return null
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    localStorage.setItem(THEME_STORAGE_KEY, theme)
  }
}

export const mockApi = new MockExamPanelApi()
