use exam_panel_core::domain::CampusId;
use exam_panel_service::dto::CreateTeacherInput;
use exam_panel_service::error::AppError;
use exam_panel_service::service::AppService;
use exam_panel_storage::{StorageError, Store};

#[test]
fn test_startup_error_classification() {
    let unsupp = StorageError::UnsupportedVersion(99);
    let app_err = AppError::from(unsupp);
    assert_eq!(app_err.code, "unsupported_database_version");
    assert_eq!(app_err.params.get("version").unwrap(), "99");

    let corrupt = StorageError::DatabaseCorrupted("header malformed".to_string());
    let corrupt_err = AppError::from(corrupt);
    assert_eq!(corrupt_err.code, "database_corrupted");
    assert_eq!(
        corrupt_err.params.get("detail").unwrap(),
        "header malformed"
    );
}

#[test]
fn test_trial_mode_isolation_real_db_untouched() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::env::set_var("EXAMPANEL_DATA_DIR", temp.path().to_str().unwrap());

    let real_db_path = temp.path().join("real-exam-panel.db");
    let real_store = Store::open_at(&real_db_path).expect("open real store");
    let service = AppService::new(real_store);

    // Initial state: not in trial mode
    assert!(!service.is_trial_mode());

    // Create campus first in real DB
    let campus = service
        .create_campus(exam_panel_service::dto::CreateCampusInput {
            code: "PH1".to_string(),
            name: "Phan hieu 1".to_string(),
            color: "#3b82f6".to_string(),
        })
        .expect("create campus");

    // Create a unique teacher in real DB
    let real_teacher = service
        .create_teacher(CreateTeacherInput {
            code: Some("T_REAL".to_string()),
            full_name: "Giaovien That".to_string(),
            campus_id: campus.id,
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .expect("create real teacher");

    assert_eq!(real_teacher.full_name, "Giaovien That");

    // Enter trial mode
    service.enter_trial_mode().expect("enter trial mode");
    assert!(service.is_trial_mode());

    let app_info = service.get_app_info().expect("get app info");
    assert_eq!(app_info.in_trial_mode, Some(true));

    // In trial mode, demo data is loaded
    let trial_teachers = service.list_teachers().expect("list trial teachers");
    // Should have seeded demo teachers, not just the real teacher
    assert!(trial_teachers.len() >= 10);
    assert!(!trial_teachers
        .iter()
        .any(|t| t.code.as_deref() == Some("T_REAL")));

    // Add a trial teacher in trial mode
    let trial_teacher = service
        .create_teacher(CreateTeacherInput {
            code: Some("T_TRIAL".to_string()),
            full_name: "Giaovien Dungthu".to_string(),
            campus_id: CampusId(1),
            load_weight: 1.0,
            active: true,
            note: None,
            display_name: None,
            quota_override: None,
            max_tasks_per_exam_override: None,
        })
        .expect("create trial teacher");
    assert_eq!(trial_teacher.full_name, "Giaovien Dungthu");

    // Exit trial mode
    service.exit_trial_mode().expect("exit trial mode");
    assert!(!service.is_trial_mode());

    let app_info_after = service.get_app_info().expect("get app info");
    assert_eq!(app_info_after.in_trial_mode, Some(false));

    // Verify real DB only has real teacher and does NOT have trial teacher!
    let real_teachers = service.list_teachers().expect("list real teachers");
    assert_eq!(real_teachers.len(), 1);
    assert_eq!(real_teachers[0].code.as_deref(), Some("T_REAL"));
    assert!(!real_teachers
        .iter()
        .any(|t| t.code.as_deref() == Some("T_TRIAL")));
}
