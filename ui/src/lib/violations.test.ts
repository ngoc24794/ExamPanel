import { describe, it, expect } from 'vitest'
import i18n from '@/i18n'
import { describeViolation } from './violations'

const lookup = {
  teacherName: (id: number) => ({ 7: 'C Quí' })[id],
  examName: (id: number) => ({ 4: 'CK2' })[id],
  gradeName: (id: number) => ({ 1: 'Khối 10' })[id],
  subjectName: (id: number) => ({ 2: 'VL' })[id],
}

describe('describeViolation (RA-010, RA-021)', () => {
  it('names the teacher, exam and numbers instead of printing the engine code', async () => {
    await i18n.changeLanguage('vi')
    const text = describeViolation(
      i18n.t.bind(i18n),
      {
        rule: 'h4',
        code: 'max_tasks_per_exam_exceeded',
        teacher: 7,
        params: { exam_id: 4, count: 3, limit: 2 },
      },
      lookup,
    )
    expect(text).toContain('C Quí')
    expect(text).toContain('CK2')
    expect(text).toMatch(/3/)
    expect(text).toMatch(/2/)
    expect(text).not.toMatch(/max_tasks_per_exam_exceeded|h4:/i)
  })

  it('translates panel violations and works in English', async () => {
    const v = {
      rule: 'h1',
      code: 'duplicate_teacher_in_panel',
      teacher: 7,
      panel: { exam_id: 4, grade_id: 1, subject_id: 2 },
      params: {},
    } as const
    await i18n.changeLanguage('en')
    const en = describeViolation(i18n.t.bind(i18n), v, lookup)
    expect(en).toContain('C Quí')
    expect(en).toContain('CK2 · Khối 10 · VL')
    expect(en).not.toMatch(/duplicate_teacher/)
    await i18n.changeLanguage('vi')
  })

  it('falls back to a neutral sentence for unknown codes', async () => {
    await i18n.changeLanguage('vi')
    const text = describeViolation(i18n.t.bind(i18n), { rule: 'h7', code: 'brand_new', params: {} })
    expect(text).toContain('H7')
  })
})
