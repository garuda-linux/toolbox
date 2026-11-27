//! Helpers the tests of several modules share.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Answer, Event, Failure, Outcome};
use toolbox_plan::{AUR_CLONE, ConfigFilePath, ConfigText, StepId};

use crate::apply::EventSink;

pub const BACKUPS: &str = "var/lib/garuda-toolbox/backups";

impl EventSink for Vec<Event> {
    fn send(&mut self, event: Event) {
        self.push(event);
    }
}

pub fn printed(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Output { step: _, data } => Some(data.as_str()),
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Started { step: _ }
            | Event::Waiting { step: _ }
            | Event::Finished {
                step: _,
                outcome: _,
            }
            | Event::Done => None,
        })
        .collect()
}

/// Runs `builtin` below `root` without a home, as the helper does, and
/// returns how it ended and what it printed.
pub fn run_builtin(root: &Path, builtin: &Builtin) -> (Outcome, String) {
    let mut events = Vec::new();
    let outcome = crate::builtin::execute(root, None, StepId::new(1), builtin, &mut events);

    (outcome, printed(&events))
}

pub fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// Fails the test unless `outcome` is a failure without a known cause.
#[track_caller]
pub fn assert_failed(outcome: Outcome) {
    let Outcome::Failed {
        cause: Failure::Unknown,
        cleanup_failed,
    } = outcome
    else {
        panic!("the step did not fail: {outcome:?}");
    };

    assert_eq!(cleanup_failed, Vec::new(), "a cleanup failed too");
}

/// Whether the process whose id a script wrote to `pid_file` has ended,
/// waiting a moment for the kernel to end it after a kill.
pub fn gone(pid_file: &Path) -> bool {
    // Far below the sleeps of the scripts, far above a loaded test machine.
    const SOON: Duration = Duration::from_secs(10);

    let pid = fs::read_to_string(pid_file).unwrap();
    let process = Path::new("/proc").join(pid.trim());
    let started = Instant::now();

    while process.exists() && started.elapsed() < SOON {
        thread::sleep(Duration::from_millis(10));
    }

    !process.exists()
}

/// A temporary root with the home of `ada` below it, 0700 and empty. The
/// home is returned canonical, so a program that prints it matches.
pub fn ada_home() -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().canonicalize().unwrap().join("home/ada");
    fs::create_dir_all(&home).unwrap();
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

    (root, home)
}

/// Sends `answer` to a runner once `delay` has passed, from a thread of its own.
pub fn answer_after(app: Sender<Answer>, delay: Duration, answer: Answer) -> JoinHandle<()> {
    thread::Builder::new()
        .name("test-answer".to_owned())
        .spawn(move || {
            thread::sleep(delay);
            app.send(answer).unwrap();
        })
        .unwrap()
}

/// An executable shell script that runs `body`, to stand in for a program.
pub fn stand_in(dir: &Path, body: &str) -> PathBuf {
    let program = dir.join("makepkg");
    fs::write(&program, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();

    program
}

pub fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

pub fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

/// The git of the test machine, found on `PATH`.
pub fn git_program() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|folder| folder.join("git"))
                .find(|candidate| candidate.is_file())
        })
        .expect("git is on PATH")
}

/// The clone of `base` below `home`, as `paru -G` leaves one, with its one
/// commit marked reviewed. A build of it passes the review's guard.
pub fn reviewed_clone(home: &Path, base: &str) -> PathBuf {
    let folder = home.join(AUR_CLONE).join(base);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("PKGBUILD"), format!("pkgname={base}\n")).unwrap();

    for args in [
        &["init", "-q"][..],
        &["add", "--all"],
        &["commit", "-q", "-m", "reviewed"],
        &["update-ref", "refs/garuda-toolbox/reviewed", "HEAD"],
    ] {
        let status = std::process::Command::new(git_program())
            .current_dir(&folder)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args([
                "-c",
                "user.name=ada",
                "-c",
                "user.email=ada@example.invalid",
                "-c",
                "commit.gpgSign=false",
            ])
            .args(args)
            .status()
            .unwrap();

        assert!(status.success(), "git {args:?}");
    }

    folder
}
