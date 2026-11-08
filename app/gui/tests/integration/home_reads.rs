//! Home when a read fails: it says so on the page and offers to read
//! again, with no note over the page.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::programs::Programs;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, reader, snapshot};
use toolbox_system::{ProbeError, Status};

use crate::common::{TALL, no_note, window};
use crate::home::{home, host, quiet};
use crate::system::step_until;

/// Home with a status and a programs reader that always fail.
fn failing(system: Snapshot) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let harness = window(TALL).build_eframe(move |cc| {
        let ctx = cc.egui_ctx.clone();
        let probe = move || (system.clone(), Vec::new());
        let status = || Err::<Status, _>("no network".to_owned());
        let programs = || Err::<Programs, _>("no PATH".to_owned());

        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(ctx.clone()),
                    probe,
                )),
                status: Some(reader::spawn_fallible(
                    &handle,
                    toolbox_gui::waking::repaint(ctx.clone()),
                    "status",
                    status,
                )),
                programs: Some(reader::spawn_fallible(
                    &handle,
                    toolbox_gui::waking::repaint(ctx),
                    "programs",
                    programs,
                )),
                ..Setup::default()
            },
        )
    });

    (harness, runtime)
}

#[test]
fn a_failed_status_read_says_so_with_read_again() {
    let (mut harness, _runtime) = failing(host(false, false));

    step_until(&mut harness, "the failed check", |harness| {
        harness.query_by_label("The update check failed.").is_some()
    });

    assert!(harness.query_by_label("Checking the system…").is_none());
    assert!(harness.query_all_by_label("Read again").count() >= 2);
    assert!(no_note(&harness));
}

#[test]
fn failed_checks_offer_their_details_and_to_read_again() {
    let mut status = quiet();
    status.health = Err(ProbeError::Failed("no garuda-health".to_owned()));
    let (harness, _runtime, _power) = home(host(false, false), status);

    let details = harness.get_by_label("Show details").rect();
    let again = harness
        .query_all_by_label("Read again")
        .map(|node| node.rect())
        .max_by(|one, other| one.top().total_cmp(&other.top()))
        .unwrap();

    assert!(details.right() <= again.left());
}

#[test]
fn the_live_iso_tells_that_its_tools_could_not_be_looked_for() {
    let (mut harness, _runtime) = failing(host(true, false));

    step_until(&mut harness, "the programs failure", |harness| {
        harness
            .query_by_label("The live system's tools could not be looked for.")
            .is_some()
    });

    assert!(no_note(&harness));
}

#[test]
fn an_installed_system_keeps_quiet_about_the_live_tools() {
    let (mut harness, _runtime) = failing(host(false, false));

    step_until(&mut harness, "the failed check", |harness| {
        harness.query_by_label("The update check failed.").is_some()
    });

    assert!(
        harness
            .query_by_label("The live system's tools could not be looked for.")
            .is_none()
    );
}
