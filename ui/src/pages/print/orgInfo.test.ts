import { describe, it, expect } from 'vitest'
import { orgInfo, signatureDate } from './orgInfo'

describe('orgInfo (RA-029)', () => {
  it('never invents organisation text', () => {
    const o = orgInfo(undefined)
    expect(o.schoolName).toBe('')
    expect(o.deptName).toBe('')
    expect(o.placeName).toBe('')
    expect(o.signerName).toBe('')
    expect(o.isEmpty).toBe(true)
  })
  it('trims blanks and reports configured data', () => {
    const o = orgInfo({ school_name: ' THPT X ', department_name: '  ' } as never)
    expect(o.schoolName).toBe('THPT X')
    expect(o.deptName).toBe('')
    expect(o.isEmpty).toBe(false)
  })
  it('formats the signature date with and without a place', () => {
    const d = new Date(2026, 9, 5)
    expect(signatureDate('Huế', d)).toBe('Huế, ngày 5 tháng 10 năm 2026')
    expect(signatureDate('', d)).toBe('Ngày 5 tháng 10 năm 2026')
  })
})
