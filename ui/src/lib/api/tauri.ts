import { invoke } from '@tauri-apps/api/core'
import type { AppInfo, ExamPanelApi, ThemeMode } from './types'

const THEME_STORAGE_KEY = 'exampanel_theme_mode'

export class TauriExamPanelApi implements ExamPanelApi {
  async ping(): Promise<string> {
    return await invoke<string>('ping')
  }

  async getAppInfo(): Promise<AppInfo> {
    return {
      name: 'ExamPanel',
      version: '0.1.0',
      identifier: 'vn.exampanel.app',
      mode: 'tauri',
    }
  }

  async getTheme(): Promise<ThemeMode | null> {
    // TODO (Phase 2): Read theme preference from SQLite settings repository
    const saved = localStorage.getItem(THEME_STORAGE_KEY)
    if (saved === 'light' || saved === 'dark' || saved === 'system') {
      return saved
    }
    return null
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    // TODO (Phase 2): Persist theme preference into SQLite settings repository
    localStorage.setItem(THEME_STORAGE_KEY, theme)
  }
}

export const tauriApi = new TauriExamPanelApi()
