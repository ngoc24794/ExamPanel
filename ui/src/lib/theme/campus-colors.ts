export const CAMPUS_PALETTE_KEYS = [
  'blue',
  'emerald',
  'amber',
  'purple',
  'rose',
  'indigo',
  'teal',
  'orange',
] as const

export type CampusPaletteKey = (typeof CAMPUS_PALETTE_KEYS)[number]

export interface CampusPaletteOption {
  key: CampusPaletteKey
  labelVi: string
  labelEn: string
  previewColor: string
}

export const CAMPUS_PALETTE_OPTIONS: CampusPaletteOption[] = [
  { key: 'blue', labelVi: 'Xanh dương', labelEn: 'Blue', previewColor: '#3b82f6' },
  { key: 'emerald', labelVi: 'Xanh ngọc', labelEn: 'Emerald', previewColor: '#10b981' },
  { key: 'amber', labelVi: 'Hổ phách', labelEn: 'Amber', previewColor: '#f59e0b' },
  { key: 'purple', labelVi: 'Tím', labelEn: 'Purple', previewColor: '#8b5cf6' },
  { key: 'rose', labelVi: 'Đỏ hồng', labelEn: 'Rose', previewColor: '#f43f5e' },
  { key: 'indigo', labelVi: 'Chàm', labelEn: 'Indigo', previewColor: '#6366f1' },
  { key: 'teal', labelVi: 'Xanh mòng két', labelEn: 'Teal', previewColor: '#14b8a6' },
  { key: 'orange', labelVi: 'Cam', labelEn: 'Orange', previewColor: '#f97316' },
]

export function normalizeCampusColorKey(color: string): CampusPaletteKey {
  const normalized = color.toLowerCase().trim()
  if ((CAMPUS_PALETTE_KEYS as readonly string[]).includes(normalized)) {
    return normalized as CampusPaletteKey
  }
  if (normalized.startsWith('palette-')) {
    const num = parseInt(normalized.replace('palette-', ''), 10)
    if (!isNaN(num) && num >= 1 && num <= CAMPUS_PALETTE_KEYS.length) {
      return CAMPUS_PALETTE_KEYS[num - 1]
    }
  }
  // Fallback for hex codes
  if (normalized === '#3b82f6') return 'blue'
  if (normalized === '#10b981') return 'emerald'
  if (normalized === '#f59e0b') return 'amber'
  if (normalized === '#8b5cf6') return 'purple'
  if (normalized === '#f43f5e') return 'rose'
  if (normalized === '#6366f1') return 'indigo'
  if (normalized === '#14b8a6') return 'teal'
  if (normalized === '#f97316') return 'orange'
  return 'blue'
}

export function getCampusColorStyle(color: string): React.CSSProperties {
  const key = normalizeCampusColorKey(color)
  return {
    backgroundColor: `hsl(var(--campus-${key}-bg))`,
    color: `hsl(var(--campus-${key}-fg))`,
    borderColor: `hsl(var(--campus-${key}-border))`,
  }
}

export function getCampusDotColor(color: string | undefined | null): string {
  if (!color) return '#3b82f6'
  const trimmed = color.trim()
  if (trimmed.startsWith('#') && trimmed.length >= 4) {
    return trimmed
  }
  const key = normalizeCampusColorKey(color)
  const opt = CAMPUS_PALETTE_OPTIONS.find((o) => o.key === key)
  return opt ? opt.previewColor : '#3b82f6'
}
