mod flood;
mod focus;
mod header;
mod mirror;
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

/// pacman asks for a provider of java-runtime, takes the default and
/// asks to proceed, captured through a PTY.
pub const PACMAN_INSTALL: &str =
    include_str!("../../../../crates/pty/tests/fixtures/pacman-provider.txt");

/// The package the install in [`PACMAN_INSTALL`] was asked for.
pub const JAVA: &[&str] = &["java-runtime"];

/// A package queued where the fake helper prints nothing.
const FIREFOX: &[&str] = &["firefox"];

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

/// The fake helper: it reads the plan, then prints `events`. A quote in
/// real output, such as pacman's round a file name, ends the shell's
/// quoting and is put back as an escaped one.
fn script(events: &[Event]) -> String {
    let lines: String = events.iter().map(|event| encode(event).unwrap()).collect();
    let quoted = lines.replace('\'', r"'\''");
    format!("read request; printf '%s' '{quoted}'")
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

/// The step prints pacman's install of a Java runtime as a PTY captured
/// it, then succeeds.
pub fn succeeding_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Output {
            step,
            data: PACMAN_INSTALL.to_owned(),
        },
        Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ];

    script(&events)
}

/// `queued` waits for Apply. The runtime runs the service, so the caller
/// keeps it as long as the harness.
pub fn app(script: String, queued: &'static [&'static str]) -> (Harness<'static, App>, Runtime) {
    app_with(script, queued, None, false)
}

/// As [`app`], with `queued` installed in the read after the run.
pub fn installing_app(
    script: String,
    queued: &'static [&'static str],
) -> (Harness<'static, App>, Runtime) {
    app_with(script, queued, None, true)
}

/// Reads nothing installed first and `names` after the run.
pub fn installed_after_the_run(
    handle: &Handle,
    ctx: &egui::Context,
    names: &'static [&'static str],
) -> SnapshotHandle {
    let read = AtomicUsize::new(0);
    let probe = move || {
        let mut system = Snapshot::default();
        if read.fetch_add(1, Ordering::SeqCst) > 0 {
            system.insert(InstalledPackages(packages(names)));
        }

        (system, Vec::new())
    };

    snapshot::spawn(handle, toolbox_gui::waking::repaint(ctx.clone()), probe)
}

fn app_with(
    script: String,
    queued: &'static [&'static str],
    notifier: Option<Arc<dyn Notifier>>,
    reads: bool,
) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let changes = installing(queued);

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let launcher = Arc::new(FakeLauncher(script));
        let setup = Setup {
            snapshot: reads.then(|| installed_after_the_run(&handle, &cc.egui_ctx, queued)),
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
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);
    assert!(harness.query_by_label("Console").is_none());

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "All changes applied");
    assert!(harness.query_by_label("Done").is_some());
    assert!(harness.query_by_label("Console output").is_some());
}

#[test]
fn ctrl_enter_applies_too() {
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.step();

    wait_for(&mut harness, "All changes applied");
}

#[test]
fn a_dismissed_password_dialog_is_told_in_words() {
    let (mut harness, _runtime) = app("exit 126".to_owned(), FIREFOX);

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
                cleanup_failed: Vec::new(),
            },
        },
        Event::Done,
    ];

    let script = format!("{}; exit 1", script(&events));
    let (mut harness, _runtime) = app(script, FIREFOX);

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "Failed (exit code 1)");
}

#[test]
fn a_run_that_ends_in_the_background_is_notified() {
    let (notifier, shown) = Recorder::pair();
    let (mut harness, _runtime) = app_with(succeeding_script(), JAVA, Some(notifier), false);

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
    let (mut harness, _runtime) = app_with(succeeding_script(), JAVA, Some(notifier), false);

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");

    shown.try_recv().unwrap_err();
}

#[test]
fn a_run_that_ends_behind_a_hidden_console_leaves_a_note() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()), JAVA);

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
    let (mut harness, _runtime) = app(succeeding_script(), JAVA);

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
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()), JAVA);

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
    let (mut harness, _runtime) = app(idle_script(), FIREFOX);

    harness.get_by_label("Apply").click();
    harness.step();

    wait_for(&mut harness, "Nothing needed to change");
    assert!(harness.query_by_label("All changes applied").is_none());
}
