//! The review of AUR build files. paru fetches the build files of the plan's
//! packages into the app's clone folder, the user sees what changed since
//! the last yes, and one explicit yes marks the shown commits reviewed.
//! Nothing is built or installed here, and a build refuses any folder that
//! differs from its reviewed commit.

mod git;
#[cfg(test)]
mod tests;

use std::io;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use rustix::fs::{AtFlags, FileType, Mode};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{AUR_CLONE, PackageName, StepId, TargetList};
use toolbox_text::OutsideText;

use self::git::{EMPTY_TREE, Git, GitError, REVIEWED};
use super::route::{RouteProblem, route_from_home};
use crate::apply::EventSink;
use crate::capture::{CaptureError, Captured, Policy};
use crate::report::{failure, note, output};
use crate::secure;

/// paru prints a few lines per base. A flood past this stops the fetch.
const FETCH_CAP: usize = 64 << 10;

const FETCH_DEADLINE: Duration = Duration::from_secs(300);

/// The app's side of a review: the Console and its answers. The helper has
/// none, so it fails a review before this runs.
pub trait Console {
    fn home(&self) -> Option<PathBuf>;

    /// Read by every git run, so a stop ends it like a cancel.
    fn stop(&self) -> Arc<AtomicBool>;

    /// Runs `command` captured while the answers are read. A cancel, a finish,
    /// a closed channel or the stop flag stops it.
    fn watched(
        &mut self,
        command: Command,
        policy: Policy<'static>,
    ) -> Result<Captured, CaptureError>;

    /// Drops the answers typed so far, so none of them answers the question.
    fn drain(&mut self);

    /// The first line typed, without its line break. `None` when the wait
    /// ends without one: a cancel, a finish, a closed channel or the stop flag.
    fn line(&mut self) -> Option<String>;
}

#[derive(Debug, thiserror::Error)]
enum ReviewError {
    #[error(transparent)]
    Route(#[from] RouteProblem),
    #[error("{base}: paru left no folder in ~/{AUR_CLONE}, so there is nothing to review")]
    MissingFolder { base: String },
    #[error(
        "{base}: its folder in ~/{AUR_CLONE} is a link or no folder, and links are not followed"
    )]
    LinkedFolder { base: String },
    #[error("{base}: its folder has no PKGBUILD")]
    MissingPkgbuild { base: String },
    #[error("{base}: its folder is no git repository")]
    MissingRepository { base: String },
    #[error("{base}: git has no commit to show")]
    MissingCommit { base: String },
    #[error("{base}: {source}")]
    Git { base: String, source: GitError },
    #[error(
        "{base}: the changes are too large to show in full, so nothing was asked and nothing was built"
    )]
    TooLarge { base: String },
}

/// Why a build may not start from a folder.
#[derive(Debug, thiserror::Error)]
pub enum Unreviewed {
    #[error("it has no review yet, so it is not built")]
    NotReviewed,
    #[error("the build files changed after the review, so review them again")]
    Changed,
    #[error("cannot check the build files against the review: {0}")]
    Check(GitError),
}

/// What the Console shows of one base. `body` is `None` when nothing changed.
struct Part {
    base: PackageName,
    folder: PathBuf,
    head: String,
    heading: String,
    body: Option<String>,
}

/// Fetches every base with one `paru -G`, shows each one and asks once.
/// Only a yes marks the shown commits reviewed.
pub fn run(
    console: &mut dyn Console,
    (paru, git): (&Path, &Path),
    step: StepId,
    bases: &TargetList<PackageName>,
    sink: &mut dyn EventSink,
) -> Outcome {
    let Some(home) = console.home() else {
        return failure(
            step,
            sink,
            "the review runs as the user, and this process has no home",
        );
    };

    if let Err(error) = route_from_home(&home, rustix::process::geteuid().as_raw()) {
        return failure(step, sink, &error.to_string());
    }

    if let Err(error) = make_clone(&home) {
        return failure(step, sink, &format!("cannot make ~/{AUR_CLONE}: {error}"));
    }

    if let Err(outcome) = fetch(console, (paru, &home), bases, step, sink) {
        return outcome;
    }

    let cancel = console.stop();
    let parts = match show_all(git, &home, bases, &cancel) {
        Ok(parts) => parts,
        Err(error) => return failure(step, sink, &error.to_string()),
    };

    for part in &parts {
        show(step, sink, part);
    }

    console.drain();
    output(step, sink, &question(bases));

    if !confirmed(console.line().as_deref()) {
        output(
            step,
            sink,
            "nothing is built, and nothing was marked reviewed\n",
        );
        return Outcome::Cancelled;
    }

    for part in &parts {
        if let Err(error) = Git::new(git, &part.folder, &cancel).set_reviewed(&part.head) {
            return failure(
                step,
                sink,
                &format!("{}: cannot mark it reviewed: {error}", part.base),
            );
        }

        output(step, sink, &format!("{} is marked reviewed\n", part.base));
    }

    Outcome::Succeeded
}

/// Whether the folder's `HEAD` is the commit the user last said yes to, and
/// no tracked file differs from it. Untracked files, such as sources the
/// build downloaded, do not count.
///
/// # Errors
///
/// [`Unreviewed`] for the first check that fails.
pub(super) fn unchanged_since_review(
    git: &Path,
    folder: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<(), Unreviewed> {
    let git = Git::new(git, folder, cancel);
    let head = git
        .commit("HEAD")
        .map_err(Unreviewed::Check)?
        .ok_or(Unreviewed::NotReviewed)?;

    let reviewed = git
        .commit(REVIEWED)
        .map_err(Unreviewed::Check)?
        .ok_or(Unreviewed::NotReviewed)?;

    if reviewed != head {
        return Err(Unreviewed::Changed);
    }

    match git.tracked_unchanged() {
        Ok(true) => Ok(()),
        Ok(false) => Err(Unreviewed::Changed),
        Err(error) => Err(Unreviewed::Check(error)),
    }
}

/// The folder for the clone, made with mode 0700, since it holds what the
/// user reviewed and what makepkg runs.
fn make_clone(home: &Path) -> io::Result<()> {
    let folder = secure::create_dirs(home, Path::new(AUR_CLONE))?;

    rustix::fs::fchmod(&folder, Mode::RWXU).map_err(io::Error::from)
}

fn show_all(
    git: &Path,
    home: &Path,
    bases: &TargetList<PackageName>,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<Part>, ReviewError> {
    let mut parts = Vec::new();

    for base in bases {
        parts.push(part(git, home, base, cancel)?);
    }

    Ok(parts)
}

fn part(
    git: &Path,
    home: &Path,
    base: &PackageName,
    cancel: &Arc<AtomicBool>,
) -> Result<Part, ReviewError> {
    let folder = base_folder(home, base)?;
    let runs = Git::new(git, &folder, cancel);
    let named = |source: GitError| ReviewError::Git {
        base: base.as_str().to_owned(),
        source,
    };

    let head = runs
        .commit("HEAD")
        .map_err(named)?
        .ok_or_else(|| ReviewError::MissingCommit {
            base: base.as_str().to_owned(),
        })?;

    let reviewed = runs.commit(REVIEWED).map_err(named)?;

    let (heading, body) = match reviewed.as_deref() {
        Some(from) if from == head => (format!("{base}: no change since the last review"), None),
        Some(from) => (
            format!("{base}: changes since the last review"),
            Some(diff(&runs, base, from, &head)?),
        ),
        None => (
            format!("{base}: all files, the first review"),
            Some(diff(&runs, base, EMPTY_TREE, &head)?),
        ),
    };

    Ok(Part {
        base: base.clone(),
        folder,
        head,
        heading,
        body,
    })
}

/// The text of the changes, kept to what the Console may show. A diff over
/// the cap is an error, since a cut diff is no review.
fn diff(runs: &Git<'_>, base: &PackageName, from: &str, to: &str) -> Result<String, ReviewError> {
    match runs.diff(from, to) {
        Ok(text) => Ok(OutsideText::keep_lines(&text).into_string()),
        Err(GitError::TooLarge) => Err(ReviewError::TooLarge {
            base: base.as_str().to_owned(),
        }),
        Err(source) => Err(ReviewError::Git {
            base: base.as_str().to_owned(),
            source,
        }),
    }
}

/// The base's folder below the clone, reached without a link, with a
/// PKGBUILD and a repository of its own.
fn base_folder(home: &Path, base: &PackageName) -> Result<PathBuf, ReviewError> {
    let name = base.as_str();
    let relative = Path::new(AUR_CLONE).join(name);

    let folder = match secure::open_dir(home, &relative) {
        Ok(folder) => folder,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(ReviewError::MissingFolder {
                base: name.to_owned(),
            });
        }
        Err(_) => {
            return Err(ReviewError::LinkedFolder {
                base: name.to_owned(),
            });
        }
    };

    if !is_regular_file(&folder, "PKGBUILD") {
        return Err(ReviewError::MissingPkgbuild {
            base: name.to_owned(),
        });
    }

    if secure::open_dir(home, &relative.join(".git")).is_err() {
        return Err(ReviewError::MissingRepository {
            base: name.to_owned(),
        });
    }

    Ok(home.join(relative))
}

fn is_regular_file(folder: &OwnedFd, name: &str) -> bool {
    rustix::fs::statat(folder, name, AtFlags::SYMLINK_NOFOLLOW)
        .is_ok_and(|stat| FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile)
}

fn show(step: StepId, sink: &mut dyn EventSink, part: &Part) {
    note(step, sink, &part.heading, Outcome::Skipped);

    match &part.body {
        Some(body) if body.is_empty() => {
            note(step, sink, "no text changed", Outcome::Skipped);
        }
        Some(body) => output(step, sink, body),
        None => {}
    }
}

fn question(bases: &TargetList<PackageName>) -> String {
    let names: Vec<&str> = bases.iter().map(PackageName::as_str).collect();

    format!(
        "Build {}? Type y and press Enter to build. Anything else cancels.\n",
        names.join(" ")
    )
}

/// Only `y` or `Y` on its own line says yes.
fn confirmed(line: Option<&str>) -> bool {
    matches!(line, Some("y" | "Y"))
}

/// `paru -G` of every base, captured. Its lines name AUR packages and
/// versions, which is outside text, so they are cleaned before they show.
fn fetch(
    console: &mut dyn Console,
    (paru, home): (&Path, &Path),
    bases: &TargetList<PackageName>,
    step: StepId,
    sink: &mut dyn EventSink,
) -> Result<(), Outcome> {
    let mut command = Command::new(paru);
    command
        .current_dir(home.join(AUR_CLONE))
        .args(["-G", "--"])
        .args(bases.iter().map(PackageName::as_str));

    let policy = Policy::tail(FETCH_CAP).capped().within(FETCH_DEADLINE);
    let fetched = match console.watched(command, policy) {
        Ok(fetched) => fetched,
        Err(CaptureError::Cancelled) => return Err(Outcome::Cancelled),
        Err(CaptureError::TooLarge) => {
            return Err(failure(
                step,
                sink,
                &format!("paru printed more than {FETCH_CAP} bytes, so the fetch is stopped"),
            ));
        }
        Err(error) => return Err(failure(step, sink, &format!("paru: {error}"))),
    };

    output(
        step,
        sink,
        &OutsideText::keep_lines(&fetched.output).into_string(),
    );
    output(
        step,
        sink,
        &OutsideText::keep_lines(&fetched.errors).into_string(),
    );

    if fetched.code != Some(0_i32) {
        return Err(failure(
            step,
            sink,
            &format!("paru exited with {:?}", fetched.code),
        ));
    }

    Ok(())
}
