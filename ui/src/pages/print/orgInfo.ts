import type { AppSettings } from '@/lib/api'

/** Organisation block of printed sheets. Never invents text: an empty setting stays empty (RA-029). */
export interface OrgInfo {
  schoolName: string
  deptName: string
  signerTitle: string
  signerName: string
  placeName: string
  /** True when none of the identifying fields has been configured in Settings. */
  isEmpty: boolean
}

const clean = (v: string | null | undefined): string => (v ?? '').trim()

export function orgInfo(settings: AppSettings | undefined | null): OrgInfo {
  const schoolName = clean(settings?.school_name)
  const deptName = clean(settings?.department_name)
  const placeName = clean(settings?.place_name)
  const signerName = clean(settings?.signer_name)
  return {
    schoolName,
    deptName,
    placeName,
    signerName,
    signerTitle: clean(settings?.signer_title) || 'TỔ TRƯỞNG CHUYÊN MÔN',
    isEmpty: !schoolName && !deptName && !placeName && !signerName,
  }
}

/** "Place, ngày d tháng m năm y", or "Ngày d tháng m năm y" without a place. */
export function signatureDate(placeName: string, date: Date): string {
  const rest = `${date.getDate()} tháng ${date.getMonth() + 1} năm ${date.getFullYear()}`
  return placeName ? `${placeName}, ngày ${rest}` : `Ngày ${rest}`
}
