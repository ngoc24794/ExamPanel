import * as React from 'react'

/**
 * Runs an async action at most once at a time. State such as `isPending` only updates after the
 * next render, so a fast double click used to send the request twice (RA-041); the ref flips
 * synchronously and the second call is ignored.
 */
export function useSingleFlight() {
  const busy = React.useRef(false)
  return React.useCallback(
    async <T>(action: () => Promise<T>): Promise<T | undefined> => {
      if (busy.current) return undefined
      busy.current = true
      try {
        return await action()
      } finally {
        busy.current = false
      }
    },
    [],
  )
}
