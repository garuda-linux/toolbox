//! Invocations whose targets a query names. Each line becomes an
//! argument, so each must pass as a package name: a line like
//! `--overwrite=*` would otherwise turn into an option.

use std::collections::BTreeSet;

use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{PackageName, StepId, protection};

use super::{EventSink, Runner};
use crate::QueryKind;
use crate::report::{failure, note};

/// Rounds until the query names nothing new to remove. A protected
/// name is kept and named once, as each round lists it again.
pub(super) fn repeated(
    step: StepId,
    query: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let mut outcome = Outcome::Skipped;
    let mut named = BTreeSet::new();
    let mut last = Vec::new();

    loop {
        let targets = match targets(step, query, QueryKind::Listing, runner, sink) {
            Ok(targets) => targets,
            Err(failed) => return failed,
        };

        let (kept, removed): (Vec<_>, Vec<_>) = targets
            .into_iter()
            .partition(|target| protection(target).is_some());

        let new: Vec<&str> = kept
            .iter()
            .filter(|target| named.insert((*target).clone()))
            .map(PackageName::as_str)
            .collect();

        if !new.is_empty() {
            note(
                step,
                sink,
                &format!("kept, as protected: {}", new.join(" ")),
                Outcome::Skipped,
            );
        }

        if removed.is_empty() {
            return outcome;
        }

        // pacman succeeded yet left them, another round would not end.
        if removed == last {
            return failure(step, sink, &format!("{run} left its targets installed"));
        }

        outcome = runner.run(
            step,
            &run.clone().args(removed.iter().map(PackageName::as_str)),
            sink,
        );

        if !matches!(outcome, Outcome::Succeeded) {
            return outcome;
        }

        last = removed;
    }
}

/// The request named its packages only. What `-s` takes along is known
/// here, at the moment of the removal.
pub(super) fn unless_protected(
    step: StepId,
    preview: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let targets = match targets(step, preview, QueryKind::Listing, runner, sink) {
        Ok(targets) => targets,
        Err(failed) => return failed,
    };

    let protected = protected_names(&targets);

    if protected.is_empty() {
        return runner.run(step, run, sink);
    }

    failure(
        step,
        sink,
        &format!(
            "nothing was removed, as this would also remove the protected {}",
            protected.join(" ")
        ),
    )
}

fn protected_names(names: &[PackageName]) -> Vec<&str> {
    names
        .iter()
        .filter(|name| protection(name).is_some())
        .map(PackageName::as_str)
        .collect()
}

/// Each line must be a name `test` asked about, as they become root `pacman -S` arguments.
pub(super) fn unmet(
    step: StepId,
    test: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let unmet = match asked_names(step, test, QueryKind::Deptest, runner, sink) {
        Ok(unmet) => unmet,
        Err(failed) => return failed,
    };

    if unmet.is_empty() {
        return Outcome::Skipped;
    }

    runner.run(
        step,
        &run.clone().args(unmet.iter().map(PackageName::as_str)),
        sink,
    )
}

/// Removes the orphans among the names `query` was asked about, in one
/// `run`. A protected name ends the step before anything is removed.
pub(super) fn orphans(
    step: StepId,
    query: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let names = match asked_names(step, query, QueryKind::Orphans, runner, sink) {
        Ok(names) => names,
        Err(failed) => return failed,
    };

    if names.is_empty() {
        return Outcome::Skipped;
    }

    let protected = protected_names(&names);
    if !protected.is_empty() {
        return failure(
            step,
            sink,
            &format!(
                "nothing was removed, as the protected {} may not go",
                protected.join(" ")
            ),
        );
    }

    runner.run(
        step,
        &run.clone().args(names.iter().map(PackageName::as_str)),
        sink,
    )
}

/// The names `query` printed, each one of the names it was asked about.
fn asked_names(
    step: StepId,
    query: &Command,
    kind: QueryKind,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Result<Vec<PackageName>, Outcome> {
    let names = targets(step, query, kind, runner, sink)?;

    stranger(step, query, &names, sink).map_or(Ok(names), Err)
}

/// A line of a query becomes an argument of the next command, so it must be
/// one of the names the query was asked about.
fn stranger(
    step: StepId,
    query: &Command,
    names: &[PackageName],
    sink: &mut dyn EventSink,
) -> Option<Outcome> {
    let stranger = names
        .iter()
        .find(|name| !query.arguments().iter().any(|asked| asked == name.as_str()))?;

    Some(failure(
        step,
        sink,
        &format!("{query} named {stranger}, which it was not asked about"),
    ))
}

fn targets(
    step: StepId,
    query: &Command,
    kind: QueryKind,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Result<Vec<PackageName>, Outcome> {
    let lines = runner
        .query(query, kind)
        .map_err(|error| failure(step, sink, &format!("{query} failed: {error}")))?;

    lines
        .iter()
        .map(PackageName::new)
        .collect::<Result<_, _>>()
        .map_err(|error| {
            failure(
                step,
                sink,
                &format!("{query} printed something unexpected: {error}"),
            )
        })
}
