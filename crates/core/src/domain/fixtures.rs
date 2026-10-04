//! Standard test and benchmark fixtures for ExamPanel.
//!
//! Provides canonical problem snapshots and assignment matrices, including
//! Q's dataset (nocampus and synthetic_campuses variants) and multi-subject synthetic datasets.

use crate::domain::*;
use serde_json::json;

/// Variant of the canonical Q-shaped problem dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QVariant {
    /// Single placeholder campus "Chưa phân hiệu", H3 disabled, min_campuses = 1.
    NoCampus,
    /// Four campuses "Phân hiệu 1–4", H3 enabled, min_campuses = 2.
    SyntheticCampuses,
}

/// Builds the canonical Q-shaped problem dataset.
#[must_use]
pub fn make_canonical_q_problem(variant: QVariant) -> Problem {
    let school_year = SchoolYear {
        id: SchoolYearId(1),
        name: "2026-2027".to_string(),
        is_current: true,
    };

    let (campuses, h3_enabled, min_campuses) = match variant {
        QVariant::NoCampus => (
            vec![Campus {
                id: CampusId(1),
                code: "CPH".to_string(),
                name: "Chưa phân hiệu".to_string(),
                color: "palette-1".to_string(),
            }],
            false,
            1,
        ),
        QVariant::SyntheticCampuses => (
            vec![
                Campus {
                    id: CampusId(1),
                    code: "PH1".to_string(),
                    name: "Phân hiệu 1 (giả lập)".to_string(),
                    color: "palette-1".to_string(),
                },
                Campus {
                    id: CampusId(2),
                    code: "PH2".to_string(),
                    name: "Phân hiệu 2 (giả lập)".to_string(),
                    color: "palette-2".to_string(),
                },
                Campus {
                    id: CampusId(3),
                    code: "PH3".to_string(),
                    name: "Phân hiệu 3 (giả lập)".to_string(),
                    color: "palette-3".to_string(),
                },
                Campus {
                    id: CampusId(4),
                    code: "PH4".to_string(),
                    name: "Phân hiệu 4 (giả lập)".to_string(),
                    color: "palette-4".to_string(),
                },
            ],
            true,
            2,
        ),
    };

    let grades = vec![
        Grade {
            id: GradeId(1),
            code: 10,
            name: "Khối 10".to_string(),
            sort_order: 1,
        },
        Grade {
            id: GradeId(2),
            code: 11,
            name: "Khối 11".to_string(),
            sort_order: 2,
        },
        Grade {
            id: GradeId(3),
            code: 12,
            name: "Khối 12".to_string(),
            sort_order: 3,
        },
    ];

    let subjects = vec![
        Subject {
            id: SubjectId(1),
            code: "VL".to_string(),
            name: "Vật lí".to_string(),
            color: "palette-1".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses,
        },
        Subject {
            id: SubjectId(2),
            code: "CN".to_string(),
            name: "Công nghệ".to_string(),
            color: "palette-2".to_string(),
            sort_order: 2,
            setters: 1,
            reviewers: 1,
            min_campuses,
        },
    ];

    let exams = vec![
        Exam {
            id: ExamId(1),
            school_year_id: SchoolYearId(1),
            code: "GK1".to_string(),
            name: "Giữa kỳ 1".to_string(),
            sort_order: 1,
        },
        Exam {
            id: ExamId(2),
            school_year_id: SchoolYearId(1),
            code: "CK1".to_string(),
            name: "Cuối kỳ 1".to_string(),
            sort_order: 2,
        },
        Exam {
            id: ExamId(3),
            school_year_id: SchoolYearId(1),
            code: "GK2".to_string(),
            name: "Giữa kỳ 2".to_string(),
            sort_order: 3,
        },
        Exam {
            id: ExamId(4),
            school_year_id: SchoolYearId(1),
            code: "CK2".to_string(),
            name: "Cuối kỳ 2".to_string(),
            sort_order: 4,
        },
    ];

    let teacher_specs = [
        (1, "Cô Hiền", "C Hiền", "HIEN", 1),
        (2, "Cô Lài", "C Lài", "LAI", 2),
        (3, "Thầy Phúc", "T Phúc", "PHUC", 3),
        (4, "Thầy Lộc", "T Lộc", "LOC", 4),
        (5, "Cô Thư", "C Thư", "THU", 1),
        (6, "Cô Na", "C Na", "NA", 2),
        (7, "Cô Bình", "C Bình", "BINH", 3),
        (8, "Cô Quí", "C Quí", "QUI", 4),
        (9, "Cô Tú", "C Tú", "TU", 1),
        (10, "Cô Như", "C Như", "NHU", 2),
        (11, "Cô Lan", "C Lan", "LAN", 3),
        (12, "Thầy Nghĩa", "T Nghĩa", "NGHIA", 1),
    ];

    let teachers: Vec<Teacher> = teacher_specs
        .iter()
        .map(|&(id, name, dname, code, synth_cid)| {
            let campus_id = match variant {
                QVariant::NoCampus => CampusId(1),
                QVariant::SyntheticCampuses => CampusId(synth_cid),
            };
            let (quota_override, max_tasks_per_exam_override) = match id {
                8 => (None, Some(3)),
                12 => (Some(12), Some(3)),
                _ => (None, None),
            };
            Teacher {
                id: TeacherId(id),
                full_name: name.to_string(),
                campus_id,
                load_weight: 1.0,
                active: true,
                note: None,
                code: Some(code.to_string()),
                display_name: Some(dname.to_string()),
                quota_override,
                max_tasks_per_exam_override,
            }
        })
        .collect();

    let teacher_grades_spec = vec![
        (1, vec![1, 2, 3]),
        (2, vec![1, 2, 3]),
        (3, vec![1, 3]),
        (4, vec![1, 2]),
        (5, vec![2]),
        (6, vec![1, 2]),
        (7, vec![2, 3]),
        (8, vec![2, 3]),
        (9, vec![1]),
        (10, vec![1]),
        (11, vec![3]),
        (12, vec![1, 2, 3]),
    ];

    let mut teacher_grades = Vec::new();
    for (tid, gids) in teacher_grades_spec {
        for gid in gids {
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: SchoolYearId(1),
                grade_id: GradeId(gid),
            });
        }
    }

    let mut competencies = Vec::new();
    for tid in 1..=11 {
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(1),
            role: Role::Setter,
            grade_scope: GradeScope::Taught,
        });
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(1),
            role: Role::Reviewer,
            grade_scope: GradeScope::Taught,
        });
        competencies.push(Competency {
            teacher_id: TeacherId(tid),
            subject_id: SubjectId(2),
            role: Role::Reviewer,
            grade_scope: GradeScope::Any,
        });
    }
    competencies.push(Competency {
        teacher_id: TeacherId(12),
        subject_id: SubjectId(2),
        role: Role::Setter,
        grade_scope: GradeScope::Any,
    });

    let mut rule_settings = RuleSetting::default_settings();
    if let Some(h3) = rule_settings.iter_mut().find(|r| r.key == RuleKey::H3) {
        h3.enabled = h3_enabled;
        h3.params = json!({ "enabled": h3_enabled });
    }

    Problem {
        school_year,
        campuses,
        grades,
        subjects,
        teachers,
        teacher_grades,
        competencies,
        exams,
        unavailabilities: Vec::new(),
        locks: Vec::new(),
        rule_settings,
    }
}

/// Returns Q's canonical 60 manual assignments.
#[must_use]
pub fn make_q_assignments() -> Vec<Assignment> {
    let mut a = Vec::with_capacity(60);
    let mut add = |e: i64, g: i64, s: i64, r: Role, pos: usize, tid: i64| {
        a.push(Assignment {
            plan_id: PlanId(1),
            exam_id: ExamId(e),
            grade_id: GradeId(g),
            subject_id: SubjectId(s),
            teacher_id: TeacherId(tid),
            role: r,
            position: pos,
        });
    };

    // GK1 (e = 1)
    // 10 (g = 1): VL [1, 2 | 3], CN [12 | 1]
    add(1, 1, 1, Role::Setter, 0, 1);
    add(1, 1, 1, Role::Setter, 1, 2);
    add(1, 1, 1, Role::Reviewer, 0, 3);
    add(1, 1, 2, Role::Setter, 0, 12);
    add(1, 1, 2, Role::Reviewer, 0, 1);
    // 11 (g = 2): VL [4, 5 | 6], CN [12 | 2]
    add(1, 2, 1, Role::Setter, 0, 4);
    add(1, 2, 1, Role::Setter, 1, 5);
    add(1, 2, 1, Role::Reviewer, 0, 6);
    add(1, 2, 2, Role::Setter, 0, 12);
    add(1, 2, 2, Role::Reviewer, 0, 2);
    // 12 (g = 3): VL [7, 3 | 8], CN [12 | 4]
    add(1, 3, 1, Role::Setter, 0, 7);
    add(1, 3, 1, Role::Setter, 1, 3);
    add(1, 3, 1, Role::Reviewer, 0, 8);
    add(1, 3, 2, Role::Setter, 0, 12);
    add(1, 3, 2, Role::Reviewer, 0, 4);

    // CK1 (e = 2)
    // 10: VL [1, 9 | 10], CN [12 | 9]
    add(2, 1, 1, Role::Setter, 0, 1);
    add(2, 1, 1, Role::Setter, 1, 9);
    add(2, 1, 1, Role::Reviewer, 0, 10);
    add(2, 1, 2, Role::Setter, 0, 12);
    add(2, 1, 2, Role::Reviewer, 0, 9);
    // 11: VL [2, 7 | 5], CN [12 | 11]
    add(2, 2, 1, Role::Setter, 0, 2);
    add(2, 2, 1, Role::Setter, 1, 7);
    add(2, 2, 1, Role::Reviewer, 0, 5);
    add(2, 2, 2, Role::Setter, 0, 12);
    add(2, 2, 2, Role::Reviewer, 0, 11);
    // 12: VL [11, 8 | 2], CN [12 | 7]
    add(2, 3, 1, Role::Setter, 0, 11);
    add(2, 3, 1, Role::Setter, 1, 8);
    add(2, 3, 1, Role::Reviewer, 0, 2);
    add(2, 3, 2, Role::Setter, 0, 12);
    add(2, 3, 2, Role::Reviewer, 0, 7);

    // GK2 (e = 3)
    // 10: VL [9, 10 | 4], CN [12 | 10]
    add(3, 1, 1, Role::Setter, 0, 9);
    add(3, 1, 1, Role::Setter, 1, 10);
    add(3, 1, 1, Role::Reviewer, 0, 4);
    add(3, 1, 2, Role::Setter, 0, 12);
    add(3, 1, 2, Role::Reviewer, 0, 10);
    // 11: VL [6, 5 | 1], CN [12 | 5]
    add(3, 2, 1, Role::Setter, 0, 6);
    add(3, 2, 1, Role::Setter, 1, 5);
    add(3, 2, 1, Role::Reviewer, 0, 1);
    add(3, 2, 2, Role::Setter, 0, 12);
    add(3, 2, 2, Role::Reviewer, 0, 5);
    // 12: VL [11, 2 | 8], CN [12 | 3]
    add(3, 3, 1, Role::Setter, 0, 11);
    add(3, 3, 1, Role::Setter, 1, 2);
    add(3, 3, 1, Role::Reviewer, 0, 8);
    add(3, 3, 2, Role::Setter, 0, 12);
    add(3, 3, 2, Role::Reviewer, 0, 3);

    // CK2 (e = 4)
    // 10: VL [6, 10 | 9], CN [12 | 8]
    add(4, 1, 1, Role::Setter, 0, 6);
    add(4, 1, 1, Role::Setter, 1, 10);
    add(4, 1, 1, Role::Reviewer, 0, 9);
    add(4, 1, 2, Role::Setter, 0, 12);
    add(4, 1, 2, Role::Reviewer, 0, 8);
    // 11: VL [4, 8 | 7], CN [12 | 6]
    add(4, 2, 1, Role::Setter, 0, 4);
    add(4, 2, 1, Role::Setter, 1, 8);
    add(4, 2, 1, Role::Reviewer, 0, 7);
    add(4, 2, 2, Role::Setter, 0, 12);
    add(4, 2, 2, Role::Reviewer, 0, 6);
    // 12: VL [3, 1 | 11], CN [12 | 8]
    add(4, 3, 1, Role::Setter, 0, 3);
    add(4, 3, 1, Role::Setter, 1, 1);
    add(4, 3, 1, Role::Reviewer, 0, 11);
    add(4, 3, 2, Role::Setter, 0, 12);
    add(4, 3, 2, Role::Reviewer, 0, 8);

    a
}

/// Builds a 40-teacher, 2-subject synthetic benchmark problem.
#[must_use]
pub fn make_synthetic_40_problem_2sub() -> Problem {
    let sy = SchoolYear {
        id: SchoolYearId(1),
        name: "2026-2027".to_string(),
        is_current: true,
    };

    let campuses: Vec<Campus> = (1..=4)
        .map(|c| Campus {
            id: CampusId(c),
            code: format!("PH{c}"),
            name: format!("Phân hiệu {c}"),
            color: "palette-1".to_string(),
        })
        .collect();

    let grades: Vec<Grade> = (1..=3)
        .map(|g| Grade {
            id: GradeId(g),
            code: (9 + g) as i32,
            name: format!("Khối {}", 9 + g),
            sort_order: g as i32,
        })
        .collect();

    let exams: Vec<Exam> = (1..=4)
        .map(|e| Exam {
            id: ExamId(e),
            school_year_id: sy.id,
            code: format!("EX{e}"),
            name: format!("Kỳ thi {e}"),
            sort_order: e as i32,
        })
        .collect();

    let mut teachers = Vec::with_capacity(40);
    let mut teacher_grades = Vec::new();

    for tid in 1..=40 {
        let cid = ((tid - 1) % 4) + 1;
        teachers.push(Teacher {
            id: TeacherId(tid),
            full_name: format!("Giáo viên {tid}"),
            display_name: None,
            campus_id: CampusId(cid),
            load_weight: 1.0,
            active: true,
            quota_override: None,
            max_tasks_per_exam_override: None,
            note: None,
            code: None,
        });

        let g1 = ((tid - 1) % 3) + 1;
        teacher_grades.push(TeacherGrade {
            teacher_id: TeacherId(tid),
            school_year_id: sy.id,
            grade_id: GradeId(g1),
        });
        if tid % 2 == 0 {
            let g2 = (g1 % 3) + 1;
            teacher_grades.push(TeacherGrade {
                teacher_id: TeacherId(tid),
                school_year_id: sy.id,
                grade_id: GradeId(g2),
            });
        }
    }

    let sub1 = Subject {
        id: SubjectId(1),
        code: "SUB1".to_string(),
        name: "Môn 1".to_string(),
        color: "blue".to_string(),
        sort_order: 1,
        setters: 2,
        reviewers: 1,
        min_campuses: 2,
    };
    let sub2 = Subject {
        id: SubjectId(2),
        code: "SUB2".to_string(),
        name: "Môn 2".to_string(),
        color: "green".to_string(),
        sort_order: 2,
        setters: 1,
        reviewers: 1,
        min_campuses: 2,
    };

    let mut competencies = Vec::new();
    for t in &teachers {
        if t.id.0 <= 26 {
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub1.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub1.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }
        if t.id.0 >= 15 {
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub2.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            competencies.push(Competency {
                teacher_id: t.id,
                subject_id: sub2.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }
    }

    Problem {
        school_year: sy,
        campuses,
        grades,
        subjects: vec![sub1, sub2],
        exams,
        teachers,
        teacher_grades,
        competencies,
        unavailabilities: vec![],
        locks: vec![],
        rule_settings: RuleSetting::default_settings(),
    }
}

/// Real campus and grade data from Q to override synthetic fixtures without code changes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct QRealData {
    /// Optional campuses to override default campuses.
    #[serde(default)]
    pub campuses: Vec<Campus>,
    /// Optional teacher-to-campus mapping. Key can be teacher code (e.g. "HIEN") or teacher id ("1").
    /// Value can be campus code (e.g. "PH1") or campus id ("1").
    #[serde(default)]
    pub teacher_campuses: std::collections::HashMap<String, String>,
    /// Optional teacher-to-grades mapping. Key can be teacher code or id.
    /// Value is a list of grade codes (e.g. [10, 11, 12]).
    #[serde(default)]
    pub teacher_grades: std::collections::HashMap<String, Vec<i64>>,
}

impl QRealData {
    /// Applies this real data overlay onto a problem instance.
    pub fn apply_to_problem(&self, problem: &mut Problem) {
        if !self.campuses.is_empty() {
            problem.campuses = self.campuses.clone();
        }

        // Apply teacher campuses
        for (teacher_key, campus_key) in &self.teacher_campuses {
            let campus_id = problem
                .campuses
                .iter()
                .find(|c| {
                    c.code.eq_ignore_ascii_case(campus_key) || c.id.0.to_string() == *campus_key
                })
                .map(|c| c.id);

            if let Some(cid) = campus_id {
                if let Some(t) = problem.teachers.iter_mut().find(|t| {
                    t.code
                        .as_deref()
                        .map(|c| c.eq_ignore_ascii_case(teacher_key))
                        .unwrap_or(false)
                        || t.id.0.to_string() == *teacher_key
                }) {
                    t.campus_id = cid;
                }
            }
        }

        // Apply teacher grades
        for (teacher_key, grade_codes) in &self.teacher_grades {
            let teacher_id = problem
                .teachers
                .iter()
                .find(|t| {
                    t.code
                        .as_deref()
                        .map(|c| c.eq_ignore_ascii_case(teacher_key))
                        .unwrap_or(false)
                        || t.id.0.to_string() == *teacher_key
                })
                .map(|t| t.id);

            if let Some(tid) = teacher_id {
                problem.teacher_grades.retain(|tg| tg.teacher_id != tid);
                for gcode in grade_codes {
                    if let Some(grade) = problem
                        .grades
                        .iter()
                        .find(|g| (g.code as i64) == *gcode || g.id.0 == *gcode)
                    {
                        problem.teacher_grades.push(TeacherGrade {
                            teacher_id: tid,
                            school_year_id: problem.school_year.id,
                            grade_id: grade.id,
                        });
                    }
                }
            }
        }
    }

    /// Loads QRealData from a JSON string.
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

/// Default path where real campus/grade data from Q may be placed.
pub const DEFAULT_Q_REAL_DATA_PATH: &str = "data/q_real_data.json";

/// Loads real data from the environment variable `EXAMPANEL_Q_DATA_PATH` or the default path if present.
#[must_use]
pub fn load_q_real_data_from_file_or_env() -> Option<QRealData> {
    let path = std::env::var("EXAMPANEL_Q_DATA_PATH")
        .unwrap_or_else(|_| DEFAULT_Q_REAL_DATA_PATH.to_string());
    let path = std::path::Path::new(&path);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(data) = QRealData::from_json_str(&content) {
                return Some(data);
            }
        }
    }
    None
}

/// Builds canonical Q problem, optionally applying real data overlay if provided or discovered.
#[must_use]
pub fn make_canonical_q_problem_with_real_data(
    variant: QVariant,
    real_data: Option<&QRealData>,
) -> Problem {
    let mut problem = make_canonical_q_problem(variant);
    if let Some(data) = real_data {
        data.apply_to_problem(&mut problem);
    }
    problem
}
