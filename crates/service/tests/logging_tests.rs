//! RA-043: app.log stayed empty because nothing in the Rust side ever logged.

use exam_panel_service::dto::{CreateExamInput, CreateSchoolYearInput};
use exam_panel_service::service::AppService;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::sync::{Mutex, Once};

struct Capture;
static LINES: Mutex<Vec<(Level, String)>> = Mutex::new(Vec::new());
static INIT: Once = Once::new();
// The logger is process-global; keep the tests of this file from interleaving.
static SERIAL: Mutex<()> = Mutex::new(());

impl Log for Capture {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        LINES
            .lock()
            .unwrap()
            .push((record.level(), format!("{}", record.args())));
    }
    fn flush(&self) {}
}

fn capture() -> std::sync::MutexGuard<'static, ()> {
    INIT.call_once(|| {
        log::set_logger(&Capture).unwrap();
        log::set_max_level(LevelFilter::Trace);
    });
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    LINES.lock().unwrap().clear();
    guard
}

fn lines() -> Vec<(Level, String)> {
    LINES.lock().unwrap().clone()
}

#[test]
fn opening_a_database_logs_path_and_schema_version() {
    let _g = capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("exam-panel.db");
    let _service = AppService::open_at(&path).unwrap();
    let all = lines();
    assert!(
        all.iter().any(|(l, m)| *l == Level::Info
            && m.contains("database opened")
            && m.contains("exam-panel.db")),
        "{all:?}"
    );
}

#[test]
fn a_rejected_duplicate_write_is_logged_with_its_error_code() {
    let _g = capture();
    let service = AppService::open_in_memory().unwrap();
    let year = service
        .create_school_year(CreateSchoolYearInput {
            name: "2026-2027".into(),
            is_current: true,
            copy_grades_from: None,
        })
        .unwrap()
        .id;
    let input = || CreateExamInput {
        school_year_id: year,
        code: "DUP".into(),
        name: "Trùng".into(),
        sort_order: 9,
    };
    service.create_exam(input()).unwrap();
    LINES.lock().unwrap().clear();
    let err = service.create_exam(input()).unwrap_err();
    assert_eq!(err.code, "duplicate_entry");
    let all = lines();
    assert!(
        all.iter()
            .any(|(l, m)| *l == Level::Warn && m.contains("duplicate_entry")),
        "{all:?}"
    );
}

#[test]
fn backup_and_restore_are_logged() {
    let _g = capture();
    let dir = tempfile::tempdir().unwrap();
    let service = AppService::open_at(dir.path().join("exam-panel.db")).unwrap();
    let backup = dir.path().join("b.db");
    service.backup_database(&backup).unwrap();
    service.restore_database(&backup).unwrap();
    let all = lines();
    assert!(
        all.iter().any(|(_, m)| m.contains("backup created")),
        "{all:?}"
    );
    assert!(
        all.iter().any(|(_, m)| m.contains("database restored")),
        "{all:?}"
    );
}
