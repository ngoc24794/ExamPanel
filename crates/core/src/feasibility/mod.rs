//! Pre-solve feasibility checks and diagnostic reporting.
//!
//! Validates whether input data (teacher rosters, qualifications, locks, quotas)
//! mathematically permits a valid assignment plan before launching the solver.
//!
//! Note: These checks are NECESSARY conditions; passing them does not guarantee
//! that a solution exists, but failing any error check guarantees infeasibility.

pub mod maxflow;

use crate::domain::{
    calculate_quotas, CampusId, ExamId, GradeId, LockKind, PanelKey, Problem, Role, RuleKey,
    TeacherId, TeacherQuota,
};
use maxflow::DinicGraph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// A structured diagnostic report item identifying an infeasibility or warning condition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub panel: Option<PanelKey>,
    pub teacher: Option<TeacherId>,
    pub params: BTreeMap<String, serde_json::Value>,
}

impl<'de> Deserialize<'de> for Diagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DiagnosticHelper {
            code: String,
            panel: Option<PanelKey>,
            teacher: Option<TeacherId>,
            params: BTreeMap<String, serde_json::Value>,
        }

        let helper = DiagnosticHelper::deserialize(deserializer)?;
        Ok(Self {
            code: Box::leak(helper.code.into_boxed_str()),
            panel: helper.panel,
            teacher: helper.teacher,
            params: helper.params,
        })
    }
}

impl Diagnostic {
    #[must_use]
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
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

/// The outcome of the pre-solve feasibility verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeasibilityReport {
    /// Blocking errors that make finding a valid schedule mathematically impossible.
    pub errors: Vec<Diagnostic>,
    /// Non-blocking warnings indicating tight bottlenecks or potential quality issues.
    pub warnings: Vec<Diagnostic>,
    /// Calculated workload quota targets and bounds for all teachers.
    pub quotas: Vec<TeacherQuota>,
}

impl FeasibilityReport {
    /// Returns true if there are zero blocking errors.
    #[must_use]
    pub fn is_feasible(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Runs all pre-solve feasibility checks on the given problem snapshot.
#[must_use]
pub fn check_feasibility(problem: &Problem) -> FeasibilityReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // -------------------------------------------------------------------------
    // F1: Structural integrity
    // -------------------------------------------------------------------------
    let structural_errors = problem.validate();
    for err in structural_errors {
        errors.push(Diagnostic::new("structural_error").with_param("message", err.to_string()));
    }

    // -------------------------------------------------------------------------
    // Roster and Map Lookups
    // -------------------------------------------------------------------------
    let teacher_map: HashMap<TeacherId, &crate::domain::Teacher> =
        problem.teachers.iter().map(|t| (t.id, t)).collect();

    let exam_map: HashMap<ExamId, &crate::domain::Exam> =
        problem.exams.iter().map(|e| (e.id, e)).collect();

    let grade_map: HashMap<GradeId, &crate::domain::Grade> =
        problem.grades.iter().map(|g| (g.id, g)).collect();

    let teacher_grades_set: HashSet<(TeacherId, GradeId)> = problem
        .teacher_grades
        .iter()
        .filter(|tg| tg.school_year_id == problem.school_year.id)
        .map(|tg| (tg.teacher_id, tg.grade_id))
        .collect();

    let unavailability_set: HashSet<(TeacherId, ExamId)> = problem
        .unavailabilities
        .iter()
        .map(|u| (u.teacher_id, u.exam_id))
        .collect();

    let h4_enabled = problem
        .rule_settings
        .iter()
        .find(|s| s.key == RuleKey::H4)
        .is_none_or(|s| s.enabled);

    // Compute quotas (H7)
    let quotas = calculate_quotas(problem);
    let quota_map: HashMap<TeacherId, &TeacherQuota> =
        quotas.iter().map(|q| (q.teacher_id, q)).collect();

    // Organize locks
    let mut forbid_any: HashSet<(ExamId, GradeId, TeacherId)> = HashSet::new();
    let mut forbid_role: HashMap<(ExamId, GradeId, TeacherId), HashSet<Role>> = HashMap::new();
    let mut pin_locks: Vec<&crate::domain::Lock> = Vec::new();
    let mut forbid_locks: Vec<&crate::domain::Lock> = Vec::new();

    for lock in &problem.locks {
        match lock.kind {
            LockKind::Forbid => {
                forbid_locks.push(lock);
                let key = (lock.exam_id, lock.grade_id, lock.teacher_id);
                match lock.role {
                    None => {
                        forbid_any.insert(key);
                    }
                    Some(r) => {
                        forbid_role.entry(key).or_default().insert(r);
                    }
                }
            }
            LockKind::Pin => {
                pin_locks.push(lock);
            }
        }
    }

    // Eligibility checker for teacher t on panel (e, g) in role r
    let is_eligible = |t_id: TeacherId, e_id: ExamId, g_id: GradeId, role: Role| -> bool {
        let t = match teacher_map.get(&t_id) {
            Some(teacher) if teacher.active && teacher.load_weight > 0.0 => teacher,
            _ => return false,
        };
        if !teacher_grades_set.contains(&(t.id, g_id)) {
            return false;
        }
        if unavailability_set.contains(&(t.id, e_id)) {
            return false;
        }
        let fkey = (e_id, g_id, t.id);
        if forbid_any.contains(&fkey) {
            return false;
        }
        if let Some(roles) = forbid_role.get(&fkey) {
            if roles.contains(&role) {
                return false;
            }
        }
        true
    };

    // -------------------------------------------------------------------------
    // F2: Per-panel adequacy
    // -------------------------------------------------------------------------
    let mut teacher_eligible_panel_count: HashMap<TeacherId, usize> = HashMap::new();
    let mut reviewer_eligible_teachers: HashSet<TeacherId> = HashSet::new();

    for exam in &problem.exams {
        for grade in &problem.grades {
            let pkey = PanelKey::new(exam.id, grade.id);
            let mut setter_count = 0usize;
            let mut reviewer_count = 0usize;
            let mut distinct_eligible: HashSet<TeacherId> = HashSet::new();
            let mut campus_set: HashSet<CampusId> = HashSet::new();

            for teacher in &problem.teachers {
                let e_setter = is_eligible(teacher.id, exam.id, grade.id, Role::Setter);
                let e_reviewer = is_eligible(teacher.id, exam.id, grade.id, Role::Reviewer);

                if e_setter {
                    setter_count += 1;
                }
                if e_reviewer {
                    reviewer_count += 1;
                    reviewer_eligible_teachers.insert(teacher.id);
                }
                if e_setter || e_reviewer {
                    distinct_eligible.insert(teacher.id);
                    campus_set.insert(teacher.campus_id);
                    *teacher_eligible_panel_count.entry(teacher.id).or_default() += 1;
                }
            }

            if setter_count < 2 {
                errors.push(
                    Diagnostic::new("insufficient_setters")
                        .with_panel(pkey)
                        .with_param("exam", exam.code.clone())
                        .with_param("grade", grade.code)
                        .with_param("count", setter_count),
                );
            }

            if reviewer_count < 1 {
                errors.push(
                    Diagnostic::new("insufficient_reviewers")
                        .with_panel(pkey)
                        .with_param("exam", exam.code.clone())
                        .with_param("grade", grade.code)
                        .with_param("count", reviewer_count),
                );
            }

            if distinct_eligible.len() < 3 {
                errors.push(
                    Diagnostic::new("insufficient_panel_teachers")
                        .with_panel(pkey)
                        .with_param("exam", exam.code.clone())
                        .with_param("grade", grade.code)
                        .with_param("count", distinct_eligible.len()),
                );
            }

            if campus_set.len() < 2 {
                errors.push(
                    Diagnostic::new("insufficient_campuses")
                        .with_panel(pkey)
                        .with_param("exam", exam.code.clone())
                        .with_param("grade", grade.code)
                        .with_param("count", campus_set.len()),
                );
            }

            // Warning: tight panel roster
            if distinct_eligible.len() == 3 {
                warnings.push(
                    Diagnostic::new("tight_panel_roster")
                        .with_panel(pkey)
                        .with_param("exam", exam.code.clone())
                        .with_param("grade", grade.code)
                        .with_param("count", 3),
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // F3: Locks validation
    // -------------------------------------------------------------------------
    let mut panel_pins: HashMap<PanelKey, Vec<&crate::domain::Lock>> = HashMap::new();
    let mut teacher_pins: HashMap<TeacherId, Vec<&crate::domain::Lock>> = HashMap::new();
    let mut exam_teacher_pins: HashMap<(ExamId, TeacherId), Vec<PanelKey>> = HashMap::new();

    for pin in &pin_locks {
        let pkey = PanelKey::new(pin.exam_id, pin.grade_id);
        panel_pins.entry(pkey).or_default().push(pin);
        teacher_pins.entry(pin.teacher_id).or_default().push(pin);
        exam_teacher_pins
            .entry((pin.exam_id, pin.teacher_id))
            .or_default()
            .push(pkey);

        let exam_code = exam_map.get(&pin.exam_id).map_or("", |e| e.code.as_str());
        let grade_code = grade_map.get(&pin.grade_id).map_or(0, |g| g.code);

        // Check PIN+FORBID conflict
        let fkey = (pin.exam_id, pin.grade_id, pin.teacher_id);
        let has_conflict = forbid_any.contains(&fkey)
            || forbid_role.get(&fkey).is_some_and(|roles| {
                pin.role.is_none() || pin.role.is_some_and(|r| roles.contains(&r))
            });

        if has_conflict {
            errors.push(
                Diagnostic::new("lock_conflict")
                    .with_panel(pkey)
                    .with_teacher(pin.teacher_id)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code),
            );
        }

        // Pinned teacher ineligible
        let teacher = teacher_map.get(&pin.teacher_id);
        let ineligible = match teacher {
            None => true,
            Some(t) => {
                !t.active
                    || t.load_weight <= 0.0
                    || !teacher_grades_set.contains(&(t.id, pin.grade_id))
                    || unavailability_set.contains(&(t.id, pin.exam_id))
            }
        };

        if ineligible {
            errors.push(
                Diagnostic::new("pinned_teacher_ineligible")
                    .with_panel(pkey)
                    .with_teacher(pin.teacher_id)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code),
            );
        }
    }

    // Per-panel PIN composition checks
    for (pkey, pins) in &panel_pins {
        let exam_code = exam_map.get(&pkey.exam_id).map_or("", |e| e.code.as_str());
        let grade_code = grade_map.get(&pkey.grade_id).map_or(0, |g| g.code);

        let mut pinned_setters = 0usize;
        let mut pinned_reviewers = 0usize;
        let mut pinned_campuses: HashSet<CampusId> = HashSet::new();

        for pin in pins {
            match pin.role {
                Some(Role::Setter) => pinned_setters += 1,
                Some(Role::Reviewer) => pinned_reviewers += 1,
                None => {}
            }
            if let Some(t) = teacher_map.get(&pin.teacher_id) {
                pinned_campuses.insert(t.campus_id);
            }
        }

        if pinned_setters > 2 {
            errors.push(
                Diagnostic::new("excess_pinned_setters")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pinned_setters),
            );
        }

        if pinned_reviewers > 1 {
            errors.push(
                Diagnostic::new("excess_pinned_reviewers")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pinned_reviewers),
            );
        }

        if pins.len() > 3 {
            errors.push(
                Diagnostic::new("excess_pins_in_panel")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pins.len()),
            );
        }

        // 3 pins in a panel all from one campus
        if pins.len() == 3 && pinned_campuses.len() == 1 {
            errors.push(
                Diagnostic::new("pinned_campus_monopoly")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code),
            );
        }
    }

    // Teacher pinned across multiple panels of the same exam under H4
    if h4_enabled {
        for ((exam_id, teacher_id), pkeys) in &exam_teacher_pins {
            if pkeys.len() > 1 {
                let exam_code = exam_map.get(exam_id).map_or("", |e| e.code.as_str());
                errors.push(
                    Diagnostic::new("pinned_teacher_multiple_panels")
                        .with_teacher(*teacher_id)
                        .with_param("exam", exam_code)
                        .with_param("count", pkeys.len()),
                );
            }
        }
    }

    // Teacher pins exceeding hi_t
    for (t_id, pins) in &teacher_pins {
        let hi = quota_map.get(t_id).map_or(0, |q| q.hi);
        if pins.len() > hi {
            errors.push(
                Diagnostic::new("pinned_quota_exceeded")
                    .with_teacher(*t_id)
                    .with_param("count", pins.len())
                    .with_param("hi", hi),
            );
        }
    }

    // -------------------------------------------------------------------------
    // F4: Capacity & Bipartite Max-Flow
    // -------------------------------------------------------------------------
    let total_slots = problem.exams.len() * problem.grades.len() * 3;
    let sum_lo: usize = quotas.iter().map(|q| q.lo).sum();
    let sum_hi: usize = quotas.iter().map(|q| q.hi).sum();

    if sum_hi < total_slots {
        errors.push(
            Diagnostic::new("insufficient_total_capacity")
                .with_param("total_slots", total_slots)
                .with_param("max_capacity", sum_hi),
        );
    }

    if sum_lo > total_slots {
        errors.push(
            Diagnostic::new("excess_minimum_capacity")
                .with_param("total_slots", total_slots)
                .with_param("min_capacity", sum_lo),
        );
    }

    // Exact bipartite max-flow per exam under H4
    if h4_enabled {
        let grades_count = problem.grades.len();
        let required_per_exam = (grades_count * 3) as i32;

        for exam in &problem.exams {
            // Source = 0, Sink = 1 + teachers.len() + grades.len()
            let num_t = problem.teachers.len();
            let source = 0;
            let sink = 1 + num_t + grades_count;
            let mut graph = DinicGraph::new(sink + 1);

            // Connect Source -> Teachers (capacity 1 for available teachers with hi > 0)
            for (t_idx, teacher) in problem.teachers.iter().enumerate() {
                let hi = quota_map.get(&teacher.id).map_or(0, |q| q.hi);
                let available = teacher.active
                    && teacher.load_weight > 0.0
                    && hi > 0
                    && !unavailability_set.contains(&(teacher.id, exam.id));

                if available {
                    let u = 1 + t_idx;
                    graph.add_edge(source, u, 1);

                    // Connect Teacher -> eligible grades of this exam
                    for (g_idx, grade) in problem.grades.iter().enumerate() {
                        let e_set = is_eligible(teacher.id, exam.id, grade.id, Role::Setter);
                        let e_rev = is_eligible(teacher.id, exam.id, grade.id, Role::Reviewer);
                        if e_set || e_rev {
                            let v = 1 + num_t + g_idx;
                            graph.add_edge(u, v, 1);
                        }
                    }
                }
            }

            // Connect Grades -> Sink (capacity 3)
            for (g_idx, _) in problem.grades.iter().enumerate() {
                let v = 1 + num_t + g_idx;
                graph.add_edge(v, sink, 3);
            }

            let max_flow = graph.max_flow(source, sink);
            if max_flow < required_per_exam {
                errors.push(
                    Diagnostic::new("exam_capacity_infeasible")
                        .with_param("exam", exam.code.clone())
                        .with_param("max_flow", max_flow)
                        .with_param("required", required_per_exam),
                );

                for (g_idx, grade) in problem.grades.iter().enumerate() {
                    let v = 1 + num_t + g_idx;
                    let pflow = graph.get_flow(v, sink);
                    if pflow < 3 {
                        errors.push(
                            Diagnostic::new("panel_unfillable_under_h4")
                                .with_panel(PanelKey::new(exam.id, grade.id))
                                .with_param("exam", exam.code.clone())
                                .with_param("grade", grade.code)
                                .with_param("flow", pflow)
                                .with_param("required", 3),
                        );
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Warnings: Non-blocking diagnostic hints
    // -------------------------------------------------------------------------
    for teacher in &problem.teachers {
        let q = quota_map.get(&teacher.id).map_or(0.0, |tq| tq.quota);
        let eligible_panels = teacher_eligible_panel_count
            .get(&teacher.id)
            .copied()
            .unwrap_or(0);
        if q > 0.0 && eligible_panels == 1 {
            warnings.push(
                Diagnostic::new("teacher_single_panel_eligibility")
                    .with_teacher(teacher.id)
                    .with_param("teacher_name", teacher.full_name.clone())
                    .with_param("count", 1),
            );
        }
    }

    let active_eligible_teachers_count = problem
        .teachers
        .iter()
        .filter(|t| t.active && t.load_weight > 0.0)
        .count();

    if reviewer_eligible_teachers.len() < active_eligible_teachers_count {
        warnings.push(
            Diagnostic::new("restricted_reviewer_pool")
                .with_param("reviewer_count", reviewer_eligible_teachers.len())
                .with_param("teacher_count", active_eligible_teachers_count),
        );
    }

    FeasibilityReport {
        errors,
        warnings,
        quotas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Campus, CampusId, Exam, ExamId, Grade, GradeId, Lock, LockId, Role, RuleSetting,
        SchoolYear, SchoolYearId, Teacher, TeacherGrade, Unavailability,
    };

    fn make_valid_problem() -> Problem {
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
        let e1 = Exam {
            id: ExamId(1),
            school_year_id: sy.id,
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };

        // 3 teachers, 2 from CS1, 1 from CS2
        let teachers = vec![
            Teacher {
                id: TeacherId(1),
                full_name: "Teacher 1".to_string(),
                campus_id: CampusId(1),
                load_weight: 1.0,
                active: true,
                note: None,
            },
            Teacher {
                id: TeacherId(2),
                full_name: "Teacher 2".to_string(),
                campus_id: CampusId(1),
                load_weight: 1.0,
                active: true,
                note: None,
            },
            Teacher {
                id: TeacherId(3),
                full_name: "Teacher 3".to_string(),
                campus_id: CampusId(2),
                load_weight: 1.0,
                active: true,
                note: None,
            },
        ];

        let teacher_grades = vec![
            TeacherGrade {
                teacher_id: TeacherId(1),
                school_year_id: sy.id,
                grade_id: GradeId(10),
            },
            TeacherGrade {
                teacher_id: TeacherId(2),
                school_year_id: sy.id,
                grade_id: GradeId(10),
            },
            TeacherGrade {
                teacher_id: TeacherId(3),
                school_year_id: sy.id,
                grade_id: GradeId(10),
            },
        ];

        Problem {
            school_year: sy,
            campuses: vec![c1, c2],
            grades: vec![g10],
            exams: vec![e1],
            teachers,
            teacher_grades,
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    #[test]
    fn test_valid_problem_feasibility() {
        let problem = make_valid_problem();
        let report = check_feasibility(&problem);
        assert!(
            report.is_feasible(),
            "expected valid problem to be feasible: {:?}",
            report.errors
        );
    }

    #[test]
    fn test_diagnostic_insufficient_panel_teachers_and_campuses() {
        let mut problem = make_valid_problem();
        // Remove teacher 3 (leaves only 2 teachers, all from CS1)
        problem.teachers.retain(|t| t.id != TeacherId(3));
        problem
            .teacher_grades
            .retain(|tg| tg.teacher_id != TeacherId(3));

        let report = check_feasibility(&problem);
        assert!(!report.is_feasible());
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "insufficient_panel_teachers"));
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "insufficient_campuses"));
    }

    #[test]
    fn test_diagnostic_insufficient_setters_and_reviewers() {
        let mut problem = make_valid_problem();
        // Forbid teacher 1 and 2 from being Setters
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Forbid,
        });
        problem.locks.push(Lock {
            id: LockId(2),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(2),
            role: Some(Role::Setter),
            kind: LockKind::Forbid,
        });

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "insufficient_setters"));
    }

    #[test]
    fn test_diagnostic_lock_conflict_and_excess_pins() {
        let mut problem = make_valid_problem();
        // PIN + FORBID conflict on Teacher 1
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });
        problem.locks.push(Lock {
            id: LockId(2),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: None,
            kind: LockKind::Forbid,
        });

        let report = check_feasibility(&problem);
        assert!(report.errors.iter().any(|d| d.code == "lock_conflict"));
    }

    #[test]
    fn test_diagnostic_pinned_campus_monopoly() {
        let mut problem = make_valid_problem();
        // Add teacher 4 from campus 1
        problem.teachers.push(Teacher {
            id: TeacherId(4),
            full_name: "Teacher 4".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
        });
        problem.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(4),
            school_year_id: problem.school_year.id,
            grade_id: GradeId(10),
        });

        // Pin 3 teachers all from Campus 1
        for (i, tid) in [1, 2, 4].into_iter().enumerate() {
            problem.locks.push(Lock {
                id: LockId((i + 1) as i64),
                exam_id: ExamId(1),
                grade_id: GradeId(10),
                teacher_id: TeacherId(tid),
                role: None,
                kind: LockKind::Pin,
            });
        }

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "pinned_campus_monopoly"));
    }

    #[test]
    fn test_diagnostic_exam_capacity_bottleneck_under_h4() {
        let mut problem = make_valid_problem();
        // Add Grade 11: now 2 grades needing 6 slots in Exam 1
        problem.grades.push(Grade {
            id: GradeId(11),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        });

        // Add 2 more teachers (total 5 teachers). All 5 teach Grade 10 and 11.
        for tid in 4..=5 {
            problem.teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Teacher {tid}"),
                campus_id: CampusId(2),
                load_weight: 1.0,
                active: true,
                note: None,
            });
        }
        for tid in 1..=5 {
            problem.teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: problem.school_year.id,
                grade_id: GradeId(10),
            });
            problem.teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: problem.school_year.id,
                grade_id: GradeId(11),
            });
        }

        // Each grade individually has 5 eligible teachers (> 3),
        // but the exam requires 2 * 3 = 6 teachers, while only 5 exist!
        let report = check_feasibility(&problem);
        assert!(!report.is_feasible());
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "exam_capacity_infeasible"));
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "panel_unfillable_under_h4"));
    }

    #[test]
    fn test_diagnostic_pinned_teacher_ineligible_due_to_unavailability() {
        let mut problem = make_valid_problem();
        problem.unavailabilities.push(Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: Some("Medical".to_string()),
        });
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "pinned_teacher_ineligible"));
    }

    #[test]
    fn test_diagnostic_excess_pinned_setters_and_reviewers_and_pins() {
        let mut problem = make_valid_problem();
        // Add teacher 4 and 5
        for tid in 4..=5 {
            problem.teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Teacher {tid}"),
                campus_id: CampusId(2),
                load_weight: 1.0,
                active: true,
                note: None,
            });
            problem.teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: problem.school_year.id,
                grade_id: GradeId(10),
            });
        }

        // 3 pinned setters
        for (i, tid) in [1, 2, 3].into_iter().enumerate() {
            problem.locks.push(Lock {
                id: LockId((i + 1) as i64),
                exam_id: ExamId(1),
                grade_id: GradeId(10),
                teacher_id: TeacherId(tid),
                role: Some(Role::Setter),
                kind: LockKind::Pin,
            });
        }
        // 2 pinned reviewers
        for (i, tid) in [4, 5].into_iter().enumerate() {
            problem.locks.push(Lock {
                id: LockId((i + 4) as i64),
                exam_id: ExamId(1),
                grade_id: GradeId(10),
                teacher_id: TeacherId(tid),
                role: Some(Role::Reviewer),
                kind: LockKind::Pin,
            });
        }

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "excess_pinned_setters"));
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "excess_pinned_reviewers"));
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "excess_pins_in_panel"));
    }

    #[test]
    fn test_diagnostic_pinned_teacher_multiple_panels() {
        let mut problem = make_valid_problem();
        problem.grades.push(Grade {
            id: GradeId(11),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        });
        problem.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(1),
            school_year_id: problem.school_year.id,
            grade_id: GradeId(11),
        });

        // Pin Teacher 1 to both Grade 10 and 11 in Exam 1
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });
        problem.locks.push(Lock {
            id: LockId(2),
            exam_id: ExamId(1),
            grade_id: GradeId(11),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "pinned_teacher_multiple_panels"));
    }

    #[test]
    fn test_diagnostic_pinned_quota_exceeded() {
        let mut problem = make_valid_problem();
        // Teacher 1 has hi = 1 (1 exam under H4). Pin them twice.
        problem.exams.push(Exam {
            id: ExamId(2),
            school_year_id: problem.school_year.id,
            code: "CK1".to_string(),
            name: "Cuối kỳ 1".to_string(),
            sort_order: 2,
        });
        // Make teacher 1 inactive so hi = 0, but pin them
        problem.teachers[0].active = false;
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "pinned_quota_exceeded"));
    }

    #[test]
    fn test_diagnostic_capacity_and_warnings() {
        let mut problem = make_valid_problem();
        // In valid problem (3 teachers, 1 panel, 3 slots), each teacher has lo=0, hi=1 (sum hi = 3 = D)
        // Set all load_weight to 0 -> sum hi = 0 < 3 (insufficient_total_capacity)
        for t in &mut problem.teachers {
            t.load_weight = 0.0;
        }

        let report = check_feasibility(&problem);
        assert!(report
            .errors
            .iter()
            .any(|d| d.code == "insufficient_total_capacity"));

        // Test warnings: tight panel roster
        let problem2 = make_valid_problem();
        let report2 = check_feasibility(&problem2);
        assert!(report2
            .warnings
            .iter()
            .any(|w| w.code == "tight_panel_roster"));
    }

    #[test]
    fn test_diagnostic_structural_error() {
        let mut problem = make_valid_problem();
        // Introduce duplicate exam code
        problem.exams.push(Exam {
            id: ExamId(2),
            school_year_id: problem.school_year.id,
            code: "GK1".to_string(), // duplicate code!
            name: "Duplicate".to_string(),
            sort_order: 2,
        });

        let report = check_feasibility(&problem);
        assert!(report.errors.iter().any(|d| d.code == "structural_error"));
    }
}
