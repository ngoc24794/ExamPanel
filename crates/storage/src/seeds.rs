//! Default and demonstration seed datasets.

use crate::StorageError;
use rusqlite::Connection;

/// Seeds default grades (10, 11, 12) and system settings. Idempotent.
pub fn seed_defaults(conn: &Connection) -> Result<(), StorageError> {
    // Default grades: 10, 11, 12
    conn.execute(
        "INSERT OR IGNORE INTO grades (code, name, sort_order) VALUES
         (10, 'Khối 10', 1),
         (11, 'Khối 11', 2),
         (12, 'Khối 12', 3);",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // Default settings
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES
         ('theme', 'system'),
         ('language', 'vi');",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    Ok(())
}

/// Seeds a comprehensive demonstration dataset with 4 campuses, 11 teachers,
/// a current school year 2026-2027, 4 exams, grade qualifications (each grade
/// having >= 3 teachers from >= 2 campuses, teachers teaching 2 grades, one
/// teacher with load_weight 0.5), and one unavailability entry.
///
/// Used by unit/integration tests and developer flags; never applied to real user databases.
pub fn seed_demo(conn: &Connection) -> Result<(), StorageError> {
    // Ensure default grades are present
    seed_defaults(conn)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(StorageError::from_sqlite)?;

    // 1. Four Campuses with distinct theme-friendly colors
    tx.execute(
        "INSERT OR IGNORE INTO campuses (id, code, name, color) VALUES
         (1, 'CS1', 'Cơ sở 1 - Ba Đình', '#3b82f6'),
         (2, 'CS2', 'Cơ sở 2 - Cầu Giấy', '#10b981'),
         (3, 'CS3', 'Cơ sở 3 - Hà Đông', '#f59e0b'),
         (4, 'CS4', 'Cơ sở 4 - Hoàn Kiếm', '#8b5cf6');",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // 2. 11 Teachers with realistic Vietnamese names
    // Note: Teacher 6 (Vũ Hải Hà) has load_weight = 0.5
    tx.execute(
        "INSERT OR IGNORE INTO teachers (id, full_name, campus_id, load_weight, active, note) VALUES
         (1, 'Nguyễn Văn An', 1, 1.0, 1, 'Tổ trưởng chuyên môn'),
         (2, 'Trần Thị Bình', 1, 1.0, 1, NULL),
         (3, 'Lê Hoàng Cường', 2, 1.0, 1, NULL),
         (4, 'Phạm Minh Đức', 2, 1.0, 1, NULL),
         (5, 'Hoàng Thu Giang', 3, 1.0, 1, NULL),
         (6, 'Vũ Hải Hà', 3, 0.5, 1, 'Bán thời gian'),
         (7, 'Đặng Quốc Hùng', 3, 1.0, 1, NULL),
         (8, 'Bùi Thị Lan', 4, 1.0, 1, NULL),
         (9, 'Đỗ Tuấn Minh', 4, 1.0, 1, NULL),
         (10, 'Ngô Phương Nam', 1, 1.0, 1, NULL),
         (11, 'Dương Thùy Trang', 2, 1.0, 1, NULL);",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // 3. School Year: 2026-2027 (current)
    tx.execute(
        "INSERT OR IGNORE INTO school_years (id, name, is_current) VALUES
         (1, '2026-2027', 1);",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // 4. Four default Exams for school year 1
    tx.execute(
        "INSERT OR IGNORE INTO exams (id, school_year_id, code, name, sort_order) VALUES
         (1, 1, 'GK1', 'Giữa kỳ 1', 1),
         (2, 1, 'CK1', 'Cuối kỳ 1', 2),
         (3, 1, 'GK2', 'Giữa kỳ 2', 3),
         (4, 1, 'CK2', 'Cuối kỳ 2', 4);",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // Resolve grade IDs
    let g10_id: i64 = tx
        .query_row("SELECT id FROM grades WHERE code = 10", [], |r| r.get(0))
        .map_err(StorageError::from_sqlite)?;
    let g11_id: i64 = tx
        .query_row("SELECT id FROM grades WHERE code = 11", [], |r| r.get(0))
        .map_err(StorageError::from_sqlite)?;
    let g12_id: i64 = tx
        .query_row("SELECT id FROM grades WHERE code = 12", [], |r| r.get(0))
        .map_err(StorageError::from_sqlite)?;

    // 5. Grade assignments for school year 1:
    // Grade 10: T1 (CS1), T3 (CS2), T5 (CS3), T8 (CS4) -> 4 teachers from 4 campuses
    // Grade 11: T2 (CS1), T4 (CS2), T6 (CS3, load 0.5), T9 (CS4), T1 (CS1) -> 5 teachers from 4 campuses
    // Grade 12: T3 (CS2), T7 (CS3), T10 (CS1), T11 (CS2) -> 4 teachers from 3 campuses
    // Teachers teaching 2 grades: T1 (10 & 11), T3 (10 & 12)
    let tg_tuples = [
        (1, 1, g10_id),
        (3, 1, g10_id),
        (5, 1, g10_id),
        (8, 1, g10_id),
        (2, 1, g11_id),
        (4, 1, g11_id),
        (6, 1, g11_id),
        (9, 1, g11_id),
        (1, 1, g11_id),
        (3, 1, g12_id),
        (7, 1, g12_id),
        (10, 1, g12_id),
        (11, 1, g12_id),
    ];

    for (t_id, sy_id, g_id) in tg_tuples {
        tx.execute(
            "INSERT OR IGNORE INTO teacher_grades (teacher_id, school_year_id, grade_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![t_id, sy_id, g_id],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 6. One Unavailability entry: Teacher 6 unavailable for Exam 3 (GK2)
    tx.execute(
        "INSERT OR IGNORE INTO unavailability (teacher_id, exam_id, reason) VALUES
         (6, 3, 'Khám sức khỏe định kỳ');",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // 7. Default RuleSettings for school year 1
    for rule in exam_panel_core::domain::RuleSetting::default_settings() {
        let params_str = serde_json::to_string(&rule.params)?;
        tx.execute(
            "INSERT OR IGNORE INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                1,
                rule.key.as_str(),
                if rule.enabled { 1 } else { 0 },
                rule.weight,
                params_str
            ],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    tx.commit().map_err(StorageError::from_sqlite)?;
    Ok(())
}
