//! Default and demonstration seed datasets.

use crate::StorageError;
use rusqlite::Connection;

/// Seeds default grades (10, 11, 12) and system settings. Idempotent.
pub fn seed_defaults(conn: &Connection) -> Result<(), StorageError> {
    // `INSERT OR IGNORE` on an AUTOINCREMENT table advances sqlite_sequence even when every row
    // is ignored, which rewrote the file on every open (RA-008). Only insert what is missing.
    let grades_present: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM grades WHERE code IN (10, 11, 12)",
            [],
            |r| r.get(0),
        )
        .map_err(StorageError::from_sqlite)?;
    if grades_present < 3 {
        // Default grades: 10, 11, 12
        conn.execute(
            "INSERT OR IGNORE INTO grades (code, name, sort_order) VALUES
             (10, 'Khối 10', 1),
             (11, 'Khối 11', 2),
             (12, 'Khối 12', 3);",
            [],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    let settings_present: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE key IN ('theme', 'language')",
            [],
            |r| r.get(0),
        )
        .map_err(StorageError::from_sqlite)?;
    if settings_present < 2 {
        // Default settings
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES
             ('theme', 'system'),
             ('language', 'vi');",
            [],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    Ok(())
}

/// Seeds the canonical Q-shaped demonstration dataset with 4 synthetic campuses
/// (or real campus/grade data overlay if `data/q_real_data.json` or `EXAMPANEL_Q_DATA_PATH` is present).
pub fn seed_demo(conn: &Connection) -> Result<(), StorageError> {
    seed_defaults(conn)?;

    let real_data = exam_panel_core::domain::fixtures::load_q_real_data_from_file_or_env();
    let problem = exam_panel_core::domain::fixtures::make_canonical_q_problem_with_real_data(
        exam_panel_core::domain::fixtures::QVariant::SyntheticCampuses,
        real_data.as_ref(),
    );

    let tx = conn
        .unchecked_transaction()
        .map_err(StorageError::from_sqlite)?;

    // 1. Four Campuses
    for c in &problem.campuses {
        tx.execute(
            "INSERT OR IGNORE INTO campuses (id, code, name, color) VALUES (?1, ?2, ?3, ?4);",
            rusqlite::params![c.id.0, c.code, c.name, c.color],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 2. Teachers (12 teachers)
    for t in &problem.teachers {
        tx.execute(
            "INSERT OR IGNORE INTO teachers (id, code, full_name, display_name, campus_id, load_weight, active, note, quota_override, max_tasks_per_exam_override)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            rusqlite::params![
                t.id.0,
                t.code,
                t.full_name,
                t.display_name,
                t.campus_id.0,
                t.load_weight,
                if t.active { 1 } else { 0 },
                t.note,
                t.quota_override,
                t.max_tasks_per_exam_override,
            ],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 3. School Year
    tx.execute(
        "INSERT OR IGNORE INTO school_years (id, name, is_current) VALUES (?1, ?2, ?3);",
        rusqlite::params![
            problem.school_year.id.0,
            problem.school_year.name,
            if problem.school_year.is_current { 1 } else { 0 },
        ],
    )
    .map_err(StorageError::from_sqlite)?;

    // 4. Exams
    for e in &problem.exams {
        tx.execute(
            "INSERT OR IGNORE INTO exams (id, school_year_id, code, name, sort_order) VALUES (?1, ?2, ?3, ?4, ?5);",
            rusqlite::params![e.id.0, e.school_year_id.0, e.code, e.name, e.sort_order],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 4b. Subjects
    for s in &problem.subjects {
        tx.execute(
            "INSERT OR IGNORE INTO subjects (id, school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
            rusqlite::params![
                s.id.0,
                problem.school_year.id.0,
                s.code,
                s.name,
                s.color,
                s.sort_order,
                s.setters,
                s.reviewers,
                s.min_campuses,
            ],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 4c. Competencies
    for c in &problem.competencies {
        tx.execute(
            "INSERT OR IGNORE INTO teacher_competencies (teacher_id, subject_id, role, grade_scope) VALUES (?1, ?2, ?3, ?4);",
            rusqlite::params![
                c.teacher_id.0,
                c.subject_id.0,
                c.role.as_str(),
                c.grade_scope.as_str(),
            ],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 5. Teacher Grades
    for tg in &problem.teacher_grades {
        tx.execute(
            "INSERT OR IGNORE INTO teacher_grades (teacher_id, school_year_id, grade_id) VALUES (?1, ?2, ?3);",
            rusqlite::params![tg.teacher_id.0, tg.school_year_id.0, tg.grade_id.0],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 6. Unavailabilities
    for u in &problem.unavailabilities {
        tx.execute(
            "INSERT OR IGNORE INTO unavailability (teacher_id, exam_id, reason) VALUES (?1, ?2, ?3);",
            rusqlite::params![u.teacher_id.0, u.exam_id.0, u.reason],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    // 7. Rule Settings
    for rule in &problem.rule_settings {
        let params_str = serde_json::to_string(&rule.params)?;
        tx.execute(
            "INSERT OR IGNORE INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            rusqlite::params![
                problem.school_year.id.0,
                rule.key.as_str(),
                if rule.enabled { 1 } else { 0 },
                rule.weight,
                params_str,
            ],
        )
        .map_err(StorageError::from_sqlite)?;
    }

    tx.commit().map_err(StorageError::from_sqlite)?;
    Ok(())
}

/// Seeds the legacy demonstration dataset with 4 campuses, 11 teachers.
pub fn seed_legacy_demo(conn: &Connection) -> Result<(), StorageError> {
    // Ensure default grades are present
    seed_defaults(conn)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(StorageError::from_sqlite)?;

    // 1. Four Campuses with distinct theme-friendly palette tokens
    tx.execute(
        "INSERT OR IGNORE INTO campuses (id, code, name, color) VALUES
         (1, 'PH1', 'Phân hiệu 1 - Ba Đình', 'blue'),
         (2, 'PH2', 'Phân hiệu 2 - Cầu Giấy', 'emerald'),
         (3, 'PH3', 'Phân hiệu 3 - Hà Đông', 'amber'),
         (4, 'PH4', 'Phân hiệu 4 - Hoàn Kiếm', 'purple');",
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

    // 4b. Subject CHUNG for school year 1
    tx.execute(
        "INSERT OR IGNORE INTO subjects (id, school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses) VALUES
         (1, 1, 'CHUNG', 'Chung', 'palette-1', 1, 2, 1, 2);",
        [],
    )
    .map_err(StorageError::from_sqlite)?;

    // 4c. Competencies for teachers 1..11
    for t_id in 1..=11 {
        tx.execute(
            "INSERT OR IGNORE INTO teacher_competencies (teacher_id, subject_id, role, grade_scope) VALUES
             (?1, 1, 'setter', 'taught'),
             (?1, 1, 'reviewer', 'taught');",
            rusqlite::params![t_id],
        )
        .map_err(StorageError::from_sqlite)?;
    }

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
