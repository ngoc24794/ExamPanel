//! Path resolution for ExamPanel storage.
//!
//! Handles portable mode:
//! - If the directory containing the executable is writable -> `./data/exam-panel.db`
//! - Otherwise -> OS app-data directory (e.g. `%APPDATA%/ExamPanel` or `~/.local/share/ExamPanel`).

use std::fs;
use std::path::{Path, PathBuf};

/// Name of the SQLite database file.
pub const DB_FILENAME: &str = "exam-panel.db";

/// Name of the application directory when fallback to OS app-data is used.
pub const APP_DIR_NAME: &str = "ExamPanel";

/// Checks if a directory is writable by attempting to create and delete a temporary probe file.
pub fn is_dir_writable<P: AsRef<Path>>(dir: P) -> bool {
    let dir = dir.as_ref();
    if !dir.exists() && fs::create_dir_all(dir).is_err() {
        return false;
    }

    let probe_file = dir.join(".write_test_probe");
    match fs::write(&probe_file, b"test") {
        Ok(_) => {
            let _ = fs::remove_file(probe_file);
            true
        }
        Err(_) => false,
    }
}

/// Resolves the fallback OS application data directory.
#[must_use]
pub fn fallback_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return PathBuf::from(app_data).join(APP_DIR_NAME).join("data");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME)
                .join("data");
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg_data).join(APP_DIR_NAME).join("data");
        } else if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(APP_DIR_NAME)
                .join("data");
        }
    }

    // Default to current directory if environment variables cannot be read
    PathBuf::from(".").join("data")
}

/// Resolves the active data directory for portable or installed mode.
///
/// If the directory containing the executable is writable, uses `<exe_dir>/data`.
/// Otherwise, falls back to the user OS application data directory.
#[must_use]
pub fn resolve_data_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_data_dir = exe_dir.join("data");
            if is_dir_writable(&portable_data_dir) {
                return portable_data_dir;
            }
        }
    }

    // Check current working directory for dev/headless runs
    let cwd_data = PathBuf::from(".").join("data");
    if is_dir_writable(&cwd_data) {
        return cwd_data;
    }

    fallback_app_data_dir()
}

/// Resolves the full path to the SQLite database file.
#[must_use]
pub fn resolve_database_path() -> PathBuf {
    resolve_data_dir().join(DB_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dir_writable_in_temp_dir() {
        let temp = tempfile::tempdir().expect("failed to create temp dir");
        assert!(is_dir_writable(temp.path()));
    }

    #[test]
    fn test_resolve_database_path_ends_with_db_filename() {
        let path = resolve_database_path();
        assert!(path.ends_with(DB_FILENAME));
    }
}
