//! Reading the system: the status bar while a read runs and after one
//! failed, and the queue meeting each new snapshot.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tokio::runtime::Runtime;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_gui::queue::Queue;
use toolbox_gui::testing::PATIENCE;
use toolbox_gui::{App, Setup, apply, snapshot};
use toolbox_system::{Probe, ProbeError, ProbeIssue};

use crate::common::{HEIGHT, installing, packages, window};
use crate::console::{FakeLauncher, silent_script};

/// Where a wait that ran out leaves its screenshot, beside the snapshots.
const FAILURES: &str = "tests/failures";

/// What the fake probe answers, one answer per read.
pub type Answer = (Snapshot, Vec<ProbeIssue>);

/// The app, its runtime, where the probe's answers go and how often it
/// was asked.
pub type Reading = (
    Harness<'static, App>,
    Runtime,
    Sender<Answer>,
    Arc<AtomicUsize>,
);

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(names)));
    snapshot
}

/// The app with firefox and vlc queued, and a probe that answers whatever
/// the test sends, counting its reads.
pub fn app() -> Reading {
    app_with(false)
}

/// As [`app`], in a dry run when `dry_run`.
pub fn app_with(dry_run: bool) -> Reading {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let (answer, answers): (Sender<Answer>, Receiver<Answer>) = crossbeam_channel::unbounded();
    let reads = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&reads);
    let changes = installing(&["firefox", "vlc"]);

    let harness = window(HEIGHT).build_eframe(move |cc| {
        let launcher = Arc::new(FakeLauncher(silent_script()));
        let probe = move || {
            counted.fetch_add(1, Ordering::SeqCst);
            answers.recv_timeout(PATIENCE).unwrap()
        };

        let setup = Setup {
            queue: Queue::in_memory(changes),
            snapshot: Some(snapshot::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                probe,
            )),
            apply: Some(apply::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                launcher,
            )),
            dry_run,
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    (harness, runtime, answer, reads)
}

/// Steps single frames until `done`, as the spinner never lets
/// `Harness::run` settle.
#[track_caller]
pub fn step_until(
    harness: &mut Harness<'static, App>,
    what: &str,
    done: impl Fn(&Harness<'static, App>) -> bool,
) {
    step_within(harness, what, done, PATIENCE);
}

/// [`step_until`] with its own patience. A wait that runs out leaves a
/// screenshot of the window in [`FAILURES`], named after `what`.
#[track_caller]
pub fn step_within<State>(
    harness: &mut Harness<'static, State>,
    what: &str,
    done: impl Fn(&Harness<'static, State>) -> bool,
    patience: Duration,
) {
    let start = Instant::now();
    while !done(harness) {
        if start.elapsed() >= patience {
            let name = failure_name(what);
            let path = std::path::Path::new(FAILURES).join(format!("{name}.new.png"));

            // Saved as it is: a snapshot comparison would take a picture an
            // earlier run left there as its reference and write nothing.
            drop(std::fs::create_dir_all(FAILURES));
            if let Ok(picture) = harness.render() {
                drop(picture.save(&path));
            }

            panic!("{what} never happened, the window is in {}", path.display());
        }

        std::thread::sleep(Duration::from_millis(10));
        harness.step();
    }
}

/// Steps until the window stops asking for frames, for a page whose
/// reads land one after the other. `Harness::run` gives up after a few.
#[track_caller]
pub fn settle(harness: &mut Harness<'static, App>) {
    let start = Instant::now();
    while harness.try_run().is_err() {
        assert!(start.elapsed() < PATIENCE, "the window never settled");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// `what` as a file name: letters and digits, the rest as dashes.
fn failure_name(what: &str) -> String {
    what.chars()
        .map(|char| if char.is_alphanumeric() { char } else { '-' })
        .collect()
}

pub fn shows(label: &'static str) -> impl Fn(&Harness<'static, App>) -> bool {
    move |harness| harness.query_by_label(label).is_some()
}

#[track_caller]
pub fn first_read(harness: &mut Harness<'static, App>) {
    step_until(harness, "the first read", |harness| {
        harness
            .query_all_by_label("Reading the system…")
            .next()
            .is_none()
    });
}

#[test]
fn the_first_read_shows_and_then_drops_what_is_done() {
    let (mut harness, _runtime, answer, _reads) = app();

    step_until(&mut harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });
    harness.get_by_label("Review").click();
    harness.step();
    step_until(&mut harness, "the popover", |harness| {
        harness.query_by_label("Install firefox").is_some()
    });

    answer.send((installed(&["firefox"]), Vec::new())).unwrap();

    step_until(&mut harness, "firefox leaving the queue", |harness| {
        harness.query_by_label("Install firefox").is_none()
    });
    first_read(&mut harness);
    assert!(harness.query_by_label("Install vlc").is_some());
}

#[test]
fn failed_probes_are_named_in_the_status_bar_which_reads_again() {
    let (mut harness, _runtime, answer, reads) = app();
    let issues = vec![
        ProbeIssue {
            probe: Probe::Units,
            error: ProbeError::Failed("no bus".to_owned()),
        },
        ProbeIssue {
            probe: Probe::Grub,
            error: ProbeError::Unavailable,
        },
        ProbeIssue {
            probe: Probe::Accounts,
            error: ProbeError::Failed("no passwd".to_owned()),
        },
    ];

    answer.send((Snapshot::default(), issues)).unwrap();

    step_until(
        &mut harness,
        "the failure notice",
        shows("Some parts of the system could not be read: system services, users and groups"),
    );

    harness.get_by_label("Read again").click();
    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer.send((Snapshot::default(), Vec::new())).unwrap();
}

#[test]
fn coming_back_to_the_window_reads_the_system_again() {
    let (mut harness, _runtime, answer, reads) = app();
    answer.send((Snapshot::default(), Vec::new())).unwrap();
    step_until(&mut harness, "the first read", |harness| {
        harness
            .query_all_by_label("Reading the system…")
            .next()
            .is_none()
            && reads.load(Ordering::SeqCst) == 1
    });

    harness.input_mut().focused = false;
    harness.step();
    harness.input_mut().focused = true;
    harness.step();

    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer.send((Snapshot::default(), Vec::new())).unwrap();
}

#[test]
fn a_finished_plan_reads_the_system_and_empties_the_queue() {
    let (mut harness, _runtime, answer, reads) = app();
    answer.send((installed(&[]), Vec::new())).unwrap();
    first_read(&mut harness);

    harness.get_by_label("Apply").click();
    harness.step();
    step_until(&mut harness, "the plan's end", shows("All changes applied"));
    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer
        .send((installed(&["firefox", "vlc"]), Vec::new()))
        .unwrap();

    step_until(&mut harness, "the queue emptying", |harness| {
        harness.query_by_label_contains("queued").is_none()
    });
}

#[test]
fn apply_waits_for_the_first_read() {
    let (mut harness, _runtime, answer, _reads) = app();
    step_until(&mut harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });

    assert!(harness.get_by_label("Apply").accesskit_node().is_disabled());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.step();
    assert!(harness.query_by_label("Console").is_none());

    answer.send((installed(&[]), Vec::new())).unwrap();
    step_until(&mut harness, "the first read", |harness| {
        !harness.get_by_label("Apply").accesskit_node().is_disabled()
    });

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    step_until(&mut harness, "the plan's end", shows("All changes applied"));
}

/// How far above the window's bottom the side panels end: the status
/// bar's height, or nothing without it.
fn bar_height(harness: &Harness<'static, App>) -> f32 {
    let margin = f32::from(
        toolbox_gui_kit::tokens::Sizes::DEFAULT
            .sidebar_padding
            .bottom,
    );

    let sidebar = harness.get_by_label("About").rect().bottom() + margin;

    harness.ctx.content_rect().bottom() - sidebar
}

#[test]
fn a_read_alone_brings_no_bar_only_a_note_over_the_page() {
    let (mut harness, _runtime, answer, _reads) = app();
    step_until(&mut harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });
    assert!(bar_height(&harness) < 1.0, "{}", bar_height(&harness));

    answer.send((installed(&[]), Vec::new())).unwrap();
    first_read(&mut harness);
    harness.run();

    assert!(bar_height(&harness) < 1.0, "{}", bar_height(&harness));
}

#[test]
fn a_wait_that_runs_out_leaves_a_screenshot() {
    let screenshot = std::path::Path::new(FAILURES).join("a-label-that-never-shows.new.png");

    // A snapshot run with UPDATE_SNAPSHOTS leaves this picture behind, and
    // the next wait must not take it as a reference that already matches.
    let leftover = std::path::Path::new(FAILURES).join("a-label-that-never-shows.png");
    drop(std::fs::remove_file(&screenshot));
    let mut harness =
        toolbox_gui_kit::testing::harness(200.0, 100.0).build_ui(|ui| drop(ui.label("Waiting")));

    std::fs::create_dir_all(FAILURES).unwrap();
    harness.run();
    harness.render().unwrap().save(&leftover).unwrap();

    let waited = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        step_within(
            &mut harness,
            "a label that never shows",
            |harness| harness.query_by_label("Done").is_some(),
            Duration::from_millis(50),
        );
    }));

    std::fs::remove_file(leftover).unwrap();
    assert!(waited.is_err());
    assert!(screenshot.exists());
    std::fs::remove_file(screenshot).unwrap();
}

/// The app in a dry run, after a read that failed on the system services,
/// with the bar that says so.
pub fn failed_units_bar() -> Reading {
    let (mut harness, runtime, answer, reads) = app_with(true);
    let issues = vec![ProbeIssue {
        probe: Probe::Units,
        error: ProbeError::Failed("no bus".to_owned()),
    }];

    answer.send((Snapshot::default(), issues)).unwrap();
    step_until(
        &mut harness,
        "the failure notice",
        shows("Some parts of the system could not be read: system services"),
    );
    (harness, runtime, answer, reads)
}
