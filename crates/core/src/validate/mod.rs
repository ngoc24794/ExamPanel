//! Hard-constraint plan validator.
//!
//! Validates candidate assignment sets against hard constraints H1–H7.
//! Supports both complete plans and partial plans during manual construction.

use crate::domain::{
    calculate_quotas, is_teacher_qualified, Assignment, LockKind, PanelKey, Problem, Role, RuleKey,
    TeacherId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Validation options controlling strictness and completion requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateOptions {
    /// If true, lower bound checks (panel completeness, quota lower bounds, all PINs placed)
    /// are strictly enforced. If false, only actively violated constraints are reported.
    pub require_complete: bool,
}

impl Default for ValidateOptions {
    fn default() -> Self {
        Self {
            require_complete: true,
        }
    }
}

/// A specific hard constraint violation found in an assignment set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Violation {
    pub rule: RuleKey,
    pub code: String,
    #[serde(default)]
    #[ts(optional)]
    pub panel: Option<PanelKey>,
    #[serde(default)]
    #[ts(optional)]
    pub teacher: Option<TeacherId>,
    #[ts(type = "Record<string, unknown>")]
    pub params: BTreeMap<String, serde_json::Value>,
}

impl Violation {
    #[must_use]
    pub fn new(rule: RuleKey, code: impl Into<String>) -> Self {
        Self {
            rule,
            code: code.into(),
            panel: None,
            teacher: None,
            params: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_panel(mut self, panel: PanelKey) -> Self {
        self.panel = Some(panel);
        self
    }

    #[must_use]
    pub fn with_teacher(mut self, teacher: TeacherId) -> Self {
        self.teacher = Some(teacher);
        self
    }

    #[must_use]
    pub fn with_param<V: Into<serde_json::Value>>(
        mut self,
        key: impl Into<String>,
        val: V,
    ) -> Self {
        self.params.insert(key.into(), val.into());
        self
    }
}

/// Validates an assignment set against all hard constraints (H1–H7) for the given problem snapshot.
#[must_use]
pub fn validate_assignments(
    problem: &Problem,
    assignments: &[Assignment],
    opts: &ValidateOptions,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    let teacher_map: HashMap<TeacherId, &crate::domain::Teacher> =
        problem.teachers.iter().map(|t| (t.id, t)).collect();

    let unavailability_set: HashSet<(TeacherId, crate::domain::ExamId)> = problem
        .unavailabilities
        .iter()
        .map(|u| (u.teacher_id, u.exam_id))
        .collect();

    let subjects = problem.effective_subjects();
    let subject_map: HashMap<crate::domain::SubjectId, &crate::domain::Subject> =
        subjects.iter().map(|s| (s.id, s)).collect();

    let h3_setting = problem.rule_settings.iter().find(|s| s.key == RuleKey::H3);
    let h3_enabled = h3_setting.is_none_or(|s| s.enabled);

    let h4_setting = problem.rule_settings.iter().find(|s| s.key == RuleKey::H4);
    let h4_enabled = h4_setting.is_none_or(|s| s.enabled);
    let default_max_tasks_per_exam = h4_setting
        .and_then(|s| {
            s.params
                .get("max_tasks_per_exam")
                .and_then(serde_json::Value::as_u64)
        })
        .map_or(2, |v| v as usize);
    let default_max_setter_per_exam = h4_setting
        .and_then(|s| {
            s.params
                .get("max_setter_per_exam")
                .and_then(serde_json::Value::as_u64)
        })
        .map_or(1, |v| v as usize);

    let forced_placements =
        crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let mut forced_tasks_map: HashMap<(TeacherId, crate::domain::ExamId), usize> = HashMap::new();
    let mut forced_setters_map: HashMap<(TeacherId, crate::domain::ExamId), usize> = HashMap::new();
    for p in &forced_placements {
        *forced_tasks_map
            .entry((p.teacher_id, p.panel.exam_id))
            .or_default() += 1;
        if p.role == Role::Setter {
            *forced_setters_map
                .entry((p.teacher_id, p.panel.exam_id))
                .or_default() += 1;
        }
    }

    // Group assignments by panel
    let mut panel_assignments: HashMap<PanelKey, Vec<&Assignment>> = HashMap::new();
    for a in assignments {
        let key = PanelKey::new(a.exam_id, a.grade_id, a.subject_id);
        panel_assignments.entry(key).or_default().push(a);
    }

    // All valid panels defined by the problem
    let all_panels = problem.all_panels();

    // -------------------------------------------------------------------------
    // H1: Panel Composition & Distinct Teachers
    // H2: Grade / Subject Qualification
    // H3: Multi-Campus Diversity
    // -------------------------------------------------------------------------
    for panel in &all_panels {
        let list = panel_assignments.get(panel).map_or(&[][..], |v| &v[..]);
        let subject = match subject_map.get(&panel.subject_id) {
            Some(s) => s,
            None => continue,
        };

        let mut setters_count = 0usize;
        let mut reviewers_count = 0usize;
        let mut seen_teachers: HashSet<TeacherId> = HashSet::new();
        let mut campus_ids: HashSet<crate::domain::CampusId> = HashSet::new();

        for a in list {
            match a.role {
                Role::Setter => setters_count += 1,
                Role::Reviewer => reviewers_count += 1,
            }

            // Distinct teachers per panel
            if !seen_teachers.insert(a.teacher_id) {
                violations.push(
                    Violation::new(RuleKey::H1, "duplicate_teacher_in_panel")
                        .with_panel(*panel)
                        .with_teacher(a.teacher_id),
                );
            }

            // Teacher qualification & status (H2)
            if let Some(teacher) = teacher_map.get(&a.teacher_id) {
                if !teacher.active {
                    violations.push(
                        Violation::new(RuleKey::H2, "inactive_teacher")
                            .with_panel(*panel)
                            .with_teacher(a.teacher_id),
                    );
                }
                if teacher.load_weight <= 0.0 {
                    violations.push(
                        Violation::new(RuleKey::H2, "zero_weight_teacher")
                            .with_panel(*panel)
                            .with_teacher(a.teacher_id),
                    );
                }
                if !is_teacher_qualified(problem, a.teacher_id, a.grade_id, a.subject_id, a.role) {
                    violations.push(
                        Violation::new(RuleKey::H2, "unqualified_grade")
                            .with_panel(*panel)
                            .with_teacher(a.teacher_id)
                            .with_param("grade_id", a.grade_id.value())
                            .with_param("subject_id", a.subject_id.0),
                    );
                }
                campus_ids.insert(teacher.campus_id);
            }
        }

        // Excess roles (reported in both partial and complete modes)
        let expected_setters = subject.setters as usize;
        let expected_reviewers = subject.reviewers as usize;

        if setters_count > expected_setters {
            violations.push(
                Violation::new(RuleKey::H1, "excess_setters")
                    .with_panel(*panel)
                    .with_param("count", setters_count)
                    .with_param("limit", expected_setters),
            );
        }
        if reviewers_count > expected_reviewers {
            violations.push(
                Violation::new(RuleKey::H1, "excess_reviewers")
                    .with_panel(*panel)
                    .with_param("count", reviewers_count)
                    .with_param("limit", expected_reviewers),
            );
        }

        // Completeness check (only when require_complete)
        let expected_total = expected_setters + expected_reviewers;
        if opts.require_complete
            && (setters_count != expected_setters
                || reviewers_count != expected_reviewers
                || list.len() != expected_total)
        {
            violations.push(
                Violation::new(RuleKey::H1, "panel_incomplete")
                    .with_panel(*panel)
                    .with_param("setters", setters_count)
                    .with_param("reviewers", reviewers_count)
                    .with_param("total", list.len()),
            );
        }

        // Multi-campus diversity (H3)
        let min_campuses = subject.min_campuses as usize;
        if h3_enabled
            && min_campuses > 0
            && (list.len() == expected_total || (opts.require_complete && !list.is_empty()))
            && campus_ids.len() < min_campuses
        {
            violations.push(
                Violation::new(RuleKey::H3, "single_campus_panel")
                    .with_panel(*panel)
                    .with_param("campus_count", campus_ids.len())
                    .with_param("min_campuses", min_campuses),
            );
        }
    }

    // -------------------------------------------------------------------------
    // H4: Per-exam task limits and setter limits
    // H5: Exam Availability
    // -------------------------------------------------------------------------
    let mut exam_teacher_tasks: HashMap<(crate::domain::ExamId, TeacherId), usize> = HashMap::new();
    let mut exam_teacher_setters: HashMap<(crate::domain::ExamId, TeacherId), usize> =
        HashMap::new();

    for a in assignments {
        let pkey = PanelKey::new(a.exam_id, a.grade_id, a.subject_id);
        *exam_teacher_tasks
            .entry((a.exam_id, a.teacher_id))
            .or_default() += 1;
        if a.role == Role::Setter {
            *exam_teacher_setters
                .entry((a.exam_id, a.teacher_id))
                .or_default() += 1;
        }

        // H5: Unavailability
        if unavailability_set.contains(&(a.teacher_id, a.exam_id)) {
            violations.push(
                Violation::new(RuleKey::H5, "teacher_unavailable")
                    .with_panel(pkey)
                    .with_teacher(a.teacher_id)
                    .with_param("exam_id", a.exam_id.value()),
            );
        }
    }

    if h4_enabled {
        for ((exam_id, teacher_id), count) in exam_teacher_tasks {
            let teacher = teacher_map.get(&teacher_id);
            let configured_tasks = teacher
                .and_then(|t| t.max_tasks_per_exam_override)
                .map_or(default_max_tasks_per_exam, |v| v as usize);
            let forced_tasks = forced_tasks_map
                .get(&(teacher_id, exam_id))
                .copied()
                .unwrap_or(0);
            let eff_max_tasks = configured_tasks.max(forced_tasks);

            if count > eff_max_tasks {
                let code = if eff_max_tasks == 1 {
                    "multiple_panels_in_exam"
                } else {
                    "max_tasks_per_exam_exceeded"
                };
                violations.push(
                    Violation::new(RuleKey::H4, code)
                        .with_teacher(teacher_id)
                        .with_param("exam_id", exam_id.value())
                        .with_param("count", count)
                        .with_param("limit", eff_max_tasks),
                );
            }
        }

        for ((exam_id, teacher_id), setter_count) in exam_teacher_setters {
            let forced_setters = forced_setters_map
                .get(&(teacher_id, exam_id))
                .copied()
                .unwrap_or(0);
            let eff_max_setters = default_max_setter_per_exam.max(forced_setters);

            if setter_count > eff_max_setters {
                violations.push(
                    Violation::new(RuleKey::H4, "max_setter_tasks_per_exam_exceeded")
                        .with_teacher(teacher_id)
                        .with_param("exam_id", exam_id.value())
                        .with_param("count", setter_count)
                        .with_param("limit", eff_max_setters),
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // H6: Lock Compliance
    // -------------------------------------------------------------------------
    let mut pin_satisfied: HashMap<crate::domain::LockId, bool> = HashMap::new();

    for lock in &problem.locks {
        match lock.kind {
            LockKind::Forbid => {
                let matching = assignments.iter().any(|a| {
                    a.exam_id == lock.exam_id
                        && a.grade_id == lock.grade_id
                        && a.subject_id == lock.subject_id
                        && a.teacher_id == lock.teacher_id
                        && (lock.role.is_none() || lock.role == Some(a.role))
                });
                if matching {
                    violations.push(
                        Violation::new(RuleKey::H6, "forbid_violation")
                            .with_panel(PanelKey::new(lock.exam_id, lock.grade_id, lock.subject_id))
                            .with_teacher(lock.teacher_id),
                    );
                }
            }
            LockKind::Pin => {
                pin_satisfied.insert(lock.id, false);

                // Check if teacher is assigned to this panel with wrong role in partial mode
                if let Some(existing) = assignments.iter().find(|a| {
                    a.exam_id == lock.exam_id
                        && a.grade_id == lock.grade_id
                        && a.subject_id == lock.subject_id
                        && a.teacher_id == lock.teacher_id
                }) {
                    if let Some(pinned_role) = lock.role {
                        if existing.role != pinned_role {
                            violations.push(
                                Violation::new(RuleKey::H6, "pin_role_mismatch")
                                    .with_panel(PanelKey::new(
                                        lock.exam_id,
                                        lock.grade_id,
                                        lock.subject_id,
                                    ))
                                    .with_teacher(lock.teacher_id)
                                    .with_param("expected_role", pinned_role.as_str())
                                    .with_param("actual_role", existing.role.as_str()),
                            );
                        }
                    }
                }

                // Check satisfaction
                let satisfied = assignments.iter().any(|a| {
                    a.exam_id == lock.exam_id
                        && a.grade_id == lock.grade_id
                        && a.subject_id == lock.subject_id
                        && a.teacher_id == lock.teacher_id
                        && (lock.role.is_none() || lock.role == Some(a.role))
                });
                if satisfied {
                    pin_satisfied.insert(lock.id, true);
                }
            }
        }
    }

    if opts.require_complete {
        for lock in &problem.locks {
            if lock.kind == LockKind::Pin && !pin_satisfied.get(&lock.id).copied().unwrap_or(false)
            {
                violations.push(
                    Violation::new(RuleKey::H6, "pin_missing")
                        .with_panel(PanelKey::new(lock.exam_id, lock.grade_id, lock.subject_id))
                        .with_teacher(lock.teacher_id),
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // H7: Workload Quota
    // -------------------------------------------------------------------------
    let quotas = calculate_quotas(problem);
    let mut teacher_assignment_counts: HashMap<TeacherId, usize> = HashMap::new();
    for a in assignments {
        *teacher_assignment_counts.entry(a.teacher_id).or_default() += 1;
    }

    for tq in quotas {
        let count = teacher_assignment_counts
            .get(&tq.teacher_id)
            .copied()
            .unwrap_or(0);

        if count > tq.hi {
            violations.push(
                Violation::new(RuleKey::H7, "quota_exceeded")
                    .with_teacher(tq.teacher_id)
                    .with_param("count", count)
                    .with_param("hi", tq.hi),
            );
        }

        if opts.require_complete && count < tq.lo {
            violations.push(
                Violation::new(RuleKey::H7, "quota_unmet")
                    .with_teacher(tq.teacher_id)
                    .with_param("count", count)
                    .with_param("lo", tq.lo),
            );
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, Lock, LockId, Role,
        SchoolYear, SchoolYearId, Subject, SubjectId, Teacher, TeacherGrade, Unavailability,
    };

    fn make_test_problem() -> Problem {
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let c1 = Campus {
            id: CampusId(1),
            code: "CS1".to_string(),
            name: "Campus 1".to_string(),
            color: "#ff0000".to_string(),
        };
        let c2 = Campus {
            id: CampusId(2),
            code: "CS2".to_string(),
            name: "Campus 2".to_string(),
            color: "#00ff00".to_string(),
        };
        let g10 = Grade {
            id: GradeId(10),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
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
        let e1 = Exam {
            id: ExamId(1),
            school_year_id: sy.id,
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };

        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "Teacher 1".to_string(),
            display_name: None,
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let t2 = Teacher {
            id: TeacherId(2),
            full_name: "Teacher 2".to_string(),
            display_name: None,
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let t3 = Teacher {
            id: TeacherId(3),
            full_name: "Teacher 3".to_string(),
            display_name: None,
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };

        let tg1 = TeacherGrade {
            teacher_id: TeacherId(1),
            school_year_id: sy.id,
            grade_id: GradeId(10),
        };
        let tg2 = TeacherGrade {
            teacher_id: TeacherId(2),
            school_year_id: sy.id,
            grade_id: GradeId(10),
        };
        let tg3 = TeacherGrade {
            teacher_id: TeacherId(3),
            school_year_id: sy.id,
            grade_id: GradeId(10),
        };

        let comps = vec![
            Competency {
                teacher_id: TeacherId(1),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: TeacherId(1),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: TeacherId(2),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: TeacherId(2),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: TeacherId(3),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: TeacherId(3),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            },
        ];

        Problem {
            school_year: sy,
            campuses: vec![c1, c2],
            grades: vec![g10],
            subjects: vec![sub1],
            exams: vec![e1],
            teachers: vec![t1, t2, t3],
            teacher_grades: vec![tg1, tg2, tg3],
            competencies: comps,
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: crate::domain::RuleSetting::default_settings(),
        }
    }

    #[test]
    fn test_valid_complete_assignment() {
        let problem = make_test_problem();
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(3),
                Role::Reviewer,
                0,
            ),
        ];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(
            violations.is_empty(),
            "expected 0 violations: {violations:?}"
        );
    }

    #[test]
    fn test_partial_mode_permits_incomplete_panel() {
        let problem = make_test_problem();
        let assignments = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(1),
            Role::Setter,
            0,
        )];

        let violations_partial = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations_partial.is_empty());

        let violations_complete = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(violations_complete
            .iter()
            .any(|v| v.code == "panel_incomplete"));
    }

    #[test]
    fn test_h1_duplicate_teacher_in_panel() {
        let problem = make_test_problem();
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Reviewer,
                0,
            ),
        ];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations
            .iter()
            .any(|v| v.code == "duplicate_teacher_in_panel"));
    }

    #[test]
    fn test_h2_unqualified_grade() {
        let mut problem = make_test_problem();
        // Remove teacher 3's grade qualification
        problem
            .teacher_grades
            .retain(|tg| tg.teacher_id != TeacherId(3));

        let assignments = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(3),
            Role::Reviewer,
            0,
        )];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations.iter().any(|v| v.code == "unqualified_grade"));
    }

    #[test]
    fn test_h2_inactive_and_zero_weight() {
        let mut problem = make_test_problem();
        problem.teachers[0].active = false;
        problem.teachers[1].load_weight = 0.0;

        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
        ];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations.iter().any(|v| v.code == "inactive_teacher"));
        assert!(violations.iter().any(|v| v.code == "zero_weight_teacher"));
    }

    #[test]
    fn test_h3_single_campus_panel() {
        let problem = make_test_problem();
        let mut p = problem;
        p.teachers.push(Teacher {
            id: TeacherId(4),
            full_name: "Teacher 4".to_string(),
            display_name: None,
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        });
        p.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(4),
            school_year_id: p.school_year.id,
            grade_id: GradeId(10),
        });
        p.competencies.push(Competency {
            teacher_id: TeacherId(4),
            subject_id: SubjectId(1),
            role: Role::Reviewer,
            grade_scope: GradeScope::Taught,
        });

        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(2),
                Role::Setter,
                1,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(4),
                Role::Reviewer,
                0,
            ),
        ];

        let violations = validate_assignments(
            &p,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(violations.iter().any(|v| v.code == "single_campus_panel"));
    }

    #[test]
    fn test_h4_multiple_panels_in_same_exam() {
        let mut problem = make_test_problem();
        let g11 = Grade {
            id: GradeId(11),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        };
        problem.grades.push(g11);
        problem.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(1),
            school_year_id: problem.school_year.id,
            grade_id: GradeId(11),
        });
        // Set H4 max_tasks_per_exam to 1 so 2 tasks triggers violation
        if let Some(s) = problem
            .rule_settings
            .iter_mut()
            .find(|s| s.key == RuleKey::H4)
        {
            s.params["max_tasks_per_exam"] = serde_json::json!(1);
        }

        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(11),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
        ];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations.iter().any(
            |v| v.code == "multiple_panels_in_exam" || v.code == "max_tasks_per_exam_exceeded"
        ));
    }

    #[test]
    fn test_h5_teacher_unavailable() {
        let mut problem = make_test_problem();
        problem.unavailabilities.push(Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: Some("Leave".to_string()),
        });

        let assignments = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(1),
            Role::Setter,
            0,
        )];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations.iter().any(|v| v.code == "teacher_unavailable"));
    }

    #[test]
    fn test_ra023_absent_teacher_reports_only_h5_not_h2() {
        let mut problem = make_test_problem();
        let baseline_assignments = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(1),
            Role::Setter,
            0,
        )];
        let opts = ValidateOptions {
            require_complete: false,
        };
        let base = validate_assignments(&problem, &baseline_assignments, &opts);
        assert!(
            !base.iter().any(|v| v.rule == RuleKey::H2),
            "baseline has no H2: {base:?}"
        );

        problem.unavailabilities.push(Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: None,
        });
        let violations = validate_assignments(&problem, &baseline_assignments, &opts);
        assert!(
            violations.iter().any(|v| v.code == "teacher_unavailable"),
            "H5 expected: {violations:?}"
        );
        assert!(
            !violations.iter().any(|v| v.rule == RuleKey::H2),
            "absence must not be reported as H2 unqualified_grade: {violations:?}"
        );
    }

    #[test]
    fn test_h6_forbid_and_pin_locks() {
        let mut problem = make_test_problem();
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            subject_id: SubjectId(1),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Forbid,
        });
        problem.locks.push(Lock {
            id: LockId(2),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            subject_id: SubjectId(1),
            teacher_id: TeacherId(2),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });

        // 1. Violate FORBID and miss PIN in complete mode
        let assignments = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(1),
            Role::Setter,
            0,
        )];
        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(violations.iter().any(|v| v.code == "forbid_violation"));
        assert!(violations.iter().any(|v| v.code == "pin_missing"));

        // 2. Role mismatch for PIN
        let assignments_mismatch = vec![Assignment::new(
            ExamId(1),
            GradeId(10),
            SubjectId(1),
            TeacherId(2),
            Role::Reviewer,
            0,
        )];
        let violations_mismatch = validate_assignments(
            &problem,
            &assignments_mismatch,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations_mismatch
            .iter()
            .any(|v| v.code == "pin_role_mismatch"));
    }

    #[test]
    fn test_h7_quota_exceeded_and_unmet() {
        let mut problem = make_test_problem();
        if let Some(s) = problem
            .rule_settings
            .iter_mut()
            .find(|s| s.key == RuleKey::H4)
        {
            s.params["max_tasks_per_exam"] = serde_json::json!(1);
        }
        let assignments = vec![
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Setter,
                0,
            ),
            Assignment::new(
                ExamId(1),
                GradeId(10),
                SubjectId(1),
                TeacherId(1),
                Role::Reviewer,
                0,
            ),
        ];

        let violations = validate_assignments(
            &problem,
            &assignments,
            &ValidateOptions {
                require_complete: false,
            },
        );
        assert!(violations.iter().any(|v| v.code == "quota_exceeded"));
    }
}
