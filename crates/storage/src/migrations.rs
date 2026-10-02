//! Embedded database migrations managed via `PRAGMA user_version`.

use crate::StorageError;
use rusqlite::Connection;

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../migrations/0001_initial.sql")),
    (2, include_str!("../migrations/0002_plans_extension.sql")),
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
    fn test_upgrade_from_v1_to_v2_preserves_data() {
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

        // Now run full migrations to upgrade to v2
        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 2);

        // Verify data intact and new columns populated with defaults
        let (id, name, seed, score, is_final, rank, score_report, run_params, source): (
            i64,
            String,
            u64,
            Option<f64>,
            i32,
            Option<i64>,
            Option<String>,
            Option<String>,
            String,
        ) = conn
            .query_row(
                "SELECT id, name, seed, score, is_final, rank, score_report_json, run_params_json, source FROM plans WHERE id = 10",
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
    }
}
