//! RA-008: opening an already initialised, current database must not modify its bytes.

use exam_panel_storage::Store;
use std::fs;

#[test]
fn reopening_a_current_database_leaves_the_file_byte_identical() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("exam-panel.db");

    drop(Store::open_at(&db).unwrap()); // creates + seeds
    let first = fs::read(&db).unwrap();

    drop(Store::open_at(&db).unwrap());
    drop(Store::open_at(&db).unwrap());
    let third = fs::read(&db).unwrap();

    assert_eq!(first, third, "an open wrote to the database file");
}
