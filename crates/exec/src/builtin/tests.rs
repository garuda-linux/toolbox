mod arms;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{GarudaUpdateFlag, Mirrorlist, ShellPath, StepId};

use super::{LockRemoval, execute, install_mirrorlist, lock_holder, remove_database_lock};
use crate::testing::{assert_failed, run_builtin};

/// `/etc/pacman.d/chaotic-mirrorlist` as the package ships it in Lima.
const SHIPPED_MIRRORLIST: &str =
    include_str!("../../../plan/src/mirrorlist/fixtures/chaotic-mirrorlist");

fn outcome_of(root: &Path, builtin: &Builtin) -> Outcome {
    run_builtin(root, builtin).0
}

fn process(proc_root: &Path, pid: &str, open: &[&Path]) {
    let files = proc_root.join(pid).join("fd");

    fs::create_dir_all(&files).unwrap();

    for (number, file) in open.iter().enumerate() {
        symlink(file, files.join(number.to_string())).unwrap();
    }
}

#[test]
fn the_lock_is_removed_when_no_process_has_it_open() {
    let dir = tempfile::tempdir().unwrap();
    let proc_root = dir.path().join("proc");
    let lock = dir.path().join("db.lck");
    let other = dir.path().join("pacman.log");

    fs::write(&lock, "").unwrap();
    fs::write(&other, "").unwrap();
    process(&proc_root, "1", &[]);
    process(&proc_root, "42", &[&other, &dir.path().join("gone")]);

    assert_eq!(
        remove_database_lock(&proc_root, &lock).unwrap(),
        LockRemoval::Removed
    );
    assert!(!lock.exists());
}

#[test]
fn a_process_that_has_the_lock_open_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    let proc_root = dir.path().join("proc");
    let lock = dir.path().join("db.lck");
    let other = dir.path().join("pacman.log");

    fs::write(&lock, "").unwrap();
    fs::write(&other, "").unwrap();
    process(&proc_root, "1", &[&other]);
    process(&proc_root, "777", &[&other, &lock]);

    assert_eq!(
        remove_database_lock(&proc_root, &lock).unwrap(),
        LockRemoval::Held { pid: 777 }
    );
    assert!(lock.exists());
}

#[test]
fn no_lock_is_nothing_to_do() {
    let dir = tempfile::tempdir().unwrap();
    let proc_root = dir.path().join("proc");

    process(&proc_root, "1", &[]);

    assert_eq!(
        remove_database_lock(&proc_root, &dir.path().join("db.lck")).unwrap(),
        LockRemoval::NoLock
    );
}

#[test]
fn entries_that_are_not_processes_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let proc_root = dir.path().join("proc");
    let lock = dir.path().join("db.lck");

    fs::write(&lock, "").unwrap();
    process(&proc_root, "self", &[&lock]);
    fs::write(proc_root.join("uptime"), "1.0 1.0\n").unwrap();

    assert_eq!(
        remove_database_lock(&proc_root, &lock).unwrap(),
        LockRemoval::Removed
    );
}

/// What `rate-mirrors --save` wrote for Chaotic-AUR in Lima.
fn mirrorlist() -> Mirrorlist {
    Mirrorlist::new(
        include_str!("../../../plan/src/mirrorlist/fixtures/rate-mirrors-chaotic").to_owned(),
    )
    .unwrap()
}

#[test]
fn the_mirrorlist_replaces_the_old_one_and_keeps_it() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("etc/pacman.d");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("chaotic-mirrorlist"), SHIPPED_MIRRORLIST).unwrap();

    let kept = install_mirrorlist(root.path(), &mirrorlist()).unwrap();

    assert!(kept);
    assert_eq!(
        fs::read_to_string(dir.join("chaotic-mirrorlist")).unwrap(),
        mirrorlist().as_str()
    );
    assert_eq!(
        fs::read_to_string(dir.join("chaotic-mirrorlist.bak")).unwrap(),
        SHIPPED_MIRRORLIST
    );
}

#[test]
fn the_first_mirrorlist_needs_no_backup() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("etc/pacman.d");
    fs::create_dir_all(&dir).unwrap();

    let kept = install_mirrorlist(root.path(), &mirrorlist()).unwrap();

    assert!(!kept);
    assert!(!dir.join("chaotic-mirrorlist.bak").exists());
}

#[test]
fn the_mirrorlist_folder_must_exist_already() {
    let root = tempfile::tempdir().unwrap();

    install_mirrorlist(root.path(), &mirrorlist()).unwrap_err();
    assert!(!root.path().join("etc").exists());
}

fn root_with_shells() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(
        root.path().join("etc/shells"),
        include_str!("../../../system/src/accounts/fixtures/shells"),
    )
    .unwrap();
    root
}

#[test]
fn an_unlisted_shell_fails_the_step_with_a_reason() {
    let root = root_with_shells();
    let mut events = Vec::new();
    let builtin = Builtin::RequireListedShell(ShellPath::new("/usr/bin/false").unwrap());

    let outcome = execute(root.path(), None, StepId::new(1), &builtin, &mut events);

    assert_failed(outcome);
    assert_eq!(
        events,
        [Event::Output {
            step: StepId::new(1),
            data: "/usr/bin/false is not listed in /etc/shells, so it cannot be a login shell\r\n"
                .to_owned()
        }]
    );
}

#[test]
fn a_listed_shell_lets_the_step_go_on() {
    let root = root_with_shells();
    let builtin = Builtin::RequireListedShell(ShellPath::new("/usr/bin/bash").unwrap());

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
}

#[test]
fn a_file_already_as_wanted_skips() {
    let root = tempfile::tempdir().unwrap();
    let builtin = Builtin::SetGarudaUpdateFlag {
        flag: GarudaUpdateFlag::UpdateAur,
        on: false,
    };

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Skipped);
}

#[test]
fn the_lock_below_the_root_is_found() {
    let root = tempfile::tempdir().unwrap();
    process(&root.path().join("proc"), "1", &[]);
    fs::create_dir_all(root.path().join("var/lib/pacman")).unwrap();
    fs::write(root.path().join("var/lib/pacman/db.lck"), "").unwrap();

    assert_eq!(
        outcome_of(root.path(), &Builtin::RemoveDatabaseLock),
        Outcome::Succeeded
    );
    assert!(!root.path().join("var/lib/pacman/db.lck").exists());
}
