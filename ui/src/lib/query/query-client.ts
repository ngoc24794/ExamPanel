import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import i18n from '@/i18n'

export function getErrorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null) {
    const err = error as {
      code?: string
      message?: string
      params?: Record<string, unknown>
    }
    if (err.code) {
      const key = `errors.${err.code}`
      if (i18n.exists(key)) {
        return i18n.t(key, err.params || {})
      }
      return err.message || err.code
    }
    if (err.message) return err.message
  }
  return String(error)
}

const REPORTED = new WeakSet<object>()

/**
 * Shows the user-facing toast for an error, at most once per error object. A failed mutation is
 * reported by the global handler; pages that also catch the same rejection call this too and must
 * not raise a second toast (RA-041, RA-025).
 */
export function reportError(error: unknown): void {
  if (typeof error === 'object' && error !== null) {
    if (REPORTED.has(error)) return
    REPORTED.add(error)
  }
  toast.error(getErrorMessage(error))
}

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      refetchOnWindowFocus: false,
      retry: false,
    },
    mutations: {
      onError: (error) => {
        reportError(error)
      },
    },
  },
})
