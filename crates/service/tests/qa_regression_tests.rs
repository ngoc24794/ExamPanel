//! Regression tests for the real-app QA run findings (docs/qa/real-run-20261005-linux).

use exam_panel_core::domain::SchoolYearId;
use exam_panel_core::optimize::OptimizationEffort;
use exam_panel_service::dto::{OptimizeBudget, OptimizeRequest};
use exam_panel_service::service::AppService;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn request(k: usize) -> OptimizeRequest {
    let (runs, iters) = OptimizationEffort::Fast.runs_and_iterations();
    OptimizeRequest {
        base_seed: Some(42),
        runs,
        budget: OptimizeBudget::Iterations(iters),
        k,
        diversity_threshold: None,
    }
}

/// RA-011: the real backend returned `Ok(plans)` after a cancel (only the mock threw
/// `cancelled`), so the UI saved partial plans. A cancelled job must be an error.
#[test]
fn ra011_cancelled_optimize_returns_cancelled_error_and_no_plans() {
    let service = AppService::open_in_memory().expect("open service");
    service.seed_demo().expect("seed demo");
    let cancel = Arc::new(AtomicBool::new(true));
    let err = service
        .run_optimize(SchoolYearId(1), request(3), None, Some(cancel))
        .expect_err("cancelled run must not return plans");
    assert_eq!(err.code, "cancelled");
    assert!(
        !service.is_optimizing(),
        "busy flag must be released after cancel"
    );
    assert!(service.list_plans(SchoolYearId(1)).unwrap().is_empty());
}

#[test]
fn ra011_cancel_during_run_returns_cancelled_error() {
    let service = Arc::new(AppService::open_in_memory().expect("open service"));
    service.seed_demo().expect("seed demo");
    let svc = Arc::clone(&service);
    let handle = std::thread::spawn(move || {
        let (runs, _) = OptimizationEffort::Thorough.runs_and_iterations();
        svc.run_optimize(
            SchoolYearId(1),
            OptimizeRequest {
                base_seed: Some(7),
                runs,
                budget: OptimizeBudget::Iterations(50_000_000),
                k: 3,
                diversity_threshold: None,
            },
            None,
            None,
        )
    });
    let mut cancelled = false;
    for _ in 0..200 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        if service.cancel_optimize() {
            cancelled = true;
            break;
        }
    }
    assert!(cancelled, "job never became active");
    let res = handle.join().unwrap();
    assert_eq!(res.expect_err("cancelled").code, "cancelled");
    assert!(!service.is_optimizing());
}
