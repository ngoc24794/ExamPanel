//! Workload quota calculations per rule H7 (v2) with availability scaling and bisection.

use super::entities::{Role, RuleKey};
use super::forced::{find_forced_placements, is_teacher_eligible};
use super::ids::{ExamId, TeacherId};
use super::problem::Problem;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Calculated annual workload quota bounds for an individual teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct TeacherQuota {
    pub teacher_id: TeacherId,
    pub quota: f64,
    pub lo: usize,
    pub hi: usize,
    #[serde(default)]
    pub available_exams: usize,
}

/// Computes the workload quota bounds for all teachers in the problem snapshot according to H7 (v2).
///
/// Quota formulation v2:
/// - $D = \text{total seats across all panels}$ ($\sum_{e,g,s} (s.\text{setters} + s.\text{reviewers})$).
/// - $F_t = \text{forced seats of } t$ from propagation.
/// - $\text{cap}_t = \min(\text{number of seats } t \text{ is eligible for}, \sum_{e \text{ avail}} \text{eff\_max\_tasks}(t, e))$.
/// - $w'_t = \text{load\_weight}_t \times \frac{\text{availability}_t}{E}$ (0 if eligible nowhere).
/// - $q_t = \text{clamp}(\lambda \cdot w'_t, F_t, \text{cap}_t)$ with $\lambda$ found by bisection so that $\sum q_t = D$.
/// - Teachers with `quota_override` use the override.
/// - Bounds with tolerance $k$:
///   $\text{lo}_t = \max(F_t, \lfloor q_t \rfloor - k, 0)$.
///   $\text{hi}_t = \min(\text{cap}_t, \max(F_t, \lceil q_t \rceil + k))$.
///   Override teachers have $\text{lo}_t = \text{hi}_t = \text{override}$.
#[must_use]
pub fn calculate_quotas(problem: &Problem) -> Vec<TeacherQuota> {
    let num_exams = problem.exams.len();
    let subjects = problem.effective_subjects();
    let panels = problem.all_panels();

    let mut total_slots = 0usize;
    for p in &panels {
        if let Some(s) = subjects.iter().find(|sub| sub.id == p.subject_id) {
            total_slots += (s.setters + s.reviewers) as usize;
        }
    }

    if total_slots == 0 || problem.teachers.is_empty() {
        return problem
            .teachers
            .iter()
            .map(|t| TeacherQuota {
                teacher_id: t.id,
                quota: 0.0,
                lo: 0,
                hi: 0,
                available_exams: 0,
            })
            .collect();
    }

    let h4_setting = problem.rule_settings.iter().find(|s| s.key == RuleKey::H4);
    let h4_enabled = h4_setting.is_none_or(|s| s.enabled);
    let default_max_tasks_per_exam = h4_setting
        .and_then(|s| {
            s.params
                .get("max_tasks_per_exam")
                .and_then(serde_json::Value::as_u64)
        })
        .map_or(2, |v| v as usize);

    let tolerance = problem
        .rule_settings
        .iter()
        .find(|s| s.key == RuleKey::H7)
        .and_then(|s| {
            s.params
                .get("tolerance")
                .and_then(serde_json::Value::as_u64)
        })
        .map_or(1, |v| v as usize);

    let unavailability_set: HashSet<(TeacherId, ExamId)> = problem
        .unavailabilities
        .iter()
        .map(|u| (u.teacher_id, u.exam_id))
        .collect();

    let forced_placements = find_forced_placements(problem).unwrap_or_default();
    let mut forced_counts: HashMap<TeacherId, usize> = HashMap::new();
    let mut forced_per_exam: HashMap<(TeacherId, ExamId), usize> = HashMap::new();
    for p in &forced_placements {
        *forced_counts.entry(p.teacher_id).or_default() += 1;
        *forced_per_exam
            .entry((p.teacher_id, p.panel.exam_id))
            .or_default() += 1;
    }

    let total_teachers = problem.teachers.len();
    let mut availabilities = Vec::with_capacity(total_teachers);
    let mut effective_weights = Vec::with_capacity(total_teachers);
    let mut caps = Vec::with_capacity(total_teachers);
    let mut forced_list = Vec::with_capacity(total_teachers);

    for teacher in &problem.teachers {
        let f_t = forced_counts.get(&teacher.id).copied().unwrap_or(0);
        forced_list.push(f_t);

        if !teacher.active || teacher.load_weight <= 0.0 {
            availabilities.push(0);
            effective_weights.push(0.0);
            caps.push(f_t);
            continue;
        }

        // Count eligible seats and availability per exam
        let mut available_exams = 0usize;
        let mut eligible_seats = 0usize;
        let mut sum_eff_max_tasks = 0usize;

        let max_tasks_configured = teacher
            .max_tasks_per_exam_override
            .map_or(default_max_tasks_per_exam, |v| v as usize);

        for exam in &problem.exams {
            if unavailability_set.contains(&(teacher.id, exam.id)) {
                continue;
            }

            let mut exam_eligible_seats = 0usize;
            for p in &panels {
                if p.exam_id != exam.id {
                    continue;
                }
                let subject = match subjects.iter().find(|sub| sub.id == p.subject_id) {
                    Some(s) => s,
                    None => continue,
                };

                let setter_elig = is_teacher_eligible(
                    problem,
                    teacher.id,
                    p.exam_id,
                    p.grade_id,
                    p.subject_id,
                    Role::Setter,
                );
                let reviewer_elig = is_teacher_eligible(
                    problem,
                    teacher.id,
                    p.exam_id,
                    p.grade_id,
                    p.subject_id,
                    Role::Reviewer,
                );

                if setter_elig {
                    exam_eligible_seats += subject.setters as usize;
                }
                if reviewer_elig {
                    exam_eligible_seats += subject.reviewers as usize;
                }
            }

            if exam_eligible_seats > 0 {
                available_exams += 1;
                eligible_seats += exam_eligible_seats;

                let forced_e = forced_per_exam
                    .get(&(teacher.id, exam.id))
                    .copied()
                    .unwrap_or(0);
                let eff_max = if h4_enabled {
                    max_tasks_configured.max(forced_e)
                } else {
                    total_slots
                };
                sum_eff_max_tasks += eff_max;
            }
        }

        availabilities.push(available_exams);

        let cap_t = eligible_seats.min(sum_eff_max_tasks).max(f_t);
        caps.push(cap_t);

        let w_prime = if num_exams > 0 && available_exams > 0 && eligible_seats > 0 {
            teacher.load_weight * (available_exams as f64) / (num_exams as f64)
        } else {
            0.0
        };
        effective_weights.push(w_prime);
    }

    // Bisection to find lambda such that sum(q_t) == D
    let target_sum = total_slots as f64;

    // Helper to evaluate sum of quotas for a given lambda
    let eval_quota = |lambda: f64, idx: usize| -> f64 {
        let teacher = &problem.teachers[idx];
        if let Some(override_val) = teacher.quota_override {
            return override_val as f64;
        }
        let w_prime = effective_weights[idx];
        let f_t = forced_list[idx] as f64;
        let cap_t = caps[idx] as f64;

        if w_prime <= 0.0 {
            return f_t;
        }

        let raw = lambda * w_prime;
        raw.clamp(f_t, cap_t)
    };

    let eval_total = |lambda: f64| -> f64 {
        let mut sum = 0.0;
        for i in 0..total_teachers {
            sum += eval_quota(lambda, i);
        }
        sum
    };

    // Find upper bound for bisection
    let mut lo_lambda = 0.0f64;
    let mut hi_lambda = 1.0f64;
    while eval_total(hi_lambda) < target_sum && hi_lambda < 1e8 {
        hi_lambda *= 2.0;
    }

    // Binary search 60 iterations
    for _ in 0..60 {
        let mid = (lo_lambda + hi_lambda) / 2.0;
        if eval_total(mid) < target_sum {
            lo_lambda = mid;
        } else {
            hi_lambda = mid;
        }
    }
    let best_lambda = (lo_lambda + hi_lambda) / 2.0;

    let mut quotas = Vec::with_capacity(total_teachers);
    for (i, teacher) in problem.teachers.iter().enumerate() {
        let avail = availabilities[i];
        let f_t = forced_list[i];
        let cap_t = caps[i];

        if let Some(override_val) = teacher.quota_override {
            let ov = override_val as usize;
            quotas.push(TeacherQuota {
                teacher_id: teacher.id,
                quota: ov as f64,
                lo: ov,
                hi: ov,
                available_exams: avail,
            });
            continue;
        }

        let _w_prime = effective_weights[i];
        if !teacher.active || teacher.load_weight <= 0.0 || (avail == 0 && f_t == 0) {
            quotas.push(TeacherQuota {
                teacher_id: teacher.id,
                quota: f_t as f64,
                lo: f_t,
                hi: f_t,
                available_exams: avail,
            });
            continue;
        }

        let q_t = eval_quota(best_lambda, i);
        let floor_q = q_t.floor() as usize;
        let lo = f_t.max(floor_q.saturating_sub(tolerance));

        let ceil_q = q_t.ceil() as usize;
        let hi = cap_t.min(f_t.max(ceil_q + tolerance));
        let lo = lo.min(hi);

        quotas.push(TeacherQuota {
            teacher_id: teacher.id,
            quota: q_t,
            lo,
            hi,
            available_exams: avail,
        });
    }

    quotas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, Role, RuleSetting,
        SchoolYear, SchoolYearId, Subject, SubjectId, Teacher, TeacherGrade,
    };

    #[test]
    fn test_quota_v2_regression_legacy_demo_equals_phase_3() {
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let campuses = vec![
            Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Campus 1".to_string(),
                color: "#111".to_string(),
            },
            Campus {
                id: CampusId(2),
                code: "CS2".to_string(),
                name: "Campus 2".to_string(),
                color: "#222".to_string(),
            },
            Campus {
                id: CampusId(3),
                code: "CS3".to_string(),
                name: "Campus 3".to_string(),
                color: "#333".to_string(),
            },
            Campus {
                id: CampusId(4),
                code: "CS4".to_string(),
                name: "Campus 4".to_string(),
                color: "#444".to_string(),
            },
        ];
        let grades: Vec<Grade> = (1..=3)
            .map(|g| Grade {
                id: GradeId(g),
                code: (9 + g) as i32,
                name: format!("Khối {}", 9 + g),
                sort_order: g as i32,
            })
            .collect();
        let exams: Vec<Exam> = (1..=4)
            .map(|e| Exam {
                id: ExamId(e),
                school_year_id: sy.id,
                code: format!("EX{e}"),
                name: format!("Kỳ thi {e}"),
                sort_order: e as i32,
            })
            .collect();
        let sub = Subject {
            id: SubjectId(1),
            code: "CHUNG".to_string(),
            name: "Chung".to_string(),
            color: "slate".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        };

        let mut teachers = Vec::new();
        let mut teacher_grades = Vec::new();
        let mut competencies = Vec::new();

        for tid in 1..=11 {
            teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Teacher {tid}"),
                display_name: None,
                campus_id: CampusId(((tid - 1) % 4) + 1),
                load_weight: 1.0,
                active: true,
                note: None,
                code: None,
                quota_override: None,
                max_tasks_per_exam_override: None,
            });

            for g in &grades {
                teacher_grades.push(TeacherGrade {
                    teacher_id: TeacherId(tid),
                    school_year_id: sy.id,
                    grade_id: g.id,
                });
            }

            competencies.push(Competency {
                teacher_id: TeacherId(tid),
                subject_id: sub.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: TeacherId(tid),
                subject_id: sub.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        let problem = Problem {
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
        };

        let quotas = calculate_quotas(&problem);
        assert_eq!(quotas.len(), 11);

        let expected_q = 36.0 / 11.0;
        for q in &quotas {
            assert!(
                (q.quota - expected_q).abs() < 1e-4,
                "Teacher {:?} quota {} != expected {}",
                q.teacher_id,
                q.quota,
                expected_q
            );
            assert_eq!(q.lo, 2, "Teacher {:?} lo != 2", q.teacher_id);
            assert_eq!(q.hi, 4, "Teacher {:?} hi != 4", q.teacher_id);
            assert_eq!(q.available_exams, 4);
        }
    }
}
