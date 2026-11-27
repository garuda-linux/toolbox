use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use rustix::fs::Mode;
use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Answer, Event, Outcome};
use toolbox_plan::{AUR_BUILT, AUR_CLONE, BuildRun, PackageName, StepId};

use super::super::{Environment, PtyRunner};
use crate::apply::Runner as _;
use crate::aur;
use crate::test_support::{
    ada_home, answer_after, assert_failed, git_program, mode, printed, reviewed_clone, stand_in,
};

const PACKAGE: &str = "octopi-git-0.16.2-1-x86_64.pkg.tar.zst";

/// A build whose makepkg writes a package file, run under the user's umask.
/// The helper's own check (`aur::stage`) must accept the file.
fn built_package_passes_the_helper(umask: u32) {
    let (root, home) = ada_home();
    reviewed_clone(&home, "octopi-git");
    let makepkg = stand_in(root.path(), &format!("printf x > \"$PKGDEST/{PACKAGE}\""));
    let (_app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Inherited).with_home(home.clone());

    let previous = rustix::process::umask(Mode::from_raw_mode(umask));
    let outcome = build(&mut runner, &makepkg, &mut Vec::new());
    rustix::process::umask(previous);

    assert_eq!(outcome, Outcome::Succeeded);
    let file = home.join(AUR_BUILT).join("1a2b").join(PACKAGE);
    assert_eq!(mode(&file) & 0o022, 0, "mode {:o}", mode(&file));
    assert_eq!(mode(&file) & 0o400, 0o400);

    let uid = rustix::process::geteuid().as_raw();
    let packages =
        toolbox_plan::TargetList::new([PackageName::new("octopi-git").unwrap()]).unwrap();

    let copies = aur::stage(root.path(), uid, Path::new("home/ada"), &run(), &packages).unwrap();
    assert_eq!(copies.len(), 1);
}

#[test]
fn a_package_file_is_accepted_by_the_helper_under_umask_002() {
    built_package_passes_the_helper(0o002);
}

#[test]
fn a_package_file_is_accepted_by_the_helper_under_umask_077() {
    built_package_passes_the_helper(0o077);
}

fn octopi() -> PackageName {
    PackageName::new("octopi-git").unwrap()
}

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

fn build(runner: &mut PtyRunner, makepkg: &Path, events: &mut Vec<Event>) -> Outcome {
    let git = git_program();

    runner.build_aur(
        StepId::new(1),
        (makepkg, git.as_path()),
        (&octopi(), &run()),
        events,
    )
}

#[test]
fn makepkg_runs_in_the_clone_with_pkgdest_and_the_flags() {
    let (root, home) = ada_home();
    reviewed_clone(&home, "octopi-git");
    let record = root.path().join("record");
    let script = format!(
        "{{ pwd -P; printf '%s\\n' \"$PKGDEST\"; printf '%s\\n' \"$@\"; }} > '{}'",
        record.display()
    );

    let makepkg = stand_in(root.path(), &script);
    let (_app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Inherited).with_home(home.clone());

    assert_eq!(
        build(&mut runner, &makepkg, &mut Vec::new()),
        Outcome::Succeeded
    );

    let clone = home
        .join(AUR_CLONE)
        .join("octopi-git")
        .display()
        .to_string();

    let built = home.join(AUR_BUILT).join("1a2b").display().to_string();
    let written = fs::read_to_string(&record).unwrap();
    assert_eq!(
        written.lines().collect::<Vec<_>>(),
        [clone.as_str(), built.as_str(), "-f", "-C"]
    );
}

#[test]
fn without_a_home_makepkg_does_not_start() {
    let (root, home) = ada_home();
    reviewed_clone(&home, "octopi-git");
    let record = root.path().join("record");
    let makepkg = stand_in(root.path(), &format!("touch '{}'", record.display()));
    let (_app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Clean);
    let mut events = Vec::new();

    assert_failed(build(&mut runner, &makepkg, &mut events));
    assert!(printed(&events).contains("home"), "{}", printed(&events));
    assert!(!record.exists());
}

#[test]
fn a_helper_runner_fails_the_build_without_running_it() {
    let (_app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Clean);
    let mut events = Vec::new();
    let step = Builtin::BuildAur {
        base: octopi(),
        run: run(),
    };

    assert_failed(runner.builtin(StepId::new(1), &step, &mut events));
    assert!(printed(&events).contains("home"), "{}", printed(&events));
}

#[test]
fn a_missing_clone_names_its_folder_and_makepkg_does_not_start() {
    let (root, home) = ada_home();
    let record = root.path().join("record");
    let makepkg = stand_in(root.path(), &format!("touch '{}'", record.display()));
    let (_app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Inherited).with_home(home);
    let mut events = Vec::new();

    assert_failed(build(&mut runner, &makepkg, &mut events));
    assert!(
        printed(&events).contains("~/.cache/garuda-toolbox/aur-clone/octopi-git"),
        "{}",
        printed(&events)
    );
    assert!(!record.exists());
}

#[test]
fn a_cancel_stops_the_running_makepkg() {
    let (root, home) = ada_home();
    reviewed_clone(&home, "octopi-git");
    let makepkg = stand_in(root.path(), "sleep 30");
    let (app, answers) = channel();
    let mut runner = PtyRunner::new(answers, Environment::Inherited).with_home(home);
    let canceller = answer_after(app, Duration::from_millis(300), Answer::Cancel);

    let started = Instant::now();
    let outcome = build(&mut runner, &makepkg, &mut Vec::new());
    canceller.join().unwrap();

    assert!(started.elapsed() < Duration::from_secs(20));
    assert_ne!(outcome, Outcome::Succeeded);
    assert!(runner.cancelled);
}
