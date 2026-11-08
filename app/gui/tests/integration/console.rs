mod focus;
mod header;
mod settle;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::process::Command as Process;
use tokio::runtime::{Handle, Runtime};
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_gui::apply::{self, Launcher};
use toolbox_gui::notifications::Notifier;
use toolbox_gui::queue::Queue;
use toolbox_gui::snapshot::{self, SnapshotHandle};
use toolbox_gui::testing::PATIENCE;
use toolbox_gui::{App, Setup};
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Mode, Outcome, encode};
use toolbox_system::Notification;

use self::settle::{Gate, SETTLE};
pub use self::settle::{unfocus, wait_for};
use crate::common::{HEIGHT, installing, packages, window};

pub struct FakeLauncher(pub String);

#[derive(Debug)]
struct Recorder(Sender<Notification>);

impl Recorder {
    fn pair() -> (Arc<dyn Notifier>, Receiver<Notification>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        (Arc::new(Self(sender)), receiver)
    }
}

impl Notifier for Recorder {
    fn show(&self, notification: Notification) {
        self.0.send(notification).unwrap();
    }
}

impl Launcher for FakeLauncher {
    fn command(&self) -> Process {
        let mut command = Process::new("sh");
        command.arg("-c").arg(&self.0);
        command
    }

    fn mode(&self) -> Mode {
        Mode::Run
    }
}

fn script(events: &[Event]) -> String {
    let lines: String = events.iter().map(|event| encode(event).unwrap()).collect();
    format!("read request; printf '%s' '{lines}'")
}

pub fn silent_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ];

    script(&events)
}

pub fn succeeding_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Output {
            step,
            data: "resolving dependencies...\r\n\u{1b}[1;32mok\u{1b}[0m installed firefox\r\n"
                .to_owned(),
        },
        Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ];

    script(&events)
}

/// The runtime runs the service, so the caller keeps it as long as the harness.
pub fn app(script: String) -> (Harness<'static, App>, Runtime) {
    app_with(script, None, false)
}

pub fn installing_app(script: String) -> (Harness<'static, App>, Runtime) {
    app_with(script, None, true)
}

pub fn firefox_after_the_run(handle: &Handle, ctx: &egui::Context) -> SnapshotHandle {
    let read = AtomicUsize::new(0);
    let probe = move || {
        let mut system = Snapshot::default();
        if read.fetch_add(1, Ordering::SeqCst) > 0 {
            system.insert(InstalledPackages(packages(&["firefox"])));
        }

        (system, Vec::new())
    };

    snapshot::spawn(handle, toolbox_gui::waking::repaint(ctx.clone()), probe)
}

fn app_with(
    script: String,
    notifier: Option<Arc<dyn Notifier>>,
    reads: bool,
) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let changes = installing(&["firefox"]);

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let launcher = Arc::new(FakeLauncher(script));
        let setup = Setup {
            snapshot: reads.then(|| firefox_after_the_run(&handle, &cc.egui_ctx)),
            queue: Queue::in_memory(changes),
            apply: Some(apply::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                launcher,
            )),
            notifier,
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    harness.run();
    (harness, runtime)
}

#[test]
fn apply_opens_the_console_and_shows_the_outcome() {
    let (mut harness, _runtime) = app(succeeding_script());
    assert!(harness.query_by_label("Console").is_none());

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "All changes applied");
    assert!(harness.query_by_label("Done").is_some());
    assert!(harness.query_by_label("Console output").is_some());
}

#[test]
fn ctrl_enter_applies_too() {
    let (mut harness, _runtime) = app(succeeding_script());

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.step();

    wait_for(&mut harness, "All changes applied");
}

#[test]
fn a_dismissed_password_dialog_is_told_in_words() {
    let (mut harness, _runtime) = app("exit 126".to_owned());

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(
        &mut harness,
        "The password dialog was closed. Nothing was changed.",
    );
    assert!(
        harness
            .query_by_label("Stopped before all changes were applied")
            .is_some()
    );
}

#[test]
fn a_failed_step_shows_its_exit_code() {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Finished {
            step,
            outcome: Outcome::Failed {
                cause: toolbox_plan::protocol::Failure::ExitCode { code: 1_i32 },
            },
        },
        Event::Done,
    ];

    let script = format!("{}; exit 1", script(&events));
    let (mut harness, _runtime) = app(script);

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "Failed (exit code 1)");
}

#[test]
fn a_run_that_ends_in_the_background_is_notified() {
    let (notifier, shown) = Recorder::pair();
    let (mut harness, _runtime) = app_with(succeeding_script(), Some(notifier), false);

    harness.input_mut().focused = false;
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    let notification = shown.try_recv().unwrap();
    assert_eq!(notification.summary, "Changes applied");
    assert_eq!(
        notification.body,
        "Garuda Toolbox finished applying your changes."
    );
}

#[test]
fn a_run_watched_in_the_window_is_not_notified() {
    let (notifier, shown) = Recorder::pair();
    let (mut harness, _runtime) = app_with(succeeding_script(), Some(notifier), false);

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    shown.try_recv().unwrap_err();
}

#[test]
fn a_run_that_ends_behind_a_hidden_console_leaves_a_note() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()));

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "Hide the Console");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    crate::system::step_until(&mut harness, "the Console to hide", |harness| {
        harness.query_by_label("Hide the Console").is_none()
    });
    gate.open();

    wait_for(&mut harness, "Your changes were applied.");
    assert!(harness.query_by_label("All changes applied").is_none());
}

#[test]
fn a_run_watched_in_the_console_leaves_no_note() {
    let (mut harness, _runtime) = app(succeeding_script());

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    assert!(
        harness
            .query_by_label("Your changes were applied.")
            .is_none()
    );
}

#[test]
fn a_running_plan_is_named_once_in_the_header() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()));

    harness.get_by_label("Apply").click();
    let start = Instant::now();
    while harness
        .query_all_by_label("Applying changes…")
        .next()
        .is_none()
    {
        assert!(start.elapsed() < PATIENCE, "the run never showed");
        harness.step();
    }

    harness.run_steps(SETTLE);

    assert_eq!(harness.query_all_by_label("Applying changes…").count(), 1);
}

pub fn idle_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Finished {
            step,
            outcome: Outcome::Skipped,
        },
        Event::Done,
    ];

    script(&events)
}

#[test]
fn a_run_where_no_step_had_work_is_not_called_applied() {
    let (mut harness, _runtime) = app(idle_script());

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "Nothing needed to change");
    assert!(harness.query_by_label("All changes applied").is_none());
}

/// The install's step failed on a file its mirror no longer has. The
/// script leaves out pacman's quotes round the file, which the shell
/// line around it would take.
fn missing_file_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Output {
            step,
            data: "error: failed retrieving file firefox-144.0-1-x86_64.pkg.tar.zst from mirror.example.org : The requested URL returned error: 404\r\n"
                .to_owned(),
        },
        Event::Finished {
            step,
            outcome: Outcome::Failed {
                cause: toolbox_plan::protocol::Failure::ExitCode { code: 1_i32 },
            },
        },
        Event::Done,
    ];

    format!("{}; exit 1", script(&events))
}

#[test]
fn an_install_that_met_a_missing_mirror_file_offers_an_update_first() {
    let (mut harness, _runtime) = app(missing_file_script());

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "Update the system and try again");
    unfocus(&mut harness);
    harness.snapshot("console/mirror_missing_dark");

    harness
        .get_by_label("Update the system and try again")
        .click();
    harness.step();

    wait_for(&mut harness, "pacman -Syu --noconfirm");
    assert!(
        harness
            .query_by_label_contains("pacman -S --needed --noconfirm -- firefox")
            .is_some()
    );
}
