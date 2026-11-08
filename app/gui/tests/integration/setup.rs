//! The Setup Assistant with the shipped catalog and the real queue,
//! opened as `--setup-assistant` opens it: a pick in a step reaches the
//! queue and the summary.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::page::PageId;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};

use crate::common::{Fixture, HEIGHT, installing, packages, window};
use crate::discover::source_catalogs;
use crate::system::first_read;

/// A system that offers three audio players and has one of them.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SyncPackages(packages(&[
        "strawberry",
        "lollypop",
        "audacious",
    ])));
    snapshot.insert(InstalledPackages(packages(&["audacious"])));
    snapshot
}

pub fn assistant() -> Fixture {
    assistant_with(ChangeSet::default())
}

/// [`assistant`] with `queued` waiting from another page.
fn assistant_with(queued: ChangeSet) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let probed = system();
    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(queued),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                catalogs: source_catalogs(),
                open: Some(PageId::Setup),
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);

    Fixture::new(harness, runtime)
}

#[test]
fn the_start_option_opens_the_welcome_with_the_offered_steps() {
    let mut fixture = assistant();
    let harness = fixture.harness();

    assert!(
        harness
            .query_by_label("Welcome to the Setup Assistant")
            .is_some()
    );
    assert!(
        harness
            .query_all_by_label_contains("There is 1 category.")
            .next()
            .is_some()
    );
}

#[test]
fn a_pick_reaches_the_queue_and_the_summary() {
    let mut fixture = assistant();
    let harness = fixture.harness();

    harness.get_by_label("Next").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Strawberry Player")
        .click();
    harness.run();
    assert!(
        harness
            .query_all_by_label_contains("with Apply")
            .next()
            .is_some()
    );

    harness.get_by_label("Next").click();
    harness.run();

    assert!(harness.query_by_label("Apps to install: 1").is_some());
}

/// A change from another page waits while the assistant ends with
/// nothing of its own picked.
#[test]
fn the_success_page_offers_no_second_apply() {
    let mut fixture = assistant_with(installing(&["htop"]));
    let harness = fixture.harness();
    assert!(harness.query_by_label("change queued").is_none());

    harness
        .get_by_role_and_label(Role::ComboBox, "Go to step")
        .click();
    harness.run();
    harness.get_by_label("Summary").click();
    harness.run();
    harness.get_by_label("Finish").click();
    harness.run();

    assert!(harness.query_by_label("Success!").is_some());
    assert!(harness.query_by_label("change queued").is_none());
}

fn summary_with_a_pick(harness: &mut Harness<'static, App>) {
    harness.get_by_label("Next").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Strawberry Player")
        .click();
    harness.run();
    harness.get_by_label("Next").click();
    harness.run();
}

#[test]
fn on_the_summary_the_queue_shows_only_for_the_review() {
    let mut fixture = assistant();
    let harness = fixture.harness();
    summary_with_a_pick(harness);

    assert!(harness.query_by_label("Apps to install: 1").is_some());
    assert!(harness.query_by_label("change queued").is_none());

    harness.get_by_label("Review and apply").click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_some());
    assert!(harness.query_by_label("Remove all…").is_some());

    harness.get_by_label("Hide").click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_none());

    harness.get_by_label("Back").click();
    harness.run();
    assert!(
        harness.query_by_label("change queued").is_none(),
        "a step shows no second Apply beside Next"
    );
}
