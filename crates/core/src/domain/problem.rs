//! Problem snapshot and structural validation.

use super::entities::{
    Campus, Competency, Exam, Grade, Lock, PanelKey, Role, RuleKey, RuleSetting, SchoolYear,
    Subject, Teacher, TeacherGrade, Unavailability,
};
use super::ids::{CampusId, ExamId, GradeId, SubjectId, TeacherId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Structural validation error representing invalid or inconsistent input data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum ValidationError {
    #[error("Duplicate campus ID: {0}")]
    DuplicateCampusId(CampusId),

    #[error("Duplicate campus code: '{0}'")]
    DuplicateCampusCode(String),

    #[error("Duplicate grade ID: {0}")]
    DuplicateGradeId(GradeId),

    #[error("Duplicate grade code: {0}")]
    DuplicateGradeCode(i32),

    #[error("Duplicate subject ID: {0}")]
    DuplicateSubjectId(SubjectId),

    #[error("Duplicate subject code: '{0}'")]
    DuplicateSubjectCode(String),

    #[error("Subject {0} has invalid setter seat count {1} (must be 1..=4)")]
    InvalidSubjectSetters(SubjectId, u8),

    #[error("Subject {0} has invalid reviewer seat count {1} (must be 0..=2)")]
    InvalidSubjectReviewers(SubjectId, u8),

    #[error("Subject {0} has invalid min_campuses {1} (must be 0..=3)")]
    InvalidSubjectMinCampuses(SubjectId, u8),

    #[error("Duplicate exam ID: {0}")]
    DuplicateExamId(ExamId),

    #[error("Duplicate exam code: '{0}' for school year")]
    DuplicateExamCode(String),

    #[error("Duplicate teacher ID: {0}")]
    DuplicateTeacherId(TeacherId),

    #[error("Teacher {0} references unknown campus {1}")]
    UnknownCampus(TeacherId, CampusId),

    #[error("Teacher {0} load_weight {1} is out of range [0.0, 1.0]")]
    InvalidLoadWeight(TeacherId, String),

    #[error("Teacher {0} max_tasks_per_exam_override must be >= 1")]
    InvalidMaxTasksOverride(TeacherId, u32),

    #[error("TeacherGrade references unknown teacher {0}")]
    TeacherGradeUnknownTeacher(TeacherId),

    #[error("TeacherGrade references unknown grade {0}")]
    TeacherGradeUnknownGrade(GradeId),

    #[error("Duplicate teacher grade entry for teacher {0}, grade {1}")]
    DuplicateTeacherGrade(TeacherId, GradeId),

    #[error("Competency references unknown teacher {0}")]
    CompetencyUnknownTeacher(TeacherId),

    #[error("Competency references unknown subject {0}")]
    CompetencyUnknownSubject(SubjectId),

    #[error("Duplicate competency entry for teacher {0}, subject {1}, role {2}")]
    DuplicateCompetency(TeacherId, SubjectId, Role),

    #[error("Lock references unknown teacher {0}")]
    LockUnknownTeacher(TeacherId),

    #[error("Lock references unknown exam {0}")]
    LockUnknownExam(ExamId),

    #[error("Lock references unknown grade {0}")]
    LockUnknownGrade(GradeId),

    #[error("Lock references unknown subject {0}")]
    LockUnknownSubject(SubjectId),

    #[error("Unavailability references unknown teacher {0}")]
    UnavailabilityUnknownTeacher(TeacherId),

    #[error("Unavailability references unknown exam {0}")]
    UnavailabilityUnknownExam(ExamId),

    #[error("Duplicate unavailability entry for teacher {0}, exam {1}")]
    DuplicateUnavailability(TeacherId, ExamId),

    #[error("Duplicate rule setting key: {0}")]
    DuplicateRuleKey(RuleKey),

    #[error("Rule setting {0} has negative weight {1}")]
    NegativeRuleWeight(RuleKey, String),
}

/// A complete problem snapshot for a single school year.
/// This is the pure domain input supplied to the pre-solve feasibility checker and solver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct Problem {
    pub school_year: SchoolYear,
    pub campuses: Vec<Campus>,
    pub grades: Vec<Grade>,
    pub subjects: Vec<Subject>,
    pub teachers: Vec<Teacher>,
    pub teacher_grades: Vec<TeacherGrade>,
    pub competencies: Vec<Competency>,
    pub exams: Vec<Exam>,
    pub unavailabilities: Vec<Unavailability>,
    pub locks: Vec<Lock>,
    pub rule_settings: Vec<RuleSetting>,
}

impl Problem {
    /// S3 (reviewer independent from setters' campus) can only matter when the active teachers
    /// span at least two campuses; with one it adds a constant penalty nobody can remove (RA-017).
    #[must_use]
    pub fn campus_independence_applicable(&self) -> bool {
        let mut campuses = self.teachers.iter().map(|t| t.campus_id);
        match campuses.next() {
            Some(first) => campuses.any(|c| c != first),
            None => false,
        }
    }

    /// Validates the structural integrity of the problem snapshot.
    ///
    /// Checks for:
    /// - Reference consistency (teacher -> campus, teacher_grade -> teacher/grade, locks -> exam/grade/teacher, etc.)
    /// - Key uniqueness (campus codes, grade codes, exam codes, teacher IDs)
    /// - Attribute bounds (load_weight in [0.0, 1.0], rule weights >= 0)
    ///
    /// Note: Does not verify feasibility or constraint satisfiability (that is done in Phase 3).
    #[must_use]
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // 1. Campuses
        let mut campus_ids = HashSet::new();
        let mut campus_codes = HashSet::new();
        for campus in &self.campuses {
            if !campus_ids.insert(campus.id) {
                errors.push(ValidationError::DuplicateCampusId(campus.id));
            }
            if !campus_codes.insert(campus.code.clone()) {
                errors.push(ValidationError::DuplicateCampusCode(campus.code.clone()));
            }
        }

        // 2. Grades
        let mut grade_ids = HashSet::new();
        let mut grade_codes = HashSet::new();
        for grade in &self.grades {
            if !grade_ids.insert(grade.id) {
                errors.push(ValidationError::DuplicateGradeId(grade.id));
            }
            if !grade_codes.insert(grade.code) {
                errors.push(ValidationError::DuplicateGradeCode(grade.code));
            }
        }

        // 3. Subjects
        let mut subject_ids = HashSet::new();
        let mut subject_codes = HashSet::new();
        for subject in &self.subjects {
            if !subject_ids.insert(subject.id) {
                errors.push(ValidationError::DuplicateSubjectId(subject.id));
            }
            if !subject_codes.insert(subject.code.clone()) {
                errors.push(ValidationError::DuplicateSubjectCode(subject.code.clone()));
            }
            if !(1..=4).contains(&subject.setters) {
                errors.push(ValidationError::InvalidSubjectSetters(
                    subject.id,
                    subject.setters,
                ));
            }
            if !(0..=2).contains(&subject.reviewers) {
                errors.push(ValidationError::InvalidSubjectReviewers(
                    subject.id,
                    subject.reviewers,
                ));
            }
            if !(0..=3).contains(&subject.min_campuses) {
                errors.push(ValidationError::InvalidSubjectMinCampuses(
                    subject.id,
                    subject.min_campuses,
                ));
            }
        }

        // 4. Exams
        let mut exam_ids = HashSet::new();
        let mut exam_codes = HashSet::new();
        for exam in &self.exams {
            if !exam_ids.insert(exam.id) {
                errors.push(ValidationError::DuplicateExamId(exam.id));
            }
            if !exam_codes.insert(exam.code.clone()) {
                errors.push(ValidationError::DuplicateExamCode(exam.code.clone()));
            }
        }

        // 5. Teachers
        let mut teacher_ids = HashSet::new();
        for teacher in &self.teachers {
            if !teacher_ids.insert(teacher.id) {
                errors.push(ValidationError::DuplicateTeacherId(teacher.id));
            }
            if !campus_ids.contains(&teacher.campus_id) {
                errors.push(ValidationError::UnknownCampus(
                    teacher.id,
                    teacher.campus_id,
                ));
            }
            if !(0.0..=1.0).contains(&teacher.load_weight) || teacher.load_weight.is_nan() {
                errors.push(ValidationError::InvalidLoadWeight(
                    teacher.id,
                    teacher.load_weight.to_string(),
                ));
            }
            if let Some(m) = teacher.max_tasks_per_exam_override {
                if m < 1 {
                    errors.push(ValidationError::InvalidMaxTasksOverride(teacher.id, m));
                }
            }
        }

        // 6. TeacherGrades
        let mut seen_teacher_grades = HashSet::new();
        for tg in &self.teacher_grades {
            if !teacher_ids.contains(&tg.teacher_id) {
                errors.push(ValidationError::TeacherGradeUnknownTeacher(tg.teacher_id));
            }
            if !grade_ids.contains(&tg.grade_id) {
                errors.push(ValidationError::TeacherGradeUnknownGrade(tg.grade_id));
            }
            if !seen_teacher_grades.insert((tg.teacher_id, tg.grade_id)) {
                errors.push(ValidationError::DuplicateTeacherGrade(
                    tg.teacher_id,
                    tg.grade_id,
                ));
            }
        }

        // 7. Competencies
        let mut seen_competencies = HashSet::new();
        for comp in &self.competencies {
            if !teacher_ids.contains(&comp.teacher_id) {
                errors.push(ValidationError::CompetencyUnknownTeacher(comp.teacher_id));
            }
            if !subject_ids.contains(&comp.subject_id) {
                errors.push(ValidationError::CompetencyUnknownSubject(comp.subject_id));
            }
            if !seen_competencies.insert((comp.teacher_id, comp.subject_id, comp.role)) {
                errors.push(ValidationError::DuplicateCompetency(
                    comp.teacher_id,
                    comp.subject_id,
                    comp.role,
                ));
            }
        }

        // 8. Unavailabilities
        let mut seen_unavailabilities = HashSet::new();
        for u in &self.unavailabilities {
            if !teacher_ids.contains(&u.teacher_id) {
                errors.push(ValidationError::UnavailabilityUnknownTeacher(u.teacher_id));
            }
            if !exam_ids.contains(&u.exam_id) {
                errors.push(ValidationError::UnavailabilityUnknownExam(u.exam_id));
            }
            if !seen_unavailabilities.insert((u.teacher_id, u.exam_id)) {
                errors.push(ValidationError::DuplicateUnavailability(
                    u.teacher_id,
                    u.exam_id,
                ));
            }
        }

        // 9. Locks
        for lock in &self.locks {
            if !teacher_ids.contains(&lock.teacher_id) {
                errors.push(ValidationError::LockUnknownTeacher(lock.teacher_id));
            }
            if !exam_ids.contains(&lock.exam_id) {
                errors.push(ValidationError::LockUnknownExam(lock.exam_id));
            }
            if !grade_ids.contains(&lock.grade_id) {
                errors.push(ValidationError::LockUnknownGrade(lock.grade_id));
            }
            if !subject_ids.contains(&lock.subject_id) {
                errors.push(ValidationError::LockUnknownSubject(lock.subject_id));
            }
        }

        // 10. RuleSettings
        let mut seen_rule_keys = HashSet::new();
        for rule in &self.rule_settings {
            if !seen_rule_keys.insert(rule.key) {
                errors.push(ValidationError::DuplicateRuleKey(rule.key));
            }
            if rule.weight < 0.0 || rule.weight.is_nan() {
                errors.push(ValidationError::NegativeRuleWeight(
                    rule.key,
                    rule.weight.to_string(),
                ));
            }
        }

        errors
    }

    /// Computes a stable, deterministic canonical SHA-256 hash representing this problem snapshot.
    #[must_use]
    pub fn data_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut campuses: Vec<_> = self
            .campuses
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id.0,
                    "code": &c.code,
                    "name": &c.name,
                    "color": &c.color,
                })
            })
            .collect();
        campuses.sort_by(|a, b| {
            a["code"]
                .as_str()
                .cmp(&b["code"].as_str())
                .then(a["id"].as_i64().cmp(&b["id"].as_i64()))
        });

        let mut grades: Vec<_> = self
            .grades
            .iter()
            .map(|g| {
                serde_json::json!({
                    "id": g.id.0,
                    "code": g.code,
                    "name": &g.name,
                    "sort_order": g.sort_order,
                })
            })
            .collect();
        grades.sort_by(|a, b| {
            a["code"]
                .as_i64()
                .cmp(&b["code"].as_i64())
                .then(a["id"].as_i64().cmp(&b["id"].as_i64()))
        });

        let mut subjects: Vec<_> = self
            .subjects
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id.0,
                    "code": &s.code,
                    "name": &s.name,
                    "color": &s.color,
                    "sort_order": s.sort_order,
                    "setters": s.setters,
                    "reviewers": s.reviewers,
                    "min_campuses": s.min_campuses,
                })
            })
            .collect();
        subjects.sort_by(|a, b| {
            a["code"]
                .as_str()
                .cmp(&b["code"].as_str())
                .then(a["id"].as_i64().cmp(&b["id"].as_i64()))
        });

        let mut teachers: Vec<_> = self
            .teachers
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id.0,
                    "full_name": &t.full_name,
                    "campus_id": t.campus_id.0,
                    "load_weight": format!("{:.4}", t.load_weight),
                    "active": t.active,
                    "note": &t.note,
                    "code": &t.code,
                    "quota_override": t.quota_override,
                    "max_tasks_per_exam_override": t.max_tasks_per_exam_override,
                })
            })
            .collect();
        teachers.sort_by(|a, b| a["id"].as_i64().cmp(&b["id"].as_i64()));

        let mut teacher_grades: Vec<_> = self
            .teacher_grades
            .iter()
            .map(|tg| {
                serde_json::json!({
                    "teacher_id": tg.teacher_id.0,
                    "grade_id": tg.grade_id.0,
                })
            })
            .collect();
        teacher_grades.sort_by(|a, b| {
            a["teacher_id"]
                .as_i64()
                .cmp(&b["teacher_id"].as_i64())
                .then(a["grade_id"].as_i64().cmp(&b["grade_id"].as_i64()))
        });

        let mut competencies: Vec<_> = self
            .competencies
            .iter()
            .map(|c| {
                serde_json::json!({
                    "teacher_id": c.teacher_id.0,
                    "subject_id": c.subject_id.0,
                    "role": c.role.as_str(),
                    "grade_scope": c.grade_scope.as_str(),
                })
            })
            .collect();
        competencies.sort_by(|a, b| {
            a["teacher_id"]
                .as_i64()
                .cmp(&b["teacher_id"].as_i64())
                .then(a["subject_id"].as_i64().cmp(&b["subject_id"].as_i64()))
                .then(a["role"].as_str().cmp(&b["role"].as_str()))
        });

        let mut exams: Vec<_> = self
            .exams
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id.0,
                    "code": &e.code,
                    "name": &e.name,
                    "sort_order": e.sort_order,
                })
            })
            .collect();
        exams.sort_by(|a, b| {
            a["sort_order"]
                .as_i64()
                .cmp(&b["sort_order"].as_i64())
                .then(a["code"].as_str().cmp(&b["code"].as_str()))
                .then(a["id"].as_i64().cmp(&b["id"].as_i64()))
        });

        let mut unavailabilities: Vec<_> = self
            .unavailabilities
            .iter()
            .map(|u| {
                serde_json::json!({
                    "teacher_id": u.teacher_id.0,
                    "exam_id": u.exam_id.0,
                    "reason": &u.reason,
                })
            })
            .collect();
        unavailabilities.sort_by(|a, b| {
            a["teacher_id"]
                .as_i64()
                .cmp(&b["teacher_id"].as_i64())
                .then(a["exam_id"].as_i64().cmp(&b["exam_id"].as_i64()))
        });

        let mut locks: Vec<_> = self
            .locks
            .iter()
            .map(|l| {
                serde_json::json!({
                    "exam_id": l.exam_id.0,
                    "grade_id": l.grade_id.0,
                    "subject_id": l.subject_id.0,
                    "teacher_id": l.teacher_id.0,
                    "role": l.role.as_ref().map(|r| r.as_str()),
                    "kind": l.kind.as_str(),
                })
            })
            .collect();
        locks.sort_by(|a, b| {
            a["exam_id"]
                .as_i64()
                .cmp(&b["exam_id"].as_i64())
                .then(a["grade_id"].as_i64().cmp(&b["grade_id"].as_i64()))
                .then(a["subject_id"].as_i64().cmp(&b["subject_id"].as_i64()))
                .then(a["teacher_id"].as_i64().cmp(&b["teacher_id"].as_i64()))
                .then(a["kind"].as_str().cmp(&b["kind"].as_str()))
        });

        let h3_enabled = self
            .rule_settings
            .iter()
            .find(|rs| rs.key == RuleKey::H3)
            .is_none_or(|rs| rs.enabled);

        let h4_setting = self.rule_settings.iter().find(|rs| rs.key == RuleKey::H4);
        let h4_enabled = h4_setting.is_none_or(|rs| rs.enabled);
        let h4_max_tasks = h4_setting
            .and_then(|rs| rs.params.get("max_tasks_per_exam").and_then(|v| v.as_u64()))
            .unwrap_or(2);
        let h4_max_setter = h4_setting
            .and_then(|rs| {
                rs.params
                    .get("max_setter_per_exam")
                    .and_then(|v| v.as_u64())
            })
            .unwrap_or(1);

        let h7_tolerance = self
            .rule_settings
            .iter()
            .find(|rs| rs.key == RuleKey::H7)
            .and_then(|rs| rs.params.get("tolerance").and_then(|v| v.as_i64()))
            .unwrap_or(1);

        let canonical_doc = serde_json::json!({
            "school_year": {
                "id": self.school_year.id.0,
                "name": &self.school_year.name,
            },
            "campuses": campuses,
            "grades": grades,
            "subjects": subjects,
            "teachers": teachers,
            "teacher_grades": teacher_grades,
            "competencies": competencies,
            "exams": exams,
            "unavailabilities": unavailabilities,
            "locks": locks,
            "hard_rules": {
                "h3_multi_campus_diversity_enabled": h3_enabled,
                "h4_single_panel_per_exam_enabled": h4_enabled,
                "h4_max_tasks_per_exam": h4_max_tasks,
                "h4_max_setter_per_exam": h4_max_setter,
                "h7_tolerance": h7_tolerance,
            },
        });

        let serialized = serde_json::to_string(&canonical_doc).unwrap_or_default();
        let hash = Sha256::digest(serialized.as_bytes());
        format!("{hash:x}")
    }

    /// Computes a canonical SHA-256 hash of problem soft RULES (weights, enabled status, params for S1..S10).
    #[must_use]
    pub fn rules_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let soft_keys = [
            RuleKey::S1,
            RuleKey::S2,
            RuleKey::S3,
            RuleKey::S4,
            RuleKey::S5,
            RuleKey::S6,
            RuleKey::S7,
            RuleKey::S8,
            RuleKey::S9,
            RuleKey::S10,
        ];

        let mut soft_rules: Vec<_> = self
            .rule_settings
            .iter()
            .filter(|rs| soft_keys.contains(&rs.key))
            .map(|rs| {
                serde_json::json!({
                    "key": rs.key.to_string(),
                    "enabled": rs.enabled,
                    "weight": format!("{:.4}", rs.weight),
                    "params": &rs.params,
                })
            })
            .collect();
        soft_rules.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));

        let canonical_doc = serde_json::json!({
            "soft_rules": soft_rules,
        });

        let serialized = serde_json::to_string(&canonical_doc).unwrap_or_default();
        let hash = Sha256::digest(serialized.as_bytes());
        format!("{hash:x}")
    }

    /// Combined canonical hash for legacy or backwards-compatible identification.
    #[must_use]
    pub fn canonical_hash(&self) -> String {
        format!("{}:{}", self.data_hash(), self.rules_hash())
    }

    /// Returns the subjects defined in the problem, or a default CHUNG subject if empty.
    #[must_use]
    pub fn effective_subjects(&self) -> Vec<Subject> {
        if self.subjects.is_empty() {
            vec![Subject {
                id: SubjectId(1),
                code: "CHUNG".to_string(),
                name: "Chung".to_string(),
                color: "slate".to_string(),
                sort_order: 1,
                setters: 2,
                reviewers: 1,
                min_campuses: 2,
            }]
        } else {
            self.subjects.clone()
        }
    }

    /// Returns all valid panels defined by the problem snapshot (Exam × Grade × Subject).
    #[must_use]
    pub fn all_panels(&self) -> Vec<PanelKey> {
        let subjects = self.effective_subjects();
        let mut panels = Vec::with_capacity(self.exams.len() * self.grades.len() * subjects.len());
        for exam in &self.exams {
            for grade in &self.grades {
                for subject in &subjects {
                    panels.push(PanelKey::new(exam.id, grade.id, subject.id));
                }
            }
        }
        panels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{GradeScope, LockKind, Role};
    use crate::domain::ids::{CampusId, ExamId, GradeId, LockId, SchoolYearId, TeacherId};

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
        let g1 = Grade {
            id: GradeId(10),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
        let s1 = Subject {
            id: SubjectId(1),
            code: "VL".to_string(),
            name: "Vật lí".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        };
        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "Nguyen Van A".to_string(),
            display_name: None,
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let tg1 = TeacherGrade {
            teacher_id: TeacherId(1),
            school_year_id: SchoolYearId(1),
            grade_id: GradeId(10),
        };
        let c_comp = Competency {
            teacher_id: TeacherId(1),
            subject_id: SubjectId(1),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        };
        let e1 = Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };
        let u1 = Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: Some("Leave".to_string()),
        };
        let lock1 = Lock {
            id: LockId(1),
            exam_id: ExamId(1),
            grade_id: GradeId(10),
            subject_id: SubjectId(1),
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        };

        Problem {
            school_year: sy,
            campuses: vec![c1],
            grades: vec![g1],
            subjects: vec![s1],
            teachers: vec![t1],
            teacher_grades: vec![tg1],
            competencies: vec![c_comp],
            exams: vec![e1],
            unavailabilities: vec![u1],
            locks: vec![lock1],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    #[test]
    fn test_valid_problem_passes() {
        let problem = make_valid_problem();
        let errors = problem.validate();
        assert!(errors.is_empty(), "expected no errors, got {errors:?}");
    }

    #[test]
    fn test_unknown_campus_error() {
        let mut problem = make_valid_problem();
        problem.teachers[0].campus_id = CampusId(999);
        let errors = problem.validate();
        assert!(errors.contains(&ValidationError::UnknownCampus(TeacherId(1), CampusId(999))));
    }

    #[test]
    fn test_invalid_load_weight_error() {
        let mut problem = make_valid_problem();
        problem.teachers[0].load_weight = 1.5;
        let errors = problem.validate();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidLoadWeight(TeacherId(1), _))));
    }

    #[test]
    fn test_duplicate_exam_code_error() {
        let mut problem = make_valid_problem();
        let e2 = Exam {
            id: ExamId(2),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Duplicate GK1".to_string(),
            sort_order: 2,
        };
        problem.exams.push(e2);
        let errors = problem.validate();
        assert!(errors.contains(&ValidationError::DuplicateExamCode("GK1".to_string())));
    }

    #[test]
    fn test_lock_unknown_teacher() {
        let mut problem = make_valid_problem();
        problem.locks[0].teacher_id = TeacherId(777);
        let errors = problem.validate();
        assert!(errors.contains(&ValidationError::LockUnknownTeacher(TeacherId(777))));
    }

    #[test]
    fn test_unavailability_unknown_exam() {
        let mut problem = make_valid_problem();
        problem.unavailabilities[0].exam_id = ExamId(888);
        let errors = problem.validate();
        assert!(errors.contains(&ValidationError::UnavailabilityUnknownExam(ExamId(888))));
    }

    #[test]
    fn test_problem_serde_roundtrip() {
        let problem = make_valid_problem();
        let json = serde_json::to_string(&problem).unwrap();
        let deserialized: Problem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, problem);
    }

    #[test]
    fn test_canonical_hash_stability_under_reordering() {
        let mut p1 = make_valid_problem();
        let mut p2 = make_valid_problem();

        // Add extra items in different order
        let t2 = Teacher {
            id: TeacherId(2),
            full_name: "Tran Thi B".to_string(),
            display_name: None,
            campus_id: CampusId(1),
            load_weight: 0.8,
            active: true,
            quota_override: None,
            max_tasks_per_exam_override: None,
            note: None,
            code: Some("GV002".to_string()),
        };
        p1.teachers.push(t2.clone());
        p2.teachers.insert(0, t2);

        // Reverse exams
        let e2 = Exam {
            id: ExamId(2),
            school_year_id: SchoolYearId(1),
            code: "CK1".to_string(),
            name: "Cuoi Ki 1".to_string(),
            sort_order: 2,
        };
        p1.exams.push(e2.clone());
        p2.exams.insert(0, e2);

        assert_eq!(p1.data_hash(), p2.data_hash());
        assert_eq!(p1.rules_hash(), p2.rules_hash());
        assert_eq!(p1.canonical_hash(), p2.canonical_hash());
    }

    #[test]
    fn test_canonical_hash_changes_on_modification() {
        let p1 = make_valid_problem();
        let d1 = p1.data_hash();
        let r1 = p1.rules_hash();

        // 1. Data change (teacher weight) changes data_hash but NOT rules_hash
        let mut p2 = make_valid_problem();
        p2.teachers[0].load_weight = 0.5;
        assert_ne!(d1, p2.data_hash());
        assert_eq!(r1, p2.rules_hash());

        // 2. Soft rule change changes rules_hash but NOT data_hash
        let mut p3 = make_valid_problem();
        let s1_idx = p3
            .rule_settings
            .iter()
            .position(|r| r.key == RuleKey::S1)
            .unwrap();
        p3.rule_settings[s1_idx].weight = 50.0;
        assert_eq!(d1, p3.data_hash());
        assert_ne!(r1, p3.rules_hash());

        // 3. Campus rename changes data_hash but NOT rules_hash
        let mut p4 = make_valid_problem();
        p4.campuses[0].name = "Renamed Campus".to_string();
        assert_ne!(d1, p4.data_hash());
        assert_eq!(r1, p4.rules_hash());

        // 4. Unavailability changes data_hash but NOT rules_hash
        let mut p5 = make_valid_problem();
        p5.unavailabilities.push(Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: Some("Off".to_string()),
        });
        assert_ne!(d1, p5.data_hash());
        assert_eq!(r1, p5.rules_hash());

        // 5. Hard rule H4 toggle changes data_hash but NOT rules_hash
        let mut p6 = make_valid_problem();
        let h4_idx = p6
            .rule_settings
            .iter()
            .position(|r| r.key == RuleKey::H4)
            .unwrap();
        p6.rule_settings[h4_idx].enabled = false;
        assert_ne!(d1, p6.data_hash());
        assert_eq!(r1, p6.rules_hash());

        // 6. Hard rule H7 tolerance changes data_hash but NOT rules_hash
        let mut p7 = make_valid_problem();
        let h7_idx = p7
            .rule_settings
            .iter()
            .position(|r| r.key == RuleKey::H7)
            .unwrap();
        p7.rule_settings[h7_idx].params = serde_json::json!({ "tolerance": 2 });
        assert_ne!(d1, p7.data_hash());
        assert_eq!(r1, p7.rules_hash());
    }
}
