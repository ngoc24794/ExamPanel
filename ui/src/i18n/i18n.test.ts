import { describe, expect, it } from 'vitest'
import vi from './locales/vi.json'
import en from './locales/en.json'

type JsonObject = { [key: string]: unknown }

function getFlatKeys(obj: JsonObject, prefix = ''): string[] {
  let keys: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const fullPath = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys = keys.concat(getFlatKeys(value as JsonObject, fullPath))
    } else {
      keys.push(fullPath)
    }
  }
  return keys
}

describe('i18n locales parity', () => {
  it('vi.json and en.json have identical key sets', () => {
    const viKeys = getFlatKeys(vi as JsonObject).sort()
    const enKeys = getFlatKeys(en as JsonObject).sort()

    expect(viKeys).toEqual(enKeys)
  })

  it('all diagnostic keys are present in both locales', () => {
    const expectedDiagnosticKeys = [
      'diagnostics.duplicate_campus_id',
      'diagnostics.duplicate_campus_code',
      'diagnostics.duplicate_grade_id',
      'diagnostics.duplicate_grade_code',
      'diagnostics.duplicate_exam_id',
      'diagnostics.duplicate_exam_code',
      'diagnostics.duplicate_teacher_id',
      'diagnostics.unknown_campus_ref',
      'diagnostics.load_weight_out_of_range',
      'diagnostics.unknown_teacher_ref',
      'diagnostics.unknown_grade_ref',
      'diagnostics.unknown_exam_ref',
      'diagnostics.duplicate_teacher_grade',
      'diagnostics.duplicate_unavailability',
      'diagnostics.duplicate_rule_key',
      'diagnostics.negative_rule_weight',
      'diagnostics.no_campuses',
      'diagnostics.no_active_teachers',
      'diagnostics.single_campus',
      'diagnostics.no_grades',
      'diagnostics.no_exams',
      'diagnostics.insufficient_setters',
      'diagnostics.insufficient_reviewers',
      'diagnostics.insufficient_panel_teachers',
      'diagnostics.insufficient_campuses',
      'diagnostics.lock_conflict',
      'diagnostics.excess_pinned_setters',
      'diagnostics.excess_pinned_reviewers',
      'diagnostics.excess_pins_in_panel',
      'diagnostics.pinned_teacher_ineligible',
      'diagnostics.pinned_teacher_multiple_panels',
      'diagnostics.pinned_campus_monopoly',
      'diagnostics.pinned_quota_exceeded',
      'diagnostics.insufficient_total_capacity',
      'diagnostics.excess_minimum_capacity',
      'diagnostics.exam_capacity_infeasible',
      'diagnostics.panel_unfillable_under_h4',
      'diagnostics.tight_panel_roster',
      'diagnostics.teacher_single_panel_eligibility',
      'diagnostics.restricted_reviewer_pool',
    ]

    const viKeys = getFlatKeys(vi as JsonObject)
    for (const key of expectedDiagnosticKeys) {
      expect(viKeys).toContain(key)
    }
  })

  it('all soft constraint violation keys are present in both locales', () => {
    const expectedSoftKeys = [
      'soft.reviewer_never',
      'soft.reviewer_too_many',
      'soft.role_imbalance',
      'soft.reviewer_same_campus',
      'soft.setter_pair_repeated',
      'soft.review_relation_repeated',
      'soft.setter_consecutive',
      'soft.grade_not_rotated',
      'soft.load_deviation',
      'soft.exam_crowding',
      'soft.review_subject_missing',
    ]

    const viKeys = getFlatKeys(vi as JsonObject)
    for (const key of expectedSoftKeys) {
      expect(viKeys).toContain(key)
    }
  })

  it('vi.json does not contain the forbidden term "cơ sở" (case-insensitive)', () => {
    const rawVi = JSON.stringify(vi).toLowerCase()
    expect(rawVi.includes('cơ sở')).toBe(false)
  })

  it('vi.json does not contain the forbidden term "kế hoạch" (case-insensitive)', () => {
    const rawVi = JSON.stringify(vi).toLowerCase()
    expect(rawVi.includes('kế hoạch')).toBe(false)
  })

  it('vi.json does not contain the forbidden terms "ràng buộc cứng" or "ràng buộc mềm" (case-insensitive)', () => {
    const rawVi = JSON.stringify(vi).toLowerCase()
    expect(rawVi.includes('ràng buộc cứng')).toBe(false)
    expect(rawVi.includes('ràng buộc mềm')).toBe(false)
  })
})

