use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use toolbox_plan::UserName;
use toolbox_plan::protocol::ReadKind;

use super::{HelperLocks, LockProblem, LockSource as _, parse};
use crate::HelperReads;
use crate::pkexec::Refusal;
use crate::test_support::runnable_script;

/// Runs its script in place of pkexec and the helper.
struct Script(PathBuf);

impl HelperReads for Script {
    fn read_command(&self, _kind: ReadKind) -> Command {
        Command::new(&self.0)
    }
}

fn helper(path: PathBuf) -> HelperLocks {
    HelperLocks(Arc::new(Script(path)))
}

#[test]
fn the_answer_becomes_a_map_of_flags() {
    let locks = parse(
        "{\"locks\":[{\"user\":\"nico\",\"locked\":false},{\"user\":\"guest\",\"locked\":true}]}\n",
    )
    .unwrap();

    assert_eq!(locks.get(&UserName::new("nico").unwrap()), Some(&false));
    assert_eq!(locks.get(&UserName::new("guest").unwrap()), Some(&true));
    assert_eq!(locks.len(), 2);
}

#[test]
fn anything_else_is_no_answer() {
    for answer in [
        "",
        "not json",
        "{\"locks\":[{\"user\":\"Bad Name\",\"locked\":true}]}",
        "{\"locks\":[{\"user\":\"nico\",\"locked\":\"$y$hash\"}]}",
        "{\"other\":1}",
    ] {
        assert_eq!(parse(answer), None, "{answer}");
    }
}

#[test]
fn a_helper_that_cannot_start_or_fails_leaves_the_flags_unknown() {
    let missing = helper(PathBuf::from("/nonexistent/helper"));

    assert!(matches!(
        missing.locks().unwrap_err(),
        LockProblem::Failed(reason) if reason.contains("cannot start")
    ));

    let folder = tempfile::tempdir().unwrap();

    assert!(matches!(
        refusing(folder.path(), "cannot answer shadow-locks", 1)
            .locks()
            .unwrap_err(),
        LockProblem::Failed(reason) if reason.contains("cannot answer")
    ));
}

/// A script standing in for pkexec or the helper behind it.
fn refusing(folder: &Path, stderr: &str, code: i32) -> HelperLocks {
    let script = folder.join(format!("helper-{code}-{}", stderr.len()));

    runnable_script(&script, &format!("printf '%s' '{stderr}' >&2\nexit {code}"));
    helper(script)
}

#[test]
fn a_refusal_of_pkexec_is_told_apart_by_its_code_and_words() {
    let folder = tempfile::tempdir().unwrap();

    for (stderr, code, refusal) in [
        ("", 126_i32, Refusal::Dismissed),
        ("Not authorized", 127_i32, Refusal::Denied),
        ("No authentication agent found.", 127_i32, Refusal::NoAgent),
        (
            "Cannot run program /helper: No such file or directory",
            127_i32,
            Refusal::HelperMissing,
        ),
    ] {
        assert_eq!(
            refusing(folder.path(), stderr, code).locks(),
            Err(LockProblem::Refused(refusal)),
            "{stderr}"
        );
    }
}

/// A cut answer could read as fewer accounts, so a long one is no answer.
#[test]
fn an_answer_past_the_cap_leaves_the_flags_unknown() {
    let folder = tempfile::tempdir().unwrap();
    let script = folder.path().join("helper");

    runnable_script(&script, "head -c 2000000 /dev/zero");

    let locks = helper(script);

    assert!(matches!(
        locks.locks().unwrap_err(),
        LockProblem::Failed(reason) if reason.contains("too long")
    ));
}
