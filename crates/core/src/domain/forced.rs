//! Forced placements discovery and fixpoint propagation.
//!
//! When the eligible set of teachers for a panel's role has exactly as many teachers
//! as available seats, those placements are structurally forced. They are propagated
//! to a fixpoint by removing forced teachers from the other role in the same panel.

use super::entities::{GradeScope, LockKind, PanelKey, Placement, Role};
use super::ids::{ExamId, GradeId, SubjectId, TeacherId};
use super::problem::Problem;
use std::collections::{HashMap, HashSet};

/// Returns whether a teacher is eligible for a specific role on an exam panel.
#[must_use]
pub fn is_teacher_eligible(
    problem: &Problem,
    teacher_id: TeacherId,
    exam_id: ExamId,
    grade_id: GradeId,
    subject_id: SubjectId,
    role: Role,
) -> bool {
    let teacher = match problem.teachers.iter().find(|t| t.id == teacher_id) {
        Some(t) if t.active && t.load_weight > 0.0 => t,
        _ => return false,
    };

    // Unavailability in this exam term
    if problem
        .unavailabilities
        .iter()
        .any(|u| u.teacher_id == teacher.id && u.exam_id == exam_id)
    {
        return false;
    }

    // Must have matching competency
    let comp = problem
        .competencies
        .iter()
        .find(|c| c.teacher_id == teacher.id && c.subject_id == subject_id && c.role == role);

    let comp = match comp {
        Some(c) => c,
        None => return false,
    };

    // Scope check: Taught requires teacher to teach this grade this year
    if comp.grade_scope == GradeScope::Taught {
        let teaches_grade = problem.teacher_grades.iter().any(|tg| {
            tg.school_year_id == problem.school_year.id
                && tg.teacher_id == teacher.id
                && tg.grade_id == grade_id
        });
        if !teaches_grade {
            return false;
        }
    }

    // FORBID lock check
    let is_forbidden = problem.locks.iter().any(|lock| {
        lock.exam_id == exam_id
            && lock.grade_id == grade_id
            && lock.subject_id == subject_id
            && lock.teacher_id == teacher.id
            && lock.kind == LockKind::Forbid
            && (lock.role.is_none() || lock.role == Some(role))
    });

    !is_forbidden
}

/// Discovers structurally forced placements across all panels using fixpoint propagation.
///
/// Returns `Ok(Vec<Placement>)` containing all forced placements, or `Err(String)` if
/// the instance is strictly infeasible due to insufficient eligible teachers for a seat.
pub fn find_forced_placements(problem: &Problem) -> Result<Vec<Placement>, String> {
    let mut eligible: HashMap<(PanelKey, Role), HashSet<TeacherId>> = HashMap::new();
    let mut seats_needed: HashMap<(PanelKey, Role), usize> = HashMap::new();
    let mut assigned: HashMap<(PanelKey, Role), Vec<TeacherId>> = HashMap::new();

    // 1. Initialize panels and eligibility
    for exam in &problem.exams {
        for grade in &problem.grades {
            for subject in &problem.subjects {
                let panel = PanelKey::new(exam.id, grade.id, subject.id);
                seats_needed.insert((panel, Role::Setter), subject.setters as usize);
                seats_needed.insert((panel, Role::Reviewer), subject.reviewers as usize);

                let mut setters_elig = HashSet::new();
                let mut reviewers_elig = HashSet::new();

                for t in &problem.teachers {
                    if is_teacher_eligible(
                        problem,
                        t.id,
                        exam.id,
                        grade.id,
                        subject.id,
                        Role::Setter,
                    ) {
                        setters_elig.insert(t.id);
                    }
                    if is_teacher_eligible(
                        problem,
                        t.id,
                        exam.id,
                        grade.id,
                        subject.id,
                        Role::Reviewer,
                    ) {
                        reviewers_elig.insert(t.id);
                    }
                }

                eligible.insert((panel, Role::Setter), setters_elig);
                eligible.insert((panel, Role::Reviewer), reviewers_elig);
            }
        }
    }

    // 2. Pre-place PIN locks
    for lock in &problem.locks {
        if lock.kind == LockKind::Pin {
            let panel = PanelKey::new(lock.exam_id, lock.grade_id, lock.subject_id);
            if let Some(role) = lock.role {
                let current = assigned.entry((panel, role)).or_default();
                if !current.contains(&lock.teacher_id) {
                    current.push(lock.teacher_id);
                }
                // Remove from the other role in the same panel
                let other_role = if role == Role::Setter {
                    Role::Reviewer
                } else {
                    Role::Setter
                };
                if let Some(other_set) = eligible.get_mut(&(panel, other_role)) {
                    other_set.remove(&lock.teacher_id);
                }
            }
        }
    }

    // Check for excess PIN locks upfront before candidate availability checks
    for (&(panel, role), &target_seats) in &seats_needed {
        let current_count = assigned.get(&(panel, role)).map_or(0, |v| v.len());
        if current_count > target_seats {
            return Err(format!(
                "Infeasible: Panel {:?} role {:?} has {} PIN locks exceeding seat count {}",
                panel, role, current_count, target_seats
            ));
        }
    }

    // 3. Fixpoint propagation
    loop {
        let mut changed = false;

        for (&(panel, role), &target_seats) in &seats_needed {
            let current = assigned.entry((panel, role)).or_default();
            let already_count = current.len();

            if already_count > target_seats {
                return Err(format!(
                    "Infeasible: Panel {:?} role {:?} has {} PIN locks exceeding seat count {}",
                    panel, role, already_count, target_seats
                ));
            }

            let needed = target_seats - already_count;
            if needed == 0 {
                continue;
            }

            let elig_set = eligible.get(&(panel, role)).cloned().unwrap_or_default();
            let available_candidates: Vec<TeacherId> = elig_set
                .into_iter()
                .filter(|tid| !current.contains(tid))
                .collect();

            if available_candidates.len() < needed {
                return Err(format!(
                    "Infeasible: Panel {:?} role {:?} requires {} seats, but only {} eligible teachers available",
                    panel, role, needed, available_candidates.len()
                ));
            }

            // Exactly as many candidates as needed -> all are forced!
            if available_candidates.len() == needed {
                let other_role = if role == Role::Setter {
                    Role::Reviewer
                } else {
                    Role::Setter
                };
                for tid in available_candidates {
                    current.push(tid);
                    if let Some(other_set) = eligible.get_mut(&(panel, other_role)) {
                        other_set.remove(&tid);
                    }
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    // 4. Construct sorted Placement list
    let mut placements = Vec::new();
    for (&(panel, role), teachers) in &assigned {
        for (pos, &tid) in teachers.iter().enumerate() {
            placements.push(Placement::new(panel, role, pos, tid));
        }
    }

    placements.sort_by(|a, b| {
        a.panel
            .exam_id
            .cmp(&b.panel.exam_id)
            .then(a.panel.grade_id.cmp(&b.panel.grade_id))
            .then(a.panel.subject_id.cmp(&b.panel.subject_id))
            .then((a.role as u8).cmp(&(b.role as u8)))
            .then(a.position.cmp(&b.position))
    });

    Ok(placements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    #[test]
    fn test_forced_placement_single_candidate_propagates() {
        let school_year = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let exam = Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };
        let grade = Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
        let subject = Subject {
            id: SubjectId(1),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 1,
            reviewers: 1,
            min_campuses: 1,
        };
        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "T Nghĩa".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let t2 = Teacher {
            id: TeacherId(2),
            full_name: "C Hiền".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let competencies = vec![
            Competency {
                teacher_id: TeacherId(1),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Any,
            },
            Competency {
                teacher_id: TeacherId(2),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Any,
            },
        ];

        let problem = Problem {
            school_year,
            campuses: vec![Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Phân hiệu 1".to_string(),
                color: "blue".to_string(),
            }],
            grades: vec![grade],
            subjects: vec![subject],
            teachers: vec![t1, t2],
            teacher_grades: vec![],
            competencies,
            exams: vec![exam],
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        };

        let forced = find_forced_placements(&problem).expect("find forced");
        assert_eq!(forced.len(), 2);
        assert!(forced
            .iter()
            .any(|p| p.teacher_id == TeacherId(1) && p.role == Role::Setter));
        assert!(forced
            .iter()
            .any(|p| p.teacher_id == TeacherId(2) && p.role == Role::Reviewer));
    }

    #[test]
    fn test_forced_placement_cascade_propagation() {
        // T1 and T2 are both eligible for Setter.
        // But only T1 is eligible for Reviewer (needed=1).
        // Since T1 is forced into Reviewer, T1 cannot take Setter.
        // That leaves only T2 for Setter (needed=1), cascading T2 to Setter!
        let school_year = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let exam = Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };
        let grade = Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
        let subject = Subject {
            id: SubjectId(1),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 1,
            reviewers: 1,
            min_campuses: 1,
        };
        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "T1".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let t2 = Teacher {
            id: TeacherId(2),
            full_name: "T2".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        // Both T1 and T2 have Setter competency; only T1 has Reviewer competency
        let competencies = vec![
            Competency {
                teacher_id: TeacherId(1),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Any,
            },
            Competency {
                teacher_id: TeacherId(2),
                subject_id: SubjectId(1),
                role: Role::Setter,
                grade_scope: GradeScope::Any,
            },
            Competency {
                teacher_id: TeacherId(1),
                subject_id: SubjectId(1),
                role: Role::Reviewer,
                grade_scope: GradeScope::Any,
            },
        ];

        let problem = Problem {
            school_year,
            campuses: vec![Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Phân hiệu 1".to_string(),
                color: "blue".to_string(),
            }],
            grades: vec![grade],
            subjects: vec![subject],
            teachers: vec![t1, t2],
            teacher_grades: vec![],
            competencies,
            exams: vec![exam],
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        };

        let forced = find_forced_placements(&problem).expect("find forced cascade");
        assert_eq!(forced.len(), 2);
        // T1 must be Reviewer, T2 must be Setter
        assert!(forced
            .iter()
            .any(|p| p.teacher_id == TeacherId(1) && p.role == Role::Reviewer));
        assert!(forced
            .iter()
            .any(|p| p.teacher_id == TeacherId(2) && p.role == Role::Setter));
    }

    #[test]
    fn test_forced_placement_infeasible_zero_candidates() {
        let school_year = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let exam = Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };
        let grade = Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
        let subject = Subject {
            id: SubjectId(1),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 1,
            reviewers: 1,
            min_campuses: 1,
        };

        let problem = Problem {
            school_year,
            campuses: vec![Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Phân hiệu 1".to_string(),
                color: "blue".to_string(),
            }],
            grades: vec![grade],
            subjects: vec![subject],
            teachers: vec![],
            teacher_grades: vec![],
            competencies: vec![],
            exams: vec![exam],
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        };

        let res = find_forced_placements(&problem);
        assert!(res.is_err(), "Expected error due to zero candidates");
        let err_msg = res.err().unwrap();
        assert!(err_msg.contains("Infeasible"));
    }

    #[test]
    fn test_forced_placement_infeasible_excess_pins() {
        let school_year = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };
        let exam = Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        };
        let grade = Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        };
        let subject = Subject {
            id: SubjectId(1),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "blue".to_string(),
            sort_order: 1,
            setters: 1,
            reviewers: 1,
            min_campuses: 1,
        };
        let t1 = Teacher {
            id: TeacherId(1),
            full_name: "T1".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let t2 = Teacher {
            id: TeacherId(2),
            full_name: "T2".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            code: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        };
        let panel = PanelKey {
            exam_id: ExamId(1),
            grade_id: GradeId(1),
            subject_id: SubjectId(1),
        };

        // Two PIN locks for Setter when setters = 1!
        let locks = vec![
            Lock {
                id: LockId(1),
                exam_id: panel.exam_id,
                grade_id: panel.grade_id,
                subject_id: panel.subject_id,
                teacher_id: TeacherId(1),
                role: Some(Role::Setter),
                kind: LockKind::Pin,
            },
            Lock {
                id: LockId(2),
                exam_id: panel.exam_id,
                grade_id: panel.grade_id,
                subject_id: panel.subject_id,
                teacher_id: TeacherId(2),
                role: Some(Role::Setter),
                kind: LockKind::Pin,
            },
        ];

        let problem = Problem {
            school_year,
            campuses: vec![Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Phân hiệu 1".to_string(),
                color: "blue".to_string(),
            }],
            grades: vec![grade],
            subjects: vec![subject],
            teachers: vec![t1, t2],
            teacher_grades: vec![],
            competencies: vec![],
            exams: vec![exam],
            unavailabilities: vec![],
            locks,
            rule_settings: RuleSetting::default_settings(),
        };

        let res = find_forced_placements(&problem);
        assert!(res.is_err(), "Expected error due to excess pins");
        let err_msg = res.err().unwrap();
        assert!(
            err_msg.contains("Infeasible") && err_msg.contains("PIN locks exceeding seat count")
        );
    }
}
