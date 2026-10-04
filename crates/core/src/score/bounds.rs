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
    // S9: Avoidable exam crowding lower bound (0.0 under even distribution)
    // -------------------------------------------------------------------------
    let s9_bound = 0.0;

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
    fn test_s6_exact_per_teacher_minimum_function() {
        let num_exams = 4usize;
        let adj_cost = |s: usize| -> usize {
            let max_non_adjacent = num_exams.div_ceil(2);
            if s > max_non_adjacent && 2 * s > num_exams + 1 {
                2 * s - num_exams - 1
            } else {
                0
            }
        };

        assert_eq!(adj_cost(0), 0);
        assert_eq!(adj_cost(1), 0);
        assert_eq!(adj_cost(2), 0);
        assert_eq!(adj_cost(3), 1);
        assert_eq!(adj_cost(4), 3);
        assert_ne!(adj_cost(4), 4 - 2);
    }

    #[test]
    fn test_lower_bounds_validity_exhaustive_tiny_instances() {
        let mut rng = ChaCha8Rng::seed_from_u64(1234567);
        let mut valid_tested_instances = 0usize;

        for attempt in 0..1000 {
            if valid_tested_instances >= 200 {
                break;
            }

            let num_exams = 2;
            let num_grades = 1usize;
            let num_teachers = rng.gen_range(4..=5);

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
                code: "SUB1".to_string(),
                name: "Subject 1".to_string(),
                color: "slate".to_string(),
                sort_order: 1,
                setters: 1,
                reviewers: 1,
                min_campuses: 1,
            };
            let sub2 = Subject {
                id: SubjectId(2),
                code: "SUB2".to_string(),
                name: "Subject 2".to_string(),
                color: "blue".to_string(),
                sort_order: 2,
                setters: 1,
                reviewers: 1,
                min_campuses: 1,
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

                // Multi-subject competencies:
                // Sub 1 competency
                if rng.gen_bool(0.8) {
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
                // Sub 2 competency
                if rng.gen_bool(0.8) {
                    competencies.push(Competency {
                        teacher_id: TeacherId(tid as i64),
                        subject_id: SubjectId(2),
                        role: Role::Setter,
                        grade_scope: GradeScope::Taught,
                    });
                    competencies.push(Competency {
                        teacher_id: TeacherId(tid as i64),
                        subject_id: SubjectId(2),
                        role: Role::Reviewer,
                        grade_scope: GradeScope::Taught,
                    });
                }
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
                subjects: vec![sub1, sub2],
                exams,
                teachers,
                teacher_grades,
                competencies,
                unavailabilities,
                locks: vec![],
                rule_settings: RuleSetting::default_settings(),
            };

            let panels = problem.all_panels();

            let is_competent = |tid: TeacherId, gid: GradeId, sid: SubjectId, role: Role| -> bool {
                problem.competencies.iter().any(|c| {
                    c.teacher_id == tid
                        && c.subject_id == sid
                        && c.role == role
                        && (c.grade_scope == GradeScope::Any
                            || problem
                                .teacher_grades
                                .iter()
                                .any(|tg| tg.teacher_id == tid && tg.grade_id == gid))
                })
            };

            // Candidate pairs (setter, reviewer) for each panel
            let mut panel_pairs: Vec<Vec<(usize, usize)>> = Vec::new();
            for p in &panels {
                let mut pairs = Vec::new();
                for u_s in 0..num_teachers {
                    let ts_id = problem.teachers[u_s].id;
                    if !is_competent(ts_id, p.grade_id, p.subject_id, Role::Setter) {
                        continue;
                    }
                    if problem
                        .unavailabilities
                        .iter()
                        .any(|u| u.teacher_id == ts_id && u.exam_id == p.exam_id)
                    {
                        continue;
                    }

                    for u_r in 0..num_teachers {
                        if u_r == u_s {
                            continue;
                        }
                        let tr_id = problem.teachers[u_r].id;
                        if !is_competent(tr_id, p.grade_id, p.subject_id, Role::Reviewer) {
                            continue;
                        }
                        if problem
                            .unavailabilities
                            .iter()
                            .any(|u| u.teacher_id == tr_id && u.exam_id == p.exam_id)
                        {
                            continue;
                        }

                        pairs.push((u_s, u_r));
                    }
                }
                panel_pairs.push(pairs);
            }

            if panel_pairs.iter().any(|v| v.is_empty()) {
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

            fn search_multi_fast(
                p_idx: usize,
                panels: &[PanelKey],
                panel_pairs: &[Vec<(usize, usize)>],
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
                for &(u_s, u_r) in &panel_pairs[p_idx] {
                    if used_counts[u_s] >= hi_limits[u_s] || used_counts[u_r] >= hi_limits[u_r] {
                        continue;
                    }

                    let ts_id = problem.teachers[u_s].id;
                    let tr_id = problem.teachers[u_r].id;

                    let mut conflict = false;
                    for a in current.iter() {
                        if a.exam_id == panel.exam_id
                            && (a.teacher_id == ts_id || a.teacher_id == tr_id)
                        {
                            conflict = true;
                            break;
                        }
                    }
                    if conflict {
                        continue;
                    }

                    used_counts[u_s] += 1;
                    used_counts[u_r] += 1;

                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        panel.subject_id,
                        ts_id,
                        Role::Setter,
                        0,
                    ));
                    current.push(Assignment::new(
                        panel.exam_id,
                        panel.grade_id,
                        panel.subject_id,
                        tr_id,
                        Role::Reviewer,
                        0,
                    ));

                    search_multi_fast(
                        p_idx + 1,
                        panels,
                        panel_pairs,
                        problem,
                        hi_limits,
                        lo_limits,
                        used_counts,
                        current,
                        solutions,
                    );

                    current.pop();
                    current.pop();

                    used_counts[u_s] -= 1;
                    used_counts[u_r] -= 1;
                }
            }

            search_multi_fast(
                0,
                &panels,
                &panel_pairs,
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
            let mut min_s6 = f64::INFINITY;
            let mut min_s8 = f64::INFINITY;

            let eligible_reviewers_count = quotas
                .iter()
                .filter(|q| {
                    if q.quota < 1.0 {
                        return false;
                    }
                    let t = match problem.teachers.iter().find(|t| t.id == q.teacher_id) {
                        Some(t) if t.active && t.load_weight > 0.0 => t,
                        _ => return false,
                    };
                    problem
                        .competencies
                        .iter()
                        .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer)
                })
                .count();
            let non_forced_reviewer_slots = panels.len();
            let auto_max = if eligible_reviewers_count > 0 {
                (non_forced_reviewer_slots as f64 / eligible_reviewers_count as f64).ceil() as usize
            } else {
                2
            };

            for sol in &all_solutions {
                let mut counts = vec![0usize; num_teachers];
                let mut rev_counts = vec![0usize; num_teachers];
                let mut setter_exams = vec![vec![]; num_teachers];

                for a in sol {
                    let t_idx = a.teacher_id.0 as usize - 1;
                    counts[t_idx] += 1;
                    if a.role == Role::Reviewer {
                        rev_counts[t_idx] += 1;
                    } else if a.role == Role::Setter {
                        setter_exams[t_idx].push(a.exam_id.0 as usize);
                    }
                }

                let mut s1 = 0.0;
                for q in &quotas {
                    if q.quota < 1.0 {
                        continue;
                    }
                    let t_idx = q.teacher_id.0 as usize - 1;
                    let is_comp = problem
                        .competencies
                        .iter()
                        .any(|c| c.teacher_id == q.teacher_id && c.role == Role::Reviewer);
                    if !is_comp {
                        continue;
                    }
                    let revs = rev_counts[t_idx];
                    if revs == 0 {
                        s1 += 1.0;
                    } else if revs > auto_max {
                        s1 += (revs - auto_max) as f64;
                    }
                }

                let mut s6 = 0.0;
                for mut exams in setter_exams {
                    exams.sort_unstable();
                    exams.dedup();
                    for w in exams.windows(2) {
                        if w[1] == w[0] + 1 {
                            s6 += 1.0;
                        }
                    }
                }

                let mut s8 = 0.0;
                for q in &quotas {
                    let t_idx = q.teacher_id.0 as usize - 1;
                    let diff = (counts[t_idx] as f64) - q.quota;
                    s8 += diff * diff;
                }

                min_s1 = min_s1.min(s1);
                min_s6 = min_s6.min(s6);
                min_s8 = min_s8.min(s8);
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
                min_s6 >= s6_bound - 1e-6,
                "Instance {attempt}: Exhaustive min S6 ({min_s6}) is LESS than computed bound ({s6_bound})! S6 bound is invalid."
            );
            assert!(
                min_s8 >= s8_bound - 1e-6,
                "Instance {attempt}: Exhaustive min S8 ({min_s8}) is LESS than computed bound ({s8_bound})! S8 bound is invalid."
            );

            valid_tested_instances += 1;
        }

        println!(
            "Tested {} feasible multi-subject tiny instances, all satisfied S1, S6, S8 lower bounds.",
            valid_tested_instances
        );
        assert!(
            valid_tested_instances >= 200,
            "Expected at least 200 feasible tiny multi-subject instances tested, got {valid_tested_instances}"
        );
    }
}
