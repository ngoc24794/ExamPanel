//! ExamPanel Storage Library
//!
//! SQLite database connection, schema migrations, and repository access.

pub mod migrations;
pub mod paths;
pub mod seeds;
pub mod store;

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

pub use migrations::{get_current_version, latest_version, run_migrations};
pub use seeds::{seed_defaults, seed_demo};
pub use store::Store;

/// Storage errors that can occur during database operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    Constraint(String),

    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("I/O error during storage operations: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl StorageError {
    /// Maps a rusqlite error into a structured StorageError,
    /// translating SQLite constraint violations into `StorageError::Constraint`.
    #[must_use]
    pub fn from_sqlite(err: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(ref f, Some(ref msg)) = err {
            if f.code == rusqlite::ErrorCode::ConstraintViolation {
                return Self::Constraint(msg.clone());
            }
        }
        Self::Sqlite(err)
    }
}

/// Opens an in-memory SQLite database connection with foreign keys enabled.
pub fn open_in_memory() -> Result<Connection, StorageError> {
    let conn = Connection::open_in_memory().map_err(StorageError::from_sqlite)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(StorageError::from_sqlite)?;
    Ok(conn)
}

/// Opens or creates an SQLite database at the specified path using the default
/// rollback journal mode (`DELETE`), keeping database storage in a single portable file.
pub fn open_connection<P: AsRef<Path>>(path: P) -> Result<Connection, StorageError> {
    let conn = Connection::open(path).map_err(StorageError::from_sqlite)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")
        .map_err(StorageError::from_sqlite)?;
    Ok(conn)
}

/// Opens or creates the SQLite database at the resolved portable/installed data path.
pub fn open_database() -> Result<Connection, StorageError> {
    let data_dir = paths::resolve_data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    let db_path = paths::resolve_database_path();
    open_connection(db_path)
}

/// Opens the high-level Store at the default resolved portable/installed database path.
pub fn open_default() -> Result<Store, StorageError> {
    Store::open_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use exam_panel_core::domain::{Assignment, Plan, PlanId, Role};

    #[test]
    fn test_open_in_memory_connection_succeeds() {
        let conn = open_in_memory().expect("failed to open in-memory database");
        let result: i32 = conn
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("failed to run sample query");
        assert_eq!(result, 1);
    }

    #[test]
    fn test_migrations_and_idempotency() {
        let mut conn = open_in_memory().unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), 0);

        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), latest_version());

        // Running twice is a complete no-op
        run_migrations(&mut conn).unwrap();
        assert_eq!(get_current_version(&conn).unwrap(), latest_version());
    }

    #[test]
    fn test_fk_and_check_constraints() {
        let store = Store::open_in_memory().unwrap();

        // 1. Create a campus and teacher
        let campus = store
            .create_campus("CS1", "Campus 1", "#ffffff")
            .expect("create campus");
        let _teacher = store
            .create_teacher("Teacher A", campus.id, 1.0, true, None)
            .expect("create teacher");

        // 2. Deleting campus that still has teachers must fail with Constraint error (ON DELETE RESTRICT)
        let delete_res = store.delete_campus(campus.id);
        assert!(
            matches!(delete_res, Err(StorageError::Constraint(_))),
            "expected Constraint violation when deleting campus with active teachers, got: {delete_res:?}"
        );

        // 3. Teacher load_weight CHECK constraint (> 1.0)
        let invalid_weight_res = store.create_teacher("Teacher B", campus.id, 1.5, true, None);
        assert!(
            matches!(invalid_weight_res, Err(StorageError::Constraint(_))),
            "expected Constraint violation for load_weight = 1.5, got: {invalid_weight_res:?}"
        );

        // 4. Invalid role CHECK constraint
        let raw_insert_invalid_role = store.conn().execute(
            "INSERT INTO assignments (plan_id, exam_id, grade_id, teacher_id, role) VALUES (1, 1, 1, 1, 'invalid_role')",
            [],
        );
        let err = raw_insert_invalid_role.map_err(StorageError::from_sqlite);
        assert!(
            matches!(err, Err(StorageError::Constraint(_))),
            "expected Constraint violation for invalid role, got: {err:?}"
        );
    }

    #[test]
    fn test_partial_unique_indexes() {
        let store = Store::open_in_memory().unwrap();

        // 1. First current school year
        store
            .conn()
            .execute(
                "INSERT INTO school_years (id, name, is_current) VALUES (1, '2025-2026', 1)",
                [],
            )
            .unwrap();

        // 2. Inserting a second school year with is_current = 1 must fail due to partial unique index
        let second_current = store.conn().execute(
            "INSERT INTO school_years (id, name, is_current) VALUES (2, '2026-2027', 1)",
            [],
        );
        let err = second_current.map_err(StorageError::from_sqlite);
        assert!(
            matches!(err, Err(StorageError::Constraint(_))),
            "expected partial unique index rejection for second current school year, got: {err:?}"
        );

        // 3. Test mark_final switches correctly
        let sy = store
            .create_school_year("2026-2027", None)
            .expect("create school year");
        let plan1 = Plan {
            id: PlanId(1),
            school_year_id: sy.id,
            name: "Candidate Plan 1".to_string(),
            created_at: "2026-10-01T00:00:00".to_string(),
            seed: 42,
            score: Some(15.0),
            is_final: false,
        };
        let plan2 = Plan {
            id: PlanId(2),
            school_year_id: sy.id,
            name: "Candidate Plan 2".to_string(),
            created_at: "2026-10-01T00:00:00".to_string(),
            seed: 43,
            score: Some(10.0),
            is_final: false,
        };
        store.save_plan(&plan1, &[]).expect("save plan 1");
        store.save_plan(&plan2, &[]).expect("save plan 2");

        // Mark plan 1 final
        store.mark_final(PlanId(1)).expect("mark plan 1 final");
        let (p1, _) = store.load_plan(PlanId(1)).unwrap();
        assert!(p1.is_final);

        // Mark plan 2 final -> plan 1 should now be non-final
        store.mark_final(PlanId(2)).expect("mark plan 2 final");
        let (p1_after, _) = store.load_plan(PlanId(1)).unwrap();
        let (p2_after, _) = store.load_plan(PlanId(2)).unwrap();
        assert!(!p1_after.is_final);
        assert!(p2_after.is_final);

        // Inserting another is_final=1 directly must fail
        let duplicate_final = store
            .conn()
            .execute("UPDATE plans SET is_final = 1 WHERE id = 1", []);
        let err2 = duplicate_final.map_err(StorageError::from_sqlite);
        assert!(
            matches!(err2, Err(StorageError::Constraint(_))),
            "expected partial unique index rejection for two final plans, got: {err2:?}"
        );
    }

    #[test]
    fn test_create_school_year_and_copy_teacher_grades() {
        let store = Store::open_in_memory().unwrap();

        // 1. Create school year 1 with 4 default exams and default rules
        let sy1 = store
            .create_school_year("2025-2026", None)
            .expect("create year 1");
        let exams1 = store.get_exams(sy1.id).expect("get exams");
        assert_eq!(exams1.len(), 4);
        assert_eq!(exams1[0].code, "GK1");
        assert_eq!(exams1[1].code, "CK1");
        assert_eq!(exams1[2].code, "GK2");
        assert_eq!(exams1[3].code, "CK2");

        let rules1 = store.get_rule_settings(sy1.id).expect("get rules");
        assert_eq!(rules1.len(), 10);
        assert!(rules1
            .iter()
            .any(|r| r.key == exam_panel_core::domain::RuleKey::S8));

        // Add campus, teacher, and grade assignment in sy1
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Nguyen Van A", campus.id, 1.0, true, None)
            .unwrap();
        let grades = store.get_grades().unwrap();
        store
            .set_teacher_grades(teacher.id, sy1.id, &[grades[0].id, grades[1].id])
            .expect("set teacher grades");

        // 2. Create school year 2 using sy1 as template
        let sy2 = store
            .create_school_year("2026-2027", Some(sy1.id))
            .expect("create year 2 with template");
        let exams2 = store.get_exams(sy2.id).expect("get exams 2");
        assert_eq!(exams2.len(), 4);

        let copied_grades = store.get_teacher_grades(teacher.id, sy2.id).unwrap();
        assert_eq!(copied_grades.len(), 2);
        assert!(copied_grades.contains(&grades[0].id));
        assert!(copied_grades.contains(&grades[1].id));

        // 3. Test copy_teacher_grades directly
        let sy3 = store
            .create_school_year("2027-2028", None)
            .expect("create year 3");
        let copied_count = store
            .copy_teacher_grades(sy1.id, sy3.id)
            .expect("copy teacher grades");
        assert_eq!(copied_count, 2);
        let grades3 = store.get_teacher_grades(teacher.id, sy3.id).unwrap();
        assert_eq!(grades3.len(), 2);
    }

    #[test]
    fn test_save_and_load_plan_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();

        let plan = Plan {
            id: PlanId(0), // Will be autoincremented
            school_year_id: sy.id,
            name: "Test Schedule Plan".to_string(),
            created_at: "2026-10-01T12:00:00Z".to_string(),
            seed: 12345,
            score: Some(8.5),
            is_final: false,
        };

        let assignments = vec![Assignment {
            plan_id: PlanId(0),
            exam_id: exams[0].id,
            grade_id: grades[0].id,
            teacher_id: teacher.id,
            role: Role::Setter,
        }];

        let saved_id = store.save_plan(&plan, &assignments).expect("save plan");
        assert!(saved_id.value() > 0);

        let (loaded_plan, loaded_assignments) = store.load_plan(saved_id).expect("load plan");
        assert_eq!(loaded_plan.id, saved_id);
        assert_eq!(loaded_plan.school_year_id, sy.id);
        assert_eq!(loaded_plan.name, "Test Schedule Plan");
        assert_eq!(loaded_plan.seed, 12345);
        assert_eq!(loaded_plan.score, Some(8.5));
        assert!(!loaded_plan.is_final);

        assert_eq!(loaded_assignments.len(), 1);
        assert_eq!(loaded_assignments[0].plan_id, saved_id);
        assert_eq!(loaded_assignments[0].exam_id, exams[0].id);
        assert_eq!(loaded_assignments[0].grade_id, grades[0].id);
        assert_eq!(loaded_assignments[0].teacher_id, teacher.id);
        assert_eq!(loaded_assignments[0].role, Role::Setter);
    }

    #[test]
    fn test_seed_demo_and_load_problem_validation() {
        let store = Store::open_in_memory().unwrap();
        seed_demo(store.conn()).expect("seed demo data");

        // School year 1 is current 2026-2027
        let current_sy = store
            .get_current_school_year()
            .expect("get current year")
            .expect("current year must exist");
        assert_eq!(current_sy.name, "2026-2027");

        let problem = store
            .load_problem(current_sy.id)
            .expect("load problem snapshot");

        // Verify counts from requirements:
        // 4 campuses, 11 teachers, 3 grades, 4 exams
        assert_eq!(problem.campuses.len(), 4, "expected 4 campuses");
        assert_eq!(problem.teachers.len(), 11, "expected 11 teachers");
        assert_eq!(problem.grades.len(), 3, "expected 3 grades");
        assert_eq!(problem.exams.len(), 4, "expected 4 exams");
        assert_eq!(
            problem.unavailabilities.len(),
            1,
            "expected 1 unavailability entry"
        );

        // Problem must pass validation with 0 errors
        let validation_errors = problem.validate();
        assert!(
            validation_errors.is_empty(),
            "demo problem snapshot failed structural validation: {validation_errors:?}"
        );
    }

    #[test]
    fn test_protect_historical_plans_on_delete_restrict() {
        let store = Store::open_in_memory().unwrap();
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();

        let plan = Plan {
            id: PlanId(0),
            school_year_id: sy.id,
            name: "Candidate Plan".to_string(),
            created_at: "2026-10-01T12:00:00Z".to_string(),
            seed: 42,
            score: Some(5.0),
            is_final: false,
        };
        let assignments = vec![Assignment {
            plan_id: PlanId(0),
            exam_id: exams[0].id,
            grade_id: grades[0].id,
            teacher_id: teacher.id,
            role: Role::Setter,
        }];
        store.save_plan(&plan, &assignments).expect("save plan");

        // Attempt to delete teacher in use -> StorageError::Constraint("teacher_in_use")
        let del_t = store.delete_teacher(teacher.id);
        assert!(
            matches!(del_t, Err(StorageError::Constraint(ref msg)) if msg == "teacher_in_use"),
            "expected StorageError::Constraint(\"teacher_in_use\"), got: {del_t:?}"
        );

        // Attempt to delete exam in use -> StorageError::Constraint("exam_in_use")
        let del_e = store.delete_exam(exams[0].id);
        assert!(
            matches!(del_e, Err(StorageError::Constraint(ref msg)) if msg == "exam_in_use"),
            "expected StorageError::Constraint(\"exam_in_use\"), got: {del_e:?}"
        );

        // Attempt to delete grade in use -> StorageError::Constraint("grade_in_use")
        let del_g = store.delete_grade(grades[0].id);
        assert!(
            matches!(del_g, Err(StorageError::Constraint(ref msg)) if msg == "grade_in_use"),
            "expected StorageError::Constraint(\"grade_in_use\"), got: {del_g:?}"
        );
    }

    #[test]
    fn test_locks_lock_id_delete_and_unique_expression_index() {
        use exam_panel_core::domain::LockKind;

        let store = Store::open_in_memory().unwrap();
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();

        // 1. Create a lock and verify it has LockId
        let lock1 = store
            .create_lock(exams[0].id, grades[0].id, teacher.id, None, LockKind::Pin)
            .expect("create lock 1");
        assert!(lock1.id.value() > 0);

        // 2. Duplicate lock with role=None must fail because COALESCE(role, 'any') will collide
        let dup1 = store.create_lock(exams[0].id, grades[0].id, teacher.id, None, LockKind::Pin);
        assert!(
            matches!(dup1, Err(StorageError::Constraint(_))),
            "duplicate lock with role=None should violate unique index, got: {dup1:?}"
        );

        // 3. Different role (e.g. Some(Setter)) does not collide with role=None
        let lock2 = store
            .create_lock(
                exams[0].id,
                grades[0].id,
                teacher.id,
                Some(Role::Setter),
                LockKind::Pin,
            )
            .expect("create lock with role");
        assert_ne!(lock1.id, lock2.id);

        // Duplicate with Some(Setter) must fail
        let dup2 = store.create_lock(
            exams[0].id,
            grades[0].id,
            teacher.id,
            Some(Role::Setter),
            LockKind::Pin,
        );
        assert!(
            matches!(dup2, Err(StorageError::Constraint(_))),
            "duplicate lock with role=Some(Setter) should violate unique index, got: {dup2:?}"
        );

        // 4. delete_lock with LockId
        store.delete_lock(lock1.id).expect("delete lock 1");
        let del_again = store.delete_lock(lock1.id);
        assert!(
            matches!(del_again, Err(StorageError::NotFound(_))),
            "deleting non-existent lock should return NotFound"
        );
    }
}
