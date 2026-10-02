//! Simulated Annealing optimization engine.
//!
//! Implements auto-calibrated T0, geometric cooling, Metropolis acceptance criterion,
//! throttled progress reporting, and cooperative cancellation.

use super::state::{DensePanel, IncrementalState};
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Optimization budget specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    /// Deterministic number of move iterations.
    Iterations(u64),
    /// Real-time deadline in milliseconds.
    TimeMs(u64),
}

use serde::{Deserialize, Serialize};

/// Progress snapshot sent to callers during optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct Progress {
    pub run: usize,
    #[ts(type = "number")]
    pub iteration: u64,
    pub best_score: f64,
    pub current_score: f64,
    #[ts(type = "number")]
    pub elapsed_ms: u64,
}

/// Auto-calibrates the initial temperature T0 so that ~80% of worsening moves are accepted.
#[must_use]
pub fn calibrate_initial_temperature(state: &mut IncrementalState, rng: &mut ChaCha8Rng) -> f64 {
    let mut worsening_deltas = Vec::with_capacity(100);

    for _ in 0..200 {
        if let Some(m) = state.sample_candidate_move(rng) {
            let delta = state.try_apply_move(m);
            if delta > 1e-9 {
                worsening_deltas.push(delta);
            }
            state.revert_move(m, delta);
            if worsening_deltas.len() >= 50 {
                break;
            }
        }
    }

    if worsening_deltas.is_empty() {
        return 5.0; // Reasonable default
    }

    let avg_delta = worsening_deltas.iter().sum::<f64>() / worsening_deltas.len() as f64;
    // We want exp(-avg_delta / T0) = 0.80 => -avg_delta / T0 = ln(0.80) => T0 = -avg_delta / ln(0.80)
    let t0 = -avg_delta / 0.80_f64.ln();
    t0.max(0.1)
}

/// Runs a single thread of simulated annealing.
pub fn run_simulated_annealing(
    mut state: IncrementalState,
    seed: u64,
    budget: Budget,
    run_idx: usize,
    cancel: Option<&Arc<AtomicBool>>,
    progress: Option<&Arc<dyn Fn(Progress) + Send + Sync>>,
) -> (Vec<DensePanel>, f64, u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let t0 = calibrate_initial_temperature(&mut state, &mut rng);
    let t_end = 1e-4_f64;

    let mut best_panels = state.panels.clone();
    let mut best_penalty = state.current_penalty;

    let total_steps: u64 = match budget {
        Budget::Iterations(n) => n.max(1),
        Budget::TimeMs(ms) => (ms * 1000).max(1), // Estimate iterations for cooling rate
    };

    let cooling_factor = (t_end / t0).powf(1.0 / total_steps as f64);
    let mut temp = t0;

    let start_time = Instant::now();
    let mut last_progress_time = Instant::now();
    let throttle_interval = std::time::Duration::from_millis(100);

    let mut iteration = 0u64;

    loop {
        // Budget and termination check
        match budget {
            Budget::Iterations(max_iter) => {
                if iteration >= max_iter {
                    break;
                }
            }
            Budget::TimeMs(max_ms) => {
                if start_time.elapsed().as_millis() >= max_ms as u128 {
                    break;
                }
            }
        }

        // Cooperative cancellation check (every 512 iterations)
        if (iteration & 511) == 0 {
            if let Some(flag) = cancel {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
            }
        }

        // Progress reporting (throttled to <= 10 calls/s)
        if (iteration & 1023) == 0 {
            if let Some(cb) = progress {
                if last_progress_time.elapsed() >= throttle_interval {
                    cb(Progress {
                        run: run_idx,
                        iteration,
                        best_score: best_penalty,
                        current_score: state.current_penalty,
                        elapsed_ms: start_time.elapsed().as_millis() as u64,
                    });
                    last_progress_time = Instant::now();
                }
            }
        }

        iteration += 1;

        // Sample and evaluate move
        if let Some(m) = state.sample_candidate_move(&mut rng) {
            let delta = state.try_apply_move(m);

            let accept = if delta <= 0.0 {
                true
            } else {
                let prob = (-delta / temp).exp();
                let r = (rng.next_u32() as f64) / (u32::MAX as f64);
                r < prob
            };

            if accept {
                if state.current_penalty < best_penalty {
                    best_penalty = state.current_penalty;
                    best_panels = state.panels.clone();
                }
            } else {
                state.revert_move(m, delta);
            }
        }

        // Cool down
        temp = (temp * cooling_factor).max(t_end);
    }

    // Final progress callback
    if let Some(cb) = progress {
        cb(Progress {
            run: run_idx,
            iteration,
            best_score: best_penalty,
            current_score: state.current_penalty,
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        });
    }

    (best_panels, best_penalty, iteration)
}
