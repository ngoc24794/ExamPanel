import { describe, it, expect } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
import vi from './locales/vi.json'
import en from './locales/en.json'

const SRC = path.resolve(__dirname, '..')

function walk(dir: string, out: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      if (entry.name === 'generated' || entry.name === 'node_modules') continue
      walk(p, out)
    } else if (/\.(ts|tsx)$/.test(entry.name) && !/\.test\.(ts|tsx)$/.test(entry.name)) {
      out.push(p)
    }
  }
  return out
}

/** Static keys used as `t('a.b')`, `i18n.t("a.b")`, `t(\n'a.b'`; dynamic keys are skipped. */
function usedKeys(): Map<string, string[]> {
  const found = new Map<string, string[]>()
  const re = /\bt\(\s*(['"`])([A-Za-z0-9_.-]+)\1/g
  for (const file of walk(SRC)) {
    const text = fs.readFileSync(file, 'utf8')
    for (const m of text.matchAll(re)) {
      const key = m[2]
      if (!key.includes('.')) continue
      const list = found.get(key) ?? []
      list.push(path.relative(SRC, file))
      found.set(key, list)
    }
  }
  return found
}

const PLURAL_SUFFIXES = ['', '_zero', '_one', '_two', '_few', '_many', '_other']

function has(tree: unknown, key: string): boolean {
  return PLURAL_SUFFIXES.some((suffix) => {
    let node = tree as Record<string, unknown> | string | undefined
    for (const part of (key + suffix).split('.')) {
      if (typeof node !== 'object' || node === null || !(part in node)) return false
      node = (node as Record<string, unknown>)[part] as typeof node
    }
    return typeof node === 'string'
  })
}

function flatten(tree: unknown, prefix = '', out: string[] = []): string[] {
  for (const [k, v] of Object.entries(tree as Record<string, unknown>)) {
    const p = prefix ? `${prefix}.${k}` : k
    if (typeof v === 'object' && v !== null) flatten(v, p, out)
    else out.push(p)
  }
  return out
}

/** Minimal JSON walker that reports duplicate keys inside one object (JSON.parse hides them). */
function duplicateKeys(text: string): string[] {
  const dups: string[] = []
  let i = 0
  const ws = () => {
    while (/\s/.test(text[i])) i++
  }
  const str = (): string => {
    const start = i++
    while (text[i] !== '"') i += text[i] === '\\' ? 2 : 1
    i++
    return JSON.parse(text.slice(start, i))
  }
  const value = (where: string): void => {
    ws()
    if (text[i] === '{') {
      i++
      const seen = new Set<string>()
      ws()
      while (text[i] !== '}') {
        ws()
        const key = str()
        const full = where ? `${where}.${key}` : key
        if (seen.has(key)) dups.push(full)
        seen.add(key)
        ws()
        i++ // :
        value(full)
        ws()
        if (text[i] === ',') i++
        ws()
      }
      i++
    } else if (text[i] === '[') {
      i++
      ws()
      while (text[i] !== ']') {
        value(where)
        ws()
        if (text[i] === ',') i++
        ws()
      }
      i++
    } else if (text[i] === '"') {
      str()
    } else {
      while (i < text.length && !/[,}\]\s]/.test(text[i])) i++
    }
  }
  value('')
  return dups
}

describe('i18n completeness (RA-040)', () => {
  const used = usedKeys()

  it('finds the translation keys used by the source', () => {
    expect(used.size).toBeGreaterThan(300)
  })

  it('every static t() key exists in BOTH vi.json and en.json', () => {
    const missing: string[] = []
    for (const [key, files] of used) {
      const where = [!has(vi, key) && 'vi', !has(en, key) && 'en']
        .filter(Boolean)
        .join('+')
      if (where) missing.push(`${key} [${where}] ← ${[...new Set(files)].join(', ')}`)
    }
    expect(missing, `missing keys:\n${missing.join('\n')}`).toEqual([])
  })

  it('vi.json and en.json define exactly the same keys', () => {
    const viKeys = new Set(flatten(vi))
    const enKeys = new Set(flatten(en))
    const onlyVi = [...viKeys].filter((k) => !enKeys.has(k))
    const onlyEn = [...enKeys].filter((k) => !viKeys.has(k))
    expect({ onlyVi, onlyEn }).toEqual({ onlyVi: [], onlyEn: [] })
  })

  it('locale files contain no duplicate JSON keys (the last one silently wins)', () => {
    for (const file of ['vi.json', 'en.json']) {
      const text = fs.readFileSync(path.join(__dirname, 'locales', file), 'utf8')
      expect(duplicateKeys(text), file).toEqual([])
    }
  })
})
