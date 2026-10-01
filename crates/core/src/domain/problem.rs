//! Problem snapshot and structural validation.

use super::entities::{
    Campus, Exam, Grade, Lock, RuleKey, RuleSetting, SchoolYear, Teacher, TeacherGrade,
    Unavailability,
};
use super::ids::{CampusId, ExamId, GradeId, TeacherId};
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

    #[error("TeacherGrade references unknown teacher {0}")]
    TeacherGradeUnknownTeacher(TeacherId),

    #[error("TeacherGrade references unknown grade {0}")]
    TeacherGradeUnknownGrade(GradeId),

    #[error("Duplicate teacher grade entry for teacher {0}, grade {1}")]
    DuplicateTeacherGrade(TeacherId, GradeId),

    #[error("Lock references unknown teacher {0}")]
    LockUnknownTeacher(TeacherId),

    #[error("Lock references unknown exam {0}")]
    LockUnknownExam(ExamId),

    #[error("Lock references unknown grade {0}")]
    LockUnknownGrade(GradeId),

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Problem {
    pub school_year: SchoolYear,
    pub campuses: Vec<Campus>,
    pub grades: Vec<Grade>,
    pub teachers: Vec<Teacher>,
    pub teacher_grades: Vec<TeacherGrade>,
    pub exams: Vec<Exam>,
    pub unavailabilities: Vec<Unavailability>,
    pub locks: Vec<Lock>,
    pub rule_settings: Vec<RuleSetting>,
}

impl Problem {
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

        // 3. Exams
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

        // 4. Teachers
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
        }

        // 5. TeacherGrades
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

        // 6. Unavailabilities
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

        // 7. Locks
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
        }

        // 8. RuleSettings
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{LockKind, Role};
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
        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "Nguyen Van A".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
        };
        let tg1 = TeacherGrade {
            teacher_id: TeacherId(1),
            school_year_id: SchoolYearId(1),
            grade_id: GradeId(10),
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
            teacher_id: TeacherId(1),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        };

        Problem {
            school_year: sy,
            campuses: vec![c1],
            grades: vec![g1],
            teachers: vec![t1],
            teacher_grades: vec![tg1],
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
}
