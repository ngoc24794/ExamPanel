import { mockApi } from './mock'
import { tauriApi } from './tauri'
import type { ExamPanelApi } from './types'

/**
 * Detects if running inside Tauri desktop environment.
 */
export function isTauriEnvironment(): boolean {
  if (typeof window === 'undefined') return false
  return '__TAURI_INTERNALS__' in window || '__TAURI__' in window
}

/**
 * Automatically selected API instance.
 * Uses Tauri IPC bridge if in desktop app, otherwise falls back to mock API for browser development.
 */
export const api: ExamPanelApi = isTauriEnvironment() ? tauriApi : mockApi

export * from './types'
export { mockApi, tauriApi }
