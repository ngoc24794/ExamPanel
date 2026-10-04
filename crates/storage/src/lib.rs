//! ExamPanel Storage Library
//!
//! SQLite database connection, schema migrations, and repository access.

pub mod backup;
pub mod migrations;
pub mod paths;
pub mod seeds;
pub mod store;

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

pub use backup::{
    backup_database, create_automatic_backup, list_automatic_backups, restore_database,
    validate_backup_file, BackupFileInfo, BackupValidationSummary, MAX_AUTO_BACKUPS,
};
pub use migrations::{get_current_version, latest_version, run_migrations};
pub use seeds::{seed_defaults, seed_demo, seed_legacy_demo};
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

    #[error("Cơ sở dữ liệu được tạo bởi phiên bản mới hơn (user_version = {0})")]
    UnsupportedVersion(u32),

    #[error("Cơ sở dữ liệu bị lỗi hoặc bị hỏng: {0}")]
    DatabaseCorrupted(String),

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
            .create_teacher("Teacher A", campus.id, 1.0, true, None, None)
            .expect("create teacher");

        // 2. Deleting campus that still has teachers must fail with Constraint error (ON DELETE RESTRICT)
        let delete_res = store.delete_campus(campus.id);
        assert!(
            matches!(delete_res, Err(StorageError::Constraint(_))),
            "expected Constraint violation when deleting campus with active teachers, got: {delete_res:?}"
        );

        // 3. Teacher load_weight CHECK constraint (> 1.0)
        let invalid_weight_res =
            store.create_teacher("Teacher B", campus.id, 1.5, true, None, None);
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
            rank: Some(1),
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
        };
        let plan2 = Plan {
            id: PlanId(2),
            school_year_id: sy.id,
            name: "Candidate Plan 2".to_string(),
            created_at: "2026-10-01T00:00:00".to_string(),
            seed: 43,
            score: Some(10.0),
            is_final: false,
            rank: Some(2),
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
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
        assert_eq!(rules1.len(), 13);
        assert!(rules1
            .iter()
            .any(|r| r.key == exam_panel_core::domain::RuleKey::S8));

        // Add campus, teacher, and grade assignment in sy1
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Nguyen Van A", campus.id, 1.0, true, None, None)
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
            .create_teacher("Teacher 1", campus.id, 1.0, true, None, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();
        let subjects = store.get_subjects(sy.id).unwrap();

        let plan = Plan {
            id: PlanId(0), // Will be autoincremented
            school_year_id: sy.id,
            name: "Test Schedule Plan".to_string(),
            created_at: "2026-10-01T12:00:00Z".to_string(),
            seed: 12345,
            score: Some(8.5),
            is_final: false,
            rank: Some(1),
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
        };

        let assignments = vec![Assignment {
            plan_id: PlanId(0),
            exam_id: exams[0].id,
            grade_id: grades[0].id,
            subject_id: subjects[0].id,
            teacher_id: teacher.id,
            role: Role::Setter,
            position: 0,
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
        assert_eq!(loaded_assignments[0].subject_id, subjects[0].id);
        assert_eq!(loaded_assignments[0].teacher_id, teacher.id);
        assert_eq!(loaded_assignments[0].role, Role::Setter);
        assert_eq!(loaded_assignments[0].position, 0);
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

        // Verify counts from Q-shaped canonical problem:
        // 4 campuses, 12 teachers, 3 grades, 4 exams, 2 subjects
        assert_eq!(problem.campuses.len(), 4, "expected 4 campuses");
        assert_eq!(problem.teachers.len(), 12, "expected 12 teachers");
        assert_eq!(problem.grades.len(), 3, "expected 3 grades");
        assert_eq!(problem.exams.len(), 4, "expected 4 exams");
        assert_eq!(problem.subjects.len(), 2, "expected 2 subjects");
        assert_eq!(
            problem.unavailabilities.len(),
            0,
            "expected 0 unavailability entries"
        );

        // Problem must pass validation with 0 errors
        let validation_errors = problem.validate();
        assert!(
            validation_errors.is_empty(),
            "demo problem snapshot failed structural validation: {validation_errors:?}"
        );
    }

    #[test]
    fn test_seed_legacy_demo_validation() {
        let store = Store::open_in_memory().unwrap();
        seed_legacy_demo(store.conn()).expect("seed legacy demo data");

        let current_sy = store
            .get_current_school_year()
            .expect("get current year")
            .expect("current year must exist");

        let problem = store
            .load_problem(current_sy.id)
            .expect("load problem snapshot");

        assert_eq!(problem.campuses.len(), 4);
        assert_eq!(problem.teachers.len(), 11);
        assert_eq!(problem.grades.len(), 3);
        assert_eq!(problem.exams.len(), 4);
        assert_eq!(problem.unavailabilities.len(), 1);

        let validation_errors = problem.validate();
        assert!(
            validation_errors.is_empty(),
            "legacy demo failed structural validation: {validation_errors:?}"
        );
    }

    #[test]
    fn test_no_forbidden_terms_in_fixtures_and_seeds() {
        use exam_panel_core::domain::fixtures::{make_canonical_q_problem, QVariant};

        fn assert_no_forbidden(text: &str, location: &str) {
            let lower = text.to_lowercase();
            let stripped = lower.replace("cơ sở dữ liệu", "");
            assert!(
                !stripped.contains("cơ sở"),
                "Forbidden term 'cơ sở' found in {location}: '{text}'"
            );
        }

        // 1. Check canonical Q problems
        for variant in [QVariant::NoCampus, QVariant::SyntheticCampuses] {
            let p = make_canonical_q_problem(variant);
            for c in &p.campuses {
                assert_no_forbidden(&c.name, "campus.name");
                assert_no_forbidden(&c.code, "campus.code");
            }
            for t in &p.teachers {
                assert_no_forbidden(&t.full_name, "teacher.full_name");
                if let Some(ref d) = t.display_name {
                    assert_no_forbidden(d, "teacher.display_name");
                }
                if let Some(ref n) = t.note {
                    assert_no_forbidden(n, "teacher.note");
                }
            }
            for e in &p.exams {
                assert_no_forbidden(&e.name, "exam.name");
            }
            for s in &p.subjects {
                assert_no_forbidden(&s.name, "subject.name");
            }
        }

        // 2. Check seed_demo DB strings
        let mut conn = open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        seed_demo(&conn).unwrap();

        let mut stmt = conn.prepare("SELECT name FROM campuses").unwrap();
        let campus_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for name in campus_names {
            assert_no_forbidden(&name, "seed_demo campuses.name");
        }

        let mut stmt = conn.prepare("SELECT full_name FROM teachers").unwrap();
        let teacher_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for name in teacher_names {
            assert_no_forbidden(&name, "seed_demo teachers.full_name");
        }

        // 3. Check seed_legacy_demo DB strings
        let mut conn_legacy = open_in_memory().unwrap();
        run_migrations(&mut conn_legacy).unwrap();
        seed_legacy_demo(&conn_legacy).unwrap();

        let mut stmt = conn_legacy.prepare("SELECT name FROM campuses").unwrap();
        let campus_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for name in campus_names {
            assert_no_forbidden(&name, "seed_legacy_demo campuses.name");
        }
    }

    #[test]
    fn test_protect_historical_plans_on_delete_restrict() {
        let store = Store::open_in_memory().unwrap();
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();
        let subjects = store.get_subjects(sy.id).unwrap();

        let plan = Plan {
            id: PlanId(0),
            school_year_id: sy.id,
            name: "Candidate Plan".to_string(),
            created_at: "2026-10-01T12:00:00Z".to_string(),
            seed: 42,
            score: Some(5.0),
            is_final: false,
            rank: None,
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
        };
        let assignments = vec![Assignment {
            plan_id: PlanId(0),
            exam_id: exams[0].id,
            grade_id: grades[0].id,
            subject_id: subjects[0].id,
            teacher_id: teacher.id,
            role: Role::Setter,
            position: 0,
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

        // Attempt to delete subject in use -> StorageError::Constraint("subject_in_use")
        let del_s = store.delete_subject(subjects[0].id);
        assert!(
            matches!(del_s, Err(StorageError::Constraint(ref msg)) if msg == "subject_in_use"),
            "expected StorageError::Constraint(\"subject_in_use\"), got: {del_s:?}"
        );
    }

    #[test]
    fn test_locks_lock_id_delete_and_unique_expression_index() {
        use exam_panel_core::domain::LockKind;

        let store = Store::open_in_memory().unwrap();
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();
        let subjects = store.get_subjects(sy.id).unwrap();

        // 1. Create a lock and verify it has LockId
        let lock1 = store
            .create_lock(
                exams[0].id,
                grades[0].id,
                subjects[0].id,
                teacher.id,
                None,
                LockKind::Pin,
            )
            .expect("create lock 1");
        assert!(lock1.id.value() > 0);

        // 2. Duplicate lock with role=None must fail because COALESCE(role, 'any') will collide
        let dup1 = store.create_lock(
            exams[0].id,
            grades[0].id,
            subjects[0].id,
            teacher.id,
            None,
            LockKind::Pin,
        );
        assert!(
            matches!(dup1, Err(StorageError::Constraint(_))),
            "duplicate lock with role=None should violate unique index, got: {dup1:?}"
        );

        // 3. Different role (e.g. Some(Setter)) does not collide with role=None
        let lock2 = store
            .create_lock(
                exams[0].id,
                grades[0].id,
                subjects[0].id,
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
            subjects[0].id,
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

    #[test]
    fn test_plan_batch_operations_and_methods() {
        let store = Store::open_in_memory().expect("open store");
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();
        let teacher = store
            .create_teacher("Teacher 1", campus.id, 1.0, true, None, None)
            .unwrap();
        let exams = store.get_exams(sy.id).unwrap();
        let grades = store.get_grades().unwrap();

        // 1. Deactivate teacher and teachers_with_grades
        store
            .set_teacher_grades(teacher.id, sy.id, &[grades[0].id])
            .unwrap();
        let twg = store.teachers_with_grades(sy.id).unwrap();
        assert_eq!(twg.len(), 1);
        assert_eq!(twg[0].grade_ids, vec![grades[0].id]);

        store.deactivate_teacher(teacher.id).unwrap();
        let updated_t = store.get_teacher(teacher.id).unwrap().unwrap();
        assert!(!updated_t.active);

        // 2. Batch save plans
        let plan1 = Plan {
            id: PlanId(0),
            school_year_id: sy.id,
            name: "Plan A".to_string(),
            created_at: "2026-10-01T10:00:00Z".to_string(),
            seed: 100,
            score: Some(20.0),
            is_final: false,
            rank: Some(1),
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
        };
        let plan2 = Plan {
            id: PlanId(0),
            school_year_id: sy.id,
            name: "Plan B".to_string(),
            created_at: "2026-10-01T10:00:00Z".to_string(),
            seed: 101,
            score: Some(25.0),
            is_final: false,
            rank: Some(2),
            score_report_json: None,
            run_params_json: None,
            source: "optimizer".to_string(),
            data_hash: None,
            rules_hash: None,
        };

        let subjects = store.get_subjects(sy.id).unwrap();

        let asg1 = vec![Assignment {
            plan_id: PlanId(0),
            exam_id: exams[0].id,
            grade_id: grades[0].id,
            subject_id: subjects[0].id,
            teacher_id: teacher.id,
            role: Role::Setter,
            position: 0,
        }];

        let ids = store
            .save_plans_batch(&[(plan1, asg1), (plan2, vec![])])
            .expect("save batch");
        assert_eq!(ids.len(), 2);

        // 3. List plans
        let list = store.list_plans(sy.id).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, ids[0]);
        assert_eq!(list[0].name, "Plan A");

        // 4. Rename plan
        store.rename_plan(ids[0], "Renamed Plan A").unwrap();
        let (p1, asg) = store.load_plan(ids[0]).unwrap();
        assert_eq!(p1.name, "Renamed Plan A");
        assert_eq!(asg.len(), 1);

        // 5. Duplicate plan
        let dup_id = store.duplicate_plan(ids[0], "Copy Plan A").unwrap();
        let (p_dup, asg_dup) = store.load_plan(dup_id).unwrap();
        assert_eq!(p_dup.name, "Copy Plan A");
        assert_eq!(p_dup.source, "duplicate");
        assert_eq!(asg_dup.len(), 1);

        // 6. Delete plan
        store.delete_plan(dup_id).unwrap();
        assert!(store.load_plan(dup_id).is_err());

        // 7. Reset rule settings to defaults
        store.reset_rule_settings_to_defaults(sy.id).unwrap();
        let settings = store.get_rule_settings(sy.id).unwrap();
        assert_eq!(settings.len(), 13);
    }

    #[test]
    fn test_subjects_and_competencies_crud() {
        use exam_panel_core::domain::{Competency, GradeScope, Role};

        let store = Store::open_in_memory().expect("open store");
        let sy = store.create_school_year("2026-2027", None).unwrap();
        let campus = store.create_campus("CS1", "Campus 1", "#fff").unwrap();

        // 1. Teacher with display_name and overrides
        let teacher = store
            .create_teacher_full(
                "Nguyen Van Nghia",
                campus.id,
                1.0,
                true,
                None,
                Some("GV001"),
                Some("T Nghĩa"),
                Some(12),
                Some(3),
            )
            .expect("create teacher full");
        assert_eq!(teacher.display_name.as_deref(), Some("T Nghĩa"));
        assert_eq!(teacher.quota_override, Some(12));
        assert_eq!(teacher.max_tasks_per_exam_override, Some(3));

        // Update teacher overrides
        let mut teacher_mod = teacher.clone();
        teacher_mod.display_name = Some("Thầy Nghĩa".to_string());
        teacher_mod.quota_override = Some(14);
        store.update_teacher(&teacher_mod).unwrap();
        let loaded_teacher = store.get_teacher(teacher.id).unwrap().unwrap();
        assert_eq!(loaded_teacher.display_name.as_deref(), Some("Thầy Nghĩa"));
        assert_eq!(loaded_teacher.quota_override, Some(14));

        let initial_subjects = store.get_subjects(sy.id).unwrap();
        assert_eq!(initial_subjects.len(), 1); // CHUNG

        // 2. Create Subjects: VL and CN
        let vl = store
            .create_subject(sy.id, "VL", "Vật lí", "blue", 2, 2, 1, 2)
            .expect("create VL");
        let cn = store
            .create_subject(sy.id, "CN", "Công nghệ", "green", 3, 1, 1, 2)
            .expect("create CN");

        assert_eq!(vl.setters, 2);
        assert_eq!(vl.reviewers, 1);
        assert_eq!(cn.setters, 1);
        assert_eq!(cn.reviewers, 1);

        // Reorder subjects: CN, VL, CHUNG
        store
            .reorder_subjects(sy.id, &[cn.id, vl.id, initial_subjects[0].id])
            .expect("reorder");
        let subjects = store.get_subjects(sy.id).unwrap();
        assert_eq!(subjects.len(), 3);
        assert_eq!(subjects[0].code, "CN");
        assert_eq!(subjects[1].code, "VL");
        assert_eq!(subjects[2].code, "CHUNG");

        // 3. Competencies
        store
            .set_competency(teacher.id, cn.id, Role::Setter, GradeScope::Any)
            .expect("set setter competency");
        store
            .set_competency(teacher.id, cn.id, Role::Reviewer, GradeScope::Taught)
            .expect("set reviewer competency");

        let comps = store.get_teacher_competencies(teacher.id, sy.id).unwrap();
        assert_eq!(comps.len(), 2);
        assert!(comps
            .iter()
            .any(|c| c.role == Role::Setter && c.grade_scope == GradeScope::Any));
        assert!(comps
            .iter()
            .any(|c| c.role == Role::Reviewer && c.grade_scope == GradeScope::Taught));

        // Delete one competency
        store
            .delete_competency(teacher.id, cn.id, Role::Reviewer)
            .expect("delete competency");
        let comps_after = store.get_teacher_competencies(teacher.id, sy.id).unwrap();
        assert_eq!(comps_after.len(), 1);

        // Replace teacher competencies
        let new_comps = vec![
            Competency {
                teacher_id: teacher.id,
                subject_id: vl.id,
                role: Role::Setter,
                grade_scope: GradeScope::Taught,
            },
            Competency {
                teacher_id: teacher.id,
                subject_id: cn.id,
                role: Role::Setter,
                grade_scope: GradeScope::Any,
            },
        ];
        store
            .replace_teacher_competencies(teacher.id, sy.id, &new_comps)
            .expect("replace competencies");
        let comps_replaced = store.get_teacher_competencies(teacher.id, sy.id).unwrap();
        assert_eq!(comps_replaced.len(), 2);

        // Delete subject
        store.delete_subject(vl.id).expect("delete VL");
        let subjects_final = store.get_subjects(sy.id).unwrap();
        assert_eq!(subjects_final.len(), 2); // CHUNG + CN
    }
}
