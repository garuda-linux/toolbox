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

    let protected: Vec<&str> = targets
        .iter()
        .filter(|target| protection(target).is_some())
        .map(PackageName::as_str)
        .collect();

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

/// Each line must be a name `test` asked about, as they become root `pacman -S` arguments.
pub(super) fn unmet(
    step: StepId,
    test: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let unmet = match targets(step, test, QueryKind::Deptest, runner, sink) {
        Ok(unmet) => unmet,
        Err(failed) => return failed,
    };

    if let Some(stranger) = unmet
        .iter()
        .find(|name| !test.arguments().iter().any(|asked| asked == name.as_str()))
    {
        return failure(
            step,
            sink,
            &format!("{test} named {stranger}, which it was not asked about"),
        );
    }

    if unmet.is_empty() {
        return Outcome::Skipped;
    }

    runner.run(
        step,
        &run.clone().args(unmet.iter().map(PackageName::as_str)),
        sink,
    )
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
