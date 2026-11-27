//! Reading the system: the status bar while a read runs and after one
//! failed, and the queue meeting each new snapshot.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use egui::Theme;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::Snapshot;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::kernels::LinuxPackages;
use toolbox_gui::queue::Queue;
use toolbox_gui::testing::PATIENCE;
use toolbox_gui::{App, Setup, apply, snapshot};
use toolbox_system::{Probe, ProbeError, ProbeIssue};

use crate::common::{HEIGHT, installing, packages, window};
use crate::console::{FakeLauncher, silent_script};

#[cfg(test)]
mod tests;

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

/// With every part the guards of the package queue check against, so a
/// restored queue may apply.
fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(names)));
    snapshot.insert(SyncPackages::default());
    snapshot.insert(LinuxPackages::default());
    snapshot
}

/// The app with firefox and vlc queued, and a probe that answers whatever
/// the test sends, counting its reads.
pub fn app() -> Reading {
    app_with(false)
}

/// As [`app`], in a dry run when `dry_run`.
pub fn app_with(dry_run: bool) -> Reading {
    app_on(dry_run, Theme::Dark)
}

pub fn app_on(dry_run: bool, theme: Theme) -> Reading {
    app_sized(HEIGHT, dry_run, theme)
}

/// As [`app`], in a window `height` tall, its first read still running.
pub fn app_at(height: f32) -> Reading {
    app_sized(height, false, Theme::Dark)
}

fn app_sized(height: f32, dry_run: bool, theme: Theme) -> Reading {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let (answer, answers): (Sender<Answer>, Receiver<Answer>) = crossbeam_channel::unbounded();
    let reads = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&reads);
    let changes = installing(&["firefox", "vlc"]);
    let look = crate::look::look_for(&runtime, theme);

    let harness = window(height).with_theme(theme).build_eframe(move |cc| {
        let launcher = Arc::new(FakeLauncher(silent_script()));
        let probe = move || {
            counted.fetch_add(1, Ordering::SeqCst);
            answers.recv_timeout(PATIENCE).unwrap()
        };

        let setup = Setup {
            look,
            queue: Queue::restored_in_memory(changes),
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

#[track_caller]
fn until_reading_notice(harness: &mut Harness<'static, App>) {
    step_until(harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });
}

/// The app in a dry run, after a read that failed on the system services,
/// with the bar that says so.
pub fn failed_units_bar() -> Reading {
    failed_units_bar_on(Theme::Dark)
}

/// The failed-units bar of [`failed_units_bar`], drawn on `theme`.
pub fn failed_units_bar_on(theme: Theme) -> Reading {
    let (mut harness, runtime, answer, reads) = app_on(true, theme);
    let issues = vec![ProbeIssue {
        probe: Probe::Units,
        error: ProbeError::fake("no bus"),
    }];

    answer.send((Snapshot::default(), issues)).unwrap();
    step_until(
        &mut harness,
        "the failure notice",
        shows("Some parts of the system could not be read: system services"),
    );
    (harness, runtime, answer, reads)
}
