import type { Progress } from '@/lib/api'

export interface AggregateProgress {
  /** Overall completion in [0, 1]; never decreases. */
  fraction: number
  /** Lowest best-score seen over all runs; never increases. */
  bestScore: number
  /** Current score of the run that reported last. */
  currentScore: number
  /** Number of runs that reported at least once. */
  runsStarted: number
}

/**
 * The solver runs several annealing runs in parallel waves and every progress event carries the
 * counters of ONE run. Showing the latest event made the bar and the best score jump backwards
 * (RA-012); this folds the events of all runs into one monotonic overall progress.
 */
export function createProgressAggregator(runs: number, iterationsPerRun: number) {
  const iterations = new Map<number, number>()
  let best = Number.POSITIVE_INFINITY
  let fraction = 0
  const total = Math.max(1, runs) * Math.max(1, iterationsPerRun)

  return {
    update(p: Progress): AggregateProgress {
      iterations.set(
        p.run,
        Math.min(iterationsPerRun, Math.max(iterations.get(p.run) ?? 0, p.iteration)),
      )
      best = Math.min(best, p.best_score)
      let done = 0
      for (const it of iterations.values()) done += it
      fraction = Math.max(fraction, Math.min(1, done / total))
      return {
        fraction,
        bestScore: best,
        currentScore: p.current_score,
        runsStarted: iterations.size,
      }
    },
  }
}
