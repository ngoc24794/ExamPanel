//! Embedded database migrations managed via `PRAGMA user_version`.

use crate::StorageError;
use rusqlite::Connection;

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../migrations/0001_initial.sql")),
    (2, include_str!("../migrations/0002_plans_extension.sql")),
    (3, include_str!("../migrations/0003_plans_problem_hash.sql")),
    (4, include_str!("../migrations/0004_phase9_data_model.sql")),
    (
        5,
        include_str!("../migrations/0005_subjects_and_competencies.sql"),
    ),
];

/// Returns the latest migration version available in the binary.
#[must_use]
pub fn latest_version() -> i32 {
    MIGRATIONS.last().map_or(0, |(v, _)| *v)
}

/// Reads the current schema version of the database.
pub fn get_current_version(conn: &Connection) -> Result<i32, StorageError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(StorageError::from_sqlite)?;
    Ok(version)
}

/// Runs all pending schema migrations in numerical order.
/// Each migration runs inside its own transaction and updates `PRAGMA user_version`.
/// Calling this multiple times is idempotent.
pub fn run_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    let current_version = get_current_version(conn)?;
    let latest = latest_version();

    if current_version > latest {
        return Err(StorageError::UnsupportedVersion(current_version as u32));
    }

    for (version, sql) in MIGRATIONS {
        if *version > current_version {
            let tx = conn.transaction().map_err(StorageError::from_sqlite)?;
            tx.execute_batch(sql).map_err(StorageError::from_sqlite)?;
            tx.pragma_update(None, "user_version", *version)
                .map_err(StorageError::from_sqlite)?;
            tx.commit().map_err(StorageError::from_sqlite)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_fresh_db_reaches_latest_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 0);

        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), latest_version());
    }

    #[test]
    fn test_migrations_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        run_migrations(&mut conn).unwrap();
        let ver1 = get_current_version(&conn).unwrap();

        // Running twice is a complete no-op
        run_migrations(&mut conn).unwrap();
        let ver2 = get_current_version(&conn).unwrap();

        assert_eq!(ver1, ver2);
        assert_eq!(ver2, latest_version());
    }

    #[test]
    fn test_upgrade_from_v1_to_v5_preserves_data() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        // Run ONLY migration 1
        let sql_v1 = include_str!("../migrations/0001_initial.sql");
        conn.execute_batch(sql_v1).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 1);

        // Insert dummy data into v1 schema
        conn.execute(
            "INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final)
             VALUES (10, 1, 'Legacy Plan', '2026-09-01T00:00:00', 42, 12.5, 1);",
            [],
        )
        .unwrap();

        // Now run full migrations to upgrade to latest version (v5)
        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), latest_version());

        // Verify data intact and new columns populated with defaults
        let (id, name, seed, score, is_final, rank, score_report, run_params, source, data_hash, rules_hash): (
            i64,
            String,
            u64,
            Option<f64>,
            i32,
            Option<i64>,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT id, name, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash FROM plans WHERE id = 10",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(id, 10);
        assert_eq!(name, "Legacy Plan");
        assert_eq!(seed, 42);
        assert_eq!(score, Some(12.5));
        assert_eq!(is_final, 1);
        assert_eq!(rank, None);
        assert_eq!(score_report, None);
        assert_eq!(run_params, None);
        assert_eq!(source, "optimizer");
        assert_eq!(data_hash, None);
        assert_eq!(rules_hash, None);
    }

    #[test]
    fn test_upgrade_from_v3_to_v4_backfills_data_hash_and_drops_problem_hash() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        // Run migrations 1, 2, 3
        let sql_v1 = include_str!("../migrations/0001_initial.sql");
        let sql_v2 = include_str!("../migrations/0002_plans_extension.sql");
        let sql_v3 = include_str!("../migrations/0003_plans_problem_hash.sql");
        conn.execute_batch(sql_v1).unwrap();
        conn.execute_batch(sql_v2).unwrap();
        conn.execute_batch(sql_v3).unwrap();
        conn.pragma_update(None, "user_version", 3).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 3);

        conn.execute(
            "INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, source, problem_hash)
             VALUES (20, 1, 'V3 Plan', '2026-09-02T00:00:00', 99, 5.0, 0, 1, 'manual', 'old_problem_hash_123');",
            [],
        )
        .unwrap();

        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), latest_version());

        let (id, name, data_hash, rules_hash): (i64, String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT id, name, data_hash, rules_hash FROM plans WHERE id = 20",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(id, 20);
        assert_eq!(name, "V3 Plan");
        assert_eq!(data_hash, Some("old_problem_hash_123".to_string()));
        assert_eq!(rules_hash, None);

        // Verify problem_hash column no longer exists
        let pragma_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(plans);")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(!pragma_cols.contains(&"problem_hash".to_string()));
        assert!(pragma_cols.contains(&"data_hash".to_string()));
        assert!(pragma_cols.contains(&"rules_hash".to_string()));
    }

    #[test]
    fn test_upgrade_from_v4_to_v5_backfills_subjects_and_competencies() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        // Run migrations 1, 2, 3, 4
        for v in 1..=4 {
            let sql = MIGRATIONS.iter().find(|(ver, _)| *ver == v).unwrap().1;
            conn.execute_batch(sql).unwrap();
            conn.pragma_update(None, "user_version", v).unwrap();
        }
        assert_eq!(get_current_version(&conn).unwrap(), 4);

        // Insert legacy data: school year, campus, teacher, exam, grade, lock, plan, assignment
        conn.execute(
            "INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO campuses (id, code, name, color) VALUES (1, 'CS1', 'Ba Đình', 'blue');",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO teachers (id, full_name, campus_id, load_weight, active) VALUES (1, 'Nguyễn Văn A', 1, 1.0, 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO exams (id, school_year_id, code, name, sort_order) VALUES (1, 1, 'GK1', 'Giữa kỳ 1', 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO grades (id, code, name, sort_order) VALUES (1, 10, 'Khối 10', 1);",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO locks (id, exam_id, grade_id, teacher_id, role, kind) VALUES (1, 1, 1, 1, 'setter', 'pin');",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, source)
             VALUES (1, 1, 'Plan 1', '2026-09-01T00:00:00', 42, 0.0, 1, 1, 'optimizer');",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO assignments (plan_id, exam_id, grade_id, teacher_id, role) VALUES (1, 1, 1, 1, 'setter');",
            [],
        )
        .unwrap();

        // Upgrade to v5
        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 5);

        // 1. Check subject 'CHUNG' exists
        let (sub_id, code, name, setters, reviewers): (i64, String, String, i32, i32) = conn
            .query_row(
                "SELECT id, code, name, setters, reviewers FROM subjects WHERE school_year_id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(code, "CHUNG");
        assert_eq!(name, "Chung");
        assert_eq!(setters, 2);
        assert_eq!(reviewers, 1);

        // 2. Check teacher competencies backfilled
        let comp_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM teacher_competencies WHERE teacher_id = 1 AND subject_id = ?1",
                [sub_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(comp_count, 2); // both setter and reviewer

        // 3. Check locks table has subject_id
        let lock_sub: i64 = conn
            .query_row("SELECT subject_id FROM locks WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(lock_sub, sub_id);

        // 4. Check assignments table has subject_id and position
        let (a_sub, pos): (i64, usize) = conn
            .query_row(
                "SELECT subject_id, position FROM assignments WHERE plan_id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(a_sub, sub_id);
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_legacy_equivalence_v4_to_v5() {
        use crate::store::Store;
        use exam_panel_core::domain::*;
        use exam_panel_core::score::evaluate;
        use exam_panel_core::solver::{solve_hard, SolveOptions};
        use exam_panel_core::validate::{validate_assignments, ValidateOptions};

        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        // 1. Run migrations 1 through 4
        for v in 1..=4 {
            let sql = MIGRATIONS.iter().find(|(ver, _)| *ver == v).unwrap().1;
            conn.execute_batch(sql).unwrap();
            conn.pragma_update(None, "user_version", v).unwrap();
        }
        assert_eq!(get_current_version(&conn).unwrap(), 4);

        // 2. Insert standard legacy demo schema data (v4)
        conn.execute(
            "INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO campuses (id, code, name, color) VALUES
             (1, 'CS1', 'Phân hiệu 1', 'blue'),
             (2, 'CS2', 'Phân hiệu 2', 'green'),
             (3, 'CS3', 'Phân hiệu 3', 'amber'),
             (4, 'CS4', 'Phân hiệu 4', 'purple');",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO grades (id, code, name, sort_order) VALUES
             (1, 10, 'Khối 10', 1),
             (2, 11, 'Khối 11', 2),
             (3, 12, 'Khối 12', 3);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO exams (id, school_year_id, code, name, sort_order) VALUES
             (1, 1, 'GK1', 'Giữa kỳ 1', 1),
             (2, 1, 'CK1', 'Cuối kỳ 1', 2),
             (3, 1, 'GK2', 'Giữa kỳ 2', 3),
             (4, 1, 'CK2', 'Cuối kỳ 2', 4);",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO teachers (id, full_name, campus_id, load_weight, active) VALUES
             (1, 'GV 1', 1, 1.0, 1),
             (2, 'GV 2', 1, 1.0, 1),
             (3, 'GV 3', 2, 1.0, 1),
             (4, 'GV 4', 2, 1.0, 1),
             (5, 'GV 5', 3, 1.0, 1),
             (6, 'GV 6', 3, 1.0, 1),
             (7, 'GV 7', 3, 1.0, 1),
             (8, 'GV 8', 4, 1.0, 1),
             (9, 'GV 9', 4, 1.0, 1),
             (10, 'GV 10', 1, 1.0, 1),
             (11, 'GV 11', 2, 1.0, 1);",
            [],
        )
        .unwrap();

        for tid in 1..=11 {
            for gid in 1..=3 {
                conn.execute(
                    "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id) VALUES (?1, 1, ?2);",
                    [tid, gid],
                )
                .unwrap();
            }
        }

        let legacy_sub = Subject {
            id: SubjectId(1),
            code: "CHUNG".to_string(),
            name: "Chung".to_string(),
            color: "palette-1".to_string(),
            sort_order: 1,
            setters: 2,
            reviewers: 1,
            min_campuses: 2,
        };

        let teachers_vec: Vec<Teacher> = (1..=11)
            .map(|id| {
                let cid = match id {
                    1 | 2 | 10 => 1,
                    3 | 4 | 11 => 2,
                    5 | 6 | 7 => 3,
                    _ => 4,
                };
                Teacher {
                    id: TeacherId(id),
                    full_name: format!("GV {id}"),
                    display_name: None,
                    campus_id: CampusId(cid),
                    load_weight: 1.0,
                    active: true,
                    note: None,
                    code: None,
                    quota_override: None,
                    max_tasks_per_exam_override: None,
                }
            })
            .collect();

        let mut comps = Vec::new();
        for t in &teachers_vec {
            comps.push(Competency {
                teacher_id: t.id,
                subject_id: legacy_sub.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            });
            comps.push(Competency {
                teacher_id: t.id,
                subject_id: legacy_sub.id,
                role: Role::Reviewer,
                grade_scope: GradeScope::Taught,
            });
        }

        let legacy_prob = Problem {
            school_year: SchoolYear {
                id: SchoolYearId(1),
                name: "2026-2027".to_string(),
                is_current: true,
            },
            campuses: vec![
                Campus {
                    id: CampusId(1),
                    code: "CS1".into(),
                    name: "Phân hiệu 1".into(),
                    color: "blue".into(),
                },
                Campus {
                    id: CampusId(2),
                    code: "CS2".into(),
                    name: "Phân hiệu 2".into(),
                    color: "green".into(),
                },
                Campus {
                    id: CampusId(3),
                    code: "CS3".into(),
                    name: "Phân hiệu 3".into(),
                    color: "amber".into(),
                },
                Campus {
                    id: CampusId(4),
                    code: "CS4".into(),
                    name: "Phân hiệu 4".into(),
                    color: "purple".into(),
                },
            ],
            grades: vec![
                Grade {
                    id: GradeId(1),
                    code: 10,
                    name: "Khối 10".into(),
                    sort_order: 1,
                },
                Grade {
                    id: GradeId(2),
                    code: 11,
                    name: "Khối 11".into(),
                    sort_order: 2,
                },
                Grade {
                    id: GradeId(3),
                    code: 12,
                    name: "Khối 12".into(),
                    sort_order: 3,
                },
            ],
            subjects: vec![legacy_sub],
            exams: (1..=4)
                .map(|e| Exam {
                    id: ExamId(e),
                    school_year_id: SchoolYearId(1),
                    code: format!("EX{e}"),
                    name: format!("Exam {e}"),
                    sort_order: e as i32,
                })
                .collect(),
            teachers: teachers_vec,
            teacher_grades: (1..=11)
                .flat_map(|t| {
                    (1..=3).map(move |g| TeacherGrade {
                        teacher_id: TeacherId(t),
                        school_year_id: SchoolYearId(1),
                        grade_id: GradeId(g),
                    })
                })
                .collect(),
            competencies: comps,
            unavailabilities: vec![],
            locks: vec![],
            rule_settings: RuleSetting::default_settings(),
        };

        let sol = solve_hard(
            &legacy_prob,
            &SolveOptions {
                seed: 42,
                time_limit_ms: 2000,
                max_nodes: 500_000,
            },
        )
        .expect("solve hard legacy demo");

        let baseline_violations = validate_assignments(
            &legacy_prob,
            &sol.assignments,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(
            baseline_violations.is_empty(),
            "Baseline solution must be valid"
        );
        let baseline_score = evaluate(&legacy_prob, &sol.assignments);

        conn.execute(
            "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, source)
             VALUES (1, 1, 'Legacy Plan', '2026-09-01T00:00:00', 42, ?1, 1, 1, 'optimizer');",
            [baseline_score.total],
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

        // 3. Perform migration v4 -> v5
        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 5);

        // 4. Verify post-migration equivalence using Store
        let store = Store::new(conn);
        let loaded_prob = store.load_problem(SchoolYearId(1)).unwrap();
        let (loaded_plan, loaded_assigns) = store.load_plan(PlanId(1)).unwrap();

        assert_eq!(loaded_assigns.len(), 36, "36 assignments preserved");
        let post_violations = validate_assignments(
            &loaded_prob,
            &loaded_assigns,
            &ValidateOptions {
                require_complete: true,
            },
        );
        assert!(
            post_violations.is_empty(),
            "Expected 0 violations after migration, got {post_violations:?}"
        );

        let post_score = evaluate(&loaded_prob, &loaded_assigns);
        let diff = (post_score.total - baseline_score.total).abs();
        assert!(
            diff < 1e-6,
            "Legacy equivalence failed: post-migration total {} != pre-migration total {}",
            post_score.total,
            baseline_score.total
        );
        assert_eq!(loaded_plan.score, Some(baseline_score.total));
    }

    #[test]
    fn test_migrations_rejects_higher_user_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 99;").unwrap();
        let res = run_migrations(&mut conn);
        match res {
            Err(StorageError::UnsupportedVersion(v)) => assert_eq!(v, 99),
            other => panic!("expected UnsupportedVersion(99), got {other:?}"),
        }
    }
}
