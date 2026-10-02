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
    fn test_upgrade_from_v1_to_v4_preserves_data() {
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

        // Now run full migrations to upgrade to latest version
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
