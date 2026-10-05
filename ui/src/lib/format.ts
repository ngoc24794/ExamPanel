/** Penalty units: whole numbers stay whole, fractions show at most two decimals (RA-016). */
export function formatUnits(units: number): string {
  if (!Number.isFinite(units)) return '-'
  return Number.isInteger(units) ? String(units) : String(Number(units.toFixed(2)))
}
