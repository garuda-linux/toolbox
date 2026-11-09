use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::launch::Launcher;
use toolbox_gui::news::SampleNews;
use toolbox_gui::power::Power;
use toolbox_gui::programs::Programs;
use toolbox_gui::queue::Queue;
use toolbox_gui::reader::ReaderHandle;
use toolbox_gui::waking::Waking;
use toolbox_gui::{App, Setup, reader, snapshot};
use toolbox_news::{News, NewsSource as _};
use toolbox_plan::{PackageName, UserName};
use toolbox_system::{
    CallError, DiskSpace, Facts, Host, PackageCount, PendingUpdate, ProbeError, Status,
};

use crate::common::{TALL, no_note, window};
use crate::system::{shows, step_until};

pub const GB: u64 = 1_000_000_000;

/// Refuses every restart, as polkit does when the user cancels.
#[derive(Debug, Default)]
pub struct RefusingPower {
    asked: AtomicUsize,
}

#[derive(Debug)]
struct BrokenLauncher;

impl Power for RefusingPower {
    fn restart(&self, refused: Waking<CallError>) {
        self.asked.fetch_add(1, Ordering::SeqCst);
        refused.send(CallError::Denied);
    }
}

impl Launcher for BrokenLauncher {
    fn start(&self, _program: &Path, _args: &[&str]) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }
}

pub fn update(name: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: "1.0-1".to_owned(),
        to: "1.1-1".to_owned(),
    }
}

pub fn quiet() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(Vec::new()),
        merges: Ok(Vec::new()),
        last_update: Ok(None),
        health: Ok(Vec::new()),
    }
}

pub fn quiet_facts() -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: Some(12),
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Ok(std::time::SystemTime::now() - std::time::Duration::from_hours(100)),
        disk: Ok(DiskSpace {
            total: 500 * GB,
            free: 200 * GB,
        }),
        package_cache: Ok(GB),
        orphans: Ok(Vec::new()),
        failed_units: Ok(Vec::new()),
    }
}

pub fn host(live_iso: bool, reboot_pending: bool) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host {
        user: Some(UserName::new("tester").unwrap()),
        live_iso,
        desktops: Vec::new(),
        edition: None,
        reboot_pending,
    });
    snapshot
}

pub fn home(
    system: Snapshot,
    status: Status,
) -> (Harness<'static, App>, Runtime, Arc<RefusingPower>) {
    home_with(system, status, quiet_facts())
}

pub fn home_with(
    system: Snapshot,
    status: Status,
    facts: Facts,
) -> (Harness<'static, App>, Runtime, Arc<RefusingPower>) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let power = Arc::new(RefusingPower::default());
    let shared = Arc::clone(&power);
    let status = Arc::new(status);
    let facts = Arc::new(facts);

    let mut harness = window(TALL).build_eframe(move |cc| {
        let ctx = cc.egui_ctx.clone();
        let probe = move || (system.clone(), Vec::new());
        let read = move || Status::clone(&status);
        let read_facts = move || Facts::clone(&facts);
        let programs = || Programs {
            installer: true,
            chroot: true,
            ..Programs::default()
        };

        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(ctx.clone()),
                    probe,
                )),
                status: Some(reader::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(ctx.clone()),
                    "status",
                    read,
                )),
                news: Some(sample_news(&handle, &ctx)),
                facts: Some(reader::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(ctx.clone()),
                    "facts",
                    read_facts,
                )),
                programs: Some(reader::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(ctx),
                    "programs",
                    programs,
                )),
                power: Some(shared),
                launcher: Some(Arc::new(BrokenLauncher)),
                ..Setup::default()
            },
        )
    });

    step_until(&mut harness, "the first read", |harness| {
        harness
            .query_all_by_label("Reading the system…")
            .next()
            .is_none()
            && harness.query_by_label("Checking the system…").is_none()
    });
    harness.run();
    (harness, runtime, power)
}

#[test]
fn a_quiet_system_greets_and_says_all_is_well() {
    let (harness, _runtime, _power) = home(host(false, false), quiet());

    assert!(harness.query_by_label("Hello, tester!").is_some());
    assert!(harness.query_by_label("All 7 checks passed").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Link, "Maintenance")
            .is_some()
    );
}

#[test]
fn updates_queue_and_list_their_packages() {
    let mut status = quiet();
    status.updates = Ok(vec![update("linux"), update("mesa")]);
    status.aur_updates = Ok(vec![update("paru-bin")]);
    let (mut harness, _runtime, _power) = home(host(false, false), status);

    assert!(harness.query_by_label("2 updates available").is_some());

    let more = "More actions for 2 updates available";
    harness.get_by_label(more).click();
    harness.run();
    harness.get_by_label("Queue update").click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_some());

    harness.get_by_label(more).click();
    harness.run();
    assert!(harness.query_by_label("Don't update").is_some());
    harness.get_by_label("Show packages").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Pending updates")
            .is_some()
    );
    assert!(harness.query_by_label("paru-bin").is_some());

    harness.get_by_label("Close").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Pending updates")
            .is_none()
    );
}

#[test]
fn a_refused_restart_says_so() {
    let (mut harness, _runtime, power) = home(host(false, true), quiet());

    harness.get_by_label("Restart…").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Restart now?")
            .is_some()
    );

    harness.get_by_label("Restart").click();
    step_until(
        &mut harness,
        "the refusal",
        shows("The restart was not authorized."),
    );

    assert_eq!(power.asked.load(Ordering::SeqCst), 1);
}

#[test]
fn a_check_that_failed_shows_its_reason_on_request() {
    let mut status = quiet();
    status.health = Err(ProbeError::Failed("no garuda-health".to_owned()));
    let (mut harness, _runtime, _power) = home(host(false, false), status);

    assert!(
        harness
            .query_by_label_contains("One check could not run")
            .is_some()
    );

    harness.get_by_label("Show details").click();
    harness.run();
    assert!(
        harness
            .query_by_label("garuda-health: no garuda-health")
            .is_some()
    );
}

#[test]
fn the_live_iso_offers_its_tools_and_tells_a_failed_start() {
    let (mut harness, _runtime, _power) = home(host(true, false), quiet());

    assert!(harness.query_by_label("System status").is_none());
    assert!(
        harness
            .query_by_label("Repair an installed system")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Link, "Setup Assistant")
            .is_none()
    );

    harness.get_by_label("Start installer").click();
    harness.run();
    assert!(
        harness
            .query_by_label_contains("The installer could not start")
            .is_some()
    );
    assert!(no_note(&harness), "a banner on Home, not a note");
}

#[test]
fn a_next_step_opens_its_page() {
    let (mut harness, _runtime, _power) = home(host(false, false), quiet());

    harness
        .get_by_role_and_label(Role::Link, "Maintenance")
        .click();
    harness.run();

    assert_eq!(harness.query_all_by_label("Maintenance").count(), 2);
}

#[test]
fn the_greeting_counts_what_needs_attention() {
    let status = Status {
        updates: Ok(vec![update("a"), update("b"), update("c")]),
        ..quiet()
    };

    let (harness, _runtime, _power) = home(host(false, false), status);

    assert!(
        harness
            .query_by_label("One thing needs attention")
            .is_some()
    );
    assert!(harness.query_by_label("3 updates available").is_some());
}

#[test]
fn a_quiet_greeting_says_all_is_fine() {
    let (harness, _runtime, _power) = home(host(false, false), quiet());

    assert!(harness.query_by_label("Everything is fine").is_some());
}

#[test]
fn a_failed_updates_check_says_so_not_none() {
    let status = Status {
        updates: Err(ProbeError::Failed("pacman is busy".to_owned())),
        ..quiet()
    };

    let (harness, _runtime, _power) = home(host(false, false), status);

    assert!(
        harness
            .query_by_label_contains("One check could not run")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("updates available")
            .is_none()
    );
}

#[test]
fn when_every_check_failed_one_banner_says_it_and_no_all_clear() {
    let failed = || ProbeError::Failed("no network".to_owned());
    let status = Status {
        updates: Err(failed()),
        aur_updates: Err(failed()),
        merges: Err(failed()),
        ..quiet()
    };

    let (harness, _runtime, _power) = home(host(false, false), status);

    assert!(harness.query_by_label_contains("checks passed").is_none());
    assert!(harness.query_all_by_label("Read again").next().is_some());
}

/// The dry run's news, as the self-test sees them.
fn sample_news(handle: &tokio::runtime::Handle, ctx: &egui::Context) -> ReaderHandle<News> {
    reader::spawn_fallible(
        handle,
        toolbox_gui::waking::repaint(ctx.clone()),
        "news",
        || SampleNews.news(),
    )
}
