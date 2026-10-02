//! Soft constraint evaluation and score reporting.
//!
//! Evaluates complete assignment plans against soft quality constraints S1–S8:
//! - S1 Reviewer count: min 1, max 2 for eligible teachers with quota >= 1.
//! - S2 Role balance: |reviews - count / 3| for teachers with count >= 2.
//! - S3 Independent reviewer: setters sharing campus with reviewer.
//! - S4 Repeated setter pair: unordered pairs working together > 1 time.
//! - S5 Repeated review relation: directed (reviewer, setter) relations > 1 time.
//! - S6 Consecutive setting: setter in consecutive exams (by sort_order).
//! - S7 Grade rotation: variety for teachers qualified for >= 2 grades.
//! - S8 Load balance: squared deviation from fair target quota (count - q_t)^2.

use crate::domain::{
    calculate_quotas, Assignment, GradeId, LockKind, PanelKey, Problem, Role, RuleKey, TeacherId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

pub mod bounds;
pub use bounds::{lower_bounds, optimal_s8_counts, RuleBound};

/// Summary score report for a complete plan evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreReport {
    /// Total penalty across all enabled soft constraint rules.
    pub total: f64,
    /// Detailed score breakdown per soft rule (S1..S8).
    pub by_rule: Vec<RuleScore>,
    /// Itemized list of soft constraint violations with diagnostic context.
    pub violations: Vec<SoftViolation>,
    /// Per-teacher assignment and quota summary statistics.
    pub per_teacher: Vec<TeacherStats>,
}

/// Score breakdown for a specific soft constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleScore {
    pub rule: RuleKey,
    pub enabled: bool,
    pub weight: f64,
    pub units: f64,
    pub penalty: f64,
    pub lower_bound: f64,
}

/// A specific soft constraint violation instance with explanatory metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftViolation {
    pub rule: RuleKey,
    pub code: String,
    pub panel: Option<PanelKey>,
    pub teachers: Vec<TeacherId>,
    pub params: BTreeMap<String, String>,
}

/// Annual assignment statistics for an individual teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherStats {
    pub teacher_id: TeacherId,
    pub count: usize,
    pub setter: usize,
    pub reviewer: usize,
    pub quota: f64,
    pub grades_assigned: Vec<GradeId>,
}

/// Evaluates a complete assignment schedule against all soft constraints (S1..S8).
#[must_use]
pub fn evaluate(problem: &Problem, assignments: &[Assignment]) -> ScoreReport {
    let quotas = calculate_quotas(problem);
    let quota_map: HashMap<TeacherId, f64> =
        quotas.iter().map(|q| (q.teacher_id, q.quota)).collect();

    let teacher_map: HashMap<TeacherId, &crate::domain::Teacher> =
        problem.teachers.iter().map(|t| (t.id, t)).collect();

    let exam_map: HashMap<crate::domain::ExamId, &crate::domain::Exam> =
        problem.exams.iter().map(|e| (e.id, e)).collect();

    let grade_map: HashMap<GradeId, &crate::domain::Grade> =
        problem.grades.iter().map(|g| (g.id, g)).collect();

    let teacher_grades_map: HashMap<TeacherId, HashSet<GradeId>> = {
        let mut map: HashMap<TeacherId, HashSet<GradeId>> = HashMap::new();
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

    // Map rule settings for quick lookup
    let rule_settings_map: HashMap<RuleKey, &crate::domain::RuleSetting> =
        problem.rule_settings.iter().map(|s| (s.key, s)).collect();

    let get_rule_setting = |key: RuleKey, default_weight: f64| -> (bool, f64) {
        if let Some(s) = rule_settings_map.get(&key) {
            (s.enabled, s.weight)
        } else {
            (true, default_weight)
        }
    };

    // Aggregate teacher assignments
    let mut count_map: HashMap<TeacherId, usize> = HashMap::new();
    let mut setter_map: HashMap<TeacherId, usize> = HashMap::new();
    let mut reviewer_map: HashMap<TeacherId, usize> = HashMap::new();
    let mut teacher_assigned_grades: HashMap<TeacherId, HashSet<GradeId>> = HashMap::new();

    // Organize panel assignments: (exam_id, grade_id) -> (setters, reviewer)
    let mut panel_map: HashMap<
        (crate::domain::ExamId, GradeId),
        (Vec<TeacherId>, Option<TeacherId>),
    > = HashMap::new();

    // Teacher-per-exam role mapping: (teacher_id, exam_id) -> Option<Role>
    let mut teacher_exam_role: HashMap<(TeacherId, crate::domain::ExamId), Role> = HashMap::new();

    for a in assignments {
        *count_map.entry(a.teacher_id).or_insert(0) += 1;
        match a.role {
            Role::Setter => *setter_map.entry(a.teacher_id).or_insert(0) += 1,
            Role::Reviewer => *reviewer_map.entry(a.teacher_id).or_insert(0) += 1,
        }
        teacher_assigned_grades
            .entry(a.teacher_id)
            .or_default()
            .insert(a.grade_id);

        let entry = panel_map.entry((a.exam_id, a.grade_id)).or_default();
        match a.role {
            Role::Setter => entry.0.push(a.teacher_id),
            Role::Reviewer => entry.1 = Some(a.teacher_id),
        }

        teacher_exam_role.insert((a.teacher_id, a.exam_id), a.role);
    }

    let mut violations = Vec::new();

    // -------------------------------------------------------------------------
    // S1: Reviewer Count
    // -------------------------------------------------------------------------
    let (s1_enabled, s1_weight) = get_rule_setting(RuleKey::S1, 10.0);
    let mut s1_units = 0.0;

    // Check which teachers are reviewer-eligible somewhere
    for t in &problem.teachers {
        if !t.active || t.load_weight <= 0.0 {
            continue;
        }
        let q_t = quota_map.get(&t.id).copied().unwrap_or(0.0);
        if q_t < 1.0 {
            continue;
        }

        // Must be eligible for at least one panel as reviewer
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
                // Check if locked out by FORBID
                !problem.locks.iter().any(|lock| {
                    lock.exam_id == e.id
                        && lock.grade_id == g.id
                        && lock.teacher_id == t.id
                        && lock.kind == LockKind::Forbid
                        && (lock.role.is_none() || lock.role == Some(Role::Reviewer))
                })
            })
        });

        if !has_eligible_panel {
            continue;
        }

        let reviews = reviewer_map.get(&t.id).copied().unwrap_or(0);
        if reviews == 0 {
            s1_units += 1.0;
            let mut params = BTreeMap::new();
            params.insert("teacher".to_string(), t.full_name.clone());
            params.insert("quota".to_string(), format!("{q_t:.2}"));
            violations.push(SoftViolation {
                rule: RuleKey::S1,
                code: "reviewer_never".to_string(),
                panel: None,
                teachers: vec![t.id],
                params,
            });
        } else if reviews > 2 {
            let excess = (reviews - 2) as f64;
            s1_units += excess;
            let mut params = BTreeMap::new();
            params.insert("teacher".to_string(), t.full_name.clone());
            params.insert("count".to_string(), reviews.to_string());
            params.insert("max".to_string(), "2".to_string());
            violations.push(SoftViolation {
                rule: RuleKey::S1,
                code: "reviewer_too_many".to_string(),
                panel: None,
                teachers: vec![t.id],
                params,
            });
        }
    }

    // -------------------------------------------------------------------------
    // S2: Role Balance
    // -------------------------------------------------------------------------
    let (s2_enabled, s2_weight) = get_rule_setting(RuleKey::S2, 3.0);
    let mut s2_units = 0.0;

    for t in &problem.teachers {
        let count_t = count_map.get(&t.id).copied().unwrap_or(0);
        if count_t >= 2 {
            let reviews = reviewer_map.get(&t.id).copied().unwrap_or(0);
            let lo = count_t / 3;
            let hi = count_t.div_ceil(3);
            let diff = if reviews < lo {
                (lo - reviews) as f64
            } else if reviews > hi {
                (reviews - hi) as f64
            } else {
                0.0
            };
            if diff > 1e-9 {
                s2_units += diff;
                let mut params = BTreeMap::new();
                params.insert("teacher".to_string(), t.full_name.clone());
                params.insert("reviews".to_string(), reviews.to_string());
                params.insert("count".to_string(), count_t.to_string());
                params.insert("lo".to_string(), lo.to_string());
                params.insert("hi".to_string(), hi.to_string());
                params.insert("ideal".to_string(), format!("[{lo}, {hi}]"));
                params.insert("diff".to_string(), format!("{diff:.0}"));
                violations.push(SoftViolation {
                    rule: RuleKey::S2,
                    code: "role_imbalance".to_string(),
                    panel: None,
                    teachers: vec![t.id],
                    params,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // S3: Independent Reviewer
    // -------------------------------------------------------------------------
    let (s3_enabled, s3_weight) = get_rule_setting(RuleKey::S3, 4.0);
    let mut s3_units = 0.0;

    for (&(eid, gid), (setters, reviewer_opt)) in &panel_map {
        if let Some(r_id) = reviewer_opt {
            let r_campus = teacher_map.get(r_id).map(|t| t.campus_id);
            let mut same_campus_setters = Vec::new();
            for s_id in setters {
                let s_campus = teacher_map.get(s_id).map(|t| t.campus_id);
                if s_campus == r_campus {
                    same_campus_setters.push(*s_id);
                }
            }
            if !same_campus_setters.is_empty() {
                let count_same = same_campus_setters.len();
                s3_units += count_same as f64;
                let exam_code = exam_map
                    .get(&eid)
                    .map(|e| e.code.clone())
                    .unwrap_or_default();
                let grade_code = grade_map.get(&gid).map(|g| g.code).unwrap_or(0);
                let mut params = BTreeMap::new();
                params.insert("exam".to_string(), exam_code);
                params.insert("grade".to_string(), grade_code.to_string());
                params.insert("count".to_string(), count_same.to_string());
                violations.push(SoftViolation {
                    rule: RuleKey::S3,
                    code: "reviewer_same_campus".to_string(),
                    panel: Some(PanelKey::new(eid, gid)),
                    teachers: [vec![*r_id], same_campus_setters].concat(),
                    params,
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // S4: Repeated Setter Pair
    // -------------------------------------------------------------------------
    let (s4_enabled, s4_weight) = get_rule_setting(RuleKey::S4, 6.0);
    let mut s4_units = 0.0;

    let mut setter_pair_counts: BTreeMap<(TeacherId, TeacherId), usize> = BTreeMap::new();
    for (setters, _) in panel_map.values() {
        if setters.len() >= 2 {
            let mut s = setters.clone();
            s.sort();
            for i in 0..s.len() {
                for j in (i + 1)..s.len() {
                    *setter_pair_counts.entry((s[i], s[j])).or_insert(0) += 1;
                }
            }
        }
    }

    for ((t1, t2), count) in setter_pair_counts {
        if count > 1 {
            let excess = count - 1;
            s4_units += excess as f64;
            let name1 = teacher_map
                .get(&t1)
                .map(|t| t.full_name.clone())
                .unwrap_or_default();
            let name2 = teacher_map
                .get(&t2)
                .map(|t| t.full_name.clone())
                .unwrap_or_default();
            let mut params = BTreeMap::new();
            params.insert("teacher1".to_string(), name1);
            params.insert("teacher2".to_string(), name2);
            params.insert("count".to_string(), count.to_string());
            violations.push(SoftViolation {
                rule: RuleKey::S4,
                code: "setter_pair_repeated".to_string(),
                panel: None,
                teachers: vec![t1, t2],
                params,
            });
        }
    }

    // -------------------------------------------------------------------------
    // S5: Repeated Review Relation
    // -------------------------------------------------------------------------
    let (s5_enabled, s5_weight) = get_rule_setting(RuleKey::S5, 6.0);
    let mut s5_units = 0.0;

    let mut review_relation_counts: BTreeMap<(TeacherId, TeacherId), usize> = BTreeMap::new();
    for (setters, reviewer_opt) in panel_map.values() {
        if let Some(r_id) = reviewer_opt {
            for s_id in setters {
                *review_relation_counts.entry((*r_id, *s_id)).or_insert(0) += 1;
            }
        }
    }

    for ((rev, set), count) in review_relation_counts {
        if count > 1 {
            let excess = count - 1;
            s5_units += excess as f64;
            let rev_name = teacher_map
                .get(&rev)
                .map(|t| t.full_name.clone())
                .unwrap_or_default();
            let set_name = teacher_map
                .get(&set)
                .map(|t| t.full_name.clone())
                .unwrap_or_default();
            let mut params = BTreeMap::new();
            params.insert("reviewer".to_string(), rev_name);
            params.insert("setter".to_string(), set_name);
            params.insert("count".to_string(), count.to_string());
            violations.push(SoftViolation {
                rule: RuleKey::S5,
                code: "review_relation_repeated".to_string(),
                panel: None,
                teachers: vec![rev, set],
                params,
            });
        }
    }

    // -------------------------------------------------------------------------
    // S6: Consecutive Setting (Redefined: Setter in consecutive exams)
    // -------------------------------------------------------------------------
    let (s6_enabled, s6_weight) = get_rule_setting(RuleKey::S6, 2.0);
    let mut s6_units = 0.0;

    let mut sorted_exams = problem.exams.clone();
    sorted_exams.sort_by_key(|e| e.sort_order);

    if sorted_exams.len() >= 2 {
        for t in &problem.teachers {
            for i in 0..(sorted_exams.len() - 1) {
                let e1 = &sorted_exams[i];
                let e2 = &sorted_exams[i + 1];

                let is_setter_e1 = teacher_exam_role.get(&(t.id, e1.id)) == Some(&Role::Setter);
                let is_setter_e2 = teacher_exam_role.get(&(t.id, e2.id)) == Some(&Role::Setter);

                if is_setter_e1 && is_setter_e2 {
                    s6_units += 1.0;
                    let mut params = BTreeMap::new();
                    params.insert("teacher".to_string(), t.full_name.clone());
                    params.insert("exam1".to_string(), e1.code.clone());
                    params.insert("exam2".to_string(), e2.code.clone());
                    violations.push(SoftViolation {
                        rule: RuleKey::S6,
                        code: "setter_consecutive".to_string(),
                        panel: None,
                        teachers: vec![t.id],
                        params,
                    });
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // S7: Grade Rotation
    // -------------------------------------------------------------------------
    let (s7_enabled, s7_weight) = get_rule_setting(RuleKey::S7, 1.0);
    let mut s7_units = 0.0;

    for t in &problem.teachers {
        if let Some(qualified_grades) = teacher_grades_map.get(&t.id) {
            if qualified_grades.len() >= 2 {
                let count_t = count_map.get(&t.id).copied().unwrap_or(0);
                let target = count_t.min(qualified_grades.len());
                let assigned = teacher_assigned_grades
                    .get(&t.id)
                    .map(|s| s.len())
                    .unwrap_or(0);
                if assigned < target {
                    let diff = (target - assigned) as f64;
                    s7_units += diff;
                    let mut params = BTreeMap::new();
                    params.insert("teacher".to_string(), t.full_name.clone());
                    params.insert("assigned".to_string(), assigned.to_string());
                    params.insert("target".to_string(), target.to_string());
                    violations.push(SoftViolation {
                        rule: RuleKey::S7,
                        code: "grade_not_rotated".to_string(),
                        panel: None,
                        teachers: vec![t.id],
                        params,
                    });
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // S8: Load Balance (Squared deviation from fair quota)
    // -------------------------------------------------------------------------
    let (s8_enabled, s8_weight) = get_rule_setting(RuleKey::S8, 8.0);
    let mut s8_units = 0.0;

    for t in &problem.teachers {
        let count_t = count_map.get(&t.id).copied().unwrap_or(0);
        let q_t = quota_map.get(&t.id).copied().unwrap_or(0.0);
        let diff = count_t as f64 - q_t;
        let sq = diff * diff;
        if sq > 1e-9 {
            s8_units += sq;
            let mut params = BTreeMap::new();
            params.insert("teacher".to_string(), t.full_name.clone());
            params.insert("count".to_string(), count_t.to_string());
            params.insert("quota".to_string(), format!("{q_t:.2}"));
            params.insert("deviation".to_string(), format!("{diff:+.2}"));
            violations.push(SoftViolation {
                rule: RuleKey::S8,
                code: "load_deviation".to_string(),
                panel: None,
                teachers: vec![t.id],
                params,
            });
        }
    }

    // Compile lower bounds and by_rule scores
    let bounds = bounds::lower_bounds(problem);
    let get_lower_bound = |key: RuleKey| -> f64 {
        bounds
            .iter()
            .find(|b| b.rule == key)
            .map_or(0.0, |b| b.units_lower_bound)
    };

    let rule_scores = vec![
        RuleScore {
            rule: RuleKey::S1,
            enabled: s1_enabled,
            weight: s1_weight,
            units: s1_units,
            penalty: if s1_enabled {
                s1_weight * s1_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S1),
        },
        RuleScore {
            rule: RuleKey::S2,
            enabled: s2_enabled,
            weight: s2_weight,
            units: s2_units,
            penalty: if s2_enabled {
                s2_weight * s2_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S2),
        },
        RuleScore {
            rule: RuleKey::S3,
            enabled: s3_enabled,
            weight: s3_weight,
            units: s3_units,
            penalty: if s3_enabled {
                s3_weight * s3_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S3),
        },
        RuleScore {
            rule: RuleKey::S4,
            enabled: s4_enabled,
            weight: s4_weight,
            units: s4_units,
            penalty: if s4_enabled {
                s4_weight * s4_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S4),
        },
        RuleScore {
            rule: RuleKey::S5,
            enabled: s5_enabled,
            weight: s5_weight,
            units: s5_units,
            penalty: if s5_enabled {
                s5_weight * s5_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S5),
        },
        RuleScore {
            rule: RuleKey::S6,
            enabled: s6_enabled,
            weight: s6_weight,
            units: s6_units,
            penalty: if s6_enabled {
                s6_weight * s6_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S6),
        },
        RuleScore {
            rule: RuleKey::S7,
            enabled: s7_enabled,
            weight: s7_weight,
            units: s7_units,
            penalty: if s7_enabled {
                s7_weight * s7_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S7),
        },
        RuleScore {
            rule: RuleKey::S8,
            enabled: s8_enabled,
            weight: s8_weight,
            units: s8_units,
            penalty: if s8_enabled {
                s8_weight * s8_units
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S8),
        },
    ];

    let total: f64 = rule_scores.iter().map(|rs| rs.penalty).sum();

    // TeacherStats for each teacher
    let mut per_teacher = Vec::with_capacity(problem.teachers.len());
    for t in &problem.teachers {
        let count = count_map.get(&t.id).copied().unwrap_or(0);
        let setter = setter_map.get(&t.id).copied().unwrap_or(0);
        let reviewer = reviewer_map.get(&t.id).copied().unwrap_or(0);
        let quota = quota_map.get(&t.id).copied().unwrap_or(0.0);
        let mut grades_assigned: Vec<GradeId> = teacher_assigned_grades
            .get(&t.id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        grades_assigned.sort();

        per_teacher.push(TeacherStats {
            teacher_id: t.id,
            count,
            setter,
            reviewer,
            quota,
            grades_assigned,
        });
    }

    ScoreReport {
        total,
        by_rule: rule_scores,
        violations,
        per_teacher,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Campus, CampusId, Exam, ExamId, Grade, GradeId, Role, RuleSetting, SchoolYear,
        SchoolYearId, Teacher, TeacherGrade,
    };

    fn make_test_problem() -> Problem {
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
                color: "#111".to_string(),
            },
            Campus {
                id: CampusId(2),
                code: "C2".to_string(),
                name: "Campus 2".to_string(),
                color: "#222".to_string(),
            },
        ];
        let grades = vec![
            Grade {
                id: GradeId(1),
                code: 10,
                name: "G10".to_string(),
                sort_order: 1,
            },
            Grade {
                id: GradeId(2),
                code: 11,
                name: "G11".to_string(),
                sort_order: 2,
            },
        ];
        let exams = vec![
            Exam {
                id: ExamId(1),
                school_year_id: sy.id,
                code: "E1".to_string(),
                name: "Exam 1".to_string(),
                sort_order: 1,
            },
            Exam {
                id: ExamId(2),
                school_year_id: sy.id,
                code: "E2".to_string(),
                name: "Exam 2".to_string(),
                sort_order: 2,
            },
        ];

        let mut teachers = Vec::new();
        let mut teacher_grades = Vec::new();
        for i in 1..=6 {
            let cid = if i <= 3 { CampusId(1) } else { CampusId(2) };
            teachers.push(Teacher {
                id: TeacherId(i),
                full_name: format!("Teacher {i}"),
                campus_id: cid,
                load_weight: 1.0,
                active: true,
                note: None,
            });
            // Each teaches both grades
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(i),
                school_year_id: sy.id,
                grade_id: GradeId(1),
            });
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(i),
                school_year_id: sy.id,
                grade_id: GradeId(2),
            });
        }

        Problem {
            school_year: sy,
            campuses,
            grades,
            exams,
            teachers,
            teacher_grades,
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    #[test]
    fn test_s1_reviewer_count_violations() {
        let problem = make_test_problem();
        // 2 exams x 2 grades = 4 panels (12 slots). 6 teachers -> quota = 2.0 each.
        // Hand-build plan where T1 has 0 reviews (reviewer_never), T2 has 3 reviews (reviewer_too_many)
        let assignments = vec![
            // E1 G1: setters T1, T3, reviewer T2
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Reviewer),
            // E1 G2: setters T4, T5, reviewer T2
            Assignment::new(ExamId(1), GradeId(2), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(2), Role::Reviewer),
            // E2 G1: setters T1, T4, reviewer T2
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(2), Role::Reviewer),
            // E2 G2: setters T5, T6, reviewer T3
            Assignment::new(ExamId(2), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(3), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s1 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S1).unwrap();
        // T1 has 0 reviews -> 1 unit
        // T2 has 3 reviews -> 3 - 2 = 1 unit
        // T4, T5, T6 have 0 reviews -> 3 units
        // T3 has 1 review -> 0 units
        // Total S1 units = 1 (T1) + 1 (T2) + 3 (T4, T5, T6) = 5 units
        assert_eq!(s1.units, 5.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "reviewer_never" && v.teachers == vec![TeacherId(1)]));
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "reviewer_too_many" && v.teachers == vec![TeacherId(2)]));
    }

    #[test]
    fn test_s2_role_balance() {
        let problem = make_test_problem();
        // T1 has count = 3, reviewer = 0 -> lo = 1, hi = 1 -> dist = 1.0
        // T2 has count = 2, reviewer = 0 -> lo = 0, hi = 1 -> dist = 0.0 (inside interval [0, 1])
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(1)]));
        // Under new definition, T2 (count 2, reviewer 0) is inside [0, 1], so no violation
        assert!(!rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(2)]));

        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(4)]));
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(6)]));

        let s2 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S2).unwrap();
        assert_eq!(s2.units, 3.0);
    }

    #[test]
    fn test_lower_bounds_calculation() {
        let problem = make_test_problem();
        let bounds = lower_bounds(&problem);
        assert_eq!(bounds.len(), 8);
        for b in &bounds {
            assert!(b.units_lower_bound >= 0.0);
        }
        let s8_bound = bounds.iter().find(|b| b.rule == RuleKey::S8).unwrap();
        assert!(s8_bound.units_lower_bound >= 0.0);
    }

    #[test]
    fn test_s3_independent_reviewer() {
        let problem = make_test_problem();
        // T1, T2, T3 are Campus 1. T4, T5, T6 are Campus 2.
        // In panel E1 G1: setters T1, T4; reviewer T2. T1 and T2 share Campus 1 -> 1 unit.
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Reviewer),
            // other panels cleanly segregated
            Assignment::new(ExamId(1), GradeId(2), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(1), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s3 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S3).unwrap();
        // Panel 1: reviewer T2 (C1), setters T1 (C1), T4 (C2) -> 1 unit
        // Panel 2: reviewer T5 (C2), setters T2 (C1), T3 (C1) -> 0 units
        // Panel 3: reviewer T1 (C1), setters T4 (C2), T5 (C2) -> 0 units
        // Panel 4: reviewer T1 (C1), setters T3 (C1), T6 (C2) -> 1 unit
        // Total S3 units = 2
        assert_eq!(s3.units, 2.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "reviewer_same_campus"
                && v.panel == Some(PanelKey::new(ExamId(1), GradeId(1)))));
    }

    #[test]
    fn test_s4_setter_pair_repeated() {
        let problem = make_test_problem();
        // T1 and T2 pair up as setters in E1 G1 and in E2 G1 -> 1 excess unit
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(5), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(4), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s4 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S4).unwrap();
        assert_eq!(s4.units, 1.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "setter_pair_repeated"
                && v.teachers == vec![TeacherId(1), TeacherId(2)]));
    }

    #[test]
    fn test_s5_review_relation_repeated() {
        let problem = make_test_problem();
        // T4 reviews T1 in E1 G1 and in E2 G1 -> 1 excess unit
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(5), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s5 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S5).unwrap();
        assert_eq!(s5.units, 1.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "review_relation_repeated"
                && v.teachers == vec![TeacherId(4), TeacherId(1)]));
    }

    #[test]
    fn test_s6_setter_consecutive() {
        let problem = make_test_problem();
        // T1 is a setter in E1 (G1) and also in E2 (G1) -> 1 unit
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(2), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(5), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s6 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S6).unwrap();
        // T1 is setter in E1 and E2 -> 1 unit
        // T3 is setter in E1 and E2 -> 1 unit
        // Total S6 = 2 units
        assert_eq!(s6.units, 2.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "setter_consecutive" && v.teachers == vec![TeacherId(1)]));
    }

    #[test]
    fn test_s7_grade_rotation() {
        let problem = make_test_problem();
        // T1 is assigned twice, but both times in GradeId(1) -> distinct = 1, target = min(2, 2) = 2. diff = 1 unit
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(5), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "grade_not_rotated" && v.teachers == vec![TeacherId(1)]));
    }

    #[test]
    fn test_s8_load_deviation() {
        let problem = make_test_problem();
        // Quota is 2.0 for all 6 teachers.
        // T1 has count = 3 -> diff = 1.0, sq = 1.0
        // T2 has count = 1 -> diff = -1.0, sq = 1.0
        // T3..T6 have count = 2 -> diff = 0.0
        let assignments = vec![
            Assignment::new(ExamId(1), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(2), Role::Setter),
            Assignment::new(ExamId(1), GradeId(1), TeacherId(4), Role::Reviewer),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(5), Role::Setter),
            Assignment::new(ExamId(1), GradeId(2), TeacherId(6), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(3), Role::Setter),
            Assignment::new(ExamId(2), GradeId(1), TeacherId(5), Role::Reviewer),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(1), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(4), Role::Setter),
            Assignment::new(ExamId(2), GradeId(2), TeacherId(6), Role::Reviewer),
        ];

        let rep = evaluate(&problem, &assignments);
        let s8 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S8).unwrap();
        // T1 has count = 3: (3 - 2)^2 = 1
        // T2 has count = 1: (1 - 2)^2 = 1
        // T3 has count = 2: 0
        // T4 has count = 2: 0
        // T5 has count = 2: 0
        // T6 has count = 2: 0
        // Total S8 units = 2.0
        assert_eq!(s8.units, 2.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "load_deviation" && v.teachers == vec![TeacherId(1)]));
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "load_deviation" && v.teachers == vec![TeacherId(2)]));
    }
}
