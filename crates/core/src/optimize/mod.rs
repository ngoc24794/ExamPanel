//! Local search optimization and incremental state evaluation.
//!
//! Provides:
//! - Dense incremental state representation with O(1) counters for S1–S8
//! - Feasibility-preserving move operators M1–M4
//! - Simulated annealing engine with auto-calibrated T0 and geometric cooling

pub mod anneal;
pub mod moves;
pub mod state;

pub use anneal::{run_simulated_annealing, Budget, Progress};
pub use moves::LocalMove;
pub use state::{DensePanel, IncrementalState, SlotRole};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::solver::{solve_hard, SolveOptions};
    use crate::validate::{validate_assignments, ValidateOptions};
    use rand_chacha::ChaCha8Rng;
    use rand_core::{RngCore, SeedableRng};

    fn make_seed_demo_problem() -> Problem {
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };

        let campuses = vec![
            Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Co so 1".to_string(),
                color: "#1e40af".to_string(),
            },
            Campus {
                id: CampusId(2),
                code: "CS2".to_string(),
                name: "Co so 2".to_string(),
                color: "#059669".to_string(),
            },
            Campus {
                id: CampusId(3),
                code: "CS3".to_string(),
                name: "Co so 3".to_string(),
                color: "#d97706".to_string(),
            },
            Campus {
                id: CampusId(4),
                code: "CS4".to_string(),
                name: "Co so 4".to_string(),
                color: "#dc2626".to_string(),
            },
        ];

        let grades = vec![
            Grade {
                id: GradeId(1),
                code: 10,
                name: "Khoi 10".to_string(),
                sort_order: 1,
            },
            Grade {
                id: GradeId(2),
                code: 11,
                name: "Khoi 11".to_string(),
                sort_order: 2,
            },
            Grade {
                id: GradeId(3),
                code: 12,
                name: "Khoi 12".to_string(),
                sort_order: 3,
            },
        ];

        let exams = vec![
            Exam {
                id: ExamId(1),
                school_year_id: sy.id,
                code: "GK1".to_string(),
                name: "Giua ky 1".to_string(),
                sort_order: 1,
            },
            Exam {
                id: ExamId(2),
                school_year_id: sy.id,
                code: "CK1".to_string(),
                name: "Cuoi ky 1".to_string(),
                sort_order: 2,
            },
            Exam {
                id: ExamId(3),
                school_year_id: sy.id,
                code: "GK2".to_string(),
                name: "Giua ky 2".to_string(),
                sort_order: 3,
            },
            Exam {
                id: ExamId(4),
                school_year_id: sy.id,
                code: "CK2".to_string(),
                name: "Cuoi ky 2".to_string(),
                sort_order: 4,
            },
        ];

        let raw_teachers = [
            (1, "Nguyen Van An", 1, 1.0),
            (2, "Tran Thi Binh", 1, 1.0),
            (3, "Le Hoang Cuong", 2, 1.0),
            (4, "Pham Minh Duc", 2, 1.0),
            (5, "Hoang Thu Giang", 3, 1.0),
            (6, "Vu Hai Ha", 3, 0.5),
            (7, "Dang Quoc Hung", 3, 1.0),
            (8, "Bui Thi Lan", 4, 1.0),
            (9, "Do Tuan Minh", 4, 1.0),
            (10, "Ngo Phuong Nam", 1, 1.0),
            (11, "Duong Thuy Trang", 2, 1.0),
        ];

        let teachers: Vec<Teacher> = raw_teachers
            .iter()
            .map(|&(id, name, campus_id, weight)| Teacher {
                id: TeacherId(id),
                full_name: name.to_string(),
                campus_id: CampusId(campus_id),
                load_weight: weight,
                active: true,
                note: None,
            })
            .collect();

        let raw_teacher_grades = [
            (1, vec![10, 11]),
            (2, vec![11, 12]),
            (3, vec![10, 12]),
            (4, vec![11]),
            (5, vec![10]),
            (6, vec![11]),
            (7, vec![12]),
            (8, vec![10]),
            (9, vec![11]),
            (10, vec![12]),
            (11, vec![12]),
        ];

        let mut teacher_grades = Vec::new();
        for &(tid, ref g_codes) in &raw_teacher_grades {
            for &g_code in g_codes {
                let gid = match g_code {
                    10 => GradeId(1),
                    11 => GradeId(2),
                    12 => GradeId(3),
                    _ => unreachable!(),
                };
                teacher_grades.push(TeacherGrade {
                    teacher_id: TeacherId(tid),
                    school_year_id: sy.id,
                    grade_id: gid,
                });
            }
        }

        let unavailabilities = vec![Unavailability {
            teacher_id: TeacherId(6),
            exam_id: ExamId(3),
            reason: Some("Medical".to_string()),
        }];

        Problem {
            school_year: sy,
            campuses,
            grades,
            exams,
            teachers,
            teacher_grades,
            unavailabilities,
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        }
    }

    fn make_synthetic_40_problem() -> Problem {
        let sy = SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        };

        let campuses: Vec<Campus> = (1..=6)
            .map(|c| Campus {
                id: CampusId(c),
                code: format!("CS{c}"),
                name: format!("Co so {c}"),
                color: "#1e40af".to_string(),
            })
            .collect();

        let grades: Vec<Grade> = (1..=3)
            .map(|g| Grade {
                id: GradeId(g),
                code: (9 + g) as i32,
                name: format!("Khoi {}", 9 + g),
                sort_order: g as i32,
            })
            .collect();

        let exams: Vec<Exam> = (1..=4)
            .map(|e| Exam {
                id: ExamId(e),
                school_year_id: sy.id,
                code: format!("EX{e}"),
                name: format!("Ky thi {e}"),
                sort_order: e as i32,
            })
            .collect();

        let mut teachers = Vec::with_capacity(40);
        let mut teacher_grades = Vec::new();

        for tid in 1..=40 {
            let cid = ((tid - 1) % 6) + 1;
            teachers.push(Teacher {
                id: TeacherId(tid),
                full_name: format!("Giao vien {tid}"),
                campus_id: CampusId(cid),
                load_weight: if tid % 10 == 0 { 0.5 } else { 1.0 },
                active: true,
                note: None,
            });

            let g1 = ((tid - 1) % 3) + 1;
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: sy.id,
                grade_id: GradeId(g1),
            });
            if tid % 2 == 0 {
                let g2 = (tid % 3) + 1;
                if g2 != g1 {
                    teacher_grades.push(TeacherGrade {
                        teacher_id: TeacherId(tid),
                        school_year_id: sy.id,
                        grade_id: GradeId(g2),
                    });
                }
            }
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
    fn test_d1_incremental_state_consistency_demo_10k_moves() {
        let problem = make_seed_demo_problem();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard demo");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let mut accepted_moves = 0;
        while accepted_moves < 10_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let delta = state.try_apply_move(m);
                let accept = rng.next_u32() % 2 == 0;
                if accept {
                    accepted_moves += 1;
                } else {
                    state.revert_move(m, delta);
                }
            }
        }

        let full_eval = state.full_evaluate(&problem);
        let diff = (state.current_penalty - full_eval.total).abs();
        assert!(
            diff < 1e-9,
            "Demo 10k moves: incremental penalty {} != full eval total {} (diff: {})",
            state.current_penalty,
            full_eval.total,
            diff
        );
    }

    #[test]
    fn test_d1_incremental_state_consistency_synthetic_40_10k_moves() {
        let problem = make_synthetic_40_problem();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 777,
                time_limit_ms: 3000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard synthetic");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(67890);

        let mut accepted_moves = 0;
        while accepted_moves < 10_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let delta = state.try_apply_move(m);
                let accept = rng.next_u32() % 2 == 0;
                if accept {
                    accepted_moves += 1;
                } else {
                    state.revert_move(m, delta);
                }
            }
        }

        let full_eval = state.full_evaluate(&problem);
        let diff = (state.current_penalty - full_eval.total).abs();
        assert!(
            diff < 1e-9,
            "Synthetic 40 10k moves: incremental penalty {} != full eval total {} (diff: {})",
            state.current_penalty,
            full_eval.total,
            diff
        );
    }

    #[test]
    fn test_d2_random_walk_hard_invariants_preserved() {
        let problem = make_seed_demo_problem();
        let sol = solve_hard(
            &problem,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard demo");

        let mut state = IncrementalState::new(&problem, &sol.assignments);
        let mut rng = ChaCha8Rng::seed_from_u64(54321);

        for _ in 0..1_000 {
            if let Some(m) = state.sample_candidate_move(&mut rng) {
                let _delta = state.try_apply_move(m);
                let assignments = state.to_assignments(PlanId(1));
                let violations = validate_assignments(
                    &problem,
                    &assignments,
                    &ValidateOptions {
                        require_complete: true,
                    },
                );
                assert!(
                    violations.is_empty(),
                    "Hard constraint violated during random walk: {violations:?}"
                );
            }
        }
    }
}
