//! The git runs of the review and of the build guard. Each one is hardened
//! against what the repository or the user's config could make git run or
//! hide, so the text shown is the text git reads.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use crate::Environment;
use crate::capture::{CaptureError, Policy, capture};

/// The app's own ref: the commit of the build files the user last said yes to.
/// No paru ref is read, since paru keeps none.
pub(super) const REVIEWED: &str = "refs/garuda-toolbox/reviewed";

/// git's empty tree, present in every SHA-1 repository. A first review
/// diffs from it, so every tracked file shows in full.
pub(super) const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// The most diff text one base may show. A review of half the text is no
/// review, so a larger diff fails the step and nothing is asked.
const DIFF_CAP: usize = 256 << 10;

const DEADLINE: Duration = Duration::from_secs(120);

/// Before the subcommand. `hooksPath` stops the reference-transaction hook
/// of `update-ref`, `fsmonitor` a program that reads the worktree, and
/// `--no-pager` a pager that `core.pager` would name.
const HARDENING: [&str; 5] = [
    "--no-pager",
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "core.fsmonitor=false",
];

/// Outside the clean environment, so the user's `~/.gitconfig` and the
/// system config are not read. A user config had `diff.external` set to a
/// program here, which the diff would have run.
const SETTINGS: [(&str, &str); 3] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_ATTR_NOSYSTEM", "1"),
];

/// Hardening of `diff`. `--no-ext-diff` stops `diff.external` and a driver's
/// `command`, `--no-textconv` a `textconv` filter the attributes name, and
/// `--text` keeps `-diff` or `binary` attributes from hiding a file as
/// "Binary files differ". `--no-color` keeps escape sequences out.
const DIFF: [&str; 5] = [
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--text",
    "--no-color",
];

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("cannot run git: {0}")]
    Run(CaptureError),
    #[error("git exited with {0:?}")]
    Exit(Option<i32>),
    #[error("git printed no commit id")]
    NoId,
    #[error("the changes are more than {DIFF_CAP} bytes")]
    TooLarge,
}

/// One repository folder, run as the user with the cancel flag of the app.
pub struct Git<'a> {
    program: &'a Path,
    folder: &'a Path,
    cancel: &'a Arc<AtomicBool>,
}

impl<'a> Git<'a> {
    pub const fn new(program: &'a Path, folder: &'a Path, cancel: &'a Arc<AtomicBool>) -> Self {
        Self {
            program,
            folder,
            cancel,
        }
    }

    /// The commit `rev` names, read from git's own `rev-parse` output, or
    /// `None` when git says there is none.
    pub fn commit(&self, rev: &str) -> Result<Option<String>, GitError> {
        let spec = format!("{rev}^{{commit}}");
        let policy = Policy::whole().within(DEADLINE);
        let captured = self.run(&["rev-parse", "--verify", "--quiet", &spec], &policy)?;

        match captured.code {
            Some(0) => {
                let id = captured.output.trim();

                if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(GitError::NoId);
                }

                Ok(Some(id.to_owned()))
            }
            Some(1) => Ok(None),
            code => Err(GitError::Exit(code)),
        }
    }

    /// The text of the changes from `from` to `to`, at most [`DIFF_CAP`]
    /// bytes, or [`GitError::TooLarge`] when there are more.
    pub(super) fn diff(&self, from: &str, to: &str) -> Result<String, GitError> {
        let policy = Policy::tail(DIFF_CAP).capped().within(DEADLINE);
        let mut args = DIFF.to_vec();
        args.extend([from, to, "--"]);

        let captured = self.run(&args, &policy).map_err(|error| match error {
            GitError::Run(CaptureError::TooLarge) => GitError::TooLarge,
            kept @ (GitError::Run(_) | GitError::Exit(_) | GitError::NoId | GitError::TooLarge) => {
                kept
            }
        })?;

        match captured.code {
            Some(0) => Ok(captured.output),
            code => Err(GitError::Exit(code)),
        }
    }

    /// Whether the tracked files equal `HEAD`. Untracked files do not count.
    pub(super) fn tracked_unchanged(&self) -> Result<bool, GitError> {
        let policy = Policy::whole().within(DEADLINE);
        let captured = self.run(
            &[
                "diff",
                "--quiet",
                "--no-ext-diff",
                "--no-textconv",
                "HEAD",
                "--",
            ],
            &policy,
        )?;

        match captured.code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            code => Err(GitError::Exit(code)),
        }
    }

    /// Points the app's ref at `id`.
    pub(super) fn set_reviewed(&self, id: &str) -> Result<(), GitError> {
        let policy = Policy::whole().within(DEADLINE);
        let captured = self.run(&["update-ref", REVIEWED, id], &policy)?;

        match captured.code {
            Some(0) => Ok(()),
            code => Err(GitError::Exit(code)),
        }
    }

    fn run(
        &self,
        args: &[&str],
        policy: &Policy<'_>,
    ) -> Result<crate::capture::Captured, GitError> {
        let (words, variables) = invocation(args);
        let mut command = Command::new(self.program);
        command.current_dir(self.folder).args(&words);

        capture(
            &mut command,
            Environment::Clean,
            &variables,
            policy,
            self.cancel,
        )
        .map_err(GitError::Run)
    }
}

/// The argv after the program and the variables added to the clean
/// environment, for one git run. A pure function, so the hardening is pinned
/// by tests without starting git.
pub fn invocation(args: &[&str]) -> (Vec<String>, Vec<(&'static str, &'static str)>) {
    let words = HARDENING
        .iter()
        .copied()
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect();

    (words, SETTINGS.to_vec())
}
