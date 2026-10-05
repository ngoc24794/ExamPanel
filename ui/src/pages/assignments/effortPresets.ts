export type EffortLevel = 'fast' | 'standard' | 'thorough'

/** Search effort presets shared by the optimize and re-optimize dialogs. */
export function getRunsAndIterations(level: EffortLevel): {
  runs: number
  iterations: number
} {
  switch (level) {
    case 'fast':
      return { runs: 4, iterations: 50000 }
    case 'standard':
      return { runs: 8, iterations: 200000 }
    case 'thorough':
      return { runs: 16, iterations: 500000 }
  }
}
