//! Soft constraint evaluation and score reporting.
//!
//! Evaluates complete assignment plans against soft quality constraints S1–S10:
//! - S1 Reviewer count: dynamic capacity Auto = ceil(R' / N_rev), or Fixed(n).
//! - S2 Role balance: distance from [floor(c' * rho), ceil(c' * rho)].
//! - S3 Independent reviewer: setters sharing campus with reviewer.
//! - S4 Repeated setter pair: unordered pairs working together > 1 time.
//! - S5 Repeated review relation: directed (reviewer, setter) relations > 1 time.
//! - S6 Consecutive setting: setter in consecutive exams (excluding forced setter presence).
//! - S7 Grade rotation: variety for teachers qualified for >= 2 grades.
//! - S8 Load balance: squared deviation from fair target quota (c' - q_t)^2 for non-forced work.
//! - S9 Exam crowding: max(0, non_forced_tasks(t, e) - 1) per teacher per exam.
//! - S10 Review subject missing: reviewer-competent teachers should review each competent subject >= 1 time.

use crate::domain::{
    calculate_quotas, Assignment, GradeId, PanelKey, Problem, Role, RuleKey, SubjectId, TeacherId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

pub mod bounds;
pub use bounds::{lower_bounds, optimal_s8_counts, RuleBound};

/// Summary score report for a complete plan evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct ScoreReport {
    /// Total penalty across all enabled soft constraint rules.
    pub total: f64,
    /// Detailed score breakdown per soft rule (S1..S10).
    pub by_rule: Vec<RuleScore>,
    /// Itemized list of soft constraint violations with diagnostic context.
    pub violations: Vec<SoftViolation>,
    /// Per-teacher assignment and quota summary statistics.
    pub per_teacher: Vec<TeacherStats>,
}

/// Score breakdown for a specific soft constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct RuleScore {
    pub rule: RuleKey,
    pub enabled: bool,
    pub weight: f64,
    pub units: f64,
    pub penalty: f64,
    pub lower_bound: f64,
}

/// A specific soft constraint violation instance with explanatory metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct SoftViolation {
    pub rule: RuleKey,
    pub code: String,
    pub panel: Option<PanelKey>,
    pub teachers: Vec<TeacherId>,
    pub params: BTreeMap<String, String>,
}

/// Annual assignment statistics for an individual teacher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct TeacherStats {
    pub teacher_id: TeacherId,
    pub count: usize,
    pub setter: usize,
    pub reviewer: usize,
    pub quota: f64,
    pub grades_assigned: Vec<GradeId>,
}

/// Evaluates a complete assignment schedule against all soft constraints (S1..S10).
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

    let subjects = problem.effective_subjects();
    let subject_map: HashMap<SubjectId, &crate::domain::Subject> =
        subjects.iter().map(|s| (s.id, s)).collect();

    let teacher_grades_map: HashMap<TeacherId, HashSet<GradeId>> = {
        let mut map: HashMap<TeacherId, HashSet<GradeId>> = HashMap::new();
        for tg in &problem.teacher_grades {
            if tg.school_year_id == problem.school_year.id {
                map.entry(tg.teacher_id).or_default().insert(tg.grade_id);
            }
        }
        map
    };

    // Forced placements
    let forced_placements =
        crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let mut forced_counts: HashMap<TeacherId, usize> = HashMap::new();
    let mut forced_reviewer_counts: HashMap<TeacherId, usize> = HashMap::new();
    let mut forced_set: HashSet<(crate::domain::ExamId, GradeId, SubjectId, Role, TeacherId)> =
        HashSet::new();

    for p in &forced_placements {
        *forced_counts.entry(p.teacher_id).or_default() += 1;
        if p.role == Role::Reviewer {
            *forced_reviewer_counts.entry(p.teacher_id).or_default() += 1;
        }
        forced_set.insert((
            p.panel.exam_id,
            p.panel.grade_id,
            p.panel.subject_id,
            p.role,
            p.teacher_id,
        ));
    }

    let is_assignment_forced = |a: &Assignment| -> bool {
        forced_set.contains(&(a.exam_id, a.grade_id, a.subject_id, a.role, a.teacher_id))
    };

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

    let mut non_forced_count_map: HashMap<TeacherId, usize> = HashMap::new();
    let mut non_forced_reviewer_map: HashMap<TeacherId, usize> = HashMap::new();
    let mut non_forced_exam_tasks: HashMap<(TeacherId, crate::domain::ExamId), usize> =
        HashMap::new();
    let mut non_forced_exam_setters: HashMap<(TeacherId, crate::domain::ExamId), usize> =
        HashMap::new();

    // Teacher subject reviews map: (teacher_id, subject_id) -> count
    let mut teacher_subject_reviews: HashMap<(TeacherId, SubjectId), usize> = HashMap::new();

    // Organize panel assignments: (exam_id, grade_id, subject_id) -> (setters, reviewers)
    type PanelAssignmentsMap =
        HashMap<(crate::domain::ExamId, GradeId, SubjectId), (Vec<TeacherId>, Vec<TeacherId>)>;
    let mut panel_map: PanelAssignmentsMap = HashMap::new();

    for a in assignments {
        *count_map.entry(a.teacher_id).or_insert(0) += 1;
        match a.role {
            Role::Setter => *setter_map.entry(a.teacher_id).or_insert(0) += 1,
            Role::Reviewer => {
                *reviewer_map.entry(a.teacher_id).or_insert(0) += 1;
                *teacher_subject_reviews
                    .entry((a.teacher_id, a.subject_id))
                    .or_default() += 1;
            }
        }
        teacher_assigned_grades
            .entry(a.teacher_id)
            .or_default()
            .insert(a.grade_id);

        let entry = panel_map
            .entry((a.exam_id, a.grade_id, a.subject_id))
            .or_default();
        match a.role {
            Role::Setter => entry.0.push(a.teacher_id),
            Role::Reviewer => entry.1.push(a.teacher_id),
        }

        let forced = is_assignment_forced(a);
        if !forced {
            *non_forced_count_map.entry(a.teacher_id).or_insert(0) += 1;
            if a.role == Role::Reviewer {
                *non_forced_reviewer_map.entry(a.teacher_id).or_insert(0) += 1;
            }
            *non_forced_exam_tasks
                .entry((a.teacher_id, a.exam_id))
                .or_default() += 1;
            if a.role == Role::Setter {
                *non_forced_exam_setters
                    .entry((a.teacher_id, a.exam_id))
                    .or_default() += 1;
            }
        }
    }

    let mut violations = Vec::new();

    // Total non-forced seats and reviewer seats calculation
    let panels = problem.all_panels();
    let mut total_slots = 0usize;
    let mut total_reviewer_slots = 0usize;
    for p in &panels {
        if let Some(sub) = subject_map.get(&p.subject_id) {
            total_slots += (sub.setters + sub.reviewers) as usize;
            total_reviewer_slots += sub.reviewers as usize;
        }
    }
    let total_forced_seats: usize = forced_counts.values().sum();
    let total_forced_reviewer_seats: usize = forced_reviewer_counts.values().sum();
    let non_forced_total_seats = total_slots.saturating_sub(total_forced_seats);
    let non_forced_reviewer_seats =
        total_reviewer_slots.saturating_sub(total_forced_reviewer_seats);

    // -------------------------------------------------------------------------
    // S1: Reviewer Count
    // -------------------------------------------------------------------------
    let (s1_enabled, s1_weight) = get_rule_setting(RuleKey::S1, 10.0);
    let mut s1_units = 0.0;

    let reviewer_capable_teachers: Vec<&crate::domain::Teacher> = problem
        .teachers
        .iter()
        .filter(|t| {
            if !t.active || t.load_weight <= 0.0 {
                return false;
            }
            let q = quota_map.get(&t.id).copied().unwrap_or(0.0);
            if q < 1.0 {
                return false;
            }
            problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer)
        })
        .collect();

    let s1_setting = rule_settings_map.get(&RuleKey::S1);
    let max_reviews_cfg = s1_setting.and_then(|s| {
        s.params
            .get("max_reviews")
            .and_then(serde_json::Value::as_u64)
    });

    let auto_max_reviews = if !reviewer_capable_teachers.is_empty() {
        (non_forced_reviewer_seats as f64 / reviewer_capable_teachers.len() as f64).ceil() as usize
    } else {
        2
    };

    let max_reviews = max_reviews_cfg.map_or(auto_max_reviews, |v| v as usize);

    for t in &reviewer_capable_teachers {
        let q_t = quota_map.get(&t.id).copied().unwrap_or(0.0);
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
        } else if reviews > max_reviews {
            let excess = (reviews - max_reviews) as f64;
            s1_units += excess;
            let mut params = BTreeMap::new();
            params.insert("teacher".to_string(), t.full_name.clone());
            params.insert("count".to_string(), reviews.to_string());
            params.insert("max".to_string(), max_reviews.to_string());
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

    let rho = if non_forced_total_seats > 0 {
        non_forced_reviewer_seats as f64 / non_forced_total_seats as f64
    } else {
        1.0 / 3.0
    };

    for t in &problem.teachers {
        // Only evaluate teachers competent in both roles
        let has_setter_comp = problem
            .competencies
            .iter()
            .any(|c| c.teacher_id == t.id && c.role == Role::Setter);
        let has_reviewer_comp = problem
            .competencies
            .iter()
            .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer);

        if !has_setter_comp || !has_reviewer_comp {
            continue;
        }

        let c_prime = non_forced_count_map.get(&t.id).copied().unwrap_or(0);
        if c_prime >= 2 {
            let r_prime = non_forced_reviewer_map.get(&t.id).copied().unwrap_or(0);
            let target = (c_prime as f64) * rho;
            let lo = target.floor() as usize;
            let hi = target.ceil() as usize;
            let diff = if r_prime < lo {
                (lo - r_prime) as f64
            } else if r_prime > hi {
                (r_prime - hi) as f64
            } else {
                0.0
            };
            if diff > 1e-9 {
                s2_units += diff;
                let mut params = BTreeMap::new();
                params.insert("teacher".to_string(), t.full_name.clone());
                params.insert("reviews".to_string(), r_prime.to_string());
                params.insert("count".to_string(), c_prime.to_string());
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

    for (&(eid, gid, sid), (setters, reviewers)) in &panel_map {
        for r_id in reviewers {
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
                    panel: Some(PanelKey::new(eid, gid, sid)),
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
    for (setters, reviewers) in panel_map.values() {
        for r_id in reviewers {
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
    // S6: Consecutive Setting (Excluding forced setter presence)
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

                let is_non_forced_setter_e1 = non_forced_exam_setters
                    .get(&(t.id, e1.id))
                    .copied()
                    .unwrap_or(0)
                    > 0;
                let is_non_forced_setter_e2 = non_forced_exam_setters
                    .get(&(t.id, e2.id))
                    .copied()
                    .unwrap_or(0)
                    > 0;

                if is_non_forced_setter_e1 && is_non_forced_setter_e2 {
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
                let c_prime = non_forced_count_map.get(&t.id).copied().unwrap_or(0);
                let target = c_prime.min(qualified_grades.len());
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
    // S8: Load Balance (Non-forced teachers: (c' - q_t)^2)
    // -------------------------------------------------------------------------
    let (s8_enabled, s8_weight) = get_rule_setting(RuleKey::S8, 8.0);
    let mut s8_units = 0.0;

    for t in &problem.teachers {
        if t.quota_override.is_some() {
            continue;
        }
        let f_t = forced_counts.get(&t.id).copied().unwrap_or(0);
        let count_t = count_map.get(&t.id).copied().unwrap_or(0);
        let c_prime = count_t.saturating_sub(f_t);
        let q_t = quota_map.get(&t.id).copied().unwrap_or(0.0);
        let q_prime = (q_t - f_t as f64).max(0.0);

        // If teacher is active and has non-forced target
        if t.active && t.load_weight > 0.0 {
            let diff = c_prime as f64 - q_prime;
            let sq = diff * diff;
            if sq > 1e-9 {
                s8_units += sq;
                let mut params = BTreeMap::new();
                params.insert("teacher".to_string(), t.full_name.clone());
                params.insert("count".to_string(), c_prime.to_string());
                params.insert("quota".to_string(), format!("{q_prime:.2}"));
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
    }

    // -------------------------------------------------------------------------
    // S9: Exam Crowding
    // -------------------------------------------------------------------------
    let (s9_enabled, s9_weight) = get_rule_setting(RuleKey::S9, 5.0);
    let mut s9_units = 0.0;

    for (&(tid, eid), &tasks) in &non_forced_exam_tasks {
        if tasks > 1 {
            let excess = (tasks - 1) as f64;
            s9_units += excess;
            let tname = teacher_map
                .get(&tid)
                .map(|t| t.full_name.clone())
                .unwrap_or_default();
            let ecode = exam_map
                .get(&eid)
                .map(|e| e.code.clone())
                .unwrap_or_default();
            let mut params = BTreeMap::new();
            params.insert("teacher".to_string(), tname);
            params.insert("exam".to_string(), ecode);
            params.insert("count".to_string(), tasks.to_string());
            violations.push(SoftViolation {
                rule: RuleKey::S9,
                code: "exam_crowding".to_string(),
                panel: None,
                teachers: vec![tid],
                params,
            });
        }
    }

    // -------------------------------------------------------------------------
    // S10: Review Subject Missing
    // -------------------------------------------------------------------------
    let (s10_enabled, s10_weight) = get_rule_setting(RuleKey::S10, 4.0);
    let mut s10_units = 0.0;

    for t in &problem.teachers {
        if !t.active || t.load_weight <= 0.0 {
            continue;
        }
        let q_t = quota_map.get(&t.id).copied().unwrap_or(0.0);
        if q_t < 1.0 {
            continue;
        }

        for sub in &subjects {
            let is_competent = problem.competencies.iter().any(|c| {
                c.teacher_id == t.id && c.subject_id == sub.id && c.role == Role::Reviewer
            });
            if is_competent {
                let reviews = teacher_subject_reviews
                    .get(&(t.id, sub.id))
                    .copied()
                    .unwrap_or(0);
                if reviews == 0 {
                    s10_units += 1.0;
                    let mut params = BTreeMap::new();
                    params.insert("teacher".to_string(), t.full_name.clone());
                    params.insert("subject".to_string(), sub.name.clone());
                    params.insert("subject_code".to_string(), sub.code.clone());
                    violations.push(SoftViolation {
                        rule: RuleKey::S10,
                        code: "review_subject_missing".to_string(),
                        panel: None,
                        teachers: vec![t.id],
                        params,
                    });
                }
            }
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
                s1_units * s1_weight
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
                s2_units * s2_weight
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
                s3_units * s3_weight
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
                s4_units * s4_weight
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
                s5_units * s5_weight
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
                s6_units * s6_weight
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
                s7_units * s7_weight
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
                s8_units * s8_weight
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S8),
        },
        RuleScore {
            rule: RuleKey::S9,
            enabled: s9_enabled,
            weight: s9_weight,
            units: s9_units,
            penalty: if s9_enabled {
                s9_units * s9_weight
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S9),
        },
        RuleScore {
            rule: RuleKey::S10,
            enabled: s10_enabled,
            weight: s10_weight,
            units: s10_units,
            penalty: if s10_enabled {
                s10_units * s10_weight
            } else {
                0.0
            },
            lower_bound: get_lower_bound(RuleKey::S10),
        },
    ];

    let total: f64 = rule_scores.iter().map(|r| r.penalty).sum();

    // Per-teacher statistics
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
        Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, Role, RuleSetting,
        SchoolYear, SchoolYearId, Subject, SubjectId, Teacher, TeacherGrade,
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
        let mut competencies = Vec::new();
        for i in 1..=6 {
            let cid = if i <= 3 { CampusId(1) } else { CampusId(2) };
            teachers.push(Teacher {
                id: TeacherId(i),
                full_name: format!("Teacher {i}"),
                display_name: None,
                campus_id: cid,
                load_weight: 1.0,
                active: true,
                note: None,
                code: None,
                quota_override: None,
                max_tasks_per_exam_override: None,
            });
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
            competencies.push(Competency {
                teacher_id: TeacherId(i),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: TeacherId(i),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        Problem {
            school_year: sy,
            campuses,
            grades,
            subjects: vec![sub1],
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
    fn test_s1_reviewer_count_violations() {
        let problem = make_test_problem();
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(2),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Reviewer,
                0,
            ),
        ];

        let rep = evaluate(&problem, &assignments);
        let s1 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S1).unwrap();
        assert_eq!(s1.units, 6.0); // T2 has 3 reviews > ceil(4/6)=1 -> excess 2; T1, T4, T5, T6 have 0 reviews -> 4; total 6
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
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
        ];

        let rep = evaluate(&problem, &assignments);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(1)]));
        assert!(!rep
            .violations
            .iter()
            .any(|v| v.code == "role_imbalance" && v.teachers == vec![TeacherId(2)]));

        let s2 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S2).unwrap();
        assert_eq!(s2.units, 3.0);
    }

    #[test]
    fn test_lower_bounds_calculation() {
        let problem = make_test_problem();
        let bounds = lower_bounds(&problem);
        assert_eq!(bounds.len(), 10);
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
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Reviewer,
                0,
            ),
            // other panels cleanly segregated
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(1),
                Role::Reviewer,
                0,
            ),
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
                && v.panel == Some(PanelKey::new(ExamId(1), GradeId(1), SubjectId(1)))));
    }

    #[test]
    fn test_s4_setter_pair_repeated() {
        let problem = make_test_problem();
        // T1 and T2 pair up as setters in E1 G1 and in E2 G1 -> 1 excess unit
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
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
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
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
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
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
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
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
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(1),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(5),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(3),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(1),
                SubjectId(1),
                TeacherId(5),
                Role::Reviewer,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(4),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(2),
                GradeId(2),
                SubjectId(1),
                TeacherId(6),
                Role::Reviewer,
                0,
            ),
        ];

        let rep = evaluate(&problem, &assignments);
        let s8 = rep.by_rule.iter().find(|r| r.rule == RuleKey::S8).unwrap();
        // T1 has count = 3: (3 - 2)^2 = 1
        // T2 has count = 1: (1 - 2)^2 = 1
        // T3..T6: 0
        // Total S8 units = 2.0
        assert_eq!(s8.units, 2.0);
        assert!(rep
            .violations
            .iter()
            .any(|v| v.code == "load_deviation" && v.teachers == vec![TeacherId(1)]));
    }
}
