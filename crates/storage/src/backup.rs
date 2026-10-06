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
    /// Stable machine-readable reason (`file_missing`, `not_a_database`, `corrupted`,
    /// `newer_version`, `missing_table`, `unreadable`) so the UI can show a localized message;
    /// `error` keeps the raw technical text for the log.
    #[serde(default)]
    pub error_code: Option<String>,
    /// Highest schema version this build understands.
    #[serde(default)]
    pub supported_version: i32,
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

/// Builds the summary of a rejected file.
fn invalid_summary(
    code: &str,
    error: String,
    user_version: i32,
) -> Result<BackupValidationSummary, StorageError> {
    Ok(BackupValidationSummary {
        valid: false,
        user_version,
        school_years_count: 0,
        teachers_count: 0,
        plans_count: 0,
        error: Some(error),
        error_code: Some(code.to_string()),
        supported_version: latest_version(),
    })
}

/// Validates an existing SQLite database file without modifying current state.
pub fn validate_backup_file(path: &Path) -> Result<BackupValidationSummary, StorageError> {
    if !path.exists() {
        return invalid_summary("file_missing", "File does not exist".to_string(), 0);
    }

    let conn = match Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ) {
        Ok(c) => c,
        Err(err) => {
            return invalid_summary("unreadable", format!("Cannot open database: {err}"), 0)
        }
    };

    // 1. PRAGMA integrity_check
    let integrity: String = match conn.query_row("PRAGMA integrity_check", [], |row| row.get(0)) {
        Ok(s) => s,
        Err(e) => {
            let text = e.to_string();
            let code = if text.contains("not a database") {
                "not_a_database"
            } else {
                "corrupted"
            };
            return invalid_summary(code, format!("Integrity check query failed: {e}"), 0);
        }
    };

    if integrity != "ok" {
        return invalid_summary(
            "corrupted",
            format!("Database integrity failure: {integrity}"),
            0,
        );
    }

    // 2. PRAGMA user_version <= latest_version()
    let user_version: i32 = match conn.query_row("PRAGMA user_version", [], |row| row.get(0)) {
        Ok(v) => v,
        Err(e) => {
            return invalid_summary("unreadable", format!("Failed to read user_version: {e}"), 0)
        }
    };

    if user_version > latest_version() {
        return invalid_summary(
            "newer_version",
            format!(
                "Unsupported future database version: {user_version} > supported {}",
                latest_version()
            ),
            user_version,
        );
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
            return invalid_summary(
                "missing_table",
                format!("Missing required table: {tbl}"),
                user_version,
            );
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
        error_code: None,
        supported_version: latest_version(),
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
    create_automatic_backup_in(conn, &backups_dir(), reason)
}

/// Same as [`create_automatic_backup`] but writes into an explicit `dir`.
///
/// File names are `exampanel-backup-<reason>-<epoch_secs>[_<n>].db`. A backup never
/// overwrites an existing file: when the second is already taken a `_<n>` suffix is added
/// so two operations in the same second each keep their own safety copy (RA-031).
pub fn create_automatic_backup_in(
    conn: &Connection,
    dir: &Path,
    reason: &str,
) -> Result<PathBuf, StorageError> {
    fs::create_dir_all(dir)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let clean_reason = reason
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();

    let mut target = dir.join(format!("exampanel-backup-{clean_reason}-{now}.db"));
    let mut seq = 1u32;
    while target.exists() {
        target = dir.join(format!("exampanel-backup-{clean_reason}-{now}_{seq}.db"));
        seq += 1;
    }

    backup_database(conn, &target)?;
    let _ = prune_backups_in(dir, MAX_AUTO_BACKUPS);

    Ok(target)
}

/// Sort key `(epoch_secs, sequence)` of an automatic backup, parsed from its file name
/// (`…-<epoch>` or `…-<epoch>_<n>`). Falls back to the file's modification time so
/// hand-named files still order sensibly.
fn backup_sort_key(filename: &str, mtime_secs: u64) -> (u64, u64) {
    let stem = filename.strip_suffix(".db").unwrap_or(filename);
    if let Some(tail) = stem.rsplit('-').next() {
        let mut parts = tail.splitn(2, '_');
        if let Some(Ok(epoch)) = parts.next().map(str::parse::<u64>) {
            let seq = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
            return (epoch, seq);
        }
    }
    (mtime_secs, 0)
}

/// Lists all automatic backups in the backups directory sorted from newest to oldest.
pub fn list_automatic_backups() -> Result<Vec<BackupFileInfo>, StorageError> {
    list_backups_in(&backups_dir())
}

/// Lists the backups in `dir`, newest first by the timestamp in the file name (not by the
/// alphabetical order of the reason prefix).
pub fn list_backups_in(dir: &Path) -> Result<Vec<BackupFileInfo>, StorageError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "db") {
            let metadata = entry.metadata()?;
            let filename = entry.file_name().to_string_lossy().to_string();
            let modified_secs = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());

            list.push((
                backup_sort_key(&filename, modified_secs),
                BackupFileInfo {
                    filename,
                    path: path.to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    modified_at: modified_secs.to_string(),
                },
            ));
        }
    }

    list.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.filename.cmp(&a.1.filename)));
    Ok(list.into_iter().map(|(_, info)| info).collect())
}

/// Prunes old automatic backup files so at most `keep_count` remain.
pub fn prune_automatic_backups(keep_count: usize) -> Result<(), StorageError> {
    prune_backups_in(&backups_dir(), keep_count)
}

/// Prunes `dir` so only the `keep_count` newest backups remain.
pub fn prune_backups_in(dir: &Path, keep_count: usize) -> Result<(), StorageError> {
    let mut backups = list_backups_in(dir)?;
    if backups.len() > keep_count {
        for old in backups.drain(keep_count..) {
            let _ = fs::remove_file(PathBuf::from(old.path));
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

    #[test]
    fn test_restore_v4_backup_triggers_migration() {
        use crate::migrations::get_current_version;
        use exam_panel_core::domain::SchoolYearId;

        let temp_dir = tempfile::tempdir().unwrap();
        let v4_backup_file = temp_dir.path().join("v4_backup.db");
        let active_db_file = temp_dir.path().join("active.db");

        // 1. Build a real v4 database file
        let v4_conn = Connection::open(&v4_backup_file).unwrap();
        v4_conn
            .execute_batch(
                r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE campuses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                color TEXT NOT NULL
            );
            CREATE TABLE school_years (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                is_current INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE grades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL
            );
            CREATE TABLE exams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL
            );
            CREATE TABLE teachers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                full_name TEXT NOT NULL,
                campus_id INTEGER NOT NULL REFERENCES campuses(id),
                load_weight REAL NOT NULL DEFAULT 1.0,
                active INTEGER NOT NULL DEFAULT 1,
                note TEXT,
                code TEXT
            );
            CREATE TABLE teacher_grades (
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
                grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE CASCADE,
                PRIMARY KEY (school_year_id, teacher_id, grade_id)
            );
            CREATE TABLE rule_settings (
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                rule_key TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                weight REAL NOT NULL DEFAULT 1.0,
                parameters TEXT NOT NULL DEFAULT '{}',
                PRIMARY KEY (school_year_id, rule_key)
            );
            CREATE TABLE plans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                seed INTEGER NOT NULL,
                score REAL,
                is_final INTEGER NOT NULL DEFAULT 0,
                rank INTEGER,
                source TEXT NOT NULL DEFAULT 'optimizer'
            );
            CREATE TABLE assignments (
                plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
                exam_id INTEGER NOT NULL REFERENCES exams(id),
                grade_id INTEGER NOT NULL REFERENCES grades(id),
                teacher_id INTEGER NOT NULL REFERENCES teachers(id),
                role TEXT NOT NULL,
                PRIMARY KEY (plan_id, exam_id, grade_id, teacher_id, role)
            );
            CREATE TABLE locks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                exam_id INTEGER NOT NULL REFERENCES exams(id),
                grade_id INTEGER NOT NULL REFERENCES grades(id),
                teacher_id INTEGER NOT NULL REFERENCES teachers(id),
                role TEXT,
                kind TEXT NOT NULL
            );
            CREATE TABLE unavailability (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
                teacher_id INTEGER NOT NULL REFERENCES teachers(id),
                exam_id INTEGER NOT NULL REFERENCES exams(id),
                reason TEXT,
                UNIQUE (school_year_id, teacher_id, exam_id)
            );
            CREATE TABLE settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT INTO campuses (id, code, name, color) VALUES (1, 'PH1', 'Phân hiệu 1', '#3b82f6');
            INSERT INTO school_years (id, name, is_current) VALUES (1, '2026-2027', 1);
            INSERT INTO grades (id, code, name) VALUES (1, 10, 'Khối 10');
            INSERT INTO teachers (id, full_name, campus_id, load_weight, active) VALUES (1, 'Thầy V4', 1, 1.0, 1);
            INSERT INTO teacher_grades (school_year_id, teacher_id, grade_id) VALUES (1, 1, 1);
            PRAGMA user_version = 4;
            "#,
            )
            .unwrap();
        drop(v4_conn);

        // 2. Open active store and restore from v4 backup
        let mut active_store = Store::open_at(&active_db_file).unwrap();
        active_store.restore_from(&v4_backup_file).unwrap();

        // 3. Verify user_version is upgraded to 5
        let ver = get_current_version(active_store.conn()).unwrap();
        assert_eq!(
            ver, 5,
            "Restoring v4 backup must upgrade to schema version 5"
        );

        // 4. Verify default subject 'CHUNG' and competency created
        let subjects = active_store.get_subjects(SchoolYearId(1)).unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].code, "CHUNG");

        let comps = active_store.get_competencies(SchoolYearId(1)).unwrap();
        assert!(
            !comps.is_empty(),
            "Competencies should be automatically populated for active teachers"
        );
    }

    #[test]
    fn test_restore_committed_v4_backup_fixture() {
        use crate::migrations::get_current_version;
        use exam_panel_core::domain::SchoolYearId;
        use std::path::PathBuf;

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixture_path = manifest_dir.join("fixtures/v4_synthetic.db");
        assert!(
            fixture_path.exists(),
            "Committed v4 backup fixture must exist at {:?}",
            fixture_path
        );

        let temp_dir = tempfile::tempdir().unwrap();
        let active_db_file = temp_dir.path().join("active_restored.db");

        let mut active_store = Store::open_at(&active_db_file).unwrap();
        active_store.restore_from(&fixture_path).unwrap();

        let ver = get_current_version(active_store.conn()).unwrap();
        assert_eq!(
            ver, 5,
            "Restoring committed v4 backup fixture must upgrade schema to version 5"
        );

        let subjects = active_store.get_subjects(SchoolYearId(1)).unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].code, "CHUNG");

        let teachers = active_store.get_teachers().unwrap();
        assert_eq!(teachers.len(), 12);
        assert_eq!(teachers[0].code.as_deref(), Some("GV01"));

        let plans = active_store.list_plans(SchoolYearId(1)).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].name, "Phương án v4 mẫu");
    }
}
