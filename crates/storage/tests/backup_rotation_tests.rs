//! Regression tests for RA-031 (backup rotation / ordering) and RA-007 (pre-migration backup).

use exam_panel_storage::backup::{
    create_automatic_backup, list_automatic_backups, prune_automatic_backups, validate_backup_file,
    MAX_AUTO_BACKUPS,
};
use exam_panel_storage::Store;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

// The backup API resolves its directory from EXAMPANEL_DATA_DIR; serialise tests touching it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_data_dir<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("EXAMPANEL_DATA_DIR", dir);
    let out = f();
    std::env::remove_var("EXAMPANEL_DATA_DIR");
    out
}

fn fake_backup(dir: &Path, reason: &str, ts: u64) -> String {
    let name = format!("exampanel-backup-{reason}-{ts}.db");
    fs::write(dir.join(&name), b"x").unwrap();
    name
}

#[test]
fn ra031_prune_keeps_the_newest_ten_across_reasons() {
    let tmp = tempfile::tempdir().unwrap();
    let backups = tmp.path().join("backups");
    fs::create_dir_all(&backups).unwrap();
    // 7 older pre-restore backups, then 9 newer pre-import backups (reason sorts BEFORE
    // "pre-restore" alphabetically, which is what broke name-based ordering).
    let mut all = Vec::new();
    for i in 0..7u64 {
        all.push((
            1_700_000_000 + i * 10,
            fake_backup(&backups, "pre-restore", 1_700_000_000 + i * 10),
        ));
    }
    for i in 0..9u64 {
        all.push((
            1_700_001_000 + i * 10,
            fake_backup(&backups, "pre-import", 1_700_001_000 + i * 10),
        ));
    }
    all.sort();
    let expected: BTreeSet<String> = all
        .iter()
        .rev()
        .take(MAX_AUTO_BACKUPS)
        .map(|(_, n)| n.clone())
        .collect();

    with_data_dir(tmp.path(), || {
        prune_automatic_backups(MAX_AUTO_BACKUPS).unwrap()
    });

    let kept: BTreeSet<String> = fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(kept, expected, "the 10 NEWEST backups must survive");
}

#[test]
fn ra031_list_is_sorted_newest_first_by_timestamp() {
    let tmp = tempfile::tempdir().unwrap();
    let backups = tmp.path().join("backups");
    fs::create_dir_all(&backups).unwrap();
    fake_backup(&backups, "pre-restore", 1_700_000_100);
    let newest = fake_backup(&backups, "pre-import", 1_700_000_300);
    fake_backup(&backups, "pre-restore", 1_700_000_200);
    let list = with_data_dir(tmp.path(), || list_automatic_backups().unwrap());
    assert_eq!(list[0].filename, newest);
    let ts: Vec<&str> = list.iter().map(|b| b.filename.as_str()).collect();
    assert_eq!(
        ts,
        vec![
            "exampanel-backup-pre-import-1700000300.db",
            "exampanel-backup-pre-restore-1700000200.db",
            "exampanel-backup-pre-restore-1700000100.db",
        ]
    );
}

#[test]
fn ra031_two_backups_in_the_same_second_both_survive() {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let (a, b) = with_data_dir(tmp.path(), || {
        let a = create_automatic_backup(store.conn(), "pre-restore").unwrap();
        let b = create_automatic_backup(store.conn(), "pre-restore").unwrap();
        (a, b)
    });
    assert_ne!(a, b, "second backup must not overwrite the first");
    assert!(a.exists() && b.exists());
}

#[test]
fn ra007_opening_an_older_database_creates_a_pre_migration_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    fs::create_dir_all(&data).unwrap();
    let db = data.join("exam-panel.db");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/v4_synthetic.db");
    fs::copy(&fixture, &db).unwrap();

    let store = Store::open_at(&db).unwrap();
    drop(store);

    let backups: Vec<_> = fs::read_dir(data.join("backups"))
        .expect("backups dir must exist after migrating an older database")
        .flatten()
        .map(|e| e.path())
        .collect();
    assert_eq!(backups.len(), 1, "{backups:?}");
    let name = backups[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert!(name.contains("pre-migration"), "{name}");
    let summary = validate_backup_file(&backups[0]).unwrap();
    assert!(summary.valid, "{:?}", summary.error);
    assert_eq!(
        summary.user_version, 4,
        "backup must hold the PRE-migration schema"
    );
}

#[test]
fn ra007_opening_a_current_database_creates_no_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    fs::create_dir_all(&data).unwrap();
    let db = data.join("exam-panel.db");
    drop(Store::open_at(&db).unwrap()); // fresh DB: nothing to protect
    drop(Store::open_at(&db).unwrap()); // already current
    assert!(!data.join("backups").exists());
}
