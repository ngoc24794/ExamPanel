//! Provable soft-constraint lower bound estimators.
//!
//! Provides mathematically sound, cheap lower bounds for soft constraint rules
//! to indicate when an objective cannot be improved further.

use crate::domain::{calculate_quotas, LockKind, Problem, Role, RuleKey, TeacherId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Lower bound evaluation for a single soft constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleBound {
    pub rule: RuleKey,
    pub units_lower_bound: f64,
    pub method: &'static str,
}

/// Computes provable, cheap lower bounds for all soft rules S1..S8.
#[must_use]
pub fn lower_bounds(problem: &Problem) -> Vec<RuleBound> {
    let quotas = calculate_quotas(problem);
    let num_exams = problem.exams.len();
    let num_grades = problem.grades.len();
    let num_panels = num_exams * num_grades;
    let total_slots = num_panels * 3;
    let total_setter_slots = num_panels * 2;
    let total_reviewer_slots = num_panels;

    let teacher_grades_map: HashMap<TeacherId, HashSet<crate::domain::GradeId>> = {
        let mut map: HashMap<TeacherId, HashSet<crate::domain::GradeId>> = HashMap::new();
        for tg in &problem.teacher_grades {
            if tg.school_year_id == problem.school_year.id {
                map.entry(tg.teacher_id).or_default().insert(tg.grade_id);
            }
        }
        map
    };

    let unavailabilities_set: HashSet<(TeacherId, crate::domain::ExamId)> = problem
        .unavailabilities
        .iter()
        .map(|u| (u.teacher_id, u.exam_id))
        .collect();

    // -------------------------------------------------------------------------
    // S1: Reviewer capacity pigeonhole
    // -------------------------------------------------------------------------
    let s1_bound = {
        let mut eligible_reviewers_count: usize = 0;
        for q in &quotas {
            if q.quota < 1.0 {
                continue;
            }
            let t = match problem.teachers.iter().find(|t| t.id == q.teacher_id) {
                Some(t) if t.active && t.load_weight > 0.0 => t,
                _ => continue,
            };
            let teaches_grades = teacher_grades_map.get(&t.id);
            let has_eligible_panel = problem.exams.iter().any(|e| {
                if unavailabilities_set.contains(&(t.id, e.id)) {
                    return false;
                }
                problem.grades.iter().any(|g| {
                    if let Some(tg) = teaches_grades {
                        if !tg.contains(&g.id) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    !problem.locks.iter().any(|lock| {
                        lock.exam_id == e.id
                            && lock.grade_id == g.id
                            && lock.teacher_id == t.id
                            && lock.kind == LockKind::Forbid
                            && (lock.role.is_none() || lock.role == Some(Role::Reviewer))
                    })
                })
            });
            if has_eligible_panel {
                eligible_reviewers_count += 1;
            }
        }

        let under_bound = eligible_reviewers_count.saturating_sub(total_reviewer_slots);
        let over_bound = total_reviewer_slots.saturating_sub(2 * eligible_reviewers_count);
        (under_bound + over_bound) as f64
    };

    // -------------------------------------------------------------------------
    // S2: Role balance (new definition [floor(c/3), ceil(c/3)])
    // -------------------------------------------------------------------------
    let s2_bound = 0.0;

    // -------------------------------------------------------------------------
    // S3: Independent reviewer
    // -------------------------------------------------------------------------
    let s3_bound = 0.0;

    // -------------------------------------------------------------------------
    // S4: Repeated setter pair
    // -------------------------------------------------------------------------
    let s4_bound = 0.0;

    // -------------------------------------------------------------------------
    // S5: Repeated review relation
    // -------------------------------------------------------------------------
    let s5_bound = 0.0;

    // -------------------------------------------------------------------------
    // S6: Pigeonhole on setter slots with consecutive setting
    // -------------------------------------------------------------------------
    // With E exams, a teacher can set at most ceil(E/2) times without adjacency.
    // For s_t setter assignments in E exams, the minimum number of adjacent pairs
    // is max(0, 2 * s_t - E - 1).
    let s6_bound = if num_exams <= 1 {
        0.0
    } else {
        let max_non_adjacent = num_exams.div_ceil(2);
        let adj_cost = |s: usize| -> usize {
            if s > max_non_adjacent && 2 * s > num_exams + 1 {
                2 * s - num_exams - 1
            } else {
                0
            }
        };

        // Compute maximum setter capacity per teacher
        let mut setter_caps = Vec::with_capacity(problem.teachers.len());
        for t in &problem.teachers {
            if !t.active || t.load_weight <= 0.0 {
                setter_caps.push(0);
                continue;
            }
            let hi = quotas
                .iter()
                .find(|q| q.teacher_id == t.id)
                .map_or(0, |q| q.hi);
            let teaches_grades = teacher_grades_map.get(&t.id);
            let eligible_setter_exams = problem
                .exams
                .iter()
                .filter(|e| {
                    if unavailabilities_set.contains(&(t.id, e.id)) {
                        return false;
                    }
                    problem.grades.iter().any(|g| {
                        if let Some(tg) = teaches_grades {
                            if !tg.contains(&g.id) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                        !problem.locks.iter().any(|lock| {
                            lock.exam_id == e.id
                                && lock.grade_id == g.id
                                && lock.teacher_id == t.id
                                && lock.kind == LockKind::Forbid
                                && (lock.role.is_none() || lock.role == Some(Role::Setter))
                        })
                    })
                })
                .count();
            let cap = hi.min(eligible_setter_exams);
            setter_caps.push(cap);
        }

        // Greedy allocation minimizing sum of adj_cost(s_t)
        let mut s_alloc = vec![0usize; setter_caps.len()];
        // Step 1: Allocate up to max_non_adjacent at zero marginal cost
        let mut allocated = 0usize;
        for i in 0..setter_caps.len() {
            let take = setter_caps[i].min(max_non_adjacent);
            s_alloc[i] = take;
            allocated += take;
        }

        if allocated >= total_setter_slots {
            0.0
        } else {
            // Step 2: Greedily allocate remaining slots based on minimal marginal cost
            while allocated < total_setter_slots {
                let mut best_idx = None;
                let mut best_marginal_cost = usize::MAX;

                for i in 0..setter_caps.len() {
                    if s_alloc[i] < setter_caps[i] {
                        let mc = adj_cost(s_alloc[i] + 1) - adj_cost(s_alloc[i]);
                        if mc < best_marginal_cost {
                            best_marginal_cost = mc;
                            best_idx = Some(i);
                        }
                    }
                }

                match best_idx {
                    Some(idx) => {
                        s_alloc[idx] += 1;
                        allocated += 1;
                    }
                    None => break, // Cannot allocate further
                }
            }

            s_alloc.iter().map(|&s| adj_cost(s)).sum::<usize>() as f64
        }
    };

    // -------------------------------------------------------------------------
    // S7: Grade rotation
    // -------------------------------------------------------------------------
    let s7_bound = 0.0;

    // -------------------------------------------------------------------------
    // S8: Load deviation lower bound via greedy marginal-cost allocation
    // -------------------------------------------------------------------------
    let (s8_bound, _) = optimal_s8_allocation(problem, total_slots);

    vec![
        RuleBound {
            rule: RuleKey::S1,
            units_lower_bound: s1_bound,
            method: "reviewer_capacity_pigeonhole",
        },
        RuleBound {
            rule: RuleKey::S2,
            units_lower_bound: s2_bound,
            method: "trivial",
        },
        RuleBound {
            rule: RuleKey::S3,
            units_lower_bound: s3_bound,
            method: "trivial",
        },
        RuleBound {
            rule: RuleKey::S4,
            units_lower_bound: s4_bound,
            method: "trivial",
        },
        RuleBound {
            rule: RuleKey::S5,
            units_lower_bound: s5_bound,
            method: "trivial",
        },
        RuleBound {
            rule: RuleKey::S6,
            units_lower_bound: s6_bound,
            method: "pigeonhole_setter_adjacency",
        },
        RuleBound {
            rule: RuleKey::S7,
            units_lower_bound: s7_bound,
            method: "trivial",
        },
        RuleBound {
            rule: RuleKey::S8,
            units_lower_bound: s8_bound,
            method: "greedy_marginal_cost_allocation",
        },
    ]
}

/// Computes the exact S8 lower bound and the optimal integer task count per teacher.
///
/// Minimizes Σ (c_t - q_t)^2 subject to Σ c_t = D and lo_t <= c_t <= hi_t.
/// Solved exactly via greedy marginal-cost allocation over separable strictly convex objectives.
#[must_use]
pub fn optimal_s8_counts(problem: &Problem) -> HashMap<TeacherId, usize> {
    let num_panels = problem.exams.len() * problem.grades.len();
    let total_slots = num_panels * 3;
    let (_, counts) = optimal_s8_allocation(problem, total_slots);
    counts
}

fn optimal_s8_allocation(
    problem: &Problem,
    total_slots: usize,
) -> (f64, HashMap<TeacherId, usize>) {
    let quotas = calculate_quotas(problem);
    if quotas.is_empty() {
        return (0.0, HashMap::new());
    }

    // Initialize counts with lo_t
    let mut counts: HashMap<TeacherId, usize> =
        quotas.iter().map(|q| (q.teacher_id, q.lo)).collect();
    let mut current_sum: usize = counts.values().sum();

    // Greedily increment teacher whose marginal cost (2*(c - q) + 1) is minimal
    while current_sum < total_slots {
        let mut best_teacher = None;
        let mut best_diff = f64::INFINITY;

        for q in &quotas {
            let c = counts[&q.teacher_id];
            if c < q.hi {
                // Marginal cost is proportional to c - q
                let diff = c as f64 - q.quota;
                if diff < best_diff {
                    best_diff = diff;
                    best_teacher = Some(q.teacher_id);
                }
            }
        }

        match best_teacher {
            Some(tid) => {
                *counts.get_mut(&tid).unwrap() += 1;
                current_sum += 1;
            }
            None => break, // Hit upper bounds
        }
    }

    let s8_units: f64 = quotas
        .iter()
        .map(|q| {
            let c = counts[&q.teacher_id] as f64;
            (c - q.quota).powi(2)
        })
        .sum();

    (s8_units, counts)
}
