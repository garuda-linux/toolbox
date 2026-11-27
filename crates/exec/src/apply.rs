//! How a single command runs is the [`Runner`]'s business, so this order of events is tested without root or real programs.
//! The helper runs root steps with it, the app user steps.

mod runner;
mod targets;
#[cfg(test)]
mod tests;

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

use toolbox_plan::procedure::{Builtin, Command, Invocation};
use toolbox_plan::protocol::{Event, Failure, Leftover, Mode, Outcome};
use toolbox_plan::{Step, StepId};

pub use self::runner::{EventSink, Procedure, Runner};
use crate::builtin::SYSTEM_ROOT;
use crate::report::{failure, note};
use crate::{aur, initramfs, merge, repair, uki};

pub fn apply<A>(
    mode: Mode,
    steps: &[Step<A>],
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> bool
where
    A: Procedure,
{
    apply_gated(mode, steps, &[], runner, sink)
}

/// Like [`apply`], but first waits before each step named in `gates`, and
/// the run ends where the wait does.
pub fn apply_gated<A>(
    mode: Mode,
    steps: &[Step<A>],
    gates: &[StepId],
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> bool
where
    A: Procedure,
{
    sink.send(Event::Accepted);

    let mut all_ok = true;

    for step in steps {
        // Built once: a config file write carries its whole text.
        let procedure = step.action.procedure();
        let _step = span(step, &procedure).entered();
        let proceeds = !gates.contains(&step.id) || runner.wait_to_proceed(step.id, sink);
        sink.send(Event::Started { step: step.id });

        // Reported, not left out: a plan whose rest is missing would read
        // as finished.
        let outcome = if proceeds {
            match mode {
                Mode::DryRun => Outcome::Skipped,
                Mode::Run if runner.finish_requested() => Outcome::Cancelled,
                Mode::Run => run_procedure(step.id, &procedure, runner, sink),
            }
        } else {
            Outcome::Cancelled
        };

        let ok = matches!(outcome, Outcome::Succeeded | Outcome::Skipped);

        sink.send(Event::Finished {
            step: step.id,
            outcome,
        });

        if !ok {
            all_ok = false;
            break;
        }
    }

    sink.send(Event::Done);
    all_ok
}

/// Never the arguments, which can hold a config file or a user's words.
/// Debug, so the helper's info-level stderr never shows it.
pub fn step_span<A>(step: &Step<A>) -> tracing::Span
where
    A: Procedure,
{
    span(step, &step.action.procedure())
}

fn span<A>(step: &Step<A>, procedure: &[Invocation]) -> tracing::Span {
    tracing::debug_span!("step", index = %step.id, program = program_name(procedure))
}

fn program_name(procedure: &[Invocation]) -> &'static str {
    procedure
        .first()
        .map_or("nothing", |invocation| match invocation {
            Invocation::Always(command)
            | Invocation::Cleanup {
                command,
                leftover: _,
            }
            | Invocation::Run(command)
            | Invocation::RunWithTargetsFrom {
                query: _,
                run: command,
            }
            | Invocation::RunWithOrphansFrom {
                query: _,
                run: command,
            }
            | Invocation::RunWithUnmetFrom {
                test: _,
                run: command,
            }
            | Invocation::RunUnlessItRemovesProtected {
                preview: _,
                run: command,
            }
            | Invocation::Feed { command, input: _ } => command.program().name(),
            Invocation::Builtin(_) => "builtin",
        })
}

/// After the first failure or cancel only [`Invocation::Always`] runs. The
/// step reports that first failure, or the always part's own one.
fn run_procedure(
    step: StepId,
    procedure: &[Invocation],
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let mut result = Outcome::Skipped;
    let mut failed = None;
    let mut left = Vec::new();

    for invocation in procedure {
        let cleanup = matches!(
            invocation,
            Invocation::Always(_)
                | Invocation::Cleanup {
                    command: _,
                    leftover: _
                }
        );

        if failed.is_some() && !cleanup {
            continue;
        }

        let outcome = guarded(step, invocation, runner, sink);

        match outcome {
            Outcome::Succeeded => result = Outcome::Succeeded,
            Outcome::Skipped => {}
            Outcome::Failed {
                cause: _,
                cleanup_failed: _,
            }
            | Outcome::Cancelled => {
                if let Invocation::Cleanup {
                    command: _,
                    leftover,
                } = invocation
                {
                    sink.send(Event::Output {
                        step,
                        data: format!("The cleanup failed too: {leftover}.\r\n"),
                    });
                    left.push(*leftover);
                }

                failed = failed.or(Some(outcome));
            }
        }
    }

    with_leftovers(failed.unwrap_or(result), left)
}

/// A step whose cleanup failed reports it as failed with what was left,
/// also after a cancel: the next repair refuses until it is gone.
fn with_leftovers(outcome: Outcome, left: Vec<Leftover>) -> Outcome {
    if left.is_empty() {
        return outcome;
    }

    match outcome {
        Outcome::Failed {
            cause,
            mut cleanup_failed,
        } => {
            cleanup_failed.extend(left);
            Outcome::Failed {
                cause,
                cleanup_failed,
            }
        }
        Outcome::Succeeded | Outcome::Skipped | Outcome::Cancelled => Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: left,
        },
    }
}

/// Runs `invocation` and turns a panic into a failed outcome, so a bug in
/// one command still leaves the step's [`Invocation::Always`] part and the
/// final [`Event::Done`] to come. A panicking cleanup fails alone, the
/// cleanups after it still run.
fn guarded(
    step: StepId,
    invocation: &Invocation,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    // What a panic left half done in the runner or the sink is not relied
    // on: after it only cleanup commands start, each from scratch.
    let caught = panic::catch_unwind(AssertUnwindSafe(|| {
        run_once(step, invocation, &mut *runner, &mut *sink)
    }));

    caught.unwrap_or_else(|_panic| {
        failure(
            step,
            sink,
            "this step stopped unexpectedly, which is an error in Garuda Toolbox itself",
        )
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "one arm per invocation, a new one fails to compile until it has its executor"
)]
fn run_once(
    step: StepId,
    invocation: &Invocation,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    match invocation {
        Invocation::Always(command)
        | Invocation::Cleanup {
            command,
            leftover: _,
        } => runner.run_always(step, command, sink),
        Invocation::Run(command) => runner.run(step, command, sink),
        Invocation::RunWithTargetsFrom { query, run } => {
            targets::repeated(step, query, run, runner, sink)
        }
        Invocation::RunWithOrphansFrom { query, run } => {
            targets::orphans(step, query, run, runner, sink)
        }
        Invocation::RunWithUnmetFrom { test, run } => targets::unmet(step, test, run, runner, sink),
        Invocation::RunUnlessItRemovesProtected { preview, run } => {
            targets::unless_protected(step, preview, run, runner, sink)
        }
        Invocation::Builtin(Builtin::RebuildInitramfs { root }) => rebuilt(
            step,
            initramfs::rebuild(Path::new(SYSTEM_ROOT), *root),
            "neither dracut-rebuild nor mkinitcpio is installed, so the initramfs was not built again",
            runner,
            sink,
        ),
        Invocation::Builtin(Builtin::RebuildUkis { root }) => rebuilt(
            step,
            uki::rebuild(Path::new(SYSTEM_ROOT), *root),
            "neither kernel-install-uki nor kernel-install is installed, so the UKIs were not built again",
            runner,
            sink,
        ),
        Invocation::Builtin(Builtin::SnapshotBeforeMerges) => {
            match merge::snapshot(Path::new(SYSTEM_ROOT)) {
                Some(command) => runner.run(step, &command, sink),
                None => note(
                    step,
                    sink,
                    "snapper has no configuration of /, so the previous versions are kept in the backup folder only",
                    Outcome::Skipped,
                ),
            }
        }
        Invocation::Builtin(Builtin::RepairBootLoader { system, work }) => {
            match repair::boot_loader_steps(Path::new(SYSTEM_ROOT), system, work) {
                Ok(steps) => run_procedure(step, &steps, runner, sink),
                Err(error) => failure(step, sink, &error.to_string()),
            }
        }
        Invocation::Builtin(builtin) => builtin_step(step, builtin, runner, sink),
        Invocation::Feed {
            command,
            input: Some(input),
        } => runner.feed(step, command, input, sink),
        Invocation::Feed {
            command,
            input: None,
        } => failure(
            step,
            sink,
            &format!("the secret for {} did not arrive", command.program().name()),
        ),
    }
}

fn builtin_step(
    step: StepId,
    builtin: &Builtin,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    if let Builtin::ResolveMerge(resolve) = builtin {
        return merge::execute(
            Path::new(SYSTEM_ROOT),
            step,
            resolve,
            sink,
            |command, text, sink| runner.check(step, command, text, sink),
        );
    }

    let Builtin::InstallBuilt {
        run,
        packages,
        reason,
    } = builtin
    else {
        return runner.builtin(step, builtin, sink);
    };

    let built = aur::Install {
        run,
        packages,
        reason: *reason,
    };

    aur::install(Path::new(SYSTEM_ROOT), runner.caller(), built, |command| {
        runner.run(step, command, sink)
    })
    .unwrap_or_else(|error| failure(step, sink, &error.to_string()))
}

fn rebuilt(
    step: StepId,
    command: Option<Command>,
    missing: &str,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    match command {
        Some(command) => runner.run(step, &command, sink),
        None => failure(step, sink, missing),
    }
}
