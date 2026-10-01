//! Embedded database migrations managed via `PRAGMA user_version`.

use crate::StorageError;
use rusqlite::Connection;

const MIGRATIONS: &[(i32, &str)] = &[(1, include_str!("../migrations/0001_initial.sql"))];

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
}
