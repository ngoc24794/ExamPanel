//! Provable soft-constraint lower bound estimators.
//!
//! Provides mathematically sound, cheap lower bounds for soft constraint rules
//! to indicate when an objective cannot be improved further.

use crate::domain::{calculate_quotas, LockKind, Problem, Role, RuleKey, TeacherId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Lower bound evaluation for a single soft constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct RuleBound {
    pub rule: RuleKey,
    pub units_lower_bound: f64,
    pub method: String,
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
            method: "reviewer_capacity_pigeonhole".to_string(),
        },
        RuleBound {
            rule: RuleKey::S2,
            units_lower_bound: s2_bound,
            method: "trivial".to_string(),
        },
        RuleBound {
            rule: RuleKey::S3,
            units_lower_bound: s3_bound,
            method: "trivial".to_string(),
        },
        RuleBound {
            rule: RuleKey::S4,
            units_lower_bound: s4_bound,
            method: "trivial".to_string(),
        },
        RuleBound {
            rule: RuleKey::S5,
            units_lower_bound: s5_bound,
            method: "trivial".to_string(),
        },
        RuleBound {
            rule: RuleKey::S6,
            units_lower_bound: s6_bound,
            method: "pigeonhole_setter_adjacency".to_string(),
        },
        RuleBound {
            rule: RuleKey::S7,
            units_lower_bound: s7_bound,
            method: "trivial".to_string(),
        },
        RuleBound {
            rule: RuleKey::S8,
            units_lower_bound: s8_bound,
            method: "greedy_marginal_cost_allocation".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_lower_bounds_validity_exhaustive_tiny_instances() {
        let mut rng = ChaCha8Rng::seed_from_u64(20261002);
        let mut valid_tested_instances = 0;
        let mut attempt = 0;

        while valid_tested_instances < 200 && attempt < 2000 {
            attempt += 1;
            let num_exams = rng.gen_range(2..=3);
            let num_grades = 1; // 1 grade keeps panels small (2..3 panels, 6..9 slots) for fast exhaustive search
            let num_teachers = rng.gen_range(4..=6);

            let sy = SchoolYear {
                id: SchoolYearId(1),
                name: "2026-2027".to_string(),
                is_current: true,
            };

            let campuses = vec![
                Campus {
                    id: CampusId(1),
                    code: "C1".to_string(),
                    name: "Campus 1".to_string(),
                    color: "blue".to_string(),
                },
                Campus {
                    id: CampusId(2),
                    code: "C2".to_string(),
                    name: "Campus 2".to_string(),
                    color: "emerald".to_string(),
                },
            ];

            let grades: Vec<Grade> = (1..=num_grades)
                .map(|g| Grade {
                    id: GradeId(g as i64),
                    code: 10 + g as i32,
                    name: format!("Grade {g}"),
                    sort_order: g as i32,
                })
                .collect();

            let exams: Vec<Exam> = (1..=num_exams)
                .map(|e| Exam {
                    id: ExamId(e as i64),
                    school_year_id: sy.id,
                    code: format!("EX{e}"),
                    name: format!("Exam {e}"),
                    sort_order: e as i32,
                })
                .collect();

            let mut teachers = Vec::with_capacity(num_teachers);
            let mut teacher_grades = Vec::new();

            for tid in 1..=num_teachers {
                let camp = if tid <= num_teachers / 2 {
                    CampusId(1)
                } else {
                    CampusId(2)
                };
                let lw = if rng.gen_bool(0.2) { 0.5 } else { 1.0 };
                teachers.push(Teacher {
                    id: TeacherId(tid as i64),
                    full_name: format!("Teacher {tid}"),
                    campus_id: camp,
                    load_weight: lw,
                    active: true,
                    note: None,
                    code: None,
                });

                // Qualified for all grades in this tiny instance
                for g in &grades {
                    teacher_grades.push(TeacherGrade {
                        teacher_id: TeacherId(tid as i64),
                        school_year_id: sy.id,
                        grade_id: g.id,
                    });
                }
            }

            // Occasional unavailability
            let mut unavailabilities = Vec::new();
            if rng.gen_bool(0.3) {
                let t_idx = rng.gen_range(1..=num_teachers);
                let e_idx = rng.gen_range(1..=num_exams);
                unavailabilities.push(Unavailability {
                    teacher_id: TeacherId(t_idx as i64),
                    exam_id: ExamId(e_idx as i64),
                    reason: Some("Absent".to_string()),
                });
            }

            let problem = Problem {
                school_year: sy,
                campuses,
                grades,
                exams,
                teachers,
                teacher_grades,
                unavailabilities,
                locks: vec![],
                rule_settings: RuleSetting::default_settings(),
            };

            // Exhaustive search over all feasible assignments
            let panels: Vec<PanelKey> = problem
                .exams
                .iter()
                .flat_map(|e| {
                    problem
                        .grades
                        .iter()
                        .map(move |g| PanelKey::new(e.id, g.id))
                })
                .collect();

            let quotas = calculate_quotas(&problem);
            let unavail_set: HashSet<(TeacherId, ExamId)> = problem
                .unavailabilities
                .iter()
                .map(|u| (u.teacher_id, u.exam_id))
                .collect();

            // Precompute valid triples per panel
            let mut panel_triples: Vec<Vec<(usize, usize, usize)>> = Vec::new();
            for p in &panels {
                let eligible: Vec<usize> = problem
                    .teachers
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.active && t.load_weight > 0.0)
                    .filter(|(_, t)| !unavail_set.contains(&(t.id, p.exam_id)))
                    .map(|(i, _)| i)
                    .collect();

                let mut triples = Vec::new();
                let n = eligible.len();
                for i in 0..n {
                    for j in (i + 1)..n {
                        for k in 0..n {
                            if k == i || k == j {
                                continue;
                            }
                            let u1 = eligible[i];
                            let u2 = eligible[j];
                            let ur = eligible[k];

                            // H3: at least 2 campuses
                            let mut c_set = HashSet::new();
                            c_set.insert(problem.teachers[u1].campus_id);
                            c_set.insert(problem.teachers[u2].campus_id);
                            c_set.insert(problem.teachers[ur].campus_id);
                            if c_set.len() >= 2 {
                                triples.push((u1, u2, ur));
                            }
                        }
                    }
                }
                panel_triples.push(triples);
            }

            let mut all_solutions: Vec<Vec<Assignment>> = Vec::new();
            let mut current = Vec::new();
            let mut used_counts = vec![0usize; problem.teachers.len()];
            let hi_limits: Vec<usize> = (0..problem.teachers.len())
                .map(|i| {
                    let tid = problem.teachers[i].id;
                    quotas
                        .iter()
                        .find(|q| q.teacher_id == tid)
                        .map_or(0, |q| q.hi)
                })
                .collect();
            let lo_limits: Vec<usize> = (0..problem.teachers.len())
                .map(|i| {
                    let tid = problem.teachers[i].id;
                    quotas
                        .iter()
                        .find(|q| q.teacher_id == tid)
                        .map_or(0, |q| q.lo)
                })
                .collect();

            fn search_fast(
                p_idx: usize,
                panels: &[PanelKey],
                panel_triples: &[Vec<(usize, usize, usize)>],
                problem: &Problem,
                hi_limits: &[usize],
                lo_limits: &[usize],
                used_counts: &mut [usize],
                current: &mut Vec<Assignment>,
                solutions: &mut Vec<Vec<Assignment>>,
            ) {
                if p_idx == panels.len() {
                    for i in 0..used_counts.len() {
                        if used_counts[i] < lo_limits[i] {
                            return;
                        }
                    }
                    solutions.push(current.clone());
                    return;
                }

                let panel = panels[p_idx];
                for &(u1, u2, ur) in &panel_triples[p_idx] {
                    if used_counts[u1] >= hi_limits[u1]
                        || used_counts[u2] >= hi_limits[u2]
                        || used_counts[ur] >= hi_limits[ur]
                    {
                        continue;
                    }

                    // H4 check across same exam
                    let t1_id = problem.teachers[u1].id;
                    let t2_id = problem.teachers[u2].id;
                    let tr_id = problem.teachers[ur].id;

                    let mut conflict = false;
                    for a in current.iter() {
                        if a.exam_id == panel.exam_id
                            && (a.teacher_id == t1_id
                                || a.teacher_id == t2_id
                                || a.teacher_id == tr_id)
                        {
                            conflict = true;
                            break;
                        }
                    }
                    if conflict {
                        continue;
                    }

                    used_counts[u1] += 1;
                    used_counts[u2] += 1;
                    used_counts[ur] += 1;

                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        t1_id,
                        Role::Setter,
                    ));
                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        t2_id,
                        Role::Setter,
                    ));
                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        tr_id,
                        Role::Reviewer,
                    ));

                    search_fast(
                        p_idx + 1,
                        panels,
                        panel_triples,
                        problem,
                        hi_limits,
                        lo_limits,
                        used_counts,
                        current,
                        solutions,
                    );

                    current.pop();
                    current.pop();
                    current.pop();

                    used_counts[u1] -= 1;
                    used_counts[u2] -= 1;
                    used_counts[ur] -= 1;
                }
            }

            search_fast(
                0,
                &panels,
                &panel_triples,
                &problem,
                &hi_limits,
                &lo_limits,
                &mut used_counts,
                &mut current,
                &mut all_solutions,
            );

            if all_solutions.is_empty() {
                continue; // Skip instances with no feasible solution
            }

            // Compute exact minimum S1, S6, and S8 across all valid solutions
            let mut min_s1 = f64::INFINITY;
            let mut min_s6 = usize::MAX;
            let mut min_s8 = f64::INFINITY;

            let exam_order: HashMap<ExamId, usize> = problem
                .exams
                .iter()
                .enumerate()
                .map(|(idx, e)| (e.id, idx))
                .collect();

            for sol in &all_solutions {
                // S6: Consecutive setter exams
                let mut teacher_setter_exams: HashMap<TeacherId, Vec<usize>> = HashMap::new();
                for a in sol {
                    if a.role == Role::Setter {
                        let e_idx = exam_order[&a.exam_id];
                        teacher_setter_exams
                            .entry(a.teacher_id)
                            .or_default()
                            .push(e_idx);
                    }
                }

                let mut s6_units = 0;
                for (_, mut exams) in teacher_setter_exams {
                    exams.sort_unstable();
                    for w in exams.windows(2) {
                        if w[1] == w[0] + 1 {
                            s6_units += 1;
                        }
                    }
                }
                min_s6 = min_s6.min(s6_units);

                // S1: Reviewer count for eligible teachers with quota >= 1
                let mut reviewer_counts = HashMap::new();
                for a in sol {
                    if a.role == Role::Reviewer {
                        *reviewer_counts.entry(a.teacher_id).or_insert(0usize) += 1;
                    }
                }
                let mut s1_units = 0.0;
                for q in &quotas {
                    if q.quota < 1.0 {
                        continue;
                    }
                    let t = match problem.teachers.iter().find(|t| t.id == q.teacher_id) {
                        Some(t) if t.active && t.load_weight > 0.0 => t,
                        _ => continue,
                    };
                    let revs = reviewer_counts.get(&t.id).copied().unwrap_or(0);
                    if revs == 0 {
                        s1_units += 1.0;
                    } else if revs > 2 {
                        s1_units += (revs - 2) as f64;
                    }
                }
                min_s1 = min_s1.min(s1_units);

                // S8: sum (count_t - q_t)^2
                let mut counts = HashMap::new();
                for a in sol {
                    *counts.entry(a.teacher_id).or_insert(0usize) += 1;
                }
                let s8_units: f64 = quotas
                    .iter()
                    .map(|q| {
                        let c = counts.get(&q.teacher_id).copied().unwrap_or(0) as f64;
                        (c - q.quota).powi(2)
                    })
                    .sum();
                if s8_units < min_s8 {
                    min_s8 = s8_units;
                }
            }

            let bounds = lower_bounds(&problem);
            let s1_bound = bounds
                .iter()
                .find(|b| b.rule == RuleKey::S1)
                .unwrap()
                .units_lower_bound;
            let s6_bound = bounds
                .iter()
                .find(|b| b.rule == RuleKey::S6)
                .unwrap()
                .units_lower_bound;
            let s8_bound = bounds
                .iter()
                .find(|b| b.rule == RuleKey::S8)
                .unwrap()
                .units_lower_bound;

            assert!(
                min_s1 >= s1_bound - 1e-6,
                "Instance {attempt}: Exhaustive min S1 ({min_s1}) is LESS than computed bound ({s1_bound})! S1 bound is invalid."
            );
            assert!(
                min_s6 as f64 >= s6_bound - 1e-6,
                "Instance {attempt}: Exhaustive min S6 ({min_s6}) is LESS than computed bound ({s6_bound})! S6 bound is invalid."
            );
            assert!(
                min_s8 >= s8_bound - 1e-6,
                "Instance {attempt}: Exhaustive min S8 ({min_s8}) is LESS than computed bound ({s8_bound})! S8 bound is invalid."
            );

            valid_tested_instances += 1;
        }

        assert!(
            valid_tested_instances >= 200,
            "Expected at least 200 feasible tiny instances tested, got {valid_tested_instances}"
        );
    }

    #[test]
    fn test_s6_exact_per_teacher_minimum_function() {
        // E = 4 exams
        let num_exams = 4usize;
        let adj_cost = |s: usize| -> usize {
            let max_non_adjacent = num_exams.div_ceil(2);
            if s > max_non_adjacent && 2 * s > num_exams + 1 {
                2 * s - num_exams - 1
            } else {
                0
            }
        };

        // For E = 4:
        // s = 0: 0
        // s = 1: 0
        // s = 2: 0 (e.g. exams 1 and 3)
        // s = 3: >= 1 adjacent pair (e.g. {1,2,4} -> (1,2) is adjacent)
        // s = 4: exactly 3 adjacent pairs ((1,2), (2,3), (3,4))
        assert_eq!(adj_cost(0), 0);
        assert_eq!(adj_cost(1), 0);
        assert_eq!(adj_cost(2), 0);
        assert_eq!(adj_cost(3), 1);
        assert_eq!(adj_cost(4), 3);

        // Verify difference with naive max(0, s - 2) which would give 2 for s = 4
        assert_ne!(adj_cost(4), 4 - 2);
    }
}
