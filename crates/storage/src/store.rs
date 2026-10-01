//! Strongly typed SQLite store and repository operations.

use crate::migrations::run_migrations;
use crate::seeds::seed_defaults;
use crate::StorageError;
use exam_panel_core::domain::{
    Assignment, Campus, CampusId, Exam, ExamId, Grade, GradeId, Lock, LockKind, Plan, PlanId,
    Problem, Role, RuleKey, RuleSetting, SchoolYear, SchoolYearId, Teacher, TeacherGrade,
    TeacherId, Unavailability,
};
use rusqlite::{params, Connection};
use std::path::Path;

/// Strongly typed storage interface for ExamPanel.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Creates a store from an established SQLite connection.
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        Self { conn }
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
        Ok(Self { conn })
    }

    /// Opens or creates a database at the specified path with foreign keys and rollback journal,
    /// runs migrations, and applies default seeds.
    pub fn open_at<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut conn = Connection::open(path).map_err(StorageError::from_sqlite)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")
            .map_err(StorageError::from_sqlite)?;
        run_migrations(&mut conn)?;
        seed_defaults(&conn)?;
        Ok(Self { conn })
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
            .map_err(StorageError::from_sqlite)?;
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
            .prepare("SELECT id, full_name, campus_id, load_weight, active, note FROM teachers ORDER BY id ASC")
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map([], |row| {
                let active_int: i32 = row.get(4)?;
                Ok(Teacher {
                    id: TeacherId(row.get(0)?),
                    full_name: row.get(1)?,
                    campus_id: CampusId(row.get(2)?),
                    load_weight: row.get(3)?,
                    active: active_int == 1,
                    note: row.get(5)?,
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
            .prepare("SELECT id, full_name, campus_id, load_weight, active, note FROM teachers WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut rows = stmt
            .query_map(params![id.value()], |row| {
                let active_int: i32 = row.get(4)?;
                Ok(Teacher {
                    id: TeacherId(row.get(0)?),
                    full_name: row.get(1)?,
                    campus_id: CampusId(row.get(2)?),
                    load_weight: row.get(3)?,
                    active: active_int == 1,
                    note: row.get(5)?,
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
    ) -> Result<Teacher, StorageError> {
        self.conn
            .execute(
                "INSERT INTO teachers (full_name, campus_id, load_weight, active, note) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    full_name,
                    campus_id.value(),
                    load_weight,
                    if active { 1 } else { 0 },
                    note,
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Teacher {
            id: TeacherId(id),
            full_name: full_name.to_string(),
            campus_id,
            load_weight,
            active,
            note: note.map(ToString::to_string),
        })
    }

    pub fn update_teacher(&self, teacher: &Teacher) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE teachers SET full_name = ?1, campus_id = ?2, load_weight = ?3, active = ?4, note = ?5, updated_at = datetime('now') WHERE id = ?6",
                params![
                    teacher.full_name,
                    teacher.campus_id.value(),
                    teacher.load_weight,
                    if teacher.active { 1 } else { 0 },
                    teacher.note,
                    teacher.id.value(),
                ],
            )
            .map_err(StorageError::from_sqlite)?;
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
            .map_err(StorageError::from_sqlite)?;
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
            .map_err(StorageError::from_sqlite)?;
        if rows == 0 {
            return Err(StorageError::NotFound(format!(
                "exam with ID {id} not found"
            )));
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Locks
    // -------------------------------------------------------------------------

    pub fn get_locks(&self, school_year_id: SchoolYearId) -> Result<Vec<Lock>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT l.id, l.exam_id, l.grade_id, l.teacher_id, l.role, l.kind
                 FROM locks l
                 JOIN exams e ON l.exam_id = e.id
                 WHERE e.school_year_id = ?1
                 ORDER BY l.id ASC",
            )
            .map_err(StorageError::from_sqlite)?;
        let rows = stmt
            .query_map(params![school_year_id.value()], |row| {
                let role_str: Option<String> = row.get(4)?;
                let kind_str: String = row.get(5)?;

                let role = role_str.as_deref().and_then(|s| s.parse::<Role>().ok());
                let kind = kind_str.parse::<LockKind>().unwrap_or(LockKind::Pin);

                Ok(Lock {
                    id: row.get(0)?,
                    exam_id: ExamId(row.get(1)?),
                    grade_id: GradeId(row.get(2)?),
                    teacher_id: TeacherId(row.get(3)?),
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
        teacher_id: TeacherId,
        role: Option<Role>,
        kind: LockKind,
    ) -> Result<Lock, StorageError> {
        self.conn
            .execute(
                "INSERT INTO locks (exam_id, grade_id, teacher_id, role, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    exam_id.value(),
                    grade_id.value(),
                    teacher_id.value(),
                    role.map(|r| r.as_str()),
                    kind.as_str(),
                ],
            )
            .map_err(StorageError::from_sqlite)?;
        let id = self.conn.last_insert_rowid();
        Ok(Lock {
            id,
            exam_id,
            grade_id,
            teacher_id,
            role,
            kind,
        })
    }

    pub fn delete_lock(&self, id: i64) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute("DELETE FROM locks WHERE id = ?1", params![id])
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
                "INSERT INTO plans (id, school_year_id, name, created_at, seed, score, is_final)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   created_at = excluded.created_at,
                   seed = excluded.seed,
                   score = excluded.score,
                   is_final = excluded.is_final",
                params![
                    plan.id.value(),
                    plan.school_year_id.value(),
                    plan.name,
                    plan.created_at,
                    plan.seed,
                    plan.score,
                    if plan.is_final { 1 } else { 0 },
                ],
            )
            .map_err(StorageError::from_sqlite)?;
            plan.id.value()
        } else {
            tx.execute(
                "INSERT INTO plans (school_year_id, name, created_at, seed, score, is_final)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    plan.school_year_id.value(),
                    plan.name,
                    plan.created_at,
                    plan.seed,
                    plan.score,
                    if plan.is_final { 1 } else { 0 },
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
                "INSERT INTO assignments (plan_id, exam_id, grade_id, teacher_id, role)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    plan_id_val,
                    a.exam_id.value(),
                    a.grade_id.value(),
                    a.teacher_id.value(),
                    a.role.as_str(),
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
            .prepare("SELECT id, school_year_id, name, created_at, seed, score, is_final FROM plans WHERE id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let mut plan_rows = plan_stmt
            .query_map(params![plan_id.value()], |row| {
                let final_int: i32 = row.get(6)?;
                Ok(Plan {
                    id: PlanId(row.get(0)?),
                    school_year_id: SchoolYearId(row.get(1)?),
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    seed: row.get(4)?,
                    score: row.get(5)?,
                    is_final: final_int == 1,
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
            .prepare("SELECT plan_id, exam_id, grade_id, teacher_id, role FROM assignments WHERE plan_id = ?1")
            .map_err(StorageError::from_sqlite)?;
        let a_rows = a_stmt
            .query_map(params![plan_id.value()], |row| {
                let role_str: String = row.get(4)?;
                let role = role_str.parse::<Role>().unwrap_or(Role::Setter);
                Ok(Assignment {
                    plan_id: PlanId(row.get(0)?),
                    exam_id: ExamId(row.get(1)?),
                    grade_id: GradeId(row.get(2)?),
                    teacher_id: TeacherId(row.get(3)?),
                    role,
                })
            })
            .map_err(StorageError::from_sqlite)?;

        let mut assignments = Vec::new();
        for r in a_rows {
            assignments.push(r.map_err(StorageError::from_sqlite)?);
        }

        Ok((plan, assignments))
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

    /// Loads the complete problem snapshot for a single school year:
    /// - School year details
    /// - All campuses
    /// - All grades
    /// - Active teachers and their grade qualifications for the year
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

        let exams = self.get_exams(school_year_id)?;
        let unavailabilities: Vec<Unavailability> = self
            .get_unavailabilities(school_year_id)?
            .into_iter()
            .filter(|u| active_ids.contains(&u.teacher_id))
            .collect();

        let locks = self.get_locks(school_year_id)?;
        let rule_settings = self.get_rule_settings(school_year_id)?;

        Ok(Problem {
            school_year,
            campuses,
            grades,
            teachers: active_teachers,
            teacher_grades,
            exams,
            unavailabilities,
            locks,
            rule_settings,
        })
    }
}
