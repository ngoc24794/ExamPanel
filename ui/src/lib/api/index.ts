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

export const api: ExamPanelApi = isTauriEnvironment() ? tauriApi : mockApi

/**
 * Formats an unknown error or AppError into a localized user-facing message.
 */
export function formatAppError(
  err: unknown,
  t: (key: string, options?: Record<string, unknown>) => string
): string {
  if (!err) return t('errors.unknown_error')
  if (typeof err === 'object' && err !== null && 'code' in err) {
    const appErr = err as { code: string; params?: Record<string, unknown> }
    const key = `errors.${appErr.code}`
    return t(key, { ...appErr.params, defaultValue: appErr.code })
  }
  if (err instanceof Error) return err.message
  return String(err)
}

export * from './types'
export { mockApi, tauriApi }

