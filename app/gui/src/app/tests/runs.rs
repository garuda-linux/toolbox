use egui_kittest::Harness;
use toolbox_core::features::bootrepair::page::Progress;
use toolbox_plan::{Plan, RootAction, UnitName, UnitScope};

use super::send;
use crate::app::{App, Setup};
use crate::apply::{ApplyHandle, PlanId, Report};
use crate::command::{AppCommand, ServiceCommand};
use crate::page::PageId;
use crate::page_service::Owner;
use crate::waking::Wake;

fn plan() -> Plan {
    Plan::root(RootAction::RefreshKeyring)
}

/// Two plans within one frame: a click, then Boot Repair's. The service
/// starts the first and refuses the second.
#[test]
fn a_refused_plan_never_takes_the_run_of_the_one_before() {
    let (reports, apply) = ApplyHandle::feeding(Wake::none());
    let mut harness = Harness::builder().build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                apply: Some(apply),
                ..Setup::default()
            },
        )
    });

    harness.run();

    let app = harness.state_mut();
    let first = app.run_at_once(plan()).unwrap();
    app.services.boot_repair.started();
    assert!(app.run_owned(plan(), Owner::BootRepair));
    let second = first.next();

    reports.send(Report::PlanStarted {
        plan: first,
        steps: Vec::new(),
    });
    reports.send(Report::Busy { plan: second });
    reports.send(Report::PlanFinished {
        plan: first,
        succeeded: true,
    });
    harness.run();

    assert_eq!(
        harness.state().services.boot_repair.progress(),
        Progress::Busy
    );
}

/// The helper refuses a second plan, so Apply says why it waits.
#[test]
fn apply_waits_while_a_plan_runs() {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();
    harness.state_mut().console.take(Report::PlanStarted {
        plan: PlanId::new(1),
        steps: Vec::new(),
    });

    assert_eq!(harness.state().apply_held(), Some("changes-wait-running"));

    harness.state_mut().console.take(Report::PlanFinished {
        plan: PlanId::new(1),
        succeeded: true,
    });

    assert_ne!(harness.state().apply_held(), Some("changes-wait-running"));
}

#[test]
fn a_journal_while_a_plan_runs_is_the_services_pages_refusal() {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();
    harness.state_mut().console.take(Report::PlanStarted {
        plan: PlanId::new(1),
        steps: Vec::new(),
    });

    let logs = ServiceCommand::Logs {
        scope: UnitScope::System,
        unit: UnitName::new("sshd.service").unwrap(),
    };

    send(&mut harness, AppCommand::Service(logs));
    assert!(harness.state().services.units.refused());

    send(&mut harness, AppCommand::DismissRefusal(PageId::Services));
    assert!(!harness.state().services.units.refused());
}
