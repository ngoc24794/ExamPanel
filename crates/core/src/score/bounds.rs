//! Provable soft-constraint lower bound estimators.
//!
//! Provides mathematically sound, cheap lower bounds for soft constraint rules
//! to indicate when an objective cannot be improved further.

use crate::domain::{calculate_quotas, Problem, Role, RuleKey, TeacherId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Lower bound evaluation for a single soft constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct RuleBound {
    pub rule: RuleKey,
    pub units_lower_bound: f64,
    pub method: String,
}

/// Computes provable, cheap lower bounds for all soft rules S1..S10.
#[must_use]
pub fn lower_bounds(problem: &Problem) -> Vec<RuleBound> {
    let quotas = calculate_quotas(problem);
    let num_exams = problem.exams.len();
    let subjects = problem.effective_subjects();
    let panels = problem.all_panels();

    let mut total_slots = 0usize;
    let mut total_setter_slots = 0usize;
    let mut total_reviewer_slots = 0usize;

    for p in &panels {
        if let Some(sub) = subjects.iter().find(|s| s.id == p.subject_id) {
            total_slots += (sub.setters + sub.reviewers) as usize;
            total_setter_slots += sub.setters as usize;
            total_reviewer_slots += sub.reviewers as usize;
        }
    }

    let forced = crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let forced_seats = forced.len();
    let forced_setters = forced.iter().filter(|p| p.role == Role::Setter).count();
    let forced_reviewers = forced.iter().filter(|p| p.role == Role::Reviewer).count();

    let non_forced_slots = total_slots.saturating_sub(forced_seats);
    let non_forced_setter_slots = total_setter_slots.saturating_sub(forced_setters);
    let non_forced_reviewer_slots = total_reviewer_slots.saturating_sub(forced_reviewers);

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
            let is_competent_reviewer = problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer);
            if is_competent_reviewer {
                eligible_reviewers_count += 1;
            }
        }

        let s1_setting = problem.rule_settings.iter().find(|s| s.key == RuleKey::S1);
        let max_reviews_cfg = s1_setting.and_then(|s| {
            s.params
                .get("max_reviews")
                .and_then(serde_json::Value::as_u64)
        });
        let auto_max = if eligible_reviewers_count > 0 {
            (non_forced_reviewer_slots as f64 / eligible_reviewers_count as f64).ceil() as usize
        } else {
            2
        };
        let max_reviews = max_reviews_cfg.map_or(auto_max, |v| v as usize);

        let under_bound = eligible_reviewers_count.saturating_sub(non_forced_reviewer_slots);
        let over_bound =
            non_forced_reviewer_slots.saturating_sub(max_reviews * eligible_reviewers_count);
        (under_bound + over_bound) as f64
    };

    // -------------------------------------------------------------------------
    // S2..S5: Trivial 0.0 bounds
    // -------------------------------------------------------------------------
    let s2_bound = 0.0;
    let s3_bound = 0.0;
    let s4_bound = 0.0;
    let s5_bound = 0.0;

    // -------------------------------------------------------------------------
    // S6: Pigeonhole on setter slots with consecutive setting
    // -------------------------------------------------------------------------
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

        // Compute maximum non-forced setter capacity per teacher
        let mut setter_caps = Vec::with_capacity(problem.teachers.len());
        for t in &problem.teachers {
            if !t.active || t.load_weight <= 0.0 {
                setter_caps.push(0);
                continue;
            }
            let is_competent_setter = problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Setter);
            if !is_competent_setter {
                setter_caps.push(0);
                continue;
            }

            let hi = quotas
                .iter()
                .find(|q| q.teacher_id == t.id)
                .map_or(0, |q| q.hi);
            let eligible_setter_exams = problem
                .exams
                .iter()
                .filter(|e| {
                    if unavailabilities_set.contains(&(t.id, e.id)) {
                        return false;
                    }
                    problem.grades.iter().any(|g| {
                        subjects.iter().any(|s| {
                            crate::domain::forced::is_teacher_eligible(
                                problem,
                                t.id,
                                e.id,
                                g.id,
                                s.id,
                                Role::Setter,
                            )
                        })
                    })
                })
                .count();
            let cap = hi.min(eligible_setter_exams);
            setter_caps.push(cap);
        }

        // Greedy allocation minimizing sum of adj_cost(s_t)
        let mut s_alloc = vec![0usize; setter_caps.len()];
        let mut allocated = 0usize;
        for i in 0..setter_caps.len() {
            let take = setter_caps[i].min(max_non_adjacent);
            s_alloc[i] = take;
            allocated += take;
        }

        if allocated >= non_forced_setter_slots {
            0.0
        } else {
            while allocated < non_forced_setter_slots {
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
                    None => break,
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
    let (s8_bound, _) = optimal_s8_allocation(problem, non_forced_slots);

    // -------------------------------------------------------------------------
    // S9: Exam crowding pigeonhole
    // -------------------------------------------------------------------------
    let s9_bound = {
        let mut total_available_pairs = 0usize;
        for t in &problem.teachers {
            if !t.active || t.load_weight <= 0.0 || t.quota_override.is_some() {
                continue;
            }
            let is_competent = problem.competencies.iter().any(|c| c.teacher_id == t.id);
            if !is_competent {
                continue;
            }
            let avail = problem
                .exams
                .iter()
                .filter(|e| !unavailabilities_set.contains(&(t.id, e.id)))
                .count();
            total_available_pairs += avail;
        }
        non_forced_slots.saturating_sub(total_available_pairs) as f64
    };

    // -------------------------------------------------------------------------
    // S10: Review subject missing
    // -------------------------------------------------------------------------
    let s10_bound = 0.0;

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
        RuleBound {
            rule: RuleKey::S9,
            units_lower_bound: s9_bound,
            method: "exam_crowding_pigeonhole".to_string(),
        },
        RuleBound {
            rule: RuleKey::S10,
            units_lower_bound: s10_bound,
            method: "trivial".to_string(),
        },
    ]
}

/// Computes the exact S8 lower bound and the optimal integer task count per teacher.
#[must_use]
pub fn optimal_s8_counts(problem: &Problem) -> HashMap<TeacherId, usize> {
    let subjects = problem.effective_subjects();
    let panels = problem.all_panels();
    let mut total_slots = 0usize;
    for p in &panels {
        if let Some(sub) = subjects.iter().find(|s| s.id == p.subject_id) {
            total_slots += (sub.setters + sub.reviewers) as usize;
        }
    }
    let forced = crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let non_forced_slots = total_slots.saturating_sub(forced.len());
    let (_, counts) = optimal_s8_allocation(problem, non_forced_slots);
    counts
}

fn optimal_s8_allocation(
    problem: &Problem,
    total_non_forced_slots: usize,
) -> (f64, HashMap<TeacherId, usize>) {
    let quotas = calculate_quotas(problem);
    if quotas.is_empty() {
        return (0.0, HashMap::new());
    }

    let forced = crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let mut forced_counts: HashMap<TeacherId, usize> = HashMap::new();
    for p in &forced {
        *forced_counts.entry(p.teacher_id).or_default() += 1;
    }

    // Filter to teachers participating in non-forced allocation
    let active_quotas: Vec<_> = quotas
        .into_iter()
        .filter(|q| {
            let t = match problem.teachers.iter().find(|t| t.id == q.teacher_id) {
                Some(t) => t,
                None => return false,
            };
            t.quota_override.is_none() && t.active && t.load_weight > 0.0
        })
        .collect();

    if active_quotas.is_empty() {
        return (0.0, HashMap::new());
    }

    // Initialize counts with (lo - f_t)
    let mut counts: HashMap<TeacherId, usize> = HashMap::new();
    for q in &active_quotas {
        let f_t = forced_counts.get(&q.teacher_id).copied().unwrap_or(0);
        let lo_prime = q.lo.saturating_sub(f_t);
        counts.insert(q.teacher_id, lo_prime);
    }

    let mut current_sum: usize = counts.values().sum();

    while current_sum < total_non_forced_slots {
        let mut best_teacher = None;
        let mut best_diff = f64::INFINITY;

        for q in &active_quotas {
            let f_t = forced_counts.get(&q.teacher_id).copied().unwrap_or(0);
            let hi_prime = q.hi.saturating_sub(f_t);
            let q_prime = (q.quota - f_t as f64).max(0.0);
            let c = counts[&q.teacher_id];
            if c < hi_prime {
                let diff = c as f64 - q_prime;
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
            None => break,
        }
    }

    let penalty: f64 = active_quotas
        .iter()
        .map(|q| {
            let f_t = forced_counts.get(&q.teacher_id).copied().unwrap_or(0);
            let q_prime = (q.quota - f_t as f64).max(0.0);
            let c = counts.get(&q.teacher_id).copied().unwrap_or(0) as f64;
            (c - q_prime).powi(2)
        })
        .sum();

    (penalty, counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Assignment, Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope,
        PanelKey, Problem, Role, RuleKey, RuleSetting, SchoolYear, SchoolYearId, Subject,
        SubjectId, Teacher, TeacherGrade, Unavailability,
    };
    use rand::Rng;
    use rand_chacha::ChaCha8Rng;
    use rand_core::SeedableRng;

    #[test]
    fn test_lower_bounds_validity_exhaustive_tiny_instances() {
        let mut rng = ChaCha8Rng::seed_from_u64(1234567);
        let mut valid_tested_instances = 0usize;

        for attempt in 0..1000 {
            if valid_tested_instances >= 200 {
                break;
            }

            let num_exams = rng.gen_range(2..=3);
            let num_grades = 1usize;
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
                    color: "#f00".to_string(),
                },
                Campus {
                    id: CampusId(2),
                    code: "C2".to_string(),
                    name: "Campus 2".to_string(),
                    color: "#0f0".to_string(),
                },
            ];

            let grades: Vec<Grade> = (1..=num_grades)
                .map(|g| Grade {
                    id: GradeId(g as i64),
                    code: (10 + g) as i32,
                    name: format!("Grade {g}"),
                    sort_order: g as i32,
                })
                .collect();

            let sub1 = Subject {
                id: SubjectId(1),
                code: "CHUNG".to_string(),
                name: "Chung".to_string(),
                color: "slate".to_string(),
                sort_order: 1,
                setters: 2,
                reviewers: 1,
                min_campuses: 2,
            };

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
            let mut competencies = Vec::new();

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
                    display_name: None,
                    campus_id: camp,
                    load_weight: lw,
                    active: true,
                    note: None,
                    code: None,
                    quota_override: None,
                    max_tasks_per_exam_override: None,
                });

                for g in &grades {
                    teacher_grades.push(TeacherGrade {
                        teacher_id: TeacherId(tid as i64),
                        school_year_id: sy.id,
                        grade_id: g.id,
                    });
                }

                competencies.push(Competency {
                    teacher_id: TeacherId(tid as i64),
                    subject_id: SubjectId(1),
                    role: Role::Setter,
                    grade_scope: GradeScope::Taught,
                });
                competencies.push(Competency {
                    teacher_id: TeacherId(tid as i64),
                    subject_id: SubjectId(1),
                    role: Role::Reviewer,
                    grade_scope: GradeScope::Taught,
                });
            }

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
                subjects: vec![sub1],
                exams,
                teachers,
                teacher_grades,
                competencies,
                unavailabilities,
                locks: vec![],
                rule_settings: RuleSetting::default_settings(),
            };

            let panels = problem.all_panels();

            // Candidate triples for each panel
            let mut panel_triples: Vec<Vec<(usize, usize, usize)>> = Vec::new();
            for p in &panels {
                let mut triples = Vec::new();
                for u1 in 0..num_teachers {
                    for u2 in (u1 + 1)..num_teachers {
                        for ur in 0..num_teachers {
                            if ur == u1 || ur == u2 {
                                continue;
                            }
                            let c1 = problem.teachers[u1].campus_id;
                            let c2 = problem.teachers[u2].campus_id;
                            let cr = problem.teachers[ur].campus_id;
                            let mut distinct_campuses = HashSet::new();
                            distinct_campuses.insert(c1);
                            distinct_campuses.insert(c2);
                            distinct_campuses.insert(cr);
                            if distinct_campuses.len() < 2 {
                                continue;
                            }

                            let t1_id = problem.teachers[u1].id;
                            let t2_id = problem.teachers[u2].id;
                            let tr_id = problem.teachers[ur].id;

                            if problem.unavailabilities.iter().any(|u| {
                                u.exam_id == p.exam_id
                                    && (u.teacher_id == t1_id
                                        || u.teacher_id == t2_id
                                        || u.teacher_id == tr_id)
                            }) {
                                continue;
                            }

                            triples.push((u1, u2, ur));
                        }
                    }
                }
                panel_triples.push(triples);
            }

            if panel_triples.iter().any(|v| v.is_empty()) {
                continue;
            }

            let quotas = calculate_quotas(&problem);
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
                        panel.subject_id,
                        t1_id,
                        Role::Setter,
                        0,
                    ));
                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        panel.subject_id,
                        t2_id,
                        Role::Setter,
                        1,
                    ));
                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        panel.subject_id,
                        tr_id,
                        Role::Reviewer,
                        0,
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
                continue;
            }

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
}
