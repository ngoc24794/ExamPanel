//! Workload quota calculations per rule H7 with availability scaling.

use super::entities::{LockKind, Role, RuleKey};
use super::ids::{ExamId, GradeId, TeacherId};
use super::problem::Problem;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Calculated annual workload quota bounds for an individual teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherQuota {
    pub teacher_id: TeacherId,
    pub quota: f64,
    pub lo: usize,
    pub hi: usize,
}

/// Computes the workload quota bounds for all teachers in the problem snapshot according to H7.
///
/// Quota formulation:
/// - $D = \text{number of panels} \times 3$ (total slots).
/// - $\text{availability}_t = \text{exams where } t \text{ is available and eligible for at least one grade}$.
/// - Effective weight: $w'_t = \text{load\_weight}_t \times \frac{\text{availability}_t}{\text{number of exams}}$ (0 if ineligible).
/// - Base quota: $q_t = D \times \frac{w'_t}{\sum w'}$.
/// - Lower bound: $\text{lo}_t = \max(0, \lfloor q_t \rfloor - k)$.
/// - Upper bound: $\text{hi}_t = \min(\text{achievable}, \lceil q_t \rceil + k)$.
///   When H4 is enabled, achievable $\le \text{availability}_t$.
/// - Teachers with `load_weight = 0`, `active = false`, or `availability = 0` have $\text{lo}_t = \text{hi}_t = 0$.
#[must_use]
pub fn calculate_quotas(problem: &Problem) -> Vec<TeacherQuota> {
    let num_exams = problem.exams.len();
    let num_grades = problem.grades.len();
    let total_panels = num_exams * num_grades;
    let total_slots = total_panels * 3;

    if total_slots == 0 || problem.teachers.is_empty() {
        return problem
            .teachers
            .iter()
            .map(|t| TeacherQuota {
                teacher_id: t.id,
                quota: 0.0,
                lo: 0,
                hi: 0,
            })
            .collect();
    }

    let h4_enabled = problem
        .rule_settings
        .iter()
        .find(|s| s.key == RuleKey::H4)
        .is_none_or(|s| s.enabled);

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

    let mut teacher_grades_map: HashMap<TeacherId, HashSet<GradeId>> = HashMap::new();
    for tg in &problem.teacher_grades {
        if tg.school_year_id == problem.school_year.id {
            teacher_grades_map
                .entry(tg.teacher_id)
                .or_default()
                .insert(tg.grade_id);
        }
    }

    let active_grades: HashSet<GradeId> = problem.grades.iter().map(|g| g.id).collect();

    // Check FORBID locks that completely block a teacher on (exam, grade)
    // A teacher is blocked on (exam, grade) if there is a FORBID with role None,
    // or both FORBID(Setter) and FORBID(Reviewer).
    let mut forbid_any_map: HashSet<(ExamId, GradeId, TeacherId)> = HashSet::new();
    let mut forbid_roles_map: HashMap<(ExamId, GradeId, TeacherId), HashSet<Role>> = HashMap::new();
    for lock in &problem.locks {
        if lock.kind == LockKind::Forbid {
            let key = (lock.exam_id, lock.grade_id, lock.teacher_id);
            match lock.role {
                None => {
                    forbid_any_map.insert(key);
                }
                Some(r) => {
                    forbid_roles_map.entry(key).or_default().insert(r);
                }
            }
        }
    }

    let is_forbidden_all_roles = |e: ExamId, g: GradeId, t: TeacherId| -> bool {
        let key = (e, g, t);
        if forbid_any_map.contains(&key) {
            return true;
        }
        if let Some(roles) = forbid_roles_map.get(&key) {
            if roles.contains(&Role::Setter) && roles.contains(&Role::Reviewer) {
                return true;
            }
        }
        false
    };

    let mut availabilities: Vec<usize> = Vec::with_capacity(problem.teachers.len());
    let mut effective_weights: Vec<f64> = Vec::with_capacity(problem.teachers.len());

    for teacher in &problem.teachers {
        if !teacher.active || teacher.load_weight <= 0.0 {
            availabilities.push(0);
            effective_weights.push(0.0);
            continue;
        }

        let taught = match teacher_grades_map.get(&teacher.id) {
            Some(set) if !set.is_empty() => set,
            _ => {
                availabilities.push(0);
                effective_weights.push(0.0);
                continue;
            }
        };

        let mut available_exams = 0usize;
        for exam in &problem.exams {
            if unavailability_set.contains(&(teacher.id, exam.id)) {
                continue;
            }

            // Must be eligible for at least one active grade in this exam
            let has_eligible_grade = taught.iter().any(|&g_id| {
                active_grades.contains(&g_id) && !is_forbidden_all_roles(exam.id, g_id, teacher.id)
            });

            if has_eligible_grade {
                available_exams += 1;
            }
        }

        let w_prime = if num_exams > 0 && available_exams > 0 {
            teacher.load_weight * (available_exams as f64) / (num_exams as f64)
        } else {
            0.0
        };

        availabilities.push(available_exams);
        effective_weights.push(w_prime);
    }

    let sum_w_prime: f64 = effective_weights.iter().sum();

    let mut quotas = Vec::with_capacity(problem.teachers.len());
    for (i, teacher) in problem.teachers.iter().enumerate() {
        let w_prime = effective_weights[i];
        let avail = availabilities[i];

        if !teacher.active || teacher.load_weight <= 0.0 || avail == 0 || sum_w_prime <= 0.0 {
            quotas.push(TeacherQuota {
                teacher_id: teacher.id,
                quota: 0.0,
                lo: 0,
                hi: 0,
            });
            continue;
        }

        let q_t = (total_slots as f64) * w_prime / sum_w_prime;
        let floor_q = q_t.floor() as usize;
        let lo = floor_q.saturating_sub(tolerance);

        let mut hi = (q_t.ceil() as usize) + tolerance;
        if h4_enabled {
            hi = hi.min(avail);
        }
        hi = hi.min(total_slots);
        let lo = lo.min(hi);

        quotas.push(TeacherQuota {
            teacher_id: teacher.id,
            quota: q_t,
            lo,
            hi,
        });
    }

    quotas
}
