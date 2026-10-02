//! Strongly typed SQLite store and repository operations.

use crate::migrations::run_migrations;
use crate::seeds::seed_defaults;
use crate::StorageError;
use exam_panel_core::domain::{
    Assignment, Campus, CampusId, Competency, Exam, ExamId, Grade, GradeId, GradeScope, Lock,
    LockId, LockKind, Plan, PlanId, PlanSummary, Problem, Role, RuleKey, RuleSetting, SchoolYear,
    SchoolYearId, Subject, SubjectId, Teacher, TeacherGrade, TeacherId, TeacherWithGrades,
    Unavailability,
};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

/// Strongly typed storage interface for ExamPanel.
pub struct Store {
    conn: Connection,
    path: Option<PathBuf>,
}

impl Store {
    /// Creates a store from an established SQLite connection.
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        Self { conn, path: None }
    }

    /// Creates a store from an established connection and known file path.
    #[must_use]
    pub fn new_with_path(conn: Connection, path: PathBuf) -> Self {
        Self {
            conn,
            path: Some(path),
        }
    }

    /// Access the underlying file path if this store is backed by a disk file.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Reopens the SQLite connection from disk if a database path is registered.
    pub fn reopen(&mut self) -> Result<(), StorageError> {
        if let Some(ref path) = self.path {
            let mut conn = Connection::open(path).map_err(StorageError::from_sqlite)?;
            conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")
                .map_err(StorageError::from_sqlite)?;

            let integrity: String = conn
                .query_row("PRAGMA quick_check(1);", [], |r| r.get(0))
                .map_err(StorageError::from_sqlite)?;
            if integrity != "ok" {
                return Err(StorageError::DatabaseCorrupted(integrity));
            }

            run_migrations(&mut conn)?;
            self.conn = conn;
        }
        Ok(())
    }

    /// Access the underlying SQLite connection.
    #[must_use]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Access the underlying SQLite connection mutably.
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Opens an in-memory database, runs migrations, and applies default seeds.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory().map_err(StorageError::from_sqlite)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(StorageError::from_sqlite)?;
        run_migrations(&mut conn)?;
        seed_defaults(&conn)?;
        Ok(Self { conn, path: None })
    }

    /// Opens or creates a database at the specified path with foreign keys and rollback journal,
    /// runs migrations, and applies default seeds.
    pub fn open_at<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let p = path.as_ref().to_path_buf();
        let mut conn = Connection::open(&p).map_err(StorageError::from_sqlite)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")
            .map_err(StorageError::from_sqlite)?;

        let integrity: String = conn
            .query_row("PRAGMA quick_check(1);", [], |r| r.get(0))
            .map_err(StorageError::from_sqlite)?;
        if integrity != "ok" {
            return Err(StorageError::DatabaseCorrupted(integrity));
        }

        run_migrations(&mut conn)?;
        seed_defaults(&conn)?;
        Ok(Self {
            conn,
            path: Some(p),
        })
    }

    /// Opens the database at the resolved portable/installed location.
    pub fn open_default() -> Result<Self, StorageError> {
        let db_path = crate::paths::resolve_database_path();
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        Self::open_at(db_path)
    }

    // -------------------------------------------------------------------------
    // Backup & Restore
    // -------------------------------------------------------------------------

    pub fn backup_to(&self, path: &Path) -> Result<(), StorageError> {
        crate::backup::backup_database(&self.conn, path)
    }

    pub fn restore_from(&mut self, path: &Path) -> Result<(), StorageError> {
        crate::backup::restore_database(&mut self.conn, path)?;
        self.reopen()?;
        Ok(())
    }

    pub fn auto_backup(&self, reason: &str) -> Result<std::path::PathBuf, StorageError> {
        crate::backup::create_automatic_backup(&self.conn, reason)
    }

    pub fn list_backups(&self) -> Result<Vec<crate::backup::BackupFileInfo>, StorageError> {
        crate::backup::list_automatic_backups()
    }

    pub fn validate_backup(
        &self,
        path: &Path,
    ) -> Result<crate::backup::BackupValidationSummary, StorageError> {
        crate::backup::validate_backup_file(path)
    }

    // -------------------------------------------------------------------------
    // Campuses
    // -------------------------------------------------------------------------

    pub fn get_campuses(&self) -> Result<Vec<Campus>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, code, name, color FROM campuses ORDER BY id ASC")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Campus {
                    id: CampusId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut campuses = Vec::new();
        for r in rows {
            campuses.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(campuses)
    }

    pub fn get_campus(&self, id: CampusId) -> Result<Option<Campus>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, code, name, color FROM campuses WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                Ok(Campus {
                    id: CampusId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn create_campus(
        &self,
        code: &str,
        name: &str,
        color: &str,
    ) -> Result<Campus, StorageError> {
        self.conn
            .execute(
                "INSERT INTO campuses (code, name, color) VALUES (?1, ?2, ?3)",
                params![code, name, color],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Campus {
            id: CampusId(id),
            code: code.to_string(),
            name: name.to_string(),
            color: color.to_string(),
        })
    }

    pub fn update_campus(&self, campus: &Campus) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE campuses SET code = ?1, name = ?2, color = ?3 WHERE id = ?4",
                params![campus.code, campus.name, campus.color, campus.id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "campus with ID {} not found",
                campus.id
            )));
        }
        Ok(())
    }

    pub fn delete_campus(&self, id: CampusId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM campuses WHERE id = ?1", params![id.value()])
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "campus with ID {id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Grades
    // -------------------------------------------------------------------------

    pub fn get_grades(&self) -> Result<Vec<Grade>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, code, name, sort_order FROM grades ORDER BY sort_order ASC, code ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Grade {
                    id: GradeId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    sort_order: row.get(3)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut grades = Vec::new();
        for r in rows {
            grades.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(grades)
    }

    pub fn get_grade(&self, id: GradeId) -> Result<Option<Grade>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, code, name, sort_order FROM grades WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                Ok(Grade {
                    id: GradeId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    sort_order: row.get(3)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn create_grade(
        &self,
        code: i32,
        name: &str,
        sort_order: i32,
    ) -> Result<Grade, StorageError> {
        self.conn
            .execute(
                "INSERT INTO grades (code, name, sort_order) VALUES (?1, ?2, ?3)",
                params![code, name, sort_order],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Grade {
            id: GradeId(id),
            code,
            name: name.to_string(),
            sort_order,
        })
    }

    pub fn update_grade(&self, grade: &Grade) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE grades SET code = ?1, name = ?2, sort_order = ?3 WHERE id = ?4",
                params![grade.code, grade.name, grade.sort_order, grade.id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "grade with ID {} not found",
                grade.id
            )));
        }
        Ok(())
    }

    pub fn delete_grade(&self, id: GradeId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM grades WHERE id = ?1", params![id.value()])
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("grade_in_use".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "grade with ID {id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Teachers
    // -------------------------------------------------------------------------

    pub fn get_teachers(&self) -> Result<Vec<Teacher>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, full_name, campus_id, load_weight, active, note, code, display_name, quota_override, max_tasks_per_exam_override FROM teachers ORDER BY id ASC")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map([], |row| {
                let active_int: i32 = row.get(4)?;
                let quota_i64: Option<i64> = row.get(8)?;
                let max_tasks_i64: Option<i64> = row.get(9)?;
                Ok(Teacher {
                    id: TeacherId(row.get(0)?),
                    full_name: row.get(1)?,
                    campus_id: CampusId(row.get(2)?),
                    load_weight: row.get(3)?,
                    active: active_int == 1,
                    note: row.get(5)?,
                    code: row.get(6)?,
                    display_name: row.get(7)?,
                    quota_override: quota_i64.map(|v| v as u32),
                    max_tasks_per_exam_override: max_tasks_i64.map(|v| v as u32),
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut teachers = Vec::new();
        for r in rows {
            teachers.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(teachers)
    }

    pub fn get_teacher(&self, id: TeacherId) -> Result<Option<Teacher>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, full_name, campus_id, load_weight, active, note, code, display_name, quota_override, max_tasks_per_exam_override FROM teachers WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                let active_int: i32 = row.get(4)?;
                let quota_i64: Option<i64> = row.get(8)?;
                let max_tasks_i64: Option<i64> = row.get(9)?;
                Ok(Teacher {
                    id: TeacherId(row.get(0)?),
                    full_name: row.get(1)?,
                    campus_id: CampusId(row.get(2)?),
                    load_weight: row.get(3)?,
                    active: active_int == 1,
                    note: row.get(5)?,
                    code: row.get(6)?,
                    display_name: row.get(7)?,
                    quota_override: quota_i64.map(|v| v as u32),
                    max_tasks_per_exam_override: max_tasks_i64.map(|v| v as u32),
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn create_teacher(
        &self,
        full_name: &str,
        campus_id: CampusId,
        load_weight: f64,
        active: bool,
        note: Option<&str>,
        code: Option<&str>,
    ) -> Result<Teacher, StorageError> {
        self.create_teacher_full(
            full_name,
            campus_id,
            load_weight,
            active,
            note,
            code,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_teacher_full(
        &self,
        full_name: &str,
        campus_id: CampusId,
        load_weight: f64,
        active: bool,
        note: Option<&str>,
        code: Option<&str>,
        display_name: Option<&str>,
        quota_override: Option<u32>,
        max_tasks_per_exam_override: Option<u32>,
    ) -> Result<Teacher, StorageError> {
        let norm_code = code.map(str::trim).filter(|c| !c.is_empty());
        self.conn
            .execute(
                "INSERT INTO teachers (full_name, campus_id, load_weight, active, note, code, display_name, quota_override, max_tasks_per_exam_override)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    full_name,
                    campus_id.value(),
                    load_weight,
                    if active { 1 } else { 0 },
                    note,
                    norm_code,
                    display_name,
                    quota_override.map(|v| v as i64),
                    max_tasks_per_exam_override.map(|v| v as i64),
                ],
            )
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("teacher_code_duplicate".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        let id = self.conn.last_insert_rowid();
        Ok(Teacher {
            id: TeacherId(id),
            full_name: full_name.to_string(),
            campus_id,
            load_weight,
            active,
            note: note.map(ToString::to_string),
            code: norm_code.map(ToString::to_string),
            display_name: display_name.map(ToString::to_string),
            quota_override,
            max_tasks_per_exam_override,
        })
    }

    pub fn update_teacher(&self, teacher: &Teacher) -> Result<(), StorageError> {
        let norm_code = teacher
            .code
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty());
        let rows = self
            .conn
            .execute(
                "UPDATE teachers SET full_name = ?1, campus_id = ?2, load_weight = ?3, active = ?4, note = ?5, code = ?6, display_name = ?7, quota_override = ?8, max_tasks_per_exam_override = ?9, updated_at = datetime('now') WHERE id = ?10",
                params![
                    teacher.full_name,
                    teacher.campus_id.value(),
                    teacher.load_weight,
                    if teacher.active { 1 } else { 0 },
                    teacher.note,
                    norm_code,
                    teacher.display_name,
                    teacher.quota_override.map(|v| v as i64),
                    teacher.max_tasks_per_exam_override.map(|v| v as i64),
                    teacher.id.value(),
                ],
            )
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("teacher_code_duplicate".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "teacher with ID {} not found",
                teacher.id
            )));
        }
        Ok(())
    }

    pub fn delete_teacher(&self, id: TeacherId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM teachers WHERE id = ?1", params![id.value()])
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("teacher_in_use".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "teacher with ID {id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // School Years
    // -------------------------------------------------------------------------

    pub fn get_school_years(&self) -> Result<Vec<SchoolYear>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, is_current FROM school_years ORDER BY id DESC")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map([], |row| {
                let current_int: i32 = row.get(2)?;
                Ok(SchoolYear {
                    id: SchoolYearId(row.get(0)?),
                    name: row.get(1)?,
                    is_current: current_int == 1,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut years = Vec::new();
        for r in rows {
            years.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(years)
    }

    pub fn get_school_year(&self, id: SchoolYearId) -> Result<Option<SchoolYear>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, is_current FROM school_years WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                let current_int: i32 = row.get(2)?;
                Ok(SchoolYear {
                    id: SchoolYearId(row.get(0)?),
                    name: row.get(1)?,
                    is_current: current_int == 1,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn get_current_school_year(&self) -> Result<Option<SchoolYear>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, is_current FROM school_years WHERE is_current = 1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map([], |row| {
                Ok(SchoolYear {
                    id: SchoolYearId(row.get(0)?),
                    name: row.get(1)?,
                    is_current: true,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn set_current_school_year(&self, id: SchoolYearId) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;
        tx.execute(
            "UPDATE school_years SET is_current = 0 WHERE is_current = 1",
            [],
        )
        .map_err(StorageError::from_sqlite)?;
        let rows = tx
            .execute(
                "UPDATE school_years SET is_current = 1 WHERE id = ?1",
                params![id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "school year with ID {id} not found"
            )));
        }
        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    pub fn delete_school_year(&self, id: SchoolYearId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "DELETE FROM school_years WHERE id = ?1",
                params![id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "school year with ID {id} not found"
            )));
        }
        Ok(())
    }

    /// Creates a new school year, its 4 default exams (GK1, CK1, GK2, CK2),
    /// default rule settings, and optionally copies teacher grade assignments from a template year.
    /// All operations run in a single transaction.
    pub fn create_school_year(
        &self,
        name: &str,
        template: Option<SchoolYearId>,
    ) -> Result<SchoolYear, StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        tx.execute(
            "INSERT INTO school_years (name, is_current) VALUES (?1, 0)",
            params![name],
        )
        .map_err(StorageError::from_sqlite)?;
        let sy_id = tx.last_insert_rowid();

        // 4 default exams
        let default_exams = [
            ("GK1", "Giữa kỳ 1", 1),
            ("CK1", "Cuối kỳ 1", 2),
            ("GK2", "Giữa kỳ 2", 3),
            ("CK2", "Cuối kỳ 2", 4),
        ];
        for (code, exam_name, sort_order) in default_exams {
            tx.execute(
                "INSERT INTO exams (school_year_id, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
                params![sy_id, code, exam_name, sort_order],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        // Default rule settings
        for rule in RuleSetting::default_settings() {
            let params_str = serde_json::to_string(&rule.params)?;
            tx.execute(
                "INSERT INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sy_id,
                    rule.key.as_str(),
                    if rule.enabled { 1 } else { 0 },
                    rule.weight,
                    params_str,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        // Template copy if requested
        if let Some(from_year) = template {
            tx.execute(
                "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id)
                 SELECT teacher_id, ?1, grade_id FROM teacher_grades WHERE school_year_id = ?2",
                params![sy_id, from_year.value()],
            )
            .map_err(StorageError::from_sqlite)?;

            // Copy subjects and map old IDs to new IDs
            let mut sub_map = std::collections::HashMap::new();
            {
                let mut stmt = tx
                    .prepare(
                        "SELECT id, code, name, color, sort_order, setters, reviewers, min_campuses FROM subjects WHERE school_year_id = ?1 ORDER BY sort_order ASC, id ASC",
                    )
                    .map_err(StorageError::from_sqlite)?;
                let rows = stmt
                    .query_map(params![from_year.value()], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i32>(4)?,
                            row.get::<_, u8>(5)?,
                            row.get::<_, u8>(6)?,
                            row.get::<_, u8>(7)?,
                        ))
                    })
                    .map_err(StorageError::from_sqlite)?;
                let mut subs = Vec::new();
                for r in rows {
                    subs.push(r.map_err(StorageError::from_sqlite)?);
                }
                drop(stmt);

                for (old_id, code, name, color, sort_order, setters, reviewers, min_campuses) in
                    subs
                {
                    tx.execute(
                        "INSERT INTO subjects (school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![sy_id, code, name, color, sort_order, setters, reviewers, min_campuses],
                    )
                    .map_err(StorageError::from_sqlite)?;
                    let new_id = tx.last_insert_rowid();
                    sub_map.insert(old_id, new_id);
                }
            }

            // Copy competencies using new subject IDs
            {
                let mut stmt = tx
                    .prepare(
                        "SELECT tc.teacher_id, tc.subject_id, tc.role, tc.grade_scope
                         FROM teacher_competencies tc
                         JOIN subjects s ON tc.subject_id = s.id
                         WHERE s.school_year_id = ?1",
                    )
                    .map_err(StorageError::from_sqlite)?;
                let rows = stmt
                    .query_map(params![from_year.value()], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })
                    .map_err(StorageError::from_sqlite)?;
                let mut comps = Vec::new();
                for r in rows {
                    comps.push(r.map_err(StorageError::from_sqlite)?);
                }
                drop(stmt);

                for (teacher_id, old_sub_id, role, grade_scope) in comps {
                    if let Some(&new_sub_id) = sub_map.get(&old_sub_id) {
                        tx.execute(
                            "INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
                             VALUES (?1, ?2, ?3, ?4)",
                            params![teacher_id, new_sub_id, role, grade_scope],
                        )
                        .map_err(StorageError::from_sqlite)?;
                    }
                }
            }
        } else {
            // Fresh school year: insert default subject CHUNG
            tx.execute(
                "INSERT INTO subjects (school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses)
                 VALUES (?1, 'CHUNG', 'Môn chung', 'primary', 1, 2, 1, 2)",
                params![sy_id],
            )
            .map_err(StorageError::from_sqlite)?;
            let chung_id = tx.last_insert_rowid();

            // Active teachers get setter and reviewer competencies for CHUNG
            tx.execute(
                "INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
                 SELECT id, ?1, 'setter', 'taught' FROM teachers WHERE active = 1
                 UNION ALL
                 SELECT id, ?1, 'reviewer', 'taught' FROM teachers WHERE active = 1",
                params![chung_id],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        tx.commit().map_err(StorageError::from_sqlite)?;

        Ok(SchoolYear {
            id: SchoolYearId(sy_id),
            name: name.to_string(),
            is_current: false,
        })
    }

    // -------------------------------------------------------------------------
    // Teacher Grades
    // -------------------------------------------------------------------------

    pub fn set_teacher_grades(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
        grade_ids: &[GradeId],
    ) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        tx.execute(
            "DELETE FROM teacher_grades WHERE teacher_id = ?1 AND school_year_id = ?2",
            params![teacher_id.value(), school_year_id.value()],
        )
        .map_err(StorageError::from_sqlite)?;

        for gid in grade_ids {
            tx.execute(
                "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id) VALUES (?1, ?2, ?3)",
                params![teacher_id.value(), school_year_id.value(), gid.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    pub fn get_teacher_grades(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<GradeId>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT grade_id FROM teacher_grades WHERE teacher_id = ?1 AND school_year_id = ?2",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![teacher_id.value(), school_year_id.value()], |row| {
                Ok(GradeId(row.get(0)?))
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn get_all_teacher_grades(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<TeacherGrade>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT teacher_id, school_year_id, grade_id FROM teacher_grades WHERE school_year_id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                Ok(TeacherGrade {
                    teacher_id: TeacherId(row.get(0)?),
                    school_year_id: SchoolYearId(row.get(1)?),
                    grade_id: GradeId(row.get(2)?),
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn copy_teacher_grades(
        &self,
        from_year: SchoolYearId,
        to_year: SchoolYearId,
    ) -> Result<usize, StorageError> {
        let rows = self
            .conn
            .execute(
                "INSERT INTO teacher_grades (teacher_id, school_year_id, grade_id)
                 SELECT teacher_id, ?1, grade_id FROM teacher_grades WHERE school_year_id = ?2",
                params![to_year.value(), from_year.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(rows)
    }

    // -------------------------------------------------------------------------
    // Exams
    // -------------------------------------------------------------------------

    pub fn get_exams(&self, school_year_id: SchoolYearId) -> Result<Vec<Exam>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, school_year_id, code, name, sort_order FROM exams WHERE school_year_id = ?1 ORDER BY sort_order ASC, id ASC")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                Ok(Exam {
                    id: ExamId(row.get(0)?),
                    school_year_id: SchoolYearId(row.get(1)?),
                    code: row.get(2)?,
                    name: row.get(3)?,
                    sort_order: row.get(4)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn get_exam(&self, id: ExamId) -> Result<Option<Exam>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, school_year_id, code, name, sort_order FROM exams WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                Ok(Exam {
                    id: ExamId(row.get(0)?),
                    school_year_id: SchoolYearId(row.get(1)?),
                    code: row.get(2)?,
                    name: row.get(3)?,
                    sort_order: row.get(4)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn create_exam(
        &self,
        school_year_id: SchoolYearId,
        code: &str,
        name: &str,
        sort_order: i32,
    ) -> Result<Exam, StorageError> {
        self.conn
            .execute(
                "INSERT INTO exams (school_year_id, code, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
                params![school_year_id.value(), code, name, sort_order],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Exam {
            id: ExamId(id),
            school_year_id,
            code: code.to_string(),
            name: name.to_string(),
            sort_order,
        })
    }

    pub fn update_exam(&self, exam: &Exam) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE exams SET code = ?1, name = ?2, sort_order = ?3 WHERE id = ?4",
                params![exam.code, exam.name, exam.sort_order, exam.id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "exam with ID {} not found",
                exam.id
            )));
        }
        Ok(())
    }

    pub fn delete_exam(&self, id: ExamId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM exams WHERE id = ?1", params![id.value()])
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("exam_in_use".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "exam with ID {id} not found"
            )));
        }
        Ok(())
    }

    pub fn reorder_exams(&self, exam_ids: &[ExamId]) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;
        for (idx, eid) in exam_ids.iter().enumerate() {
            tx.execute(
                "UPDATE exams SET sort_order = ?1 WHERE id = ?2",
                params![(idx + 1) as i32, eid.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        }
        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Locks
    // -------------------------------------------------------------------------

    pub fn get_locks(&self, school_year_id: SchoolYearId) -> Result<Vec<Lock>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT l.id, l.exam_id, l.grade_id, l.subject_id, l.teacher_id, l.role, l.kind
                 FROM locks l
                 JOIN exams e ON l.exam_id = e.id
                 WHERE e.school_year_id = ?1
                 ORDER BY l.id ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let role_str: Option<String> = row.get(5)?;
                let kind_str: String = row.get(6)?;

                let role = role_str.as_deref().and_then(|s| s.parse::<Role>().ok());
                let kind = kind_str.parse::<LockKind>().unwrap_or(LockKind::Pin);

                Ok(Lock {
                    id: LockId(row.get(0)?),
                    exam_id: ExamId(row.get(1)?),
                    grade_id: GradeId(row.get(2)?),
                    subject_id: SubjectId(row.get(3)?),
                    teacher_id: TeacherId(row.get(4)?),
                    role,
                    kind,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn create_lock(
        &self,
        exam_id: ExamId,
        grade_id: GradeId,
        subject_id: SubjectId,
        teacher_id: TeacherId,
        role: Option<Role>,
        kind: LockKind,
    ) -> Result<Lock, StorageError> {
        self.conn
            .execute(
                "INSERT INTO locks (exam_id, grade_id, subject_id, teacher_id, role, kind) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    exam_id.value(),
                    grade_id.value(),
                    subject_id.value(),
                    teacher_id.value(),
                    role.map(|r| r.as_str()),
                    kind.as_str(),
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Lock {
            id: LockId(id),
            exam_id,
            grade_id,
            subject_id,
            teacher_id,
            role,
            kind,
        })
    }

    pub fn delete_lock(&self, id: LockId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM locks WHERE id = ?1", params![id.value()])
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "lock with ID {id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Unavailability
    // -------------------------------------------------------------------------

    pub fn get_unavailabilities(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Unavailability>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT u.teacher_id, u.exam_id, u.reason
                 FROM unavailability u
                 JOIN exams e ON u.exam_id = e.id
                 WHERE e.school_year_id = ?1
                 ORDER BY u.teacher_id ASC, u.exam_id ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                Ok(Unavailability {
                    teacher_id: TeacherId(row.get(0)?),
                    exam_id: ExamId(row.get(1)?),
                    reason: row.get(2)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn set_unavailability(
        &self,
        teacher_id: TeacherId,
        exam_id: ExamId,
        reason: Option<&str>,
    ) -> Result<(), StorageError> {
        self.conn
            .execute(
                "INSERT INTO unavailability (teacher_id, exam_id, reason) VALUES (?1, ?2, ?3)
                 ON CONFLICT(teacher_id, exam_id) DO UPDATE SET reason = excluded.reason",
                params![teacher_id.value(), exam_id.value(), reason],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    pub fn delete_unavailability(
        &self,
        teacher_id: TeacherId,
        exam_id: ExamId,
    ) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "DELETE FROM unavailability WHERE teacher_id = ?1 AND exam_id = ?2",
                params![teacher_id.value(), exam_id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "unavailability for teacher {teacher_id} on exam {exam_id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Rule Settings
    // -------------------------------------------------------------------------

    pub fn get_rule_settings(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<RuleSetting>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT rule_key, enabled, weight, params_json FROM rule_settings WHERE school_year_id = ?1",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let key_str: String = row.get(0)?;
                let enabled_int: i32 = row.get(1)?;
                let weight: f64 = row.get(2)?;
                let params_json: String = row.get(3)?;

                let key = key_str.parse::<RuleKey>().map_err(|e| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(e)))
                })?;
                let params: serde_json::Value =
                    serde_json::from_str(&params_json).unwrap_or(serde_json::json!({}));

                Ok(RuleSetting {
                    key,
                    enabled: enabled_int == 1,
                    weight,
                    params,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn save_rule_setting(
        &self,
        school_year_id: SchoolYearId,
        setting: &RuleSetting,
    ) -> Result<(), StorageError> {
        let params_json = serde_json::to_string(&setting.params)?;
        self.conn
            .execute(
                "INSERT INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(school_year_id, rule_key) DO UPDATE SET
                   enabled = excluded.enabled,
                   weight = excluded.weight,
                   params_json = excluded.params_json",
                params![
                    school_year_id.value(),
                    setting.key.as_str(),
                    if setting.enabled { 1 } else { 0 },
                    setting.weight,
                    params_json,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Settings (KV)
    // -------------------------------------------------------------------------

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![key], |row| row.get(0))
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Plans and Assignments
    // -------------------------------------------------------------------------

    pub fn save_plan(
        &self,
        plan: &Plan,
        assignments: &[Assignment],
    ) -> Result<PlanId, StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        let plan_id_val = if plan.id.value() > 0 {
            tx.execute(
                "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   created_at = excluded.created_at,
                   seed = excluded.seed,
                   score = excluded.score,
                   is_final = excluded.is_final,
                   rank = excluded.rank,
                   score_report_json = excluded.score_report_json,
                   run_params_json = excluded.run_params_json,
                   source = excluded.source,
                   data_hash = excluded.data_hash,
                   rules_hash = excluded.rules_hash",
                params![
                    plan.id.value(),
                    plan.school_year_id.value(),
                    plan.name,
                    plan.created_at,
                    plan.seed as i64,
                    plan.score,
                    if plan.is_final { 1 } else { 0 },
                    plan.rank,
                    plan.score_report_json,
                    plan.run_params_json,
                    plan.source,
                    plan.data_hash,
                    plan.rules_hash,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
            plan.id.value()
        } else {
            tx.execute(
                "INSERT INTO plans (school_year_id, name, created_at, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    plan.school_year_id.value(),
                    plan.name,
                    plan.created_at,
                    plan.seed as i64,
                    plan.score,
                    if plan.is_final { 1 } else { 0 },
                    plan.rank,
                    plan.score_report_json,
                    plan.run_params_json,
                    plan.source,
                    plan.data_hash,
                    plan.rules_hash,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
            tx.last_insert_rowid()
        };

        // Clear and insert assignments
        tx.execute(
            "DELETE FROM assignments WHERE plan_id = ?1",
            params![plan_id_val],
        )
        .map_err(StorageError::from_sqlite)?;

        for a in assignments {
            tx.execute(
                "INSERT INTO assignments (plan_id, exam_id, grade_id, subject_id, teacher_id, role, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    plan_id_val,
                    a.exam_id.value(),
                    a.grade_id.value(),
                    a.subject_id.value(),
                    a.teacher_id.value(),
                    a.role.as_str(),
                    a.position as i64,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(PlanId(plan_id_val))
    }

    pub fn load_plan(&self, plan_id: PlanId) -> Result<(Plan, Vec<Assignment>), StorageError> {
        let mut plan_stmt = self
            .conn
            .prepare("SELECT id, school_year_id, name, created_at, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash FROM plans WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut plan_rows = plan_stmt
            .query_map(params![plan_id.value()], |row| {
                let final_int: i32 = row.get(6)?;
                let seed_i64: i64 = row.get(4)?;
                Ok(Plan {
                    id: PlanId(row.get(0)?),
                    school_year_id: SchoolYearId(row.get(1)?),
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    seed: seed_i64 as u64,
                    score: row.get(5)?,
                    is_final: final_int == 1,
                    rank: row.get(7)?,
                    score_report_json: row.get(8)?,
                    run_params_json: row.get(9)?,
                    source: row.get(10)?,
                    data_hash: row.get(11)?,
                    rules_hash: row.get(12)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let plan = match plan_rows.next() {
            Some(res) => res.map_err(StorageError::from_sqlite)?,
            None => {
                return Err(StorageError::NotFound(format!(
                    "plan with ID {plan_id} not found"
                )))
            }
        };

        let mut a_stmt = self
            .conn
            .prepare("SELECT plan_id, exam_id, grade_id, subject_id, teacher_id, role, position FROM assignments WHERE plan_id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let a_rows = a_stmt
            .query_map(params![plan_id.value()], |row| {
                let role_str: String = row.get(5)?;
                let role = role_str.parse::<Role>().unwrap_or(Role::Setter);
                let pos_i64: i64 = row.get(6)?;
                Ok(Assignment {
                    plan_id: PlanId(row.get(0)?),
                    exam_id: ExamId(row.get(1)?),
                    grade_id: GradeId(row.get(2)?),
                    subject_id: SubjectId(row.get(3)?),
                    teacher_id: TeacherId(row.get(4)?),
                    role,
                    position: pos_i64 as usize,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut assignments = Vec::new();
        for r in a_rows {
            assignments.push(r.map_err(StorageError::from_sqlite)?);
        }

        Ok((plan, assignments))
    }

    pub fn list_plans(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<PlanSummary>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, rank, score, created_at, is_final, source, data_hash, rules_hash
                 FROM plans
                 WHERE school_year_id = ?1
                 ORDER BY is_final DESC, rank ASC NULLS LAST, id DESC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let final_int: i32 = row.get(5)?;
                Ok(PlanSummary {
                    id: PlanId(row.get(0)?),
                    name: row.get(1)?,
                    rank: row.get(2)?,
                    score: row.get(3)?,
                    created_at: row.get(4)?,
                    is_final: final_int == 1,
                    source: row.get(6)?,
                    data_hash: row.get(7)?,
                    rules_hash: row.get(8)?,
                    is_stale: false,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn rename_plan(&self, id: PlanId, new_name: &str) -> Result<(), StorageError> {
        let affected = self
            .conn
            .execute(
                "UPDATE plans SET name = ?1 WHERE id = ?2",
                params![new_name, id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "plan with ID {id} not found"
            )));
        }
        Ok(())
    }

    pub fn delete_plan(&self, id: PlanId) -> Result<(), StorageError> {
        let affected = self
            .conn
            .execute("DELETE FROM plans WHERE id = ?1", params![id.value()])
            .map_err(StorageError::from_sqlite)?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "plan with ID {id} not found"
            )));
        }
        Ok(())
    }

    pub fn duplicate_plan(&self, id: PlanId, new_name: &str) -> Result<PlanId, StorageError> {
        let (original_plan, assignments) = self.load_plan(id)?;
        let new_plan = Plan {
            id: PlanId(0),
            school_year_id: original_plan.school_year_id,
            name: new_name.to_string(),
            created_at: original_plan.created_at,
            seed: original_plan.seed,
            score: original_plan.score,
            is_final: false,
            rank: None,
            score_report_json: original_plan.score_report_json,
            run_params_json: original_plan.run_params_json,
            source: "duplicate".to_string(),
            data_hash: original_plan.data_hash,
            rules_hash: original_plan.rules_hash,
        };
        self.save_plan(&new_plan, &assignments)
    }

    pub fn save_plans_batch(
        &self,
        plans: &[(Plan, Vec<Assignment>)],
    ) -> Result<Vec<PlanId>, StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        let mut ids = Vec::with_capacity(plans.len());
        for (plan, assignments) in plans {
            let plan_id_val = if plan.id.value() > 0 {
                tx.execute(
                    "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                     ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name,
                       created_at = excluded.created_at,
                       seed = excluded.seed,
                       score = excluded.score,
                       is_final = excluded.is_final,
                       rank = excluded.rank,
                       score_report_json = excluded.score_report_json,
                       run_params_json = excluded.run_params_json,
                       source = excluded.source,
                       data_hash = excluded.data_hash,
                       rules_hash = excluded.rules_hash",
                    params![
                        plan.id.value(),
                        plan.school_year_id.value(),
                        plan.name,
                        plan.created_at,
                        plan.seed as i64,
                        plan.score,
                        if plan.is_final { 1 } else { 0 },
                        plan.rank,
                        plan.score_report_json,
                        plan.run_params_json,
                        plan.source,
                        plan.data_hash,
                        plan.rules_hash,
                    ],
                )
                .map_err(StorageError::from_sqlite)?;
                plan.id.value()
            } else {
                tx.execute(
                    "INSERT INTO plans (school_year_id, name, created_at, seed, score, is_final, rank, score_report_json, run_params_json, source, data_hash, rules_hash)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        plan.school_year_id.value(),
                        plan.name,
                        plan.created_at,
                        plan.seed as i64,
                        plan.score,
                        if plan.is_final { 1 } else { 0 },
                        plan.rank,
                        plan.score_report_json,
                        plan.run_params_json,
                        plan.source,
                        plan.data_hash,
                        plan.rules_hash,
                    ],
                    )
                .map_err(StorageError::from_sqlite)?;
                tx.last_insert_rowid()
            };

            // Clear and insert assignments
            tx.execute(
                "DELETE FROM assignments WHERE plan_id = ?1",
                params![plan_id_val],
            )
            .map_err(StorageError::from_sqlite)?;

            for a in assignments {
                tx.execute(
                    "INSERT INTO assignments (plan_id, exam_id, grade_id, subject_id, teacher_id, role, position)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        plan_id_val,
                        a.exam_id.value(),
                        a.grade_id.value(),
                        a.subject_id.value(),
                        a.teacher_id.value(),
                        a.role.as_str(),
                        a.position as i64,
                    ],
                )
                .map_err(StorageError::from_sqlite)?;
            }

            ids.push(PlanId(plan_id_val));
        }

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(ids)
    }

    pub fn deactivate_teacher(&self, id: TeacherId) -> Result<(), StorageError> {
        let affected = self
            .conn
            .execute(
                "UPDATE teachers SET active = 0, updated_at = datetime('now') WHERE id = ?1",
                params![id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "teacher with ID {id} not found"
            )));
        }
        Ok(())
    }

    pub fn teachers_with_grades(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<TeacherWithGrades>, StorageError> {
        let teachers = self.get_teachers()?;
        let all_tg = self.get_all_teacher_grades(school_year_id)?;
        let mut tg_map: std::collections::HashMap<TeacherId, Vec<GradeId>> =
            std::collections::HashMap::new();
        for tg in all_tg {
            tg_map.entry(tg.teacher_id).or_default().push(tg.grade_id);
        }

        let result = teachers
            .into_iter()
            .map(|t| {
                let tid = t.id;
                let grade_ids = tg_map.remove(&tid).unwrap_or_default();
                TeacherWithGrades {
                    teacher: t,
                    grade_ids,
                }
            })
            .collect();
        Ok(result)
    }

    pub fn reset_rule_settings_to_defaults(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        tx.execute(
            "DELETE FROM rule_settings WHERE school_year_id = ?1",
            params![school_year_id.value()],
        )
        .map_err(StorageError::from_sqlite)?;

        for setting in RuleSetting::default_settings() {
            tx.execute(
                "INSERT INTO rule_settings (school_year_id, rule_key, enabled, weight, params_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    school_year_id.value(),
                    setting.key.as_str(),
                    if setting.enabled { 1 } else { 0 },
                    setting.weight,
                    setting.params.to_string(),
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    /// Marks the specified plan as final, transactionally unmarking any other final plan
    /// in the same school year.
    pub fn mark_final(&self, plan_id: PlanId) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        // Find school_year_id
        let sy_id: i64 = tx
            .query_row(
                "SELECT school_year_id FROM plans WHERE id = ?1",
                params![plan_id.value()],
                |r| r.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StorageError::NotFound(format!("plan with ID {plan_id} not found"))
                }
                other => StorageError::from_sqlite(other),
            })?;

        // Unmark other final plans in that school year
        tx.execute(
            "UPDATE plans SET is_final = 0 WHERE school_year_id = ?1 AND is_final = 1",
            params![sy_id],
        )
        .map_err(StorageError::from_sqlite)?;

        // Mark this plan as final
        tx.execute(
            "UPDATE plans SET is_final = 1 WHERE id = ?1",
            params![plan_id.value()],
        )
        .map_err(StorageError::from_sqlite)?;

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Problem Snapshot Loader
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // Subjects
    // -------------------------------------------------------------------------

    pub fn get_subjects(&self, school_year_id: SchoolYearId) -> Result<Vec<Subject>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, code, name, color, sort_order, setters, reviewers, min_campuses
                 FROM subjects
                 WHERE school_year_id = ?1
                 ORDER BY sort_order ASC, id ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let sort_order_i64: i64 = row.get(4)?;
                Ok(Subject {
                    id: SubjectId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    sort_order: sort_order_i64 as u32,
                    setters: row.get(5)?,
                    reviewers: row.get(6)?,
                    min_campuses: row.get(7)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn get_subject(&self, id: SubjectId) -> Result<Option<Subject>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, code, name, color, sort_order, setters, reviewers, min_campuses
                 FROM subjects
                 WHERE id = ?1",
            )
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                let sort_order_i64: i64 = row.get(4)?;
                Ok(Subject {
                    id: SubjectId(row.get(0)?),
                    code: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    sort_order: sort_order_i64 as u32,
                    setters: row.get(5)?,
                    reviewers: row.get(6)?,
                    min_campuses: row.get(7)?,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        match rows.next() {
            Some(res) => Ok(Some(res.map_err(StorageError::from_sqlite)?)),
            None => Ok(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_subject(
        &self,
        school_year_id: SchoolYearId,
        code: &str,
        name: &str,
        color: &str,
        sort_order: u32,
        setters: u8,
        reviewers: u8,
        min_campuses: u8,
    ) -> Result<Subject, StorageError> {
        self.conn
            .execute(
                "INSERT INTO subjects (school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    school_year_id.value(),
                    code,
                    name,
                    color,
                    sort_order as i64,
                    setters,
                    reviewers,
                    min_campuses,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Subject {
            id: SubjectId(id),
            code: code.to_string(),
            name: name.to_string(),
            color: color.to_string(),
            sort_order,
            setters,
            reviewers,
            min_campuses,
        })
    }

    pub fn update_subject(&self, subject: &Subject) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE subjects
                 SET code = ?1, name = ?2, color = ?3, sort_order = ?4, setters = ?5, reviewers = ?6, min_campuses = ?7
                 WHERE id = ?8",
                params![
                    subject.code,
                    subject.name,
                    subject.color,
                    subject.sort_order,
                    subject.setters,
                    subject.reviewers,
                    subject.min_campuses,
                    subject.id.value(),
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "subject with ID {} not found",
                subject.id
            )));
        }
        Ok(())
    }

    pub fn delete_subject(&self, id: SubjectId) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM subjects WHERE id = ?1", params![id.value()])
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(ref f, _)
                    if f.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    StorageError::Constraint("subject_in_use".to_string())
                }
                other => StorageError::from_sqlite(other),
            })?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "subject with ID {id} not found"
            )));
        }
        Ok(())
    }

    pub fn reorder_subjects(
        &self,
        school_year_id: SchoolYearId,
        ordered_ids: &[SubjectId],
    ) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;
        for (idx, sid) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE subjects SET sort_order = ?1 WHERE id = ?2 AND school_year_id = ?3",
                params![(idx + 1) as i32, sid.value(), school_year_id.value()],
            )
            .map_err(StorageError::from_sqlite)?;
        }
        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Teacher Competencies
    // -------------------------------------------------------------------------

    pub fn get_competencies(
        &self,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Competency>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT tc.teacher_id, tc.subject_id, tc.role, tc.grade_scope
                 FROM teacher_competencies tc
                 JOIN subjects s ON tc.subject_id = s.id
                 WHERE s.school_year_id = ?1
                 ORDER BY tc.teacher_id ASC, tc.subject_id ASC, tc.role ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let role_str: String = row.get(2)?;
                let scope_str: String = row.get(3)?;
                let role = role_str.parse::<Role>().unwrap_or(Role::Setter);
                let grade_scope = match scope_str.as_str() {
                    "any" => GradeScope::Any,
                    _ => GradeScope::Taught,
                };
                Ok(Competency {
                    teacher_id: TeacherId(row.get(0)?),
                    subject_id: SubjectId(row.get(1)?),
                    role,
                    grade_scope,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn get_teacher_competencies(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
    ) -> Result<Vec<Competency>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT tc.teacher_id, tc.subject_id, tc.role, tc.grade_scope
                 FROM teacher_competencies tc
                 JOIN subjects s ON tc.subject_id = s.id
                 WHERE tc.teacher_id = ?1 AND s.school_year_id = ?2
                 ORDER BY tc.subject_id ASC, tc.role ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![teacher_id.value(), school_year_id.value()], |row| {
                let role_str: String = row.get(2)?;
                let scope_str: String = row.get(3)?;
                let role = role_str.parse::<Role>().unwrap_or(Role::Setter);
                let grade_scope = match scope_str.as_str() {
                    "any" => GradeScope::Any,
                    _ => GradeScope::Taught,
                };
                Ok(Competency {
                    teacher_id: TeacherId(row.get(0)?),
                    subject_id: SubjectId(row.get(1)?),
                    role,
                    grade_scope,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(StorageError::from_sqlite)?);
        }
        Ok(list)
    }

    pub fn set_competency(
        &self,
        teacher_id: TeacherId,
        subject_id: SubjectId,
        role: Role,
        grade_scope: GradeScope,
    ) -> Result<(), StorageError> {
        let scope_str = match grade_scope {
            GradeScope::Taught => "taught",
            GradeScope::Any => "any",
        };
        self.conn
            .execute(
                "INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(teacher_id, subject_id, role) DO UPDATE SET
                   grade_scope = excluded.grade_scope",
                params![
                    teacher_id.value(),
                    subject_id.value(),
                    role.as_str(),
                    scope_str,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    pub fn delete_competency(
        &self,
        teacher_id: TeacherId,
        subject_id: SubjectId,
        role: Role,
    ) -> Result<(), StorageError> {
        self.conn
            .execute(
                "DELETE FROM teacher_competencies
                 WHERE teacher_id = ?1 AND subject_id = ?2 AND role = ?3",
                params![teacher_id.value(), subject_id.value(), role.as_str(),],
            )
            .map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    pub fn replace_teacher_competencies(
        &self,
        teacher_id: TeacherId,
        school_year_id: SchoolYearId,
        competencies: &[Competency],
    ) -> Result<(), StorageError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StorageError::from_sqlite)?;

        tx.execute(
            "DELETE FROM teacher_competencies
             WHERE teacher_id = ?1 AND subject_id IN (SELECT id FROM subjects WHERE school_year_id = ?2)",
            params![teacher_id.value(), school_year_id.value()],
        )
        .map_err(StorageError::from_sqlite)?;

        for c in competencies {
            let scope_str = match c.grade_scope {
                GradeScope::Taught => "taught",
                GradeScope::Any => "any",
            };
            tx.execute(
                "INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    teacher_id.value(),
                    c.subject_id.value(),
                    c.role.as_str(),
                    scope_str,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        }

        tx.commit().map_err(StorageError::from_sqlite)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Problem Snapshot
    // -------------------------------------------------------------------------

    /// Loads the complete problem snapshot for a single school year:
    /// - School year details
    /// - All campuses
    /// - All grades
    /// - All subjects for the year
    /// - Active teachers and their grade qualifications for the year
    /// - Active teacher competencies for the year
    /// - Exams for the year
    /// - Unavailability entries for the year
    /// - Panel locks for the year
    /// - Rule settings for the year
    pub fn load_problem(&self, school_year_id: SchoolYearId) -> Result<Problem, StorageError> {
        let school_year = self.get_school_year(school_year_id)?.ok_or_else(|| {
            StorageError::NotFound(format!("school year with ID {school_year_id} not found"))
        })?;

        let campuses = self.get_campuses()?;
        let grades = self.get_grades()?;
        let subjects = self.get_subjects(school_year_id)?;

        // Active teachers only
        let all_teachers = self.get_teachers()?;
        let active_teachers: Vec<Teacher> = all_teachers.into_iter().filter(|t| t.active).collect();
        let active_ids: std::collections::HashSet<TeacherId> =
            active_teachers.iter().map(|t| t.id).collect();

        // Teacher grades filtered to active teachers and this school year
        let teacher_grades: Vec<TeacherGrade> = self
            .get_all_teacher_grades(school_year_id)?
            .into_iter()
            .filter(|tg| active_ids.contains(&tg.teacher_id))
            .collect();

        // Competencies filtered to active teachers and this school year
        let competencies: Vec<Competency> = self
            .get_competencies(school_year_id)?
            .into_iter()
            .filter(|c| active_ids.contains(&c.teacher_id))
            .collect();

        let exams = self.get_exams(school_year_id)?;
        let unavailabilities: Vec<Unavailability> = self
            .get_unavailabilities(school_year_id)?
            .into_iter()
            .filter(|u| active_ids.contains(&u.teacher_id))
            .collect();

        let locks = self.get_locks(school_year_id)?;
        let mut rule_settings = self.get_rule_settings(school_year_id)?;
        for default_rule in RuleSetting::default_settings() {
            if !rule_settings.iter().any(|r| r.key == default_rule.key) {
                rule_settings.push(default_rule);
            }
        }

        Ok(Problem {
            school_year,
            campuses,
            grades,
            subjects,
            teachers: active_teachers,
            teacher_grades,
            competencies,
            exams,
            unavailabilities,
            locks,
            rule_settings,
        })
    }
}
