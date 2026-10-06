/**
 * Header of a grade group. Grades are stored as "Khối 10"; a bare number gets the localized
 * prefix so the header never reads "Khối Khối 10" (RA-003).
 */
export function gradeGroupLabel(name: string, prefix: string): string {
  return /^\d+$/.test(name.trim()) ? `${prefix} ${name.trim()}` : name
}
