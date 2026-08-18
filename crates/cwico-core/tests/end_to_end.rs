//! End-to-end coverage of scan -> classify -> plan -> execute against the
//! mock backend. These are the tests that would catch a regression in the
//! part of the tool that can break someone's computer.

use cwico_core::backend::{Event, PlatformBackend, StepStatus};
use cwico_core::mock::{MockBackend, RecordingSink};
use cwico_core::{
    Action, Engine, PlanOptions, RemovalPlan, SafetyClass, SafetyDatabase, ScanOptions, Selection,
    SourceKind,
};

fn setup() -> (MockBackend, SafetyDatabase) {
    (MockBackend::new(), SafetyDatabase::builtin())
}

#[test]
fn a_scan_classifies_every_fixture() {
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let sink = RecordingSink::new();

    let report = engine.scan(&ScanOptions::default(), &sink).unwrap();

    assert!(report.stats.total >= 12, "got {}", report.stats.total);
    assert!(report.items.iter().all(|i| i.safety_reason.is_some()));

    // The fixture set must exercise every class, or the UI is being developed
    // against a world that never shows a warning.
    for class in [
        SafetyClass::Safe,
        SafetyClass::Caution,
        SafetyClass::Critical,
        SafetyClass::Unknown,
    ] {
        assert!(
            report.items.iter().any(|i| i.safety == class),
            "no fixture is classified {}",
            class.slug()
        );
    }

    // Progress events reached the sink.
    let started = sink
        .events()
        .iter()
        .filter(|e| matches!(e, Event::ScanPassStarted { .. }))
        .count();
    assert!(
        started >= 4,
        "expected per-pass progress events, got {started}"
    );
}

#[test]
fn known_bloatware_lands_in_the_right_class() {
    let (backend, db) = setup();
    let report = Engine::new(&backend, &db)
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let class_of = |needle: &str| {
        report
            .items
            .iter()
            .find(|i| i.name.to_lowercase().contains(needle))
            .unwrap_or_else(|| panic!("fixture `{needle}` missing"))
            .safety
    };

    assert_eq!(class_of("onedrive"), SafetyClass::Safe);
    assert_eq!(class_of("candy crush"), SafetyClass::Safe);
    assert_eq!(class_of("xbox console"), SafetyClass::Safe);
    assert_eq!(class_of("edge"), SafetyClass::Caution);
    assert_eq!(class_of("photos"), SafetyClass::Caution);
    assert_eq!(class_of("defender"), SafetyClass::Critical);
    assert_eq!(class_of("remote procedure call"), SafetyClass::Critical);
    assert_eq!(class_of("visual c++"), SafetyClass::Critical);
    assert_eq!(class_of("acme ledger"), SafetyClass::Unknown);
}

#[test]
fn selecting_everything_still_cannot_remove_a_critical_component() {
    // The worst realistic user behaviour: tick every row, confirm every
    // prompt, press go. Windows must survive it.
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let everything: Vec<Selection> = report
        .items
        .iter()
        .map(|i| {
            Selection::uninstall(&i.id)
                .with_action(Action::UninstallAndDeepClean)
                .confirmed()
        })
        .collect();

    let plan = RemovalPlan::build(&report, &everything, PlanOptions::default());

    let critical_names: Vec<&str> = report
        .items
        .iter()
        .filter(|i| i.safety == SafetyClass::Critical)
        .map(|i| i.name.as_str())
        .collect();
    assert!(
        !critical_names.is_empty(),
        "the fixture must contain critical items"
    );

    for name in &critical_names {
        assert!(
            !plan.items.iter().any(|p| p.name == *name),
            "`{name}` was planned for removal"
        );
        assert!(
            plan.rejected
                .iter()
                .any(|r| r.name == *name && r.code == "protected_component"),
            "`{name}` was dropped without telling the user why"
        );
    }

    let run = engine.execute(&plan, &RecordingSink::new()).unwrap();
    let journal = backend.journal();

    // Nothing touched Defender or RPC by any route.
    for forbidden in ["WinDefend", "RpcSs"] {
        assert!(
            !journal
                .services_changed
                .iter()
                .any(|(s, _)| s.eq_ignore_ascii_case(forbidden)),
            "the run reconfigured `{forbidden}`"
        );
    }
    assert!(run.failed == 0, "unexpected failures: {run:#?}");
}

#[test]
fn caution_items_are_dropped_until_individually_confirmed() {
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let edge = report
        .items
        .iter()
        .find(|i| i.name.contains("Edge"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&edge.id)],
        PlanOptions::default(),
    );
    assert!(plan.items.is_empty());
    assert_eq!(plan.rejected[0].code, "needs_confirmation");

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&edge.id).confirmed()],
        PlanOptions::default(),
    );
    assert_eq!(plan.items.len(), 1);
}

#[test]
fn a_full_run_takes_a_restore_point_and_backs_up_the_registry_first() {
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let onedrive = report
        .items
        .iter()
        .find(|i| i.name.contains("OneDrive"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&onedrive.id).with_action(Action::UninstallAndDeepClean)],
        PlanOptions::default(),
    );
    let sink = RecordingSink::new();
    let run = engine.execute(&plan, &sink).unwrap();

    assert!(run.restore_point.is_some(), "no restore point was created");
    assert!(!run.registry_backups.is_empty(), "no registry backup taken");

    // Order matters: the snapshot has to precede the first destructive step.
    let order: Vec<String> = sink
        .events()
        .into_iter()
        .filter_map(|e| match e {
            Event::StepStarted { step, .. } => Some(step),
            _ => None,
        })
        .collect();
    let restore_at = order
        .iter()
        .position(|s| s == "create_restore_point")
        .unwrap();
    let backup_at = order.iter().position(|s| s == "backup_registry").unwrap();
    let first_destructive = order
        .iter()
        .position(|s| s == "kill_processes" || s == "run_official_uninstaller")
        .unwrap();
    assert!(restore_at < backup_at);
    assert!(backup_at < first_destructive);

    let journal = backend.journal();
    assert!(journal
        .uninstallers_run
        .iter()
        .any(|c| c.contains("/silent")));
    assert!(journal.killed.iter().any(|e| e == "OneDrive.exe"));
    assert!(journal.paths_deleted.iter().any(|p| p.contains("OneDrive")));
}

#[test]
fn a_required_restore_point_that_fails_aborts_before_anything_is_touched() {
    let backend = MockBackend::new().with_failing_restore_point();
    let db = SafetyDatabase::builtin();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();
    let onedrive = report
        .items
        .iter()
        .find(|i| i.name.contains("OneDrive"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&onedrive.id)],
        PlanOptions::default(),
    );
    let err = engine.execute(&plan, &RecordingSink::new()).unwrap_err();
    assert_eq!(err.code(), "safety_precondition");

    let journal = backend.journal();
    assert!(
        journal.uninstallers_run.is_empty(),
        "the run started anyway"
    );
    assert!(journal.killed.is_empty());
    assert!(journal.paths_deleted.is_empty());
}

#[test]
fn a_dry_run_changes_nothing() {
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let picks: Vec<Selection> = report
        .bulk_selectable()
        .iter()
        .map(|i| Selection::uninstall(&i.id).with_action(Action::UninstallAndDeepClean))
        .collect();
    assert!(!picks.is_empty());

    let plan = RemovalPlan::build(&report, &picks, PlanOptions::dry_run());
    let run = engine.execute(&plan, &RecordingSink::new()).unwrap();

    assert!(run.dry_run);
    assert_eq!(run.failed, 0);
    let journal = backend.journal();
    assert!(journal.uninstallers_run.is_empty());
    assert!(journal.killed.is_empty());
    assert!(journal.paths_deleted.is_empty());
    assert!(journal.appx_removed.is_empty());
    assert!(journal.keys_deleted.is_empty());
}

#[test]
fn a_failing_uninstaller_stops_that_item_but_not_the_run() {
    let backend = MockBackend::new().with_failing_uninstaller("uninstall.exe");
    let db = SafetyDatabase::builtin();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let acme = report
        .items
        .iter()
        .find(|i| i.name.contains("Acme"))
        .unwrap();
    let onedrive = report
        .items
        .iter()
        .find(|i| i.name.contains("OneDrive"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[
            Selection::uninstall(&acme.id)
                .with_action(Action::UninstallAndDeepClean)
                .confirmed(),
            Selection::uninstall(&onedrive.id).with_action(Action::UninstallAndDeepClean),
        ],
        PlanOptions::default(),
    );
    let run = engine.execute(&plan, &RecordingSink::new()).unwrap();

    assert_eq!(run.failed, 1);
    assert_eq!(run.succeeded, 1);

    // The critical guarantee: a failed uninstall must not be followed by a
    // deep clean of that product's folders.
    let acme_outcome = run.items.iter().find(|i| i.name.contains("Acme")).unwrap();
    assert_eq!(acme_outcome.status, StepStatus::Failed);
    assert!(
        !acme_outcome
            .steps
            .iter()
            .any(|s| s.step.starts_with("deep_clean")),
        "deep clean ran after the uninstaller failed: {:#?}",
        acme_outcome.steps
    );
    assert!(!backend
        .journal()
        .paths_deleted
        .iter()
        .any(|p| p.contains("Acme")));
}

#[test]
fn appx_removal_also_deprovisions_so_it_does_not_come_back() {
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();

    let news = report
        .items
        .iter()
        .find(|i| i.source == SourceKind::AppxProvisioned)
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&news.id)],
        PlanOptions::default(),
    );
    engine.execute(&plan, &RecordingSink::new()).unwrap();

    let removed = backend.journal().appx_removed;
    assert!(
        removed.iter().any(|p| p.starts_with("provisioned:")),
        "{removed:?}"
    );
}

#[test]
fn an_unelevated_process_refuses_to_run_a_real_plan() {
    let backend = MockBackend::new().unelevated();
    let db = SafetyDatabase::builtin();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();
    let onedrive = report
        .items
        .iter()
        .find(|i| i.name.contains("OneDrive"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&onedrive.id)],
        PlanOptions::default(),
    );
    let err = engine.execute(&plan, &RecordingSink::new()).unwrap_err();
    assert_eq!(err.code(), "needs_elevation");

    // …but a dry run is still allowed, so users can plan before elevating.
    let preview = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&onedrive.id)],
        PlanOptions::dry_run(),
    );
    assert!(engine.execute(&preview, &RecordingSink::new()).is_ok());
}

#[test]
fn shared_host_processes_are_never_terminated() {
    // DiagTrack runs inside svchost.exe. Killing svchost takes a dozen
    // unrelated services with it, so the backend must refuse.
    let (backend, db) = setup();
    let engine = Engine::new(&backend, &db);
    let report = engine
        .scan(&ScanOptions::default(), &RecordingSink::new())
        .unwrap();
    let diagtrack = report
        .items
        .iter()
        .find(|i| i.name.contains("Telemetry"))
        .unwrap();

    let plan = RemovalPlan::build(
        &report,
        &[Selection::uninstall(&diagtrack.id)],
        PlanOptions::default(),
    );
    engine.execute(&plan, &RecordingSink::new()).unwrap();

    assert!(!backend
        .journal()
        .killed
        .iter()
        .any(|e| e.eq_ignore_ascii_case("svchost.exe")));
}

#[test]
fn platform_info_is_reported() {
    let (backend, _db) = setup();
    let info = backend.platform_info();
    assert_eq!(info.platform, "mock");
    assert!(info.elevated);
    assert!(info.os_description.contains("Windows"));
}
