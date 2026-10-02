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

/// Name of the marker file that explicitly triggers portable mode.
pub const PORTABLE_MARKER: &str = "ExamPanel.portable";

/// Detailed outcome of resolving the data location mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLocationStatus {
    /// Portable mode active: marker present and data directory is writable.
    Portable(PathBuf),
    /// Portable mode requested by marker, but data directory is read-only (e.g. locked USB drive).
    PortableReadOnly {
        requested: PathBuf,
        fallback: PathBuf,
    },
    /// Standard installed mode: marker not present, using OS app-data directory.
    Installed(PathBuf),
}

/// Evaluates the data directory for a given base executable folder.
#[must_use]
pub fn inspect_data_location_for_exe_dir(exe_dir: &Path) -> DataLocationStatus {
    let marker_file = exe_dir.join(PORTABLE_MARKER);
    let fallback = fallback_app_data_dir();

    if marker_file.exists() {
        let portable_data_dir = exe_dir.join("data");
        if is_dir_writable(&portable_data_dir) {
            DataLocationStatus::Portable(portable_data_dir)
        } else {
            DataLocationStatus::PortableReadOnly {
                requested: portable_data_dir,
                fallback,
            }
        }
    } else {
        DataLocationStatus::Installed(fallback)
    }
}

/// Inspects the active data location mode based on environment and executable location.
#[must_use]
pub fn inspect_data_location() -> DataLocationStatus {
    if let Ok(dir_override) = std::env::var("EXAMPANEL_DATA_DIR") {
        return DataLocationStatus::Portable(PathBuf::from(dir_override));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let status = inspect_data_location_for_exe_dir(exe_dir);
            // In unit tests/dev, exe_dir is target/debug/deps where marker is absent.
            // If marker is in current working directory, respect it:
            if matches!(status, DataLocationStatus::Installed(_))
                && Path::new(PORTABLE_MARKER).exists()
            {
                return inspect_data_location_for_exe_dir(Path::new("."));
            }
            return status;
        }
    }

    if Path::new(PORTABLE_MARKER).exists() {
        inspect_data_location_for_exe_dir(Path::new("."))
    } else {
        DataLocationStatus::Installed(fallback_app_data_dir())
    }
}

/// Resolves the active data directory. If portable mode is requested but read-only,
/// falls back to the user OS application data directory.
#[must_use]
pub fn resolve_data_dir() -> PathBuf {
    match inspect_data_location() {
        DataLocationStatus::Portable(path) => path,
        DataLocationStatus::PortableReadOnly { fallback, .. } => fallback,
        DataLocationStatus::Installed(path) => path,
    }
}

/// Checks whether the resolved data directory is in portable mode.
#[must_use]
pub fn is_portable_mode() -> bool {
    matches!(inspect_data_location(), DataLocationStatus::Portable(_))
}

/// Resolves the full path to the SQLite database file.
#[must_use]
pub fn resolve_database_path() -> PathBuf {
    resolve_data_dir().join(DB_FILENAME)
}

/// Resolves the path to the demo sandbox database file.
#[must_use]
pub fn resolve_demo_database_path() -> PathBuf {
    resolve_data_dir().join("demo.db")
}

/// Resolves the path to the application logs directory.
#[must_use]
pub fn resolve_logs_dir() -> PathBuf {
    resolve_data_dir().join("logs")
}

/// Resolves the path to the database backups directory.
#[must_use]
pub fn resolve_backups_dir() -> PathBuf {
    resolve_data_dir().join("backups")
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

    #[test]
    fn test_portable_marker_present_and_writable() {
        let temp = tempfile::tempdir().expect("failed to create temp dir");
        let marker = temp.path().join(PORTABLE_MARKER);
        fs::write(&marker, b"").expect("write marker");

        let status = inspect_data_location_for_exe_dir(temp.path());
        match status {
            DataLocationStatus::Portable(dir) => {
                assert_eq!(dir, temp.path().join("data"));
            }
            other => panic!("expected Portable, got {other:?}"),
        }
    }

    #[test]
    fn test_portable_marker_absent_defaults_to_installed() {
        let temp = tempfile::tempdir().expect("failed to create temp dir");
        let status = inspect_data_location_for_exe_dir(temp.path());
        match status {
            DataLocationStatus::Installed(dir) => {
                assert_eq!(dir, fallback_app_data_dir());
            }
            other => panic!("expected Installed, got {other:?}"),
        }
    }

    #[test]
    fn test_portable_marker_read_only_folder() {
        let temp = tempfile::tempdir().expect("failed to create temp dir");
        let marker = temp.path().join(PORTABLE_MARKER);
        fs::write(&marker, b"").expect("write marker");

        // Create a file where data directory should be to make it uncreatable/unwritable as a directory
        let fake_data = temp.path().join("data");
        fs::write(&fake_data, b"blocking file").expect("write blocking file");

        // is_dir_writable will fail since fs::create_dir_all fails on an existing file
        assert!(!is_dir_writable(&fake_data));

        let status = inspect_data_location_for_exe_dir(temp.path());
        match status {
            DataLocationStatus::PortableReadOnly {
                requested,
                fallback,
            } => {
                assert_eq!(requested, fake_data);
                assert_eq!(fallback, fallback_app_data_dir());
            }
            other => panic!("expected PortableReadOnly, got {other:?}"),
        }
    }
}
