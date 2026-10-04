//! Local search optimization and multi-plan generation.
//!
//! Provides simulated annealing local search with:
//! - Strict H1–H7 constraint preservation
//! - O(1) incremental delta evaluation
//! - Parallel multi-start execution via Rayon
//! - Deterministic seed derivation
//! - Max-min diversity plan selection

pub mod anneal;
pub mod eval_edit;
pub mod moves;
pub mod state;

pub use anneal::{Budget, Progress};
pub use eval_edit::{evaluate_candidates, evaluate_swap, CandidateEval, SlotRef};
pub use moves::LocalMove;
pub use state::IncrementalState;

/// Pre-defined search effort presets balancing runtime vs solution depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationEffort {
    /// Fast: 4 independent runs x 25,000 iterations (Nhanh)
    Fast,
    /// Standard: 8 independent runs x 50,000 iterations (Chuẩn)
    Standard,
    /// Thorough: 16 independent runs x 100,000 iterations (Kỹ)
    Thorough,
}

impl OptimizationEffort {
    #[must_use]
    pub const fn runs_and_iterations(&self) -> (usize, u64) {
        match self {
            Self::Fast => (4, 25_000),
            Self::Standard => (8, 50_000),
            Self::Thorough => (16, 100_000),
        }
    }
}

use crate::domain::{Assignment, Problem};
use crate::score::{evaluate, ScoreReport};
use crate::solver::{solve_hard, SolveError, SolveOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

/// Configuration options for the local search optimizer.
#[derive(Clone)]
pub struct OptimizeOptions {
    /// PRNG base seed for deterministic reproducibility.
    pub base_seed: u64,
    /// Optimization budget (iterations or time deadline).
    pub budget: Budget,
    /// Number of independent parallel annealing runs (default 8).
    pub num_runs: usize,
    /// Maximum number of diverse plans to return (default 3).
    pub max_plans: usize,
    /// Minimum fractional slot diversity threshold between returned plans (default 0.20).
    pub diversity_threshold: f64,
    /// Optional cancellation token for responsive GUI abortion.
    pub cancel: Option<Arc<AtomicBool>>,
    /// Optional progress callback throttled to <= 10 calls/s.
    pub progress: Option<Arc<dyn Fn(Progress) + Send + Sync>>,
    /// Optional initial plan (if None, `solve_hard` is used).
    pub initial_assignments: Option<Vec<Assignment>>,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            base_seed: 42,
            budget: Budget::Iterations(50_000),
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        }
    }
}

/// A ranked solution plan returned from multi-start optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct RankedPlan {
    pub rank: usize,
    #[ts(type = "number")]
    pub seed: u64,
    pub assignments: Vec<Assignment>,
    pub report: ScoreReport,
}

/// Overall execution statistics from the multi-plan optimization run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct OptimizeStats {
    pub total_runs: usize,
    #[ts(type = "number")]
    pub total_iterations: u64,
    #[ts(type = "number")]
    pub elapsed_ms: u64,
}

/// The complete output of multi-start local search optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizeResult {
    pub plans: Vec<RankedPlan>,
    pub initial_report: ScoreReport,
    pub stats: OptimizeStats,
}

/// Calculates the normalized distance between two complete plans in [0.0, 1.0].
///
/// Distance is defined as the fraction of (exam, grade, role, teacher) slots that differ.
#[must_use]
pub fn plan_distance(p1: &[Assignment], p2: &[Assignment]) -> f64 {
    if p1.is_empty() || p2.is_empty() {
        return 0.0;
    }

    let mut set2 = HashSet::with_capacity(p2.len());
    for a in p2 {
        set2.insert((a.exam_id, a.grade_id, a.role, a.teacher_id));
    }

    let mut differing = 0usize;
    for a in p1 {
        if !set2.contains(&(a.exam_id, a.grade_id, a.role, a.teacher_id)) {
            differing += 1;
        }
    }

    differing as f64 / p1.len() as f64
}

/// Runs parallel simulated annealing multi-plan optimization.
pub fn optimize(problem: &Problem, opts: &OptimizeOptions) -> Result<OptimizeResult, SolveError> {
    let start_time = Instant::now();

    // 1. Obtain initial valid plan
    let initial_assignments = if let Some(ref assigns) = opts.initial_assignments {
        assigns.clone()
    } else {
        let hard_opts = SolveOptions {
            seed: opts.base_seed,
            time_limit_ms: 3000,
            max_nodes: 1_000_000,
        };
        let sol = solve_hard(problem, &hard_opts)?;
        sol.assignments
    };

    let initial_report = evaluate(problem, &initial_assignments);

    // 2. Prepare independent seed tasks
    let num_runs = opts.num_runs.max(1);
    let run_seeds: Vec<(usize, u64)> = (0..num_runs)
        .map(|r| {
            let seed = opts
                .base_seed
                .wrapping_add((r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) + 1);
            (r, seed)
        })
        .collect();

    // 3. Execute parallel annealing runs
    let base_state = IncrementalState::new(problem, &initial_assignments);

    let cancel_ref = opts.cancel.as_ref();
    let progress_ref = opts.progress.as_ref();

    let completed_runs: Vec<(usize, u64, Vec<Assignment>, ScoreReport, u64)> = run_seeds
        .into_par_iter()
        .map(|(run_idx, seed)| {
            let state = base_state.clone();
            let (best_panels, _best_penalty, iters) = anneal::run_simulated_annealing(
                state,
                seed,
                opts.budget,
                run_idx,
                cancel_ref,
                progress_ref,
            );

            // Reconstruct state to export assignments
            let mut final_state = base_state.clone();
            final_state.panels = best_panels;
            let assignments = final_state.to_assignments(crate::domain::PlanId(0));
            let report = evaluate(problem, &assignments);

            (run_idx, seed, assignments, report, iters)
        })
        .collect();

    let total_iterations: u64 = completed_runs
        .iter()
        .map(|(_, _, _, _, iters)| *iters)
        .sum();

    // 4. Sort completed runs by score (lowest penalty first)
    let mut candidates = completed_runs;
    candidates.sort_by(|a, b| {
        a.3.total
            .partial_cmp(&b.3.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 5. Diversity selection (greedy max-min diversity)
    let max_k = opts.max_plans.max(1);
    let mut selected: Vec<(usize, u64, Vec<Assignment>, ScoreReport)> = Vec::with_capacity(max_k);

    if let Some((_, seed, assigns, report, _)) = candidates.first() {
        selected.push((1, *seed, assigns.clone(), report.clone()));
    }

    // Pick subsequent plans
    while selected.len() < max_k {
        let mut best_candidate_idx = None;
        let mut best_min_dist = -1.0;

        for (idx, (_, _, assigns, _, _)) in candidates.iter().enumerate() {
            // Check if already selected (by seed or exact assignments)
            if selected.iter().any(|s| s.1 == candidates[idx].1) {
                continue;
            }

            // Min distance to already selected plans
            let mut min_d = f64::MAX;
            for s in &selected {
                let d = plan_distance(assigns, &s.2);
                if d < min_d {
                    min_d = d;
                }
            }

            if min_d >= opts.diversity_threshold && min_d > best_min_dist {
                best_min_dist = min_d;
                best_candidate_idx = Some(idx);
            }
        }

        if let Some(idx) = best_candidate_idx {
            let (_, seed, assigns, report, _) = &candidates[idx];
            let next_rank = selected.len() + 1;
            selected.push((next_rank, *seed, assigns.clone(), report.clone()));
        } else {
            // No remaining candidate satisfies the diversity threshold
            break;
        }
    }

    let plans: Vec<RankedPlan> = selected
        .into_iter()
        .map(|(rank, seed, assignments, report)| RankedPlan {
            rank,
            seed,
            assignments,
            report,
        })
        .collect();

    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    Ok(OptimizeResult {
        plans,
        initial_report,
        stats: OptimizeStats {
            total_runs: num_runs,
            total_iterations,
            elapsed_ms,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::validate::{validate_assignments, ValidateOptions};
    use rand_chacha::ChaCha8Rng;
    use rand_core::{RngCore, SeedableRng};

    pub(crate) fn make_seed_demo_problem() -> Problem {
        use crate::domain::*;
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };

        let campuses = vec![
            Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Co so 1".to_string(),
                color: "#1e40af".to_string(),
            },
            Campus {
                id: CampusId(2),
                code: "CS2".to_string(),
                name: "Co so 2".to_string(),
                color: "#047857".to_string(),
            },
            Campus {
                id: CampusId(3),
                code: "CS3".to_string(),
                name: "Co so 3".to_string(),
                color: "#b45309".to_string(),
            },
            Campus {
                id: CampusId(4),
                code: "CS4".to_string(),
                name: "Co so 4".to_string(),
                color: "#6d28d9".to_string(),
            },
        ];

        let grades = vec![
            Grade {
                id: GradeId(1),
                code: 10,
                name: "Khoi 10".to_string(),
                sort_order: 1,
            },
            Grade {
                id: GradeId(2),
                code: 11,
                name: "Khoi 11".to_string(),
                sort_order: 2,
            },
            Grade {
                id: GradeId(3),
                code: 12,
                name: "Khoi 12".to_string(),
                sort_order: 3,
            },
        ];

        let exams = vec![
            Exam {
                id: ExamId(1),
                school_year_id: sy.id,
                code: "GK1".to_string(),
                name: "Giua ky 1".to_string(),
                sort_order: 1,
            },
            Exam {
                id: ExamId(2),
                school_year_id: sy.id,
                code: "CK1".to_string(),
                name: "Cuoi ky 1".to_string(),
                sort_order: 2,
            },
            Exam {
                id: ExamId(3),
                school_year_id: sy.id,
                code: "GK2".to_string(),
                name: "Giua ky 2".to_string(),
                sort_order: 3,
            },
            Exam {
                id: ExamId(4),
                school_year_id: sy.id,
                code: "CK2".to_string(),
                name: "Cuoi ky 2".to_string(),
                sort_order: 4,
            },
        ];

        let raw_teachers = [
            (1, "Nguyen Van An", 1, 1.0),
            (2, "Tran Thi Binh", 1, 1.0),
            (3, "Le Hoang Cuong", 2, 1.0),
            (4, "Pham Minh Duc", 2, 1.0),
            (5, "Hoang Thu Giang", 3, 1.0),
            (6, "Vu Hai Ha", 3, 0.5),
            (7, "Dang Quoc Hung", 3, 1.0),
            (8, "Bui Thi Lan", 4, 1.0),
            (9, "Do Tuan Minh", 4, 1.0),
            (10, "Ngo Phuong Nam", 1, 1.0),
            (11, "Duong Thuy Trang", 2, 1.0),
        ];

        let teachers: Vec<Teacher> = raw_teachers
            .iter()
            .map(|&(id, name, campus_id, weight)| Teacher {
                id: TeacherId(id),
                full_name: name.to_string(),
                display_name: None,
                campus_id: CampusId(campus_id),
                load_weight: weight,
                active: true,
                quota_override: None,
                max_tasks_per_exam_override: None,
                note: None,
                code: None,
            })
            .collect();

        let raw_teacher_grades = [
            (1, vec![10, 11]),
            (2, vec![11, 12]),
            (3, vec![10, 12]),
            (4, vec![11]),
            (5, vec![10]),
            (6, vec![11]),
            (7, vec![12]),
            (8, vec![10]),
            (9, vec![11]),
            (10, vec![12]),
            (11, vec![12]),
        ];

        let mut teacher_grades = Vec::new();
        for &(tid, ref g_codes) in &raw_teacher_grades {
            for &g_code in g_codes {
                let gid = match g_code {
                    10 => GradeId(1),
                    11 => GradeId(2),
                    12 => GradeId(3),
                    _ => unreachable!(),
                };
                teacher_grades.push(TeacherGrade {
                    teacher_id: TeacherId(tid),
                    school_year_id: sy.id,
                    grade_id: gid,
                });
            }
        }

        let sub = Subject {
            id: SubjectId(1),
            code: "CHUNG".to_string(),
            name: "Chung".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        };
        let mut competencies = Vec::new();
        for t in &teachers {
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        let unavailabilities = vec![Unavailability {
            teacher_id: TeacherId(6),
            exam_id: ExamId(3),
            reason: Some("Medical".to_string()),
        }];

        Problem {
            school_year: sy,
            campuses,
            grades,
            subjects: vec![sub],
            exams,
            teachers,
            teacher_grades,
            competencies,
            unavailabilities,
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    fn make_synthetic_40_problem() -> Problem {
        use crate::domain::*;
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };

        let campuses: Vec<Campus> = (1..=6)
            .map(|c| Campus {
                id: CampusId(c),
                code: format!("CS{c}"),
                name: format!("Co so {c}"),
                color: "#1e40af".to_string(),
            })
            .collect();

        let grades: Vec<Grade> = (1..=3)
            .map(|g| Grade {
                id: GradeId(g),
                code: (9 + g) as i32,
                name: format!("Khoi {}", 9 + g),
                sort_order: g as i32,
            })
            .collect();

        let exams: Vec<Exam> = (1..=4)
            .map(|e| Exam {
                id: ExamId(e),
                school_year_id: sy.id,
                code: format!("EX{e}"),
                name: format!("Ky thi {e}"),
                sort_order: e as i32,
            })
            .collect();

        let mut teachers = Vec::with_capacity(40);
        let mut teacher_grades = Vec::new();

        for tid in 1..=40 {
            let cid = ((tid - 1) % 6) + 1;
            teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Giao vien {tid}"),
                display_name: None,
                campus_id: CampusId(cid),
                load_weight: 1.0,
                active: true,
                quota_override: None,
                max_tasks_per_exam_override: None,
                note: None,
                code: None,
            });

            let g1 = ((tid - 1) % 3) + 1;
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: sy.id,
                grade_id: GradeId(g1),
            });
            if tid % 2 == 0 {
                let g2 = (g1 % 3) + 1;
                teacher_grades.push(TeacherGrade {
                    teacher_id: TeacherId(tid),
                    school_year_id: sy.id,
                    grade_id: GradeId(g2),
                });
            }
        }

        let sub = Subject {
            id: SubjectId(1),
            code: "CHUNG".to_string(),
            name: "Chung".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        };
        let mut competencies = Vec::new();
        for t in &teachers {
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        Problem {
            school_year: sy,
            campuses,
            grades,
            subjects: vec![sub],
            exams,
            teachers,
            teacher_grades,
            competencies,
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    #[test]
    fn test_d1_incremental_state_consistency_demo_10k_moves() {
        let problem = make_seed_demo_problem();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard demo");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let mut accepted_moves = 0;
        while accepted_moves < 10_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let delta = state.try_apply_move(m);
                let accept = rng.next_u32() % 2 == 0;
                if accept {
                    accepted_moves += 1;
                    let assigns = state.to_assignments(crate::domain::PlanId(0));
                    let violations = validate_assignments(
                        &problem,
                        &assigns,
                        &ValidateOptions {
                            require_complete: true,
                        },
                    );
                    assert!(
                        violations.is_empty(),
                        "Hard constraint violated after accepted move {accepted_moves}: {violations:?}"
                    );
                } else {
                    state.revert_move(m, delta);
                }
            }
        }

        let full_eval = state.full_evaluate(&problem);
        let diff = (state.current_penalty - full_eval.total).abs();
        assert!(
            diff < 1e-9,
            "Demo 10k moves: incremental penalty {} != full eval total {} (diff: {})",
            state.current_penalty,
            full_eval.total,
            diff
        );
    }

    #[test]
    fn test_d1_incremental_state_consistency_synthetic_40_10k_moves() {
        let problem = make_synthetic_40_problem_2sub();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 777,
                time_limit_ms: 3000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard synthetic");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(67890);

        let mut accepted_moves = 0;
        while accepted_moves < 10_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let delta = state.try_apply_move(m);
                let accept = rng.next_u32() % 2 == 0;
                if accept {
                    accepted_moves += 1;
                    let assigns = state.to_assignments(crate::domain::PlanId(0));
                    let violations = validate_assignments(
                        &problem,
                        &assigns,
                        &ValidateOptions {
                            require_complete: true,
                        },
                    );
                    assert!(
                        violations.is_empty(),
                        "Hard constraint violated after accepted move {accepted_moves}: {violations:?}"
                    );
                } else {
                    state.revert_move(m, delta);
                }
            }
        }

        let full_eval = state.full_evaluate(&problem);
        let diff = (state.current_penalty - full_eval.total).abs();
        assert!(
            diff < 1e-9,
            "Synthetic 40 2-subject 10k moves: incremental penalty {} != full eval total {} (diff: {})",
            state.current_penalty,
            full_eval.total,
            diff
        );
    }

    #[test]
    fn test_d1_incremental_state_consistency_q_shaped_10k_moves() {
        let problem = make_canonical_q_problem(QVariant::NoCampus);
        let assignments = make_q_assignments();
        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(
            violations.is_empty(),
            "Q manual plan has violations on nocampus: {violations:?}"
        );

        let mut state = IncrementalState::new(&problem, &assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(54321);

        let mut accepted_moves = 0;
        while accepted_moves < 10_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let delta = state.try_apply_move(m);
                let accept = rng.next_u32() % 2 == 0;
                if accept {
                    accepted_moves += 1;
                    let assigns = state.to_assignments(crate::domain::PlanId(0));
                    let v = validate_assignments(
                        &problem,
                        &assigns,
                        &ValidateOptions {
                            require_complete: true,
                        },
                    );
                    assert!(
                        v.is_empty(),
                        "Hard constraint violated after accepted move {accepted_moves}: {v:?}"
                    );
                } else {
                    state.revert_move(m, delta);
                }
            }
        }

        let full_eval = state.full_evaluate(&problem);
        let diff = (state.current_penalty - full_eval.total).abs();
        assert!(
            diff < 1e-9,
            "Q-shaped 10k moves: incremental penalty {} != full eval total {} (diff: {})",
            state.current_penalty,
            full_eval.total,
            diff
        );
    }

    #[test]
    fn test_d2_random_walk_hard_invariants_preserved() {
        let problem = make_seed_demo_problem();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(999);

        for _ in 0..500 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let _ = state.try_apply_move(m);
                let assigns = state.to_assignments(crate::domain::PlanId(0));
                let violations = validate_assignments(
                    &problem,
                    &assigns,
                    &ValidateOptions {
                        require_complete: true,
                    },
                );
                assert!(
                    violations.is_empty(),
                    "Hard constraint violated during random walk: {violations:?}"
                );
            }
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(20))]

        #[test]
        fn test_proptest_d1_incremental_state_consistency(seed in 1u64..100_000u64, num_moves in 200usize..1000usize) {
            let problem = make_seed_demo_problem();
            let sol = solve_hard(
                &problem,
                &SolveOptions {
                    seed: 42,
                    time_limit_ms: 2000,
                    max_nodes: 500_000,
                },
            )
            .expect("solve hard demo");

            let mut state = IncrementalState::new(&problem, &sol.assignments);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            let mut accepted_moves = 0;
            while accepted_moves < num_moves {
                if let Some(m) = state.sample_candidate_move(&mut rng) {
                    let delta = state.try_apply_move(m);
                    let accept = rng.next_u32() % 2 == 0;
                    if accept {
                        accepted_moves += 1;
                    } else {
                        state.revert_move(m, delta);
                    }
                }
            }

            let full_eval = state.full_evaluate(&problem);
            let diff = (state.current_penalty - full_eval.total).abs();
            proptest::prop_assert!(
                diff < 1e-9,
                "Proptest incremental penalty {} != full eval total {} (diff: {})",
                state.current_penalty,
                full_eval.total,
                diff
            );
        }

        #[test]
        fn test_proptest_d2_random_walk_hard_invariants(seed in 1u64..100_000u64, num_moves in 100usize..500usize) {
            let problem = make_seed_demo_problem();
            let sol = solve_hard(
                &problem,
                &SolveOptions {
                    seed: 42,
                    time_limit_ms: 2000,
                    max_nodes: 500_000,
                },
            )
            .expect("solve hard");

            let mut state = IncrementalState::new(&problem, &sol.assignments);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            for _ in 0..num_moves {
                if let Some(m) = state.sample_candidate_move(&mut rng) {
                    let _ = state.try_apply_move(m);
                    let assigns = state.to_assignments(crate::domain::PlanId(0));
                    let violations = validate_assignments(
                        &problem,
                        &assigns,
                        &ValidateOptions {
                            require_complete: true,
                        },
                    );
                    proptest::prop_assert!(
                        violations.is_empty(),
                        "Proptest hard constraint violated during random walk: {:?}",
                        violations
                    );
                }
            }
        }
    }

    #[test]
    fn test_improvement_over_solve_hard_demo_seed() {
        let problem = make_seed_demo_problem();
        let hard_sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard");

        let hard_report = evaluate(&problem, &hard_sol.assignments);

        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(25_000),
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: Some(hard_sol.assignments),
        };

        let res = optimize(&problem, &opts).expect("optimize demo");
        assert!(!res.plans.is_empty());
        let best_plan = &res.plans[0];

        println!(
            "Hard solve seed 42 penalty: {:.2} -> Optimized best plan penalty: {:.2}",
            hard_report.total, best_plan.report.total
        );
        assert!(
            best_plan.report.total < hard_report.total,
            "Expected optimized score {} < hard solve score {}",
            best_plan.report.total,
            hard_report.total
        );
    }

    #[test]
    fn test_lower_bound_awareness_demo_seed() {
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(50_000),
            num_runs: 8,
            max_plans: 3,
            // Explicit 0.15 threshold for test sensitivity; default is 0.20
            diversity_threshold: 0.15,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let res = optimize(&problem, &opts).expect("optimize demo lower bounds");
        assert!(!res.plans.is_empty());
        let best_plan = &res.plans[0];

        // Obvious bound 1: reviewer_slots (12) <= 2 * 11 eligible reviewers,
        // so reviewer_too_many (reviews > 2) should be 0.
        let reviewer_too_many = best_plan
            .report
            .violations
            .iter()
            .filter(|v| v.code == "reviewer_too_many")
            .count();
        assert_eq!(
            reviewer_too_many, 0,
            "Expected 0 reviewer_too_many violations"
        );

        // Obvious bound 2: each grade has >= 4 qualified teachers (so >= 6 distinct pairs).
        // 4 exams per grade means S4 (repeated setter pair) can be 0.
        let s4_score = best_plan
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == crate::domain::RuleKey::S4)
            .expect("S4 score");
        assert_eq!(
            s4_score.units, 0.0,
            "Expected S4 units == 0 (no repeated setter pairs), got {}",
            s4_score.units
        );
    }

    #[test]
    fn test_determinism_across_runs() {
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 12345,
            budget: Budget::Iterations(5_000),
            num_runs: 4,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let res1 = optimize(&problem, &opts).expect("opt 1");
        let res2 = optimize(&problem, &opts).expect("opt 2");

        assert_eq!(res1.plans.len(), res2.plans.len());
        for i in 0..res1.plans.len() {
            assert_eq!(res1.plans[i].seed, res2.plans[i].seed);
            assert_eq!(res1.plans[i].report.total, res2.plans[i].report.total);
            assert_eq!(res1.plans[i].assignments, res2.plans[i].assignments);
        }
    }

    #[test]
    fn test_diversity_threshold_respected() {
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(10_000),
            num_runs: 8,
            max_plans: 3,
            // Explicit 0.15 threshold for test verification; default is 0.20
            diversity_threshold: 0.15,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let res = optimize(&problem, &opts).expect("opt diversity");
        for i in 0..res.plans.len() {
            for j in (i + 1)..res.plans.len() {
                let d = plan_distance(&res.plans[i].assignments, &res.plans[j].assignments);
                assert!(
                    d >= 0.15,
                    "Plans {i} and {j} have distance {d} < threshold 0.15"
                );
            }
        }
    }

    #[test]
    fn test_cancellation_stops_promptly() {
        let problem = make_seed_demo_problem();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel.clone();

        // Spawn a thread to cancel after 20ms
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            cancel_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(5_000_000), // very high budget
            num_runs: 1,
            max_plans: 1,
            diversity_threshold: 0.20,
            cancel: Some(cancel),
            progress: None,
            initial_assignments: None,
        };

        let start = Instant::now();
        let res = optimize(&problem, &opts).expect("cancel opt");
        let elapsed = start.elapsed();
        // Should terminate well before 5 million iterations
        assert!(elapsed.as_millis() < 500, "Elapsed was {:?}", elapsed);
        assert!(res.stats.total_iterations < 5_000_000);
    }

    #[test]
    fn test_pinned_slots_never_move() {
        let mut problem = make_seed_demo_problem();
        use crate::domain::*;
        // PIN teacher 1 as Setter on Exam 1 Grade 10
        problem.locks.push(Lock {
            id: LockId(101),
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            subject_id: problem.effective_subjects()[0].id,
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });

        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(10_000),
            num_runs: 4,
            max_plans: 3,
            // Explicit 0.15 threshold for test sensitivity; default is 0.20
            diversity_threshold: 0.15,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let res = optimize(&problem, &opts).expect("pinned opt");
        for plan in &res.plans {
            let found_pin = plan.assignments.iter().any(|a| {
                a.exam_id == ExamId(1)
                    && a.grade_id == GradeId(1)
                    && a.teacher_id == TeacherId(1)
                    && a.role == Role::Setter
            });
            assert!(found_pin, "Pinned slot moved in plan rank {}", plan.rank);
        }
    }

    #[test]
    fn test_performance_timing_demo_default_run() {
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(50_000), // 50k x 8 runs = 400k iterations
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let start = Instant::now();
        let res = optimize(&problem, &opts).expect("perf run");
        let elapsed = start.elapsed();

        println!(
            "Demo optimization (R=8, 50k iters): {:?} (total iters: {})",
            elapsed, res.stats.total_iterations
        );
        assert!(!res.plans.is_empty());
    }

    #[test]
    fn test_fairness_regression_demo_seed() {
        use crate::domain::{RuleKey, TeacherId};
        use crate::score::bounds::{lower_bounds, optimal_s8_counts};
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(200_000), // R=8, 200k each = 1.6M iterations
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let res = optimize(&problem, &opts).expect("optimize demo fairness");
        assert!(!res.plans.is_empty());
        let best_plan = &res.plans[0];

        // Bounds check
        let bounds = lower_bounds(&problem);
        let s8_bound = bounds
            .iter()
            .find(|b| b.rule == RuleKey::S8)
            .expect("S8 lower bound")
            .units_lower_bound;

        let s8_score = best_plan
            .report
            .by_rule
            .iter()
            .find(|r| r.rule == RuleKey::S8)
            .expect("S8 score");

        // Vũ Hải Hà (weight 0.5, unavailable for GK2) receives 1 task
        let vu_hai_ha_stats = best_plan
            .report
            .per_teacher
            .iter()
            .find(|t| t.teacher_id == TeacherId(6))
            .expect("Vũ Hải Hà stats");
        assert_eq!(
            vu_hai_ha_stats.count, 1,
            "Expected Vũ Hải Hà (ID 6) to receive exactly 1 task, got {}",
            vu_hai_ha_stats.count
        );

        // S8 units <= lower bound + 1e-6
        assert!(
            s8_score.units <= s8_bound + 1e-6,
            "Expected S8 units ({}) <= lower bound ({}) + 1e-6",
            s8_score.units,
            s8_bound
        );

        // No teacher's count_t exceeds their S8-optimal count by more than 1
        let opt_counts = optimal_s8_counts(&problem);
        for ts in &best_plan.report.per_teacher {
            let opt_c = opt_counts.get(&ts.teacher_id).copied().unwrap_or(0);
            let diff = (ts.count as isize - opt_c as isize).abs();
            assert!(
                diff <= 1,
                "Teacher {:?} count {} differs from optimal count {} by more than 1",
                ts.teacher_id,
                ts.count,
                opt_c
            );
        }
    }

    #[test]
    #[ignore]
    fn bench_demo_200k_iterations() {
        let problem = make_seed_demo_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(200_000), // R=8, 200k each = 1.6M iterations
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let start = Instant::now();
        let res = optimize(&problem, &opts).expect("perf run 200k");
        let elapsed = start.elapsed();

        println!(
            "BENCH Demo (R=8, 200k iters each, total {} iters): {:?}",
            res.stats.total_iterations, elapsed
        );
    }

    #[test]
    #[ignore]
    fn bench_synthetic_40_200k_iterations() {
        let problem = make_synthetic_40_problem();
        let opts = OptimizeOptions {
            base_seed: 42,
            budget: Budget::Iterations(200_000), // R=8, 200k each = 1.6M iterations
            num_runs: 8,
            max_plans: 3,
            diversity_threshold: 0.20,
            cancel: None,
            progress: None,
            initial_assignments: None,
        };

        let start = Instant::now();
        let res = optimize(&problem, &opts).expect("perf synthetic 200k");
        let elapsed = start.elapsed();

        println!(
            "BENCH Synthetic 40 (R=8, 200k iters each, total {} iters): {:?}",
            res.stats.total_iterations, elapsed
        );
    }
}
