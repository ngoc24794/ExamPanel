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
    SubjectId, TeacherId, TeacherQuota, ValidationError,
};
use maxflow::DinicGraph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// A structured diagnostic report item identifying an infeasibility or warning condition.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
pub struct Diagnostic {
    #[serde(default)]
    #[ts(optional)]
    pub rule: Option<RuleKey>,
    pub code: &'static str,
    #[serde(default)]
    #[ts(optional)]
    pub panel: Option<PanelKey>,
    #[serde(default)]
    #[ts(optional)]
    pub teacher: Option<TeacherId>,
    #[ts(type = "Record<string, unknown>")]
    pub params: BTreeMap<String, serde_json::Value>,
}

impl<'de> Deserialize<'de> for Diagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DiagnosticHelper {
            #[serde(default)]
            rule: Option<RuleKey>,
            code: String,
            #[serde(default)]
            panel: Option<PanelKey>,
            #[serde(default)]
            teacher: Option<TeacherId>,
            #[serde(default)]
            params: BTreeMap<String, serde_json::Value>,
        }

        let helper = DiagnosticHelper::deserialize(deserializer)?;
        Ok(Self {
            rule: helper.rule,
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
            rule: None,
            code,
            panel: None,
            teacher: None,
            params: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_rule(mut self, rule: RuleKey) -> Self {
        self.rule = Some(rule);
        self
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

impl From<&ValidationError> for Diagnostic {
    fn from(err: &ValidationError) -> Self {
        match err {
            ValidationError::DuplicateCampusId(id) => {
                Diagnostic::new("duplicate_campus_id").with_param("campus_id", id.0)
            }
            ValidationError::DuplicateCampusCode(code) => {
                Diagnostic::new("duplicate_campus_code").with_param("code", code.clone())
            }
            ValidationError::DuplicateGradeId(id) => {
                Diagnostic::new("duplicate_grade_id").with_param("grade_id", id.0)
            }
            ValidationError::DuplicateGradeCode(code) => {
                Diagnostic::new("duplicate_grade_code").with_param("code", *code)
            }
            ValidationError::DuplicateExamId(id) => {
                Diagnostic::new("duplicate_exam_id").with_param("exam_id", id.0)
            }
            ValidationError::DuplicateExamCode(code) => {
                Diagnostic::new("duplicate_exam_code").with_param("code", code.clone())
            }
            ValidationError::DuplicateTeacherId(id) => Diagnostic::new("duplicate_teacher_id")
                .with_teacher(*id)
                .with_param("teacher_id", id.0),
            ValidationError::UnknownCampus(tid, cid) => Diagnostic::new("unknown_campus_ref")
                .with_teacher(*tid)
                .with_param("teacher_id", tid.0)
                .with_param("campus_id", cid.0),
            ValidationError::InvalidLoadWeight(tid, weight) => {
                Diagnostic::new("load_weight_out_of_range")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("load_weight", weight.clone())
            }
            ValidationError::TeacherGradeUnknownTeacher(tid) => {
                Diagnostic::new("unknown_teacher_ref")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("context", "teacher_grade")
            }
            ValidationError::TeacherGradeUnknownGrade(gid) => Diagnostic::new("unknown_grade_ref")
                .with_param("grade_id", gid.0)
                .with_param("context", "teacher_grade"),
            ValidationError::DuplicateTeacherGrade(tid, gid) => {
                Diagnostic::new("duplicate_teacher_grade")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("grade_id", gid.0)
            }
            ValidationError::LockUnknownTeacher(tid) => Diagnostic::new("unknown_teacher_ref")
                .with_teacher(*tid)
                .with_param("teacher_id", tid.0)
                .with_param("context", "lock"),
            ValidationError::LockUnknownExam(eid) => Diagnostic::new("unknown_exam_ref")
                .with_param("exam_id", eid.0)
                .with_param("context", "lock"),
            ValidationError::LockUnknownGrade(gid) => Diagnostic::new("unknown_grade_ref")
                .with_param("grade_id", gid.0)
                .with_param("context", "lock"),
            ValidationError::UnavailabilityUnknownTeacher(tid) => {
                Diagnostic::new("unknown_teacher_ref")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("context", "unavailability")
            }
            ValidationError::UnavailabilityUnknownExam(eid) => Diagnostic::new("unknown_exam_ref")
                .with_param("exam_id", eid.0)
                .with_param("context", "unavailability"),
            ValidationError::DuplicateUnavailability(tid, eid) => {
                Diagnostic::new("duplicate_unavailability")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("exam_id", eid.0)
            }
            ValidationError::DuplicateRuleKey(rule_key) => {
                Diagnostic::new("duplicate_rule_key").with_param("rule_key", rule_key.as_str())
            }
            ValidationError::NegativeRuleWeight(rule_key, weight) => {
                Diagnostic::new("negative_rule_weight")
                    .with_param("rule_key", rule_key.as_str())
                    .with_param("weight", weight.clone())
            }
            ValidationError::DuplicateSubjectId(id) => {
                Diagnostic::new("duplicate_subject_id").with_param("subject_id", id.0)
            }
            ValidationError::DuplicateSubjectCode(code) => {
                Diagnostic::new("duplicate_subject_code").with_param("code", code.clone())
            }
            ValidationError::InvalidSubjectSetters(id, s) => {
                Diagnostic::new("invalid_subject_setters")
                    .with_param("subject_id", id.0)
                    .with_param("setters", *s)
            }
            ValidationError::InvalidSubjectReviewers(id, r) => {
                Diagnostic::new("invalid_subject_reviewers")
                    .with_param("subject_id", id.0)
                    .with_param("reviewers", *r)
            }
            ValidationError::InvalidSubjectMinCampuses(id, m) => {
                Diagnostic::new("invalid_subject_min_campuses")
                    .with_param("subject_id", id.0)
                    .with_param("min_campuses", *m)
            }
            ValidationError::InvalidMaxTasksOverride(tid, val) => {
                Diagnostic::new("invalid_max_tasks_override")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("max_tasks", *val)
            }
            ValidationError::CompetencyUnknownTeacher(tid) => {
                Diagnostic::new("unknown_teacher_ref")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("context", "competency")
            }
            ValidationError::CompetencyUnknownSubject(sid) => {
                Diagnostic::new("unknown_subject_ref")
                    .with_param("subject_id", sid.0)
                    .with_param("context", "competency")
            }
            ValidationError::DuplicateCompetency(tid, sid, role) => {
                Diagnostic::new("duplicate_competency")
                    .with_teacher(*tid)
                    .with_param("teacher_id", tid.0)
                    .with_param("subject_id", sid.0)
                    .with_param("role", role.as_str())
            }
            ValidationError::LockUnknownSubject(sid) => Diagnostic::new("unknown_subject_ref")
                .with_param("subject_id", sid.0)
                .with_param("context", "lock"),
        }
    }
}

impl From<ValidationError> for Diagnostic {
    fn from(err: ValidationError) -> Self {
        Diagnostic::from(&err)
    }
}

/// The outcome of the pre-solve feasibility verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct FeasibilityReport {
    /// True if there are zero blocking errors.
    pub is_feasible: bool,
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
        self.is_feasible
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
    for err in &structural_errors {
        errors.push(Diagnostic::from(err));
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

    let subjects = problem.effective_subjects();
    let subject_map: HashMap<SubjectId, &crate::domain::Subject> =
        subjects.iter().map(|s| (s.id, s)).collect();

    let unavailability_set: HashSet<(TeacherId, ExamId)> = problem
        .unavailabilities
        .iter()
        .map(|u| (u.teacher_id, u.exam_id))
        .collect();

    let h4_setting = problem.rule_settings.iter().find(|s| s.key == RuleKey::H4);
    let h4_enabled = h4_setting.is_none_or(|s| s.enabled);
    let default_max_tasks_per_exam = h4_setting
        .and_then(|s| {
            s.params
                .get("max_tasks_per_exam")
                .and_then(serde_json::Value::as_u64)
        })
        .map_or(2, |v| v as usize);

    // Compute quotas (H7)
    let quotas = calculate_quotas(problem);
    let quota_map: HashMap<TeacherId, &TeacherQuota> =
        quotas.iter().map(|q| (q.teacher_id, q)).collect();

    // -------------------------------------------------------------------------
    // Degenerate / Empty Database Check
    // -------------------------------------------------------------------------
    let mut degenerate = false;
    if problem.campuses.is_empty() {
        errors.push(Diagnostic::new("no_campuses"));
        degenerate = true;
    }

    let active_teachers: Vec<&crate::domain::Teacher> = problem
        .teachers
        .iter()
        .filter(|t| t.active && t.load_weight > 0.0)
        .collect();

    if active_teachers.is_empty() {
        errors.push(Diagnostic::new("no_active_teachers"));
        degenerate = true;
    } else if !problem.campuses.is_empty() {
        let active_campuses: HashSet<CampusId> =
            active_teachers.iter().map(|t| t.campus_id).collect();
        if active_campuses.len() < 2 {
            errors.push(Diagnostic::new("single_campus"));
            degenerate = true;
        }
    }

    if problem.grades.is_empty() {
        errors.push(Diagnostic::new("no_grades"));
        degenerate = true;
    }

    if problem.exams.is_empty() {
        errors.push(Diagnostic::new("no_exams"));
        degenerate = true;
    }

    if problem.subjects.is_empty() {
        errors.push(Diagnostic::new("no_subjects"));
        degenerate = true;
    }

    if degenerate {
        return FeasibilityReport {
            is_feasible: false,
            errors,
            warnings,
            quotas,
        };
    }

    // Organize locks
    let mut pin_locks: Vec<&crate::domain::Lock> = Vec::new();
    let mut forbid_locks: Vec<&crate::domain::Lock> = Vec::new();

    for lock in &problem.locks {
        match lock.kind {
            LockKind::Forbid => forbid_locks.push(lock),
            LockKind::Pin => pin_locks.push(lock),
        }
    }

    // Forced placements
    let forced = crate::domain::forced::find_forced_placements(problem).unwrap_or_default();
    let mut forced_counts: HashMap<TeacherId, usize> = HashMap::new();
    let mut forced_per_exam: HashMap<(TeacherId, ExamId), usize> = HashMap::new();
    for p in &forced {
        warnings.push(
            Diagnostic::new("forced_placement")
                .with_panel(p.panel)
                .with_teacher(p.teacher_id)
                .with_param("role", p.role.as_str())
                .with_param("position", p.position),
        );
        *forced_counts.entry(p.teacher_id).or_default() += 1;
        *forced_per_exam
            .entry((p.teacher_id, p.panel.exam_id))
            .or_default() += 1;
    }

    // Check quota overrides against [F_t, cap_t]
    for teacher in &problem.teachers {
        if let Some(ov) = teacher.quota_override {
            let f_t = forced_counts.get(&teacher.id).copied().unwrap_or(0);
            let cap_t = quota_map.get(&teacher.id).map_or(0, |q| q.hi);
            if (ov as usize) < f_t || (ov as usize) > cap_t {
                errors.push(
                    Diagnostic::new("quota_override_out_of_range")
                        .with_teacher(teacher.id)
                        .with_param("override", ov)
                        .with_param("min", f_t)
                        .with_param("max", cap_t),
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // F2: Per-panel adequacy
    // -------------------------------------------------------------------------
    let all_panels = problem.all_panels();
    let mut teacher_eligible_panel_count: HashMap<TeacherId, usize> = HashMap::new();

    for pkey in &all_panels {
        let subject = match subject_map.get(&pkey.subject_id) {
            Some(s) => s,
            None => continue,
        };

        let mut setter_count = 0usize;
        let mut reviewer_count = 0usize;
        let mut distinct_eligible: HashSet<TeacherId> = HashSet::new();
        let mut campus_set: HashSet<CampusId> = HashSet::new();

        for teacher in &problem.teachers {
            let e_setter = crate::domain::forced::is_teacher_eligible(
                problem,
                teacher.id,
                pkey.exam_id,
                pkey.grade_id,
                pkey.subject_id,
                Role::Setter,
            );
            let e_reviewer = crate::domain::forced::is_teacher_eligible(
                problem,
                teacher.id,
                pkey.exam_id,
                pkey.grade_id,
                pkey.subject_id,
                Role::Reviewer,
            );

            if e_setter {
                setter_count += 1;
            }
            if e_reviewer {
                reviewer_count += 1;
            }
            if e_setter || e_reviewer {
                distinct_eligible.insert(teacher.id);
                campus_set.insert(teacher.campus_id);
                *teacher_eligible_panel_count.entry(teacher.id).or_default() += 1;
            }
        }

        let expected_setters = subject.setters as usize;
        let expected_reviewers = subject.reviewers as usize;
        let expected_total = expected_setters + expected_reviewers;
        let min_campuses = subject.min_campuses as usize;

        let exam_code = exam_map.get(&pkey.exam_id).map_or("", |e| e.code.as_str());
        let grade_code = grade_map.get(&pkey.grade_id).map_or(0, |g| g.code);

        if expected_setters > 0 && setter_count == 0 {
            errors.push(
                Diagnostic::new("no_competent_teacher")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("role", "setter"),
            );
        } else if setter_count < expected_setters {
            errors.push(
                Diagnostic::new("insufficient_setters")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("count", setter_count)
                    .with_param("expected", expected_setters),
            );
        }

        if expected_reviewers > 0 && reviewer_count == 0 {
            errors.push(
                Diagnostic::new("no_competent_teacher")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("role", "reviewer"),
            );
        } else if reviewer_count < expected_reviewers {
            errors.push(
                Diagnostic::new("insufficient_reviewers")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("count", reviewer_count)
                    .with_param("expected", expected_reviewers),
            );
        }

        if distinct_eligible.len() < expected_total {
            errors.push(
                Diagnostic::new("insufficient_panel_teachers")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("count", distinct_eligible.len())
                    .with_param("expected", expected_total),
            );
        }

        if min_campuses > 0 && campus_set.len() < min_campuses {
            errors.push(
                Diagnostic::new("insufficient_campuses")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("count", campus_set.len())
                    .with_param("min_campuses", min_campuses),
            );
        }

        if distinct_eligible.len() == expected_total {
            warnings.push(
                Diagnostic::new("tight_panel_roster")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("subject", subject.code.clone())
                    .with_param("count", expected_total),
            );
        }
    }

    // -------------------------------------------------------------------------
    // F3: Locks validation
    // -------------------------------------------------------------------------
    let mut panel_pins: HashMap<PanelKey, Vec<&crate::domain::Lock>> = HashMap::new();
    let mut teacher_pins: HashMap<TeacherId, Vec<&crate::domain::Lock>> = HashMap::new();
    let mut exam_teacher_pins: HashMap<(ExamId, TeacherId), Vec<PanelKey>> = HashMap::new();

    for pin in &pin_locks {
        let pkey = PanelKey::new(pin.exam_id, pin.grade_id, pin.subject_id);
        panel_pins.entry(pkey).or_default().push(pin);
        teacher_pins.entry(pin.teacher_id).or_default().push(pin);
        exam_teacher_pins
            .entry((pin.exam_id, pin.teacher_id))
            .or_default()
            .push(pkey);

        let exam_code = exam_map.get(&pin.exam_id).map_or("", |e| e.code.as_str());
        let grade_code = grade_map.get(&pin.grade_id).map_or(0, |g| g.code);

        // Check PIN+FORBID conflict
        let has_conflict = forbid_locks.iter().any(|f| {
            f.exam_id == pin.exam_id
                && f.grade_id == pin.grade_id
                && f.subject_id == pin.subject_id
                && f.teacher_id == pin.teacher_id
                && (f.role.is_none() || pin.role.is_none() || f.role == pin.role)
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
        let role = pin.role.unwrap_or(Role::Setter);
        let eligible = crate::domain::forced::is_teacher_eligible(
            problem,
            pin.teacher_id,
            pin.exam_id,
            pin.grade_id,
            pin.subject_id,
            role,
        );

        if !eligible {
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
        let subject = match subject_map.get(&pkey.subject_id) {
            Some(s) => s,
            None => continue,
        };
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

        let expected_setters = subject.setters as usize;
        let expected_reviewers = subject.reviewers as usize;
        let expected_total = expected_setters + expected_reviewers;
        let min_campuses = subject.min_campuses as usize;

        if pinned_setters > expected_setters {
            errors.push(
                Diagnostic::new("excess_pinned_setters")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pinned_setters),
            );
        }

        if pinned_reviewers > expected_reviewers {
            errors.push(
                Diagnostic::new("excess_pinned_reviewers")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pinned_reviewers),
            );
        }

        if pins.len() > expected_total {
            errors.push(
                Diagnostic::new("excess_pins_in_panel")
                    .with_panel(*pkey)
                    .with_param("exam", exam_code)
                    .with_param("grade", grade_code)
                    .with_param("count", pins.len()),
            );
        }

        if pins.len() == expected_total && min_campuses > 0 && pinned_campuses.len() < min_campuses
        {
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
            let configured = teacher_map
                .get(teacher_id)
                .and_then(|t| t.max_tasks_per_exam_override)
                .map_or(default_max_tasks_per_exam, |v| v as usize);
            let forced_count = forced_per_exam
                .get(&(*teacher_id, *exam_id))
                .copied()
                .unwrap_or(0);
            let eff_max = configured.max(forced_count);

            if pkeys.len() > eff_max {
                let exam_code = exam_map.get(exam_id).map_or("", |e| e.code.as_str());
                errors.push(
                    Diagnostic::new("pinned_teacher_multiple_panels")
                        .with_teacher(*teacher_id)
                        .with_param("exam", exam_code)
                        .with_param("count", pkeys.len())
                        .with_param("limit", eff_max),
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
    let mut total_slots = 0usize;
    for p in &all_panels {
        if let Some(sub) = subject_map.get(&p.subject_id) {
            total_slots += (sub.setters + sub.reviewers) as usize;
        }
    }
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

    // Bipartite max-flow per exam under H4
    if h4_enabled {
        let num_t = problem.teachers.len();

        for exam in &problem.exams {
            let exam_panels: Vec<&PanelKey> =
                all_panels.iter().filter(|p| p.exam_id == exam.id).collect();
            let mut required_per_exam = 0i32;
            for p in &exam_panels {
                if let Some(sub) = subject_map.get(&p.subject_id) {
                    required_per_exam += (sub.setters + sub.reviewers) as i32;
                }
            }

            let num_p = exam_panels.len();
            let source = 0;
            let sink = 1 + num_t + num_p;
            let mut graph = DinicGraph::new(sink + 1);

            // Connect Source -> Teachers (capacity = eff_max_tasks)
            for (t_idx, teacher) in problem.teachers.iter().enumerate() {
                let hi = quota_map.get(&teacher.id).map_or(0, |q| q.hi);
                let available = teacher.active
                    && teacher.load_weight > 0.0
                    && hi > 0
                    && !unavailability_set.contains(&(teacher.id, exam.id));

                if available {
                    let u = 1 + t_idx;
                    let configured = teacher
                        .max_tasks_per_exam_override
                        .map_or(default_max_tasks_per_exam, |v| v as usize);
                    let forced_count = forced_per_exam
                        .get(&(teacher.id, exam.id))
                        .copied()
                        .unwrap_or(0);
                    let eff_max = configured.max(forced_count) as i32;

                    graph.add_edge(source, u, eff_max);

                    // Connect Teacher -> eligible panels in this exam
                    for (p_idx, pkey) in exam_panels.iter().enumerate() {
                        let e_set = crate::domain::forced::is_teacher_eligible(
                            problem,
                            teacher.id,
                            pkey.exam_id,
                            pkey.grade_id,
                            pkey.subject_id,
                            Role::Setter,
                        );
                        let e_rev = crate::domain::forced::is_teacher_eligible(
                            problem,
                            teacher.id,
                            pkey.exam_id,
                            pkey.grade_id,
                            pkey.subject_id,
                            Role::Reviewer,
                        );

                        if e_set || e_rev {
                            let v = 1 + num_t + p_idx;
                            graph.add_edge(u, v, 1);
                        }
                    }
                }
            }

            // Connect Panels -> Sink (capacity = subject.setters + subject.reviewers)
            for (p_idx, pkey) in exam_panels.iter().enumerate() {
                let v = 1 + num_t + p_idx;
                let cap = subject_map
                    .get(&pkey.subject_id)
                    .map_or(3, |s| (s.setters + s.reviewers) as i32);
                graph.add_edge(v, sink, cap);
            }

            let max_flow = graph.max_flow(source, sink);
            if max_flow < required_per_exam {
                errors.push(
                    Diagnostic::new("exam_capacity_infeasible")
                        .with_param("exam", exam.code.clone())
                        .with_param("max_flow", max_flow)
                        .with_param("required", required_per_exam),
                );

                for (p_idx, pkey) in exam_panels.iter().enumerate() {
                    let v = 1 + num_t + p_idx;
                    let pflow = graph.get_flow(v, sink);
                    let required = subject_map
                        .get(&pkey.subject_id)
                        .map_or(3, |s| (s.setters + s.reviewers) as i32);
                    if pflow < required {
                        let grade_code = grade_map.get(&pkey.grade_id).map_or(0, |g| g.code);
                        errors.push(
                            Diagnostic::new("panel_unfillable_under_h4")
                                .with_panel(**pkey)
                                .with_param("exam", exam.code.clone())
                                .with_param("grade", grade_code)
                                .with_param("flow", pflow)
                                .with_param("required", required),
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

    let is_feasible = errors.is_empty();

    FeasibilityReport {
        is_feasible,
        errors,
        warnings,
        quotas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, Lock, LockId, Role,
        RuleSetting, SchoolYear, SchoolYearId, Subject, SubjectId, Teacher, TeacherGrade,
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

        let teachers = vec![
            Teacher {
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
            },
            Teacher {
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
            },
            Teacher {
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

        let competencies = vec![
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
            teachers,
            teacher_grades,
            competencies,
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
        let g11 = Grade {
            id: GradeId(11),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        };
        problem.grades.push(g11);
        problem.teachers.retain(|t| t.id != TeacherId(3));
        problem
            .teacher_grades
            .retain(|tg| tg.teacher_id != TeacherId(3));
        problem
            .competencies
            .retain(|c| c.teacher_id != TeacherId(3));

        let t4 = Teacher {
            id: TeacherId(4),
            full_name: "Teacher 4".to_string(),
            display_name: None,
            campus_id: CampusId(2),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        problem.teachers.push(t4);
        problem.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(4),
            school_year_id: problem.school_year.id,
            grade_id: GradeId(11),
        });
        problem.competencies.push(Competency {
            teacher_id: TeacherId(4),
            subject_id: SubjectId(1),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        });
        problem.competencies.push(Competency {
            teacher_id: TeacherId(4),
            subject_id: SubjectId(1),
            role: Role::Reviewer,
            grade_scope: GradeScope::Taught,
        });

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
        problem.locks.push(Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            subject_id: SubjectId(1),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        });
        problem.locks.push(Lock {
            id: LockId(2),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            subject_id: SubjectId(1),
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
        problem.teachers.push(Teacher {
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
        problem.teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(4),
            school_year_id: problem.school_year.id,
            grade_id: GradeId(10),
        });
        problem.competencies.push(Competency {
            teacher_id: TeacherId(4),
            subject_id: SubjectId(1),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        });

        for (i, tid) in [1, 2, 4].into_iter().enumerate() {
            problem.locks.push(Lock {
                id: LockId((i + 1) as i64),
                exam_id: ExamId(1),
                grade_id: GradeId(10),
                subject_id: SubjectId(1),
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
        problem.grades.push(Grade {
            id: GradeId(11),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        });

        for tid in 4..=5 {
            problem.teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Teacher {tid}"),
                display_name: None,
                campus_id: CampusId(2),
                load_weight: 1.0,
                active: true,
                note: None,
                code: None,
                quota_override: None,
                max_tasks_per_exam_override: None,
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
            problem.competencies.push(Competency {
                teacher_id: TeacherId(tid),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            problem.competencies.push(Competency {
                teacher_id: TeacherId(tid),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        // Set max_tasks_per_exam = 1 so 5 teachers cannot fill 6 seats
        if let Some(s) = problem
            .rule_settings
            .iter_mut()
            .find(|s| s.key == RuleKey::H4)
        {
            s.params["max_tasks_per_exam"] = serde_json::json!(1);
        }

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
}
