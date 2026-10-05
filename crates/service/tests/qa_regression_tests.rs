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

fn repo_file(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// RA-036: `default = ["dev-tools"]` shipped the `seed_demo` IPC command in release builds.
#[test]
fn ra036_release_manifest_does_not_enable_dev_tools_by_default() {
    let manifest = repo_file("src-tauri/Cargo.toml");
    let default_line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("default"))
        .expect("src-tauri/Cargo.toml must declare `default` features");
    assert!(
        !default_line.contains("dev-tools"),
        "dev-tools must not be a default feature: {default_line}"
    );
}

/// RA-034: the webview capability must stay narrow; opening folders goes through Rust commands.
#[test]
fn ra034_capability_does_not_grant_frontend_open_path() {
    let caps = repo_file("src-tauri/capabilities/default.json");
    assert!(!caps.contains("allow-open-path"), "{caps}");
    assert!(!caps.contains("fs:") && !caps.contains("shell:"), "{caps}");
}

// ---------------------------------------------------------------------------------------------
// Excel export of a two-subject plan (RA-028, RA-009)
// ---------------------------------------------------------------------------------------------

mod export {
    use calamine::{open_workbook_auto, Data, Reader};
    use exam_panel_core::domain::fixtures::{
        make_canonical_q_problem, make_q_assignments, QVariant,
    };
    use exam_panel_core::domain::{Plan, PlanId, Role, SchoolYearId};
    use exam_panel_core::score::evaluate;
    use exam_panel_service::dto::{AppSettings, PlanDetails};
    use exam_panel_service::excel::export::export_plan_workbook;
    use std::collections::BTreeSet;

    fn export_q_plan() -> (std::path::PathBuf, tempfile::TempDir, PlanDetails) {
        let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);
        let assignments = make_q_assignments();
        let report = evaluate(&problem, &assignments);
        let details = PlanDetails {
            plan: Plan {
                id: PlanId(1),
                school_year_id: SchoolYearId(1),
                name: "Q".into(),
                source: "manual".into(),
                created_at: "2026-10-04T12:00:00Z".into(),
                seed: 1,
                score: Some(report.total),
                is_final: false,
                rank: None,
                score_report_json: None,
                run_params_json: None,
                data_hash: None,
                rules_hash: None,
            },
            assignments,
            score_report: Some(report),
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("q.xlsx");
        let settings = AppSettings {
            theme: "light".into(),
            language: "vi".into(),
            current_school_year_id: Some(SchoolYearId(1)),
            school_name: Some("Trường".into()),
            department_name: Some("Tổ".into()),
            signer_title: Some("TT".into()),
            signer_name: Some("A".into()),
            place_name: Some("HN".into()),
        };
        export_plan_workbook(
            &path,
            &details,
            &problem.school_year,
            &problem.campuses,
            &problem.grades,
            &problem.exams,
            &problem.subjects,
            &problem.teachers,
            &settings,
            &problem.rule_settings,
        )
        .expect("export");
        (path, dir, details)
    }

    fn text(d: &Data) -> String {
        match d {
            Data::Empty => String::new(),
            other => other.to_string(),
        }
    }

    /// RA-028: "Cùng ban với" lists only the members of the SAME SUBJECT panel.
    #[test]
    fn ra028_co_panelists_are_limited_to_the_same_subject_panel() {
        let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);
        let (path, _dir, details) = export_q_plan();
        let mut wb = open_workbook_auto(&path).unwrap();
        let range = wb.worksheet_range("Theo giáo viên").unwrap();
        let name_of = |id| {
            problem
                .teachers
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.full_name.clone())
                .unwrap()
        };
        let role_text = |r: Role| {
            if r == Role::Setter {
                "Ra đề"
            } else {
                "Phản biện"
            }
        };

        let mut checked = 0;
        for row in 1..range.height() {
            let teacher = text(range.get((row, 1)).unwrap());
            let subject = text(range.get((row, 3)).unwrap());
            let exam = text(range.get((row, 4)).unwrap());
            let grade = text(range.get((row, 5)).unwrap());
            let role = text(range.get((row, 6)).unwrap());
            let co: BTreeSet<String> = text(range.get((row, 7)).unwrap())
                .split("; ")
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();

            // Locate the seat in the plan to know its panel.
            let seat = details
                .assignments
                .iter()
                .find(|a| {
                    name_of(a.teacher_id) == teacher
                        && problem
                            .exams
                            .iter()
                            .any(|e| e.id == a.exam_id && e.code == exam)
                        && problem
                            .grades
                            .iter()
                            .any(|g| g.id == a.grade_id && g.name == grade)
                        && problem
                            .subjects
                            .iter()
                            .any(|s| s.id == a.subject_id && s.name == subject)
                        && role_text(a.role) == role
                })
                .unwrap_or_else(|| {
                    panic!("row {row}: no seat for {teacher}/{subject}/{exam}/{grade}")
                });
            let expected: BTreeSet<String> = details
                .assignments
                .iter()
                .filter(|o| {
                    o.exam_id == seat.exam_id
                        && o.grade_id == seat.grade_id
                        && o.subject_id == seat.subject_id
                        && o.teacher_id != seat.teacher_id
                })
                .map(|o| format!("{} ({})", name_of(o.teacher_id), role_text(o.role)))
                .collect();
            assert_eq!(
                co, expected,
                "row {row}: {teacher} {subject} {exam} {grade}"
            );
            checked += 1;
        }
        assert_eq!(checked, 60);
    }

    fn vi_rule_names() -> std::collections::HashMap<String, String> {
        let raw = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../ui/src/i18n/locales/vi.json"),
        )
        .unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut out = std::collections::HashMap::new();
        for n in 1..=10 {
            if let Some(t) = json["rules"][format!("s{n}Title")].as_str() {
                out.insert(format!("S{n}"), t.split_once(": ").unwrap().1.to_string());
            }
        }
        out
    }

    /// RA-009: criterion names equal the Rules screen, units are not truncated, bounds shown.
    #[test]
    fn ra009_criteria_sheet_uses_rules_screen_names_units_and_lower_bounds() {
        let (path, _dir, details) = export_q_plan();
        let report = details.score_report.unwrap();
        let names = vi_rule_names();
        let mut wb = open_workbook_auto(&path).unwrap();
        let range = wb.worksheet_range("Tiêu chí").unwrap();

        let header: Vec<String> = (0..range.width())
            .map(|c| text(range.get((0, c)).unwrap()))
            .collect();
        let lb_col = header
            .iter()
            .position(|h| h.contains("Cận dưới"))
            .unwrap_or_else(|| panic!("lower-bound column missing: {header:?}"));

        let mut seen = 0;
        for row in 1..range.height() {
            let code = text(range.get((row, 0)).unwrap());
            if let Some(expected) = names.get(&code) {
                let label = text(range.get((row, 1)).unwrap());
                assert_eq!(label, format!("{code} - {expected}"), "name of {code}");
                seen += 1;
            }
            let Some(rs) = report
                .by_rule
                .iter()
                .find(|r| format!("{:?}", r.rule) == code)
            else {
                continue;
            };
            let units = match range.get((row, 4)).unwrap() {
                Data::Float(f) => *f,
                Data::Int(i) => *i as f64,
                other => panic!("units of {code}: {other:?}"),
            };
            assert!(
                (units - rs.units).abs() < 0.0006,
                "{code}: sheet {units} vs engine {}",
                rs.units
            );
            let lb = match range.get((row, lb_col)).unwrap() {
                Data::Float(f) => *f,
                Data::Int(i) => *i as f64,
                other => panic!("lower bound of {code}: {other:?}"),
            };
            assert!(
                (lb - rs.lower_bound).abs() < 0.0006,
                "{code}: lb {lb} vs {}",
                rs.lower_bound
            );
        }
        assert_eq!(
            seen, 10,
            "S1..S10 must all be labelled with the Rules screen names"
        );
    }
}
