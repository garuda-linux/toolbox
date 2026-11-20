use std::fs;

use tempfile::TempDir;
use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{ConfigText, MergeResolve, Resolution, StepId};

use super::{merge_of, pacnew, system};
use crate::config_file::WriteError;
use crate::merge::{MergeError, Resolved, execute, resolve};
use crate::test_support::{BACKUPS, printed, text};

const CURRENT: &str = "root ALL=(ALL:ALL) ALL\n";
const NEW: &str = "root ALL=(ALL:ALL) ALL\n@includedir /etc/sudoers.d\n";
const VISUDO: &str = "visudo -c -f -";

/// Stands in for the runner's check, recording what it was asked.
struct Checks {
    seen: Vec<(String, String)>,
    answer: Outcome,
}

impl Checks {
    const fn answering(answer: Outcome) -> Self {
        Self {
            seen: Vec::new(),
            answer,
        }
    }

    fn check(&mut self, command: &Command, text: &ConfigText) -> Outcome {
        self.seen
            .push((command.to_string(), text.as_str().to_owned()));
        self.answer.clone()
    }

    fn saw(&self, text: &str) -> bool {
        self.seen == [(VISUDO.to_owned(), text.to_owned())]
    }
}

fn refused() -> Outcome {
    Outcome::Failed {
        cause: Failure::ExitCode { code: 1_i32 },
        cleanup_failed: Vec::new(),
    }
}

fn sudoers() -> TempDir {
    let root = system();
    fs::write(root.path().join("etc/sudoers"), CURRENT).unwrap();
    fs::write(root.path().join("etc/sudoers.pacnew"), NEW).unwrap();
    root
}

fn merge(resolution: Resolution, current: Option<&str>) -> MergeResolve {
    merge_of("/etc/sudoers.pacnew", NEW, resolution, current)
}

fn bytes(root: &TempDir, file: &str) -> Option<Vec<u8>> {
    fs::read(root.path().join(file)).ok()
}

fn resolved(
    root: &TempDir,
    merge: &MergeResolve,
    checks: &mut Checks,
) -> Result<Resolved, MergeError> {
    resolve(root.path(), merge, |command, text| {
        checks.check(command, text)
    })
}

#[test]
fn a_text_visudo_accepts_takes_the_files_place() {
    let root = sudoers();
    let mut checks = Checks::answering(Outcome::Succeeded);

    let done = resolved(
        &root,
        &merge(Resolution::TakeCopy, Some(CURRENT)),
        &mut checks,
    );

    assert_eq!(done.unwrap(), Resolved::Written);
    assert!(checks.saw(NEW), "{:?}", checks.seen);
    assert_eq!(bytes(&root, "etc/sudoers"), Some(NEW.into()));
    assert_eq!(bytes(&root, "etc/sudoers.pacnew"), None);
}

#[test]
fn a_text_visudo_refuses_leaves_both_files_and_fails_the_step() {
    let root = sudoers();
    let mut checks = Checks::answering(refused());
    let mut events = Vec::new();
    let merged = "root ALL=(ALL:ALL\n";

    let outcome = execute(
        root.path(),
        StepId::new(1),
        &merge(Resolution::Merged { text: text(merged) }, Some(CURRENT)),
        &mut events,
        |command, text, _sink| checks.check(command, text),
    );

    assert_eq!(outcome, refused());
    assert!(checks.saw(merged), "{:?}", checks.seen);
    assert_eq!(bytes(&root, "etc/sudoers"), Some(CURRENT.into()));
    assert_eq!(bytes(&root, "etc/sudoers.pacnew"), Some(NEW.into()));
    assert_eq!(bytes(&root, &format!("{BACKUPS}/etc/sudoers")), None);

    let said = printed(&events);
    assert!(said.contains("visudo refused"), "{said}");
}

#[test]
fn a_copy_that_moves_in_is_checked_with_its_own_text() {
    let root = sudoers();
    fs::remove_file(root.path().join("etc/sudoers")).unwrap();
    let mut checks = Checks::answering(Outcome::Succeeded);

    let done = resolved(&root, &merge(Resolution::TakeCopy, None), &mut checks);

    assert_eq!(done.unwrap(), Resolved::Moved);
    assert!(checks.saw(NEW), "{:?}", checks.seen);
    assert_eq!(bytes(&root, "etc/sudoers"), Some(NEW.into()));
}

#[test]
fn a_refused_copy_does_not_move_in() {
    let root = sudoers();
    fs::remove_file(root.path().join("etc/sudoers")).unwrap();
    let mut checks = Checks::answering(refused());

    let done = resolved(&root, &merge(Resolution::TakeCopy, None), &mut checks);

    assert!(matches!(done, Err(MergeError::Refused(_))), "{done:?}");
    assert_eq!(bytes(&root, "etc/sudoers"), None);
    assert_eq!(bytes(&root, "etc/sudoers.pacnew"), Some(NEW.into()));
}

#[test]
fn keeping_the_current_sudoers_file_runs_no_check() {
    let root = sudoers();
    let mut checks = Checks::answering(refused());

    let done = resolved(
        &root,
        &merge(Resolution::KeepCurrent, Some(CURRENT)),
        &mut checks,
    );

    assert_eq!(done.unwrap(), Resolved::Removed);
    assert_eq!(checks.seen, []);
}

#[test]
fn a_file_that_is_no_sudoers_file_runs_no_check() {
    let root = system();
    let mut checks = Checks::answering(refused());

    let done = resolved(&root, &pacnew(Resolution::TakeCopy), &mut checks);

    assert_eq!(done.unwrap(), Resolved::Written);
    assert_eq!(checks.seen, []);
}

#[test]
fn a_file_that_changed_fails_before_any_check() {
    let root = sudoers();
    fs::write(root.path().join("etc/sudoers"), "edited meanwhile\n").unwrap();
    let mut checks = Checks::answering(Outcome::Succeeded);

    let done = resolved(
        &root,
        &merge(Resolution::TakeCopy, Some(CURRENT)),
        &mut checks,
    );

    assert!(matches!(done, Err(MergeError::Write(WriteError::Changed))));
    assert_eq!(checks.seen, []);
}
