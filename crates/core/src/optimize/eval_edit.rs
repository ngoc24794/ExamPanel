//! Manual editing evaluation routines for candidate suggestions and slot swaps.

use super::moves::LocalMove;
use super::state::{IncrementalState, SlotRole};
use crate::domain::{Assignment, ExamId, GradeId, Problem, Role, TeacherId};
use crate::score::evaluate;
use crate::validate::{validate_assignments, ValidateOptions, Violation};
use serde::{Deserialize, Serialize};

/// Reference to a specific panel slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct SlotRef {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub role: Role,
    pub position: usize,
}

/// Evaluation outcome for assigning a candidate teacher to a slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct CandidateEval {
    pub teacher_id: TeacherId,
    pub hard_violations: Vec<Violation>,
    pub delta_score: f64,
    pub new_total: f64,
}

/// Evaluates all teachers as candidates for filling a designated slot.
///
/// Uses `IncrementalState` for high-throughput O(1) soft score delta calculation.
/// Correctness guarantee: `delta_score` matches full `evaluate()` outcome.
#[must_use]
pub fn evaluate_candidates(
    problem: &Problem,
    assignments: &[Assignment],
    slot: SlotRef,
) -> Vec<CandidateEval> {
    let mut state = IncrementalState::new(problem, assignments);
    let base_penalty = state.current_penalty;

    // Find panel index
    let exam_map: std::collections::HashMap<_, _> = state
        .exam_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();
    let grade_map: std::collections::HashMap<_, _> = state
        .grade_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    let e_idx = match exam_map.get(&slot.exam_id) {
        Some(&idx) => idx,
        None => return Vec::new(),
    };
    let g_idx = match grade_map.get(&slot.grade_id) {
        Some(&idx) => idx,
        None => return Vec::new(),
    };
    let panel_idx = e_idx * state.grade_ids.len() + g_idx;

    let slot_role = match slot.role {
        Role::Reviewer => SlotRole::Reviewer,
        Role::Setter => {
            if slot.position == 0 {
                SlotRole::Setter1
            } else {
                SlotRole::Setter2
            }
        }
    };

    let old_t_idx = state.panels[panel_idx].get_slot(slot_role);

    let mut results = Vec::with_capacity(problem.teachers.len());

    for teacher in &problem.teachers {
        let new_t_idx = match state.teacher_map.get(&teacher.id) {
            Some(&idx) => idx,
            None => continue,
        };

        // Construct modified assignments to check hard violations
        let mut modified = assignments.to_vec();
        let mut replaced = false;
        let mut setter_count = 0;
        for a in &mut modified {
            if a.exam_id == slot.exam_id && a.grade_id == slot.grade_id && a.role == slot.role {
                if slot.role == Role::Reviewer {
                    a.teacher_id = teacher.id;
                    replaced = true;
                    break;
                } else {
                    if setter_count == slot.position {
                        a.teacher_id = teacher.id;
                        replaced = true;
                        break;
                    }
                    setter_count += 1;
                }
            }
        }

        if !replaced {
            modified.push(Assignment {
                plan_id: assignments
                    .first()
                    .map_or(crate::domain::PlanId(0), |a| a.plan_id),
                exam_id: slot.exam_id,
                grade_id: slot.grade_id,
                teacher_id: teacher.id,
                role: slot.role,
            });
        }

        // Validate hard violations specifically relevant to this placement
        let all_violations = validate_assignments(problem, &modified, &ValidateOptions::default());
        let hard_violations: Vec<Violation> = all_violations
            .into_iter()
            .filter(|v| {
                if v.teacher == Some(teacher.id) {
                    return true;
                }
                if let Some(ref p) = v.panel {
                    if p.exam_id == slot.exam_id && p.grade_id == slot.grade_id {
                        return true;
                    }
                }
                false
            })
            .collect();

        // Calculate delta score: use IncrementalState for valid moves, full evaluate for violating moves
        let (delta, new_total) = if new_t_idx == old_t_idx {
            (0.0, base_penalty)
        } else if hard_violations.is_empty() {
            let m = LocalMove::Replace {
                panel_idx,
                slot: slot_role,
                old_t: old_t_idx,
                new_t: new_t_idx,
            };
            let d = state.try_apply_move(m);
            let tot = state.current_penalty;
            state.revert_move(m, d);
            (d, tot)
        } else {
            let full_report = evaluate(problem, &modified);
            (full_report.total - base_penalty, full_report.total)
        };

        results.push(CandidateEval {
            teacher_id: teacher.id,
            hard_violations,
            delta_score: delta,
            new_total,
        });
    }

    results
}

/// Evaluates swapping the teachers assigned to two slots.
#[must_use]
pub fn evaluate_swap(
    problem: &Problem,
    assignments: &[Assignment],
    slot_a: SlotRef,
    slot_b: SlotRef,
) -> CandidateEval {
    let mut state = IncrementalState::new(problem, assignments);
    let base_penalty = state.current_penalty;

    let exam_map: std::collections::HashMap<_, _> = state
        .exam_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();
    let grade_map: std::collections::HashMap<_, _> = state
        .grade_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    let e1 = exam_map.get(&slot_a.exam_id).copied().unwrap_or(0);
    let g1 = grade_map.get(&slot_a.grade_id).copied().unwrap_or(0);
    let p1 = e1 * state.grade_ids.len() + g1;

    let e2 = exam_map.get(&slot_b.exam_id).copied().unwrap_or(0);
    let g2 = grade_map.get(&slot_b.grade_id).copied().unwrap_or(0);
    let p2 = e2 * state.grade_ids.len() + g2;

    let role1 = match slot_a.role {
        Role::Reviewer => SlotRole::Reviewer,
        Role::Setter => {
            if slot_a.position == 0 {
                SlotRole::Setter1
            } else {
                SlotRole::Setter2
            }
        }
    };
    let role2 = match slot_b.role {
        Role::Reviewer => SlotRole::Reviewer,
        Role::Setter => {
            if slot_b.position == 0 {
                SlotRole::Setter1
            } else {
                SlotRole::Setter2
            }
        }
    };

    let t1 = state.panels[p1].get_slot(role1);
    let t2 = state.panels[p2].get_slot(role2);

    if t1 == t2 {
        return CandidateEval {
            teacher_id: state.teacher_ids[t1],
            hard_violations: Vec::new(),
            delta_score: 0.0,
            new_total: base_penalty,
        };
    }

    // Build modified assignments
    let mut modified = assignments.to_vec();
    let t1_id = state.teacher_ids[t1];
    let t2_id = state.teacher_ids[t2];

    let mut setter1_count = 0;
    let mut setter2_count = 0;
    for a in &mut modified {
        if a.exam_id == slot_a.exam_id && a.grade_id == slot_a.grade_id && a.role == slot_a.role {
            if slot_a.role == Role::Reviewer || setter1_count == slot_a.position {
                a.teacher_id = t2_id;
            }
            setter1_count += 1;
        } else if a.exam_id == slot_b.exam_id
            && a.grade_id == slot_b.grade_id
            && a.role == slot_b.role
        {
            if slot_b.role == Role::Reviewer || setter2_count == slot_b.position {
                a.teacher_id = t1_id;
            }
            setter2_count += 1;
        }
    }

    let all_violations = validate_assignments(problem, &modified, &ValidateOptions::default());
    let hard_violations: Vec<Violation> = all_violations
        .into_iter()
        .filter(|v| {
            if v.teacher == Some(t1_id) || v.teacher == Some(t2_id) {
                return true;
            }
            if let Some(ref p) = v.panel {
                if (p.exam_id == slot_a.exam_id && p.grade_id == slot_a.grade_id)
                    || (p.exam_id == slot_b.exam_id && p.grade_id == slot_b.grade_id)
                {
                    return true;
                }
            }
            false
        })
        .collect();

    // Determine move type for fast delta
    let local_move = if p1 == p2 {
        if role1 == SlotRole::Reviewer {
            LocalMove::RoleSwap {
                panel_idx: p1,
                setter_slot: role2,
                setter_t: t2,
                reviewer_t: t1,
            }
        } else {
            LocalMove::RoleSwap {
                panel_idx: p1,
                setter_slot: role1,
                setter_t: t1,
                reviewer_t: t2,
            }
        }
    } else if e1 == e2 {
        LocalMove::IntraExamSwap {
            panel1_idx: p1,
            slot1: role1,
            t1,
            panel2_idx: p2,
            slot2: role2,
            t2,
        }
    } else {
        LocalMove::CrossExamSwap {
            panel1_idx: p1,
            slot1: role1,
            t1,
            panel2_idx: p2,
            slot2: role2,
            t2,
        }
    };

    let (delta, new_total) = if hard_violations.is_empty() {
        let d = state.try_apply_move(local_move);
        let tot = state.current_penalty;
        state.revert_move(local_move, d);
        (d, tot)
    } else {
        let full_report = evaluate(problem, &modified);
        (full_report.total - base_penalty, full_report.total)
    };

    CandidateEval {
        teacher_id: t2_id,
        hard_violations,
        delta_score: delta,
        new_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::tests::make_seed_demo_problem;
    use crate::solver::{solve_hard, SolveOptions};

    #[test]
    fn test_evaluate_candidates_delta_matches_full_eval() {
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

        let base_report = evaluate(&problem, &sol.assignments);

        // Test slot 1: Setter 0 on Exam 1, Grade 1
        let slot = SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            role: Role::Setter,
            position: 0,
        };

        for a in &sol.assignments {
            if a.exam_id == slot.exam_id && a.grade_id == slot.grade_id {
                println!(
                    "Slot panel assignment: {:?} role={:?}",
                    a.teacher_id, a.role
                );
            }
        }

        let candidates = evaluate_candidates(&problem, &sol.assignments, slot);
        assert_eq!(candidates.len(), problem.teachers.len());

        for c in &candidates {
            let mut modified = sol.assignments.clone();
            let mut setter_count = 0;
            for a in &mut modified {
                if a.exam_id == slot.exam_id && a.grade_id == slot.grade_id && a.role == slot.role {
                    if setter_count == slot.position {
                        a.teacher_id = c.teacher_id;
                        break;
                    }
                    setter_count += 1;
                }
            }

            let full_report = evaluate(&problem, &modified);
            let expected_delta = full_report.total - base_report.total;

            println!(
                "Candidate {:?}: c.delta={:.4}, expected_delta={:.4}",
                c.teacher_id, c.delta_score, expected_delta
            );
            assert!(
                (c.delta_score - expected_delta).abs() < 1e-6,
                "Teacher {:?} candidate delta {:.4} != expected full eval delta {:.4}",
                c.teacher_id,
                c.delta_score,
                expected_delta
            );
            assert!(
                (c.new_total - full_report.total).abs() < 1e-6,
                "Teacher {:?} new_total {:.4} != expected full eval total {:.4}",
                c.teacher_id,
                c.new_total,
                full_report.total
            );
        }

        // Test slot 2: Reviewer on Exam 1, Grade 2
        let rev_slot = SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(2),
            role: Role::Reviewer,
            position: 0,
        };
        let rev_candidates = evaluate_candidates(&problem, &sol.assignments, rev_slot);
        for c in &rev_candidates {
            let mut modified = sol.assignments.clone();
            for a in &mut modified {
                if a.exam_id == rev_slot.exam_id
                    && a.grade_id == rev_slot.grade_id
                    && a.role == rev_slot.role
                {
                    a.teacher_id = c.teacher_id;
                    break;
                }
            }
            let full_report = evaluate(&problem, &modified);
            let expected_delta = full_report.total - base_report.total;
            assert!((c.delta_score - expected_delta).abs() < 1e-6);
        }
    }

    #[test]
    fn test_evaluate_swap_delta_matches_full_eval() {
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

        let base_report = evaluate(&problem, &sol.assignments);

        // Case 1: Intra-exam swap (two panels in Exam 1)
        let slot_a = SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            role: Role::Setter,
            position: 0,
        };
        let slot_b = SlotRef {
            exam_id: ExamId(1),
            grade_id: GradeId(2),
            role: Role::Setter,
            position: 1,
        };

        let swap_eval = evaluate_swap(&problem, &sol.assignments, slot_a, slot_b);

        let mut modified = sol.assignments.clone();
        let mut t_a_opt = None;
        let mut t_b_opt = None;
        let mut s_a_count = 0;
        let mut s_b_count = 0;
        for a in &sol.assignments {
            if a.exam_id == slot_a.exam_id && a.grade_id == slot_a.grade_id && a.role == slot_a.role
            {
                if slot_a.role == Role::Reviewer || s_a_count == slot_a.position {
                    t_a_opt = Some(a.teacher_id);
                }
                s_a_count += 1;
            }
            if a.exam_id == slot_b.exam_id && a.grade_id == slot_b.grade_id && a.role == slot_b.role
            {
                if slot_b.role == Role::Reviewer || s_b_count == slot_b.position {
                    t_b_opt = Some(a.teacher_id);
                }
                s_b_count += 1;
            }
        }
        let t_a = t_a_opt.unwrap();
        let t_b = t_b_opt.unwrap();

        for a in &mut modified {
            if a.exam_id == slot_a.exam_id
                && a.grade_id == slot_a.grade_id
                && a.role == slot_a.role
                && a.teacher_id == t_a
            {
                a.teacher_id = t_b;
                break;
            }
        }
        for a in &mut modified {
            if a.exam_id == slot_b.exam_id
                && a.grade_id == slot_b.grade_id
                && a.role == slot_b.role
                && a.teacher_id == t_b
            {
                a.teacher_id = t_a;
                break;
            }
        }

        let full_report = evaluate(&problem, &modified);
        let expected_delta = full_report.total - base_report.total;
        assert!(
            (swap_eval.delta_score - expected_delta).abs() < 1e-6,
            "Intra-exam swap delta {:.4} != expected {:.4}",
            swap_eval.delta_score,
            expected_delta
        );

        // Case 2: Cross-exam swap (Exam 1 vs Exam 2)
        let slot_c = SlotRef {
            exam_id: ExamId(2),
            grade_id: GradeId(1),
            role: Role::Reviewer,
            position: 0,
        };
        let cross_swap = evaluate_swap(&problem, &sol.assignments, slot_a, slot_c);
        let mut modified2 = sol.assignments.clone();
        let t_c = modified2
            .iter()
            .find(|a| {
                a.exam_id == slot_c.exam_id
                    && a.grade_id == slot_c.grade_id
                    && a.role == slot_c.role
            })
            .unwrap()
            .teacher_id;
        for a in &mut modified2 {
            if a.exam_id == slot_a.exam_id
                && a.grade_id == slot_a.grade_id
                && a.role == slot_a.role
                && a.teacher_id == t_a
            {
                a.teacher_id = t_c;
                break;
            }
        }
        for a in &mut modified2 {
            if a.exam_id == slot_c.exam_id
                && a.grade_id == slot_c.grade_id
                && a.role == slot_c.role
                && a.teacher_id == t_c
            {
                a.teacher_id = t_a;
                break;
            }
        }
        let full_report2 = evaluate(&problem, &modified2);
        let expected_delta2 = full_report2.total - base_report.total;
        assert!(
            (cross_swap.delta_score - expected_delta2).abs() < 1e-6,
            "Cross-exam swap delta {:.4} != expected {:.4}",
            cross_swap.delta_score,
            expected_delta2
        );
    }

    #[test]
    fn test_reoptimize_keeps_slots_fixed_and_valid() {
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

        // Keep 4 slots fixed
        let kept_slots = vec![
            SlotRef {
                exam_id: ExamId(1),
                grade_id: GradeId(1),
                role: Role::Setter,
                position: 0,
            },
            SlotRef {
                exam_id: ExamId(1),
                grade_id: GradeId(1),
                role: Role::Reviewer,
                position: 0,
            },
            SlotRef {
                exam_id: ExamId(2),
                grade_id: GradeId(2),
                role: Role::Setter,
                position: 1,
            },
            SlotRef {
                exam_id: ExamId(3),
                grade_id: GradeId(3),
                role: Role::Reviewer,
                position: 0,
            },
        ];

        let mut problem_with_pins = problem.clone();
        for slot in &kept_slots {
            let mut s_count = 0;
            for a in &sol.assignments {
                if a.exam_id == slot.exam_id && a.grade_id == slot.grade_id && a.role == slot.role {
                    if slot.role == Role::Reviewer || s_count == slot.position {
                        problem_with_pins.locks.push(crate::domain::Lock {
                            id: crate::domain::LockId(0),
                            exam_id: slot.exam_id,
                            grade_id: slot.grade_id,
                            teacher_id: a.teacher_id,
                            role: Some(slot.role),
                            kind: crate::domain::LockKind::Pin,
                        });
                        break;
                    }
                    s_count += 1;
                }
            }
        }

        let opt_opts = crate::optimize::OptimizeOptions {
            base_seed: 1234,
            budget: crate::optimize::Budget::Iterations(2_000),
            num_runs: 2,
            max_plans: 2,
            diversity_threshold: 0.1,
            cancel: None,
            progress: None,
            initial_assignments: Some(sol.assignments.clone()),
        };

        let res = crate::optimize::optimize(&problem_with_pins, &opt_opts).expect("reoptimize");
        assert!(!res.plans.is_empty());

        for plan in &res.plans {
            let val = validate_assignments(
                &problem_with_pins,
                &plan.assignments,
                &ValidateOptions::default(),
            );
            assert!(
                val.is_empty(),
                "Reoptimized plan has hard violations: {:?}",
                val
            );

            for lock in &problem_with_pins.locks {
                if lock.kind == crate::domain::LockKind::Pin {
                    let found = plan.assignments.iter().any(|a| {
                        a.exam_id == lock.exam_id
                            && a.grade_id == lock.grade_id
                            && a.teacher_id == lock.teacher_id
                            && lock.role.map_or(true, |r| a.role == r)
                    });
                    assert!(
                        found,
                        "Pinned teacher {:?} on exam {:?} grade {:?} role {:?} was not preserved in reoptimized plan!",
                        lock.teacher_id, lock.exam_id, lock.grade_id, lock.role
                    );
                }
            }
        }
    }
}
