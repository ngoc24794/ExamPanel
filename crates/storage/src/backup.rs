//! SQLite online backup and restore utilities.

use crate::migrations::{latest_version, run_migrations};
use crate::paths::resolve_data_dir;
use crate::StorageError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Maximum number of automatic backups kept in storage.
pub const MAX_AUTO_BACKUPS: usize = 10;

/// Validation summary for an SQLite backup file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupValidationSummary {
    pub valid: bool,
    pub user_version: i32,
    pub school_years_count: usize,
    pub teachers_count: usize,
    pub plans_count: usize,
    pub error: Option<String>,
}

/// Metadata about an automatic backup file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

/// Required tables that must exist in an authentic ExamPanel database.
const REQUIRED_TABLES: &[&str] = &[
    "campuses",
    "teachers",
    "school_years",
    "exams",
    "grades",
    "teacher_grades",
    "unavailability",
    "locks",
    "plans",
    "assignments",
    "rule_settings",
    "settings",
];

/// Copies the active database to the specified target path using SQLite online backup API.
pub fn backup_database(conn: &Connection, target_path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut dst_conn = Connection::open(target_path).map_err(StorageError::from_sqlite)?;
    dst_conn
        .execute_batch("PRAGMA journal_mode = DELETE;")
        .map_err(StorageError::from_sqlite)?;

    let backup =
        rusqlite::backup::Backup::new(conn, &mut dst_conn).map_err(StorageError::from_sqlite)?;
    backup
        .run_to_completion(100, std::time::Duration::from_millis(5), None)
        .map_err(StorageError::from_sqlite)?;

    Ok(())
}

/// Validates an existing SQLite database file without modifying current state.
pub fn validate_backup_file(path: &Path) -> Result<BackupValidationSummary, StorageError> {
    if !path.exists() {
        return Ok(BackupValidationSummary {
            valid: false,
            user_version: 0,
            school_years_count: 0,
            teachers_count: 0,
            plans_count: 0,
            error: Some("File does not exist".to_string()),
        });
    }

    let conn = match Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ) {
        Ok(c) => c,
        Err(err) => {
            return Ok(BackupValidationSummary {
                valid: false,
                user_version: 0,
                school_years_count: 0,
                teachers_count: 0,
                plans_count: 0,
                error: Some(format!("Cannot open database: {err}")),
            });
        }
    };

    // 1. PRAGMA integrity_check
    let integrity: String = match conn.query_row("PRAGMA integrity_check", [], |row| row.get(0)) {
        Ok(s) => s,
        Err(e) => {
            return Ok(BackupValidationSummary {
                valid: false,
                user_version: 0,
                school_years_count: 0,
                teachers_count: 0,
                plans_count: 0,
                error: Some(format!("Integrity check query failed: {e}")),
            });
        }
    };

    if integrity != "ok" {
        return Ok(BackupValidationSummary {
            valid: false,
            user_version: 0,
            school_years_count: 0,
            teachers_count: 0,
            plans_count: 0,
            error: Some(format!("Database integrity failure: {integrity}")),
        });
    }

    // 2. PRAGMA user_version <= latest_version()
    let user_version: i32 = match conn.query_row("PRAGMA user_version", [], |row| row.get(0)) {
        Ok(v) => v,
        Err(e) => {
            return Ok(BackupValidationSummary {
                valid: false,
                user_version: 0,
                school_years_count: 0,
                teachers_count: 0,
                plans_count: 0,
                error: Some(format!("Failed to read user_version: {e}")),
            });
        }
    };

    if user_version > latest_version() {
        return Ok(BackupValidationSummary {
            valid: false,
            user_version,
            school_years_count: 0,
            teachers_count: 0,
            plans_count: 0,
            error: Some(format!(
                "Unsupported future database version: {user_version} > supported {}",
                latest_version()
            )),
        });
    }

    // 3. Expected tables check
    for tbl in REQUIRED_TABLES {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [tbl],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !exists {
            return Ok(BackupValidationSummary {
                valid: false,
                user_version,
                school_years_count: 0,
                teachers_count: 0,
                plans_count: 0,
                error: Some(format!("Missing required table: {tbl}")),
            });
        }
    }

    // 4. Summaries
    let school_years_count: usize = conn
        .query_row("SELECT COUNT(*) FROM school_years", [], |row| row.get(0))
        .unwrap_or(0);
    let teachers_count: usize = conn
        .query_row("SELECT COUNT(*) FROM teachers", [], |row| row.get(0))
        .unwrap_or(0);
    let plans_count: usize = conn
        .query_row("SELECT COUNT(*) FROM plans", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(BackupValidationSummary {
        valid: true,
        user_version,
        school_years_count,
        teachers_count,
        plans_count,
        error: None,
    })
}

/// Restores database content from `source_path` into the active `conn`.
///
/// Automatically creates a pre-restore backup first.
pub fn restore_database(conn: &mut Connection, source_path: &Path) -> Result<(), StorageError> {
    let summary = validate_backup_file(source_path)?;
    if !summary.valid {
        return Err(StorageError::Constraint(
            summary
                .error
                .unwrap_or_else(|| "Invalid backup file".to_string()),
        ));
    }

    // Auto-backup current active DB before applying restore
    let _ = create_automatic_backup(conn, "pre-restore");

    let src_conn = Connection::open(source_path).map_err(StorageError::from_sqlite)?;
    {
        let backup =
            rusqlite::backup::Backup::new(&src_conn, conn).map_err(StorageError::from_sqlite)?;
        backup
            .run_to_completion(100, std::time::Duration::from_millis(5), None)
            .map_err(StorageError::from_sqlite)?;
    }

    // Ensure migrations are up to date if restoring an older schema version
    run_migrations(conn)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(StorageError::from_sqlite)?;

    Ok(())
}

/// Returns the directory where automatic backups are stored: `<data_dir>/backups`.
pub fn backups_dir() -> PathBuf {
    let dir = resolve_data_dir().join("backups");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

/// Creates a timestamped automatic backup with the specified reason tag.
/// Also prunes backups to keep only the latest `MAX_AUTO_BACKUPS`.
pub fn create_automatic_backup(conn: &Connection, reason: &str) -> Result<PathBuf, StorageError> {
    let dir = backups_dir();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let clean_reason = reason
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();

    let filename = format!("exampanel-backup-{clean_reason}-{now}.db");
    let target = dir.join(&filename);

    backup_database(conn, &target)?;
    let _ = prune_automatic_backups(MAX_AUTO_BACKUPS);

    Ok(target)
}

/// Lists all automatic backups in the backups directory sorted from newest to oldest.
pub fn list_automatic_backups() -> Result<Vec<BackupFileInfo>, StorageError> {
    let dir = backups_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    let entries = fs::read_dir(&dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "db" {
                    let metadata = entry.metadata()?;
                    let filename = entry.file_name().to_string_lossy().to_string();
                    let modified_at = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs().to_string())
                        .unwrap_or_default();

                    list.push(BackupFileInfo {
                        filename,
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len(),
                        modified_at,
                    });
                }
            }
        }
    }

    // Sort newest first by filename/modified_at
    list.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(list)
}

/// Prunes old automatic backup files so at most `keep_count` remain.
pub fn prune_automatic_backups(keep_count: usize) -> Result<(), StorageError> {
    let mut backups = list_automatic_backups()?;
    if backups.len() > keep_count {
        for old in backups.drain(keep_count..) {
            let path = PathBuf::from(old.path);
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;

    #[test]
    fn test_backup_and_restore_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_path = temp_dir.path().join("test_backup.db");

        let mut store = Store::open_in_memory().unwrap();
        let _sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Nguyen Van A", campus.id, 1.0, true, None, None)
            .unwrap();

        // 1. Backup
        store.backup_to(&backup_path).expect("backup failed");
        assert!(backup_path.exists());

        // 2. Validate backup
        let summary = validate_backup_file(&backup_path).expect("validate failed");
        assert!(summary.valid, "validate failed: {:?}", summary.error);
        assert_eq!(summary.teachers_count, 1);
        assert!(summary.school_years_count >= 1);

        // 3. Modify active database
        let teacher2 = store
            .create_teacher("Tran Thi B", campus.id, 1.0, true, None, None)
            .unwrap();
        assert_eq!(store.get_teachers().unwrap().len(), 2);

        // 4. Restore from backup
        store.restore_from(&backup_path).expect("restore failed");

        // 5. Verify teacher2 is gone and teacher is back
        let teachers = store.get_teachers().unwrap();
        assert_eq!(teachers.len(), 1);
        assert_eq!(teachers[0].id, teacher.id);
        assert_ne!(teachers[0].id, teacher2.id);
    }

    #[test]
    fn test_corrupted_file_rejection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bad_file = temp_dir.path().join("corrupted.db");
        fs::write(&bad_file, b"THIS_IS_NOT_A_VALID_SQLITE_DATABASE").unwrap();

        let summary = validate_backup_file(&bad_file).unwrap();
        assert!(!summary.valid);
        assert!(summary.error.is_some());
    }

    #[test]
    fn test_newer_user_version_rejection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let future_file = temp_dir.path().join("future.db");

        let store = Store::open_in_memory().unwrap();
        store.backup_to(&future_file).unwrap();

        // Set user_version = 9999
        let conn = Connection::open(&future_file).unwrap();
        conn.execute_batch("PRAGMA user_version = 9999;").unwrap();
        drop(conn);

        let summary = validate_backup_file(&future_file).unwrap();
        assert!(!summary.valid);
        let err = summary.error.unwrap();
        assert!(err.contains("Unsupported future database version"));
    }

    #[test]
    fn test_missing_table_rejection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let incomplete_file = temp_dir.path().join("incomplete.db");

        let conn = Connection::open(&incomplete_file).unwrap();
        conn.execute_batch("CREATE TABLE campuses (id INTEGER PRIMARY KEY);")
            .unwrap();
        drop(conn);

        let summary = validate_backup_file(&incomplete_file).unwrap();
        assert!(!summary.valid);
        let err = summary.error.unwrap();
        assert!(err.contains("Missing required table"));
    }

    #[test]
    fn test_restore_reopens_connection_on_file_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_file = temp_dir.path().join("active.db");
        let backup_file = temp_dir.path().join("backup.db");

        // Create initial db and backup
        let mut store = Store::open_at(&db_file).unwrap();
        assert_eq!(store.path(), Some(db_file.as_path()));
        let campus = store.create_campus("CS1", "Campus CS1", "#fff").unwrap();
        store
            .create_teacher("Teacher In Backup", campus.id, 1.0, true, None, None)
            .unwrap();
        store.backup_to(&backup_file).unwrap();

        // Mutate store with new teacher
        store
            .create_teacher("Teacher After Backup", campus.id, 1.0, true, None, None)
            .unwrap();
        assert_eq!(store.get_teachers().unwrap().len(), 2);

        // Restore: this reopens the connection
        store.restore_from(&backup_file).unwrap();

        // Check restored state
        let teachers = store.get_teachers().unwrap();
        assert_eq!(teachers.len(), 1);
        assert_eq!(teachers[0].full_name, "Teacher In Backup");

        // Verify we can still write to the reopened connection
        store
            .create_teacher("New Post Restore Teacher", campus.id, 1.0, true, None, None)
            .unwrap();
        assert_eq!(store.get_teachers().unwrap().len(), 2);
    }
}
