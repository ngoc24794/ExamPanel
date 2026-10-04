//! Verification and generation tests for v4 synthetic backup fixture.

use exam_panel_core::domain::*;
use exam_panel_core::score::evaluate;
use exam_panel_core::solver::{solve_hard, SolveOptions};
use exam_panel_core::validate::{validate_assignments, ValidateOptions};
use exam_panel_storage::migrations::{get_current_version, MIGRATIONS};
use exam_panel_storage::store::Store;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

fn get_schema_summary(conn: &Connection) -> Vec<(String, String, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT type, name, sql FROM sqlite_master 
             WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' 
             ORDER BY type, name;",
        )
        .unwrap();

    stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })
    .unwrap()
    .map(|r| r.unwrap())
    .collect()
}

pub fn create_v4_fixture_at(db_path: &Path) {
    if db_path.exists() {
        let _ = fs::remove_file(db_path);
    }

    let conn = Connection::open(db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    // 1. Run migrations 1 through 4 ONLY
    for v in 1..=4 {
        let sql = MIGRATIONS.iter().find(|(ver, _)| *ver == v).unwrap().1;
        conn.execute_batch(sql).unwrap();
        conn.pragma_update(None, "user_version", v).unwrap();
    }
    assert_eq!(get_current_version(&conn).unwrap(), 4);

    // 2. Insert School Year
    conn.execute(
        "INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);",
        [],
    )
    .unwrap();

    // 3. Insert Campuses
    conn.execute(
        "INSERT INTO campuses (id, code, name, color) VALUES
         (1, 'CS1', 'Phân hiệu 1', 'blue'),
         (2, 'CS2', 'Phân hiệu 2', 'green'),
         (3, 'CS3', 'Phân hiệu 3', 'amber'),
         (4, 'CS4', 'Phân hiệu 4', 'purple');",
        [],
    )
    .unwrap();

    // 4. Insert Grades
    conn.execute(
        "INSERT INTO grades (id, code, name, sort_order) VALUES
         (1, 10, 'Khối 10', 1),
         (2, 11, 'Khối 11', 2),
         (3, 12, 'Khối 12', 3);",
        [],
    )
    .unwrap();

    // 5. Insert Exams
    conn.execute(
        "INSERT INTO exams (id, school_year_id, code, name, sort_order) VALUES
         (1, 1, 'GK1', 'Giữa kỳ 1', 1),
         (2, 1, 'CK1', 'Cuối kỳ 1', 2),
         (3, 1, 'GK2', 'Giữa kỳ 2', 3),
         (4, 1, 'CK2', 'Cuối kỳ 2', 4);",
        [],
    )
    .unwrap();

    // 6. Insert Teachers with code
    let teachers = [
        (1, "Nguyễn Văn A", 1, "GV01"),
        (2, "Trần Thị B", 1, "GV02"),
        (3, "Lê Văn C", 2, "GV03"),
        (4, "Phạm Thị D", 2, "GV04"),
        (5, "Hoàng Văn E", 3, "GV05"),
        (6, "Vũ Thị F", 3, "GV06"),
        (7, "Đỗ Văn G", 4, "GV07"),
        (8, "Bùi Thị H", 4, "GV08"),
        (9, "Ngô Văn I", 1, "GV09"),
        (10, "Dương Thị K", 2, "GV10"),
        (11, "Lý Văn L", 3, "GV11"),
        (12, "Mai Thị M", 4, "GV12"),
    ];

    for &(id, name, cid, code) in &teachers {
        conn.execute(
            "INSERT INTO teachers (id, full_name, campus_id, load_weight, active, code) 
             VALUES (?1, ?2, ?3, 1.0, 1, ?4);",
            rusqlite::params![id, name, cid, code],
        )
        .unwrap();
    }

    // 7. Insert Teacher Grades (each teacher teaches all grades 1, 2, 3 for feasibility)
    for &(id, _, _, _) in &teachers {
        for gid in 1..=3 {
            conn.execute(
                "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id) VALUES (?1, 1, ?2);",
                rusqlite::params![id, gid],
            )
            .unwrap();
        }
    }

    // 8. Insert Unavailability
    conn.execute(
        "INSERT INTO unavailability (teacher_id, exam_id, reason) VALUES (1, 1, 'Bận công tác');",
        [],
    )
    .unwrap();

    // 9. Insert Locks (matching v4 schema without school_year_id)
    conn.execute(
        "INSERT INTO locks (id, exam_id, grade_id, teacher_id, role, kind) 
         VALUES (1, 2, 1, 3, 'setter', 'pin');",
        [],
    )
    .unwrap();

    // 10. Insert Rule Settings
    let rules = [
        ("h1", 1, 0.0),
        ("h2", 1, 0.0),
        ("h3", 1, 0.0),
        ("h4", 1, 0.0),
        ("h5", 1, 0.0),
        ("h6", 1, 0.0),
        ("h7", 1, 0.0),
        ("s1", 1, 10.0),
        ("s2", 1, 3.0),
        ("s3", 1, 4.0),
        ("s4", 1, 6.0),
        ("s5", 1, 6.0),
        ("s6", 1, 2.0),
        ("s7", 1, 1.0),
        ("s8", 1, 8.0),
    ];
    for &(rk, en, w) in &rules {
        conn.execute(
            "INSERT INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json) 
             VALUES (1, ?1, ?2, ?3, '{}');",
            rusqlite::params![rk, en, w],
        )
        .unwrap();
    }

    // 11. Insert Settings
    conn.execute(
        "INSERT INTO settings (key, value) VALUES 
         ('theme', 'light'), 
         ('language', 'vi'), 
         ('current_school_year_id', '1');",
        [],
    )
    .unwrap();

    // 12. Build and solve a valid v4 plan (12 panels x 3 = 36 assignments)
    let domain_teachers: Vec<Teacher> = teachers
        .iter()
        .map(|&(id, name, cid, code)| Teacher {
            id: TeacherId(id),
            full_name: name.to_string(),
            display_name: None,
            campus_id: CampusId(cid),
            load_weight: 1.0,
            active: true,
            note: None,
            code: Some(code.to_string()),
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .collect();

    let legacy_prob = Problem {
        school_year: SchoolYear {
            id: SchoolYearId(1),
            name: "2026-2027".to_string(),
            is_current: true,
        },
        campuses: vec![
            Campus {
                id: CampusId(1),
                code: "CS1".to_string(),
                name: "Phân hiệu 1".to_string(),
                color: "blue".to_string(),
            },
            Campus {
                id: CampusId(2),
                code: "CS2".to_string(),
                name: "Phân hiệu 2".to_string(),
                color: "green".to_string(),
            },
            Campus {
                id: CampusId(3),
                code: "CS3".to_string(),
                name: "Phân hiệu 3".to_string(),
                color: "amber".to_string(),
            },
            Campus {
                id: CampusId(4),
                code: "CS4".to_string(),
                name: "Phân hiệu 4".to_string(),
                color: "purple".to_string(),
            },
        ],
        grades: vec![
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
        ],
        subjects: vec![Subject {
            id: SubjectId(1),
            code: "CHUNG".to_string(),
            name: "Chung".to_string(),
            color: "palette-1".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        }],
        exams: vec![
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
        ],
        teachers: domain_teachers,
        teacher_grades: teachers
            .iter()
            .flat_map(|&(id, _, _, _)| {
                (1..=3).map(move |gid| TeacherGrade {
                    teacher_id: TeacherId(id),
                    school_year_id: SchoolYearId(1),
                    grade_id: GradeId(gid),
                })
            })
            .collect(),
        competencies: teachers
            .iter()
            .flat_map(|&(id, _, _, _)| {
                vec![
                    Competency {
                        teacher_id: TeacherId(id),
                        subject_id: SubjectId(1),
                        role: Role::Setter,
                        grade_scope: GradeScope::Taught,
                    },
                    Competency {
                        teacher_id: TeacherId(id),
                        subject_id: SubjectId(1),
                        role: Role::Reviewer,
                        grade_scope: GradeScope::Taught,
                    },
                ]
            })
            .collect(),
        unavailabilities: vec![Unavailability {
            teacher_id: TeacherId(1),
            exam_id: ExamId(1),
            reason: Some("Bận công tác".to_string()),
        }],
        locks: vec![Lock {
            id: LockId(1),
            exam_id: ExamId(2),
            grade_id: GradeId(1),
            subject_id: SubjectId(1),
            teacher_id: TeacherId(3),
            role: Some(Role::Setter),
            kind: LockKind::Pin,
        }],
        rule_settings: RuleSetting::default_settings(),
    };

    let sol = solve_hard(
        &legacy_prob,
        &SolveOptions {
            seed: 42,
            time_limit_ms: 3000,
            max_nodes: 500_000,
        },
    )
    .expect("solve valid v4 plan");
    let initial_score = evaluate(&legacy_prob, &sol.assignments);

    conn.execute(
        "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, source, data_hash, rules_hash) 
         VALUES (1, 1, 'Phương án v4 mẫu', '2026-09-01T00:00:00', 42, ?1, 1, 1, 'optimizer', ?2, ?3);",
        rusqlite::params![initial_score.total, legacy_prob.data_hash(), legacy_prob.rules_hash()],
    )
    .unwrap();

    for a in &sol.assignments {
        conn.execute(
            "INSERT INTO assignments (plan_id, exam_id, grade_id, teacher_id, role) 
             VALUES (1, ?1, ?2, ?3, ?4);",
            rusqlite::params![a.exam_id.0, a.grade_id.0, a.teacher_id.0, a.role.as_str()],
        )
        .unwrap();
    }

    // Verify PRAGMA user_version is strictly 4
    let ver: i32 = conn
        .query_row("PRAGMA user_version;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(ver, 4, "PRAGMA user_version must be exactly 4");
}

#[test]
fn test_v4_synthetic_fixture_schema_integrity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.join("fixtures/v4_synthetic.db");

    assert!(
        fixture_path.exists(),
        "Committed v4 fixture must exist at {:?}",
        fixture_path
    );

    // 1. Connect to committed fixture
    let fixture_conn = Connection::open(&fixture_path).expect("open committed v4 fixture");
    let fixture_ver: i32 = fixture_conn
        .query_row("PRAGMA user_version;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        fixture_ver, 4,
        "Committed fixture must have user_version = 4"
    );

    let fixture_schema = get_schema_summary(&fixture_conn);

    // 2. Connect to in-memory DB and run migrations 1..=4
    let mem_conn = Connection::open_in_memory().unwrap();
    mem_conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    for v in 1..=4 {
        let sql = MIGRATIONS.iter().find(|(ver, _)| *ver == v).unwrap().1;
        mem_conn.execute_batch(sql).unwrap();
        mem_conn.pragma_update(None, "user_version", v).unwrap();
    }
    assert_eq!(get_current_version(&mem_conn).unwrap(), 4);

    let mem_schema = get_schema_summary(&mem_conn);

    // 3. Assert EXACT match of sqlite_master tables, indices, and views
    assert_eq!(
        fixture_schema.len(),
        mem_schema.len(),
        "Schema object count mismatch"
    );

    for (f_type, f_name, f_sql) in &fixture_schema {
        let matched = mem_schema
            .iter()
            .find(|(m_type, m_name, _)| m_type == f_type && m_name == f_name);
        assert!(
            matched.is_some(),
            "Fixture contains unexpected schema object: {} {}",
            f_type,
            f_name
        );
        let (_, _, m_sql) = matched.unwrap();
        assert_eq!(
            f_sql.trim(),
            m_sql.trim(),
            "SQL schema mismatch for {} {}",
            f_type,
            f_name
        );
    }
    println!("Schema verification: Committed v4 fixture sqlite_master SQL matches migrations 1..=4 perfectly!");
}

#[test]
fn test_restore_v4_synthetic_fixture_with_migration_and_legacy_equivalence() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.join("fixtures/v4_synthetic.db");
    assert!(fixture_path.exists(), "v4_synthetic.db fixture must exist");

    let temp_dir = tempfile::tempdir().unwrap();
    let active_db_path = temp_dir.path().join("active.db");

    // Open active store (which initializes at version 5)
    let mut store = Store::open_at(&active_db_path).expect("open active store");
    assert_eq!(get_current_version(store.conn()).unwrap(), 5);

    // Pre-migration baseline check: load problem and plan directly from v4 fixture
    let v4_conn = Connection::open(&fixture_path).unwrap();
    let v4_plan_score: f64 = v4_conn
        .query_row("SELECT score FROM plans WHERE id = 1;", [], |r| r.get(0))
        .unwrap();
    drop(v4_conn);

    // Perform restore from v4 fixture
    store
        .restore_from(&fixture_path)
        .expect("restore from v4 fixture");

    // 1. Verify automatic pre-restore backup was created and exists
    let backups = store.list_backups().unwrap();
    assert!(
        backups.iter().any(|b| b.filename.contains("pre-restore")),
        "Pre-restore backup must exist in backups list"
    );

    // 2. Verify schema version upgraded to 5
    let ver = get_current_version(store.conn()).unwrap();
    assert_eq!(
        ver, 5,
        "Restoring v4 fixture must upgrade schema to version 5"
    );

    // 3. Verify all data preserved
    let campuses = store.get_campuses().unwrap();
    assert_eq!(campuses.len(), 4);

    let teachers = store.get_teachers().unwrap();
    assert_eq!(teachers.len(), 12);
    assert_eq!(teachers[0].code.as_deref(), Some("GV01"));

    let subjects = store.get_subjects(SchoolYearId(1)).unwrap();
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].code, "CHUNG");

    let comps = store.get_competencies(SchoolYearId(1)).unwrap();
    assert!(!comps.is_empty(), "Competencies backfilled for teacher");

    let locks = store.get_locks(SchoolYearId(1)).unwrap();
    assert_eq!(locks.len(), 1);

    let unavails = store.get_unavailabilities(SchoolYearId(1)).unwrap();
    assert_eq!(unavails.len(), 1);

    let plans = store.list_plans(SchoolYearId(1)).unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].name, "Phương án v4 mẫu");

    // 4. Verify legacy-equivalence of the plan
    let problem = store.load_problem(SchoolYearId(1)).unwrap();
    let (loaded_plan, assignments) = store.load_plan(PlanId(1)).unwrap();

    assert_eq!(assignments.len(), 36, "36 assignments preserved");

    let violations = validate_assignments(
        &problem,
        &assignments,
        &ValidateOptions {
            require_complete: true,
        },
    );
    assert!(
        violations.is_empty(),
        "Plan must have 0 hard constraint violations after migration, got {:?}",
        violations
    );

    let post_score = evaluate(&problem, &assignments);
    assert!(
        (post_score.total - v4_plan_score).abs() < 1e-4,
        "Post-migration score ({:.2}) must equal pre-migration baseline score ({:.2})",
        post_score.total,
        v4_plan_score
    );
    assert_eq!(loaded_plan.score, Some(v4_plan_score));
    println!(
        "Legacy-equivalence verified: score = {:.2}, 0 violations, 36 assignments intact.",
        post_score.total
    );
}
