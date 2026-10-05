import { describe, it, expect } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

// RA-042: user-facing Vietnamese text must come from t('domain.key') so the English UI is
// complete. Files below intentionally contain Vietnamese and are exempt:
//  - api/mock.ts and fixtures: demo/mock data shown as user data, not UI chrome
//  - print/*: the printed sheets are official Vietnamese documents
//  - theme/campus-colors.ts: a bilingual colour-name table (labelVi / labelEn)
const EXEMPT = [
  'lib/api/mock.ts',
  'lib/api/fixtures/',
  'lib/api/generated/',
  'pages/print/',
  'lib/theme/campus-colors.ts',
  'i18n/locales/',
]

const SRC = path.resolve(__dirname, '..')
const VIETNAMESE =
  /[ăâđêôơưàáạảãằắặẳẵầấậẩẫèéẹẻẽềếệểễìíịỉĩòóọỏõồốộổỗờớợởỡùúụủũừứựửữỳýỵỷỹĂÂĐÊÔƠƯÀÁẠẢÃẰẮẶẲẴẦẤẬẨẪÈÉẸẺẼỀẾỆỂỄÌÍỊỈĨÒÓỌỎÕỒỐỘỔỖỜỚỢỞỠÙÚỤỦŨỪỨỰỬỮỲÝỴỶỸ]/

function walk(dir: string, out: string[] = []): string[] {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name)
    if (e.isDirectory()) walk(p, out)
    else if (/\.(ts|tsx)$/.test(e.name) && !/\.test\.(ts|tsx)$/.test(e.name)) out.push(p)
  }
  return out
}

function codeLines(text: string): string[] {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .split('\n')
    .filter((l) => !/^\s*\/\//.test(l))
    .map((l) => l.replace(/\s\/\/\s.*$/, ''))
}

describe('no hard-coded Vietnamese UI text (RA-042)', () => {
  it('every component renders text through t()', () => {
    const offenders: string[] = []
    for (const file of walk(SRC)) {
      const rel = path.relative(SRC, file).replace(/\\/g, '/')
      if (EXEMPT.some((e) => rel.startsWith(e))) continue
      codeLines(fs.readFileSync(file, 'utf8')).forEach((line) => {
        if (VIETNAMESE.test(line)) offenders.push(`${rel}: ${line.trim().slice(0, 110)}`)
      })
    }
    expect(offenders, `hard-coded Vietnamese:\n${offenders.join('\n')}`).toEqual([])
  })
})
