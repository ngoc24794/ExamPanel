/**
 * SQLite `datetime('now')` yields `YYYY-MM-DD HH:MM:SS` in UTC. WebKit (WebKitGTK/Safari) cannot
 * parse that form and returns "Invalid Date", so it is normalised to ISO 8601 first (RA-024).
 */
export function toIsoUtc(value: string): string {
  const m =
    /^(\d{4}-\d{2}-\d{2})[ T](\d{2}:\d{2}:\d{2}(?:\.\d+)?)(Z|[+-]\d{2}:?\d{2})?$/.exec(
      value.trim(),
    )
  if (!m) return value
  return `${m[1]}T${m[2]}${m[3] ?? 'Z'}`
}

/** Parses a database timestamp; returns null when it is not a valid date. */
export function parseDbDate(value: string | null | undefined): Date | null {
  if (!value) return null
  const d = new Date(toIsoUtc(value))
  return Number.isNaN(d.getTime()) ? null : d
}

/** Locale date/time text for a database timestamp, or an empty string if unparsable. */
export function formatDbDateTime(
  value: string | null | undefined,
  locale?: string,
): string {
  const d = parseDbDate(value)
  return d ? d.toLocaleString(locale) : ''
}
