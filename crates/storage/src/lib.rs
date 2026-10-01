//! ExamPanel Storage Library
//!
//! SQLite database connection, schema migrations, and repository access.

pub mod paths;

use rusqlite::Connection;
use thiserror::Error;

/// Storage errors that can occur during database operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("I/O error during storage operations: {0}")]
    Io(#[from] std::io::Error),
}

/// Opens an in-memory SQLite database connection.
pub fn open_in_memory() -> Result<Connection, StorageError> {
    let conn = Connection::open_in_memory()?;
    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// Opens or creates the SQLite database at the resolved portable/installed data path.
pub fn open_database() -> Result<Connection, StorageError> {
    let data_dir = paths::resolve_data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    let db_path = paths::resolve_database_path();
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_connection_succeeds() {
        let conn = open_in_memory().expect("failed to open in-memory database");
        let result: i32 = conn
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("failed to run sample query");
        assert_eq!(result, 1);
    }
}
