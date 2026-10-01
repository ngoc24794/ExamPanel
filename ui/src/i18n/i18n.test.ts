import { describe, expect, it } from 'vitest';
import vi from './locales/vi.json';
import en from './locales/en.json';

type JsonObject = { [key: string]: unknown };

function getFlatKeys(obj: JsonObject, prefix = ''): string[] {
  let keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullPath = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys = keys.concat(getFlatKeys(value as JsonObject, fullPath));
    } else {
      keys.push(fullPath);
    }
  }
  return keys;
}

describe('i18n locales parity', () => {
  it('vi.json and en.json have identical key sets', () => {
    const viKeys = getFlatKeys(vi as JsonObject).sort();
    const enKeys = getFlatKeys(en as JsonObject).sort();

    expect(viKeys).toEqual(enKeys);
  });

  it('all diagnostic keys are present in both locales', () => {
    const expectedDiagnosticKeys = [
      'diagnostics.structural_error',
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
    ];

    const viKeys = getFlatKeys(vi as JsonObject);
    for (const key of expectedDiagnosticKeys) {
      expect(viKeys).toContain(key);
    }
  });
});
