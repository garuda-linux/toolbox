//! How a single command runs is the [`Runner`]'s business, so this order of events is tested without root or real programs.
//! The helper runs root steps with it, the app user steps.

#[cfg(test)]
mod tests;

use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

use toolbox_plan::procedure::{Builtin, Command, Invocation};
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{PackageName, RootAction, SecretInput, Step, StepId, UserAction};

use crate::builtin::SYSTEM_ROOT;
use crate::initramfs;
use crate::report::failure;

pub trait Procedure {
    fn procedure(&self) -> Vec<Invocation>;
}

impl Procedure for RootAction {
    fn procedure(&self) -> Vec<Invocation> {
        Self::procedure(self)
    }
}

impl Procedure for UserAction {
    fn procedure(&self) -> Vec<Invocation> {
        Self::procedure(self)
    }
}

pub trait EventSink {
    fn send(&mut self, event: Event);
}

pub trait Runner {
    fn run(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome;

    /// Like [`Runner::run`], but also after a cancel, and a cancel while
    /// it runs does not stop it ([`Invocation::Always`]).
    fn run_always(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome;

    /// # Errors
    ///
    /// When the command cannot start or fails.
    fn query(&mut self, command: &Command) -> io::Result<Vec<String>>;

    fn builtin(&mut self, step: StepId, builtin: &Builtin, sink: &mut dyn EventSink) -> Outcome;

    /// Whatever the program prints is reported with the secret taken out.
    fn feed(
        &mut self,
        step: StepId,
        command: &Command,
        input: &SecretInput,
        sink: &mut dyn EventSink,
    ) -> Outcome;

    fn finish_requested(&mut self) -> bool;
}

pub fn apply<A>(
    mode: Mode,
    steps: &[Step<A>],
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
        sink.send(Event::Started { step: step.id });

        // Reported, not left out: a plan whose rest is missing would read
        // as finished.
        let outcome = match mode {
            Mode::DryRun => Outcome::Skipped,
            Mode::Run if runner.finish_requested() => Outcome::Cancelled,
            Mode::Run => run_procedure(step.id, &procedure, runner, sink),
        };

        sink.send(Event::Finished {
            step: step.id,
            outcome,
        });

        if !matches!(outcome, Outcome::Succeeded | Outcome::Skipped) {
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
            | Invocation::Run(command)
            | Invocation::RunWithTargetsFrom {
                query: _,
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

    for invocation in procedure {
        if failed.is_some() && !matches!(invocation, Invocation::Always(_)) {
            continue;
        }

        let outcome = guarded(step, invocation, runner, sink);

        match outcome {
            Outcome::Succeeded => result = Outcome::Succeeded,
            Outcome::Skipped => {}
            Outcome::Failed { cause: _ } | Outcome::Cancelled => {
                failed = failed.or(Some(outcome));
            }
        }
    }

    failed.unwrap_or(result)
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

fn run_once(
    step: StepId,
    invocation: &Invocation,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    match invocation {
        Invocation::Always(command) => runner.run_always(step, command, sink),
        Invocation::Run(command) => runner.run(step, command, sink),
        Invocation::RunWithTargetsFrom { query, run } => {
            with_targets(step, query, run, runner, sink)
        }
        Invocation::Builtin(Builtin::RebuildInitramfs { root }) => {
            match initramfs::rebuild(Path::new(SYSTEM_ROOT), *root) {
                Some(command) => runner.run(step, &command, sink),
                None => failure(
                    step,
                    sink,
                    "neither dracut-rebuild nor mkinitcpio is installed, so the initramfs was not built again",
                ),
            }
        }
        Invocation::Builtin(builtin) => runner.builtin(step, builtin, sink),
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

/// The query's lines become arguments, so each must pass as a package name:
/// a line like `--overwrite=*` would otherwise turn into an option.
fn with_targets(
    step: StepId,
    query: &Command,
    run: &Command,
    runner: &mut dyn Runner,
    sink: &mut dyn EventSink,
) -> Outcome {
    let lines = match runner.query(query) {
        Ok(lines) => lines,
        Err(error) => return failure(step, sink, &format!("{query} failed: {error}")),
    };

    if lines.is_empty() {
        return Outcome::Skipped;
    }

    let targets: Result<Vec<PackageName>, _> = lines.iter().map(PackageName::new).collect();

    match targets {
        Ok(targets) => runner.run(
            step,
            &run.clone().args(targets.iter().map(PackageName::as_str)),
            sink,
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("{query} printed something unexpected: {error}"),
        ),
    }
}
