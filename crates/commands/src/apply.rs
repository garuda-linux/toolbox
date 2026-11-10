//! A change from the command line: the plan's steps, the question, then
//! the same apply service the window's Console uses, its output streamed
//! to the terminal.

#[cfg(test)]
mod tests;

mod elapsed;
mod events;
mod gate;
mod planned;
mod show;
mod typing;

use std::io::{IsTerminal as _, Write as _};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::RecvTimeoutError;
use tokio::runtime::Handle;
use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::confirm::{Prompt, consent};
use toolbox_cli::output::{ColorMode, Paint};
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{Plan, StepId};
use toolbox_services::apply::{self, ApplyCommand, ApplyHandle, Launcher, Problem, Report};
use toolbox_services::waking::Wake;
use toolbox_system::pkexec::Refusal;

use self::events::Events;
pub use self::events::{error as error_line, nothing as nothing_lines};
use self::gate::Gate;
pub(super) use self::planned::planned;
use self::planned::planned_out;
use self::show::Show;
use super::interrupt::{self, Stopper};
use crate::out::{Out, Settings};

const LOOK_AGAIN: Duration = Duration::from_millis(100);

struct Running {
    service: ApplyHandle,
    gate: Gate,
}

struct Attempt<'a> {
    runtime: &'a Handle,
    launcher: Arc<dyn Launcher>,
    /// A pkexec that could not ask stays unprinted, as the caller tries
    /// another way.
    quiet: bool,
    json: bool,
}

enum Look {
    Text(Show),
    /// With the paint of notes on standard error.
    Json(Events, Paint),
}

impl Look {
    fn report(&mut self, report: &Report) -> String {
        match self {
            Self::Text(show) => show.report(report),
            Self::Json(events, _) => events.report(report),
        }
    }

    /// A note of the command line. JSON Lines keep it on standard error.
    fn note(&mut self, text: &str) -> String {
        match self {
            Self::Text(show) => show.note(text),
            Self::Json(_, warned) => {
                writeln!(std::io::stderr(), "{}", warned.warning(text)).ok();
                String::new()
            }
        }
    }
}

/// How a run ended, and the steps that succeeded on the way.
pub struct Ended {
    pub exit: Exit,
    pub asked_not: bool,
    pub succeeded: Vec<StepId>,
    /// Steps that found nothing to do, which ended well too.
    pub skipped: Vec<StepId>,
}

impl Ended {
    const fn early(exit: Exit) -> Self {
        Self {
            exit,
            asked_not: false,
            succeeded: Vec::new(),
            skipped: Vec::new(),
        }
    }

    pub fn ended_well(&self, step: StepId) -> bool {
        self.succeeded.contains(&step) || self.skipped.contains(&step)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the ports and the terminal, each a seam the tests use"
)]
pub fn carry_out(
    plan: Plan,
    confirm: Confirm,
    typed: Option<&str>,
    runtime: &Handle,
    launcher: Arc<dyn Launcher>,
    prompt: &mut dyn Prompt,
    out: &mut Out<'_>,
) -> Exit {
    carry_out_steps(plan, confirm, typed, runtime, launcher, prompt, out).exit
}

/// [`carry_out`], also naming the steps that succeeded, so the queue
/// settles what ran.
#[expect(
    clippy::too_many_arguments,
    reason = "the ports and the terminal, each a seam the tests use"
)]
pub fn carry_out_steps(
    plan: Plan,
    confirm: Confirm,
    typed: Option<&str>,
    runtime: &Handle,
    launcher: Arc<dyn Launcher>,
    prompt: &mut dyn Prompt,
    out: &mut Out<'_>,
) -> Ended {
    if confirm.cannot_ask() {
        return Ended::early(cannot_ask(out.settings()));
    }

    let paint = out.paint();
    if let Some(exit) = planned_out(paint, &plan, confirm, out) {
        return Ended::early(exit);
    }

    if let Some(exit) = consent(prompt, confirm.yes, typed).exit() {
        if exit == Exit::NeedsYes {
            let error = paint.error("no terminal to ask on");
            let hint = paint.hint("pass --yes to carry it out");
            writeln!(out, "\n{error}\n{hint}").ok();
        }

        return Ended::early(exit);
    }

    let fallback = prompt.is_terminal().then(|| launcher.fallback()).flatten();
    let json = confirm.json;
    let first = Attempt {
        runtime,
        launcher,
        quiet: fallback.is_some(),
        json,
    };

    let first_run = run_once(plan.clone(), first, out);

    match fallback {
        Some(sudo) if first_run.asked_not => {
            if json {
                writeln!(
                    std::io::stderr(),
                    "{}",
                    warned(out.settings().color).warning(super::root::SUDO_NOTE)
                )
                .ok();
            } else {
                writeln!(out, "\n{}", paint.warning(super::root::SUDO_NOTE)).ok();
            }

            let sudo = Attempt {
                runtime,
                launcher: sudo,
                quiet: false,
                json,
            };

            // Steps of the first try ran too, before pkexec could not ask.
            let mut second_run = run_once(plan, sudo, out);
            second_run.succeeded.extend(first_run.succeeded);
            second_run.skipped.extend(first_run.skipped);
            second_run
        }
        Some(_) | None => first_run,
    }
}

/// `--json` without `--yes` or `--dry-run`: nothing on standard output, the
/// reason on standard error.
pub fn cannot_ask(settings: Settings) -> Exit {
    let paint = warned(settings.color);
    let error = paint.error("--json cannot ask before a change");
    let hint = paint.hint("pass --yes to carry it out, or --dry-run to see the plan");
    writeln!(std::io::stderr(), "{error}\n{hint}").ok();
    Exit::NeedsYes
}

fn warned(color: ColorMode) -> Paint {
    Paint::new(super::out::style_on(color, std::io::stderr().is_terminal()))
}

fn run_once(plan: Plan, attempt: Attempt<'_>, out: &mut Out<'_>) -> Ended {
    let Attempt {
        runtime,
        launcher,
        quiet,
        json,
    } = attempt;

    let look = if json {
        Look::Json(
            Events::new(Box::new(Instant::now)),
            warned(out.settings().color),
        )
    } else {
        let dry_run = launcher.mode() == Mode::DryRun;
        Look::Text(
            Show::new(
                out.paint().style(),
                Mode::of_dry_run(dry_run),
                Box::new(Instant::now),
            )
            .verbose(out.verbose()),
        )
    };

    let stopper = Stopper::new(interrupt::listen(runtime));

    // The service only asks the context for a repaint, a context without
    // a window takes that and does nothing.
    let run = Arc::new(Running {
        service: apply::spawn(runtime, Wake::none(), launcher),
        gate: Gate::default(),
    });

    // Under `--json` no step waits for typing, and standard input stays
    // the script's.
    let typing = if json {
        None
    } else {
        typing::forward_input(&run)
    };

    run.service.send(ApplyCommand::Run(plan));

    let ended = follow(&run, stopper, look, quiet, out);

    // The reader puts the terminal's echo back before the command ends.
    run.gate.note(&Report::PlanFinished { succeeded: false });
    if let Some(typing) = typing
        && typing.join().is_err()
    {
        tracing::warn!("the reader of typed answers stopped unexpectedly");
    }

    ended
}

/// Whether pkexec could not even ask: no agent, a broken pkexec, or none
/// at all. A closed dialog or a wrong password is an answer, not this.
const fn could_not_ask(problem: &Problem) -> bool {
    matches!(
        problem,
        Problem::Refused(Refusal::NoAgent | Refusal::Broken)
            | Problem::HelperFailed {
                kind: HelperFailure::Pkexec | HelperFailure::Start,
                detail: _,
            }
    )
}

fn follow(
    run: &Running,
    mut stopper: Stopper,
    mut look: Look,
    quiet: bool,
    out: &mut Out<'_>,
) -> Ended {
    let mut ended = Ended::early(Exit::Failure);
    let mut cancelled = false;

    loop {
        for press in stopper.take() {
            stop(run, press, &mut look, out);
        }

        let report = match run.service.wait_for(LOOK_AGAIN) {
            Ok(report) => report,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };

        run.gate.note(&report);

        if let Report::Problem(problem) = &report
            && could_not_ask(problem)
        {
            ended.asked_not = true;
            if quiet {
                continue;
            }
        }

        if quiet && ended.asked_not && matches!(report, Report::PlanFinished { succeeded: _ }) {
            break;
        }

        out.write_all(look.report(&report).as_bytes()).ok();
        out.flush().ok();

        match report {
            Report::Problem(Problem::Refused(_)) => ended.exit = Exit::Refused,
            Report::Step(Event::Finished {
                step: _,
                outcome: Outcome::Cancelled,
            }) => cancelled = true,
            Report::Step(Event::Finished {
                step,
                outcome: Outcome::Succeeded,
            }) => ended.succeeded.push(step),
            Report::Step(Event::Finished {
                step,
                outcome: Outcome::Skipped,
            }) => ended.skipped.push(step),
            Report::PlanFinished { succeeded } => {
                if ended.exit != Exit::Refused {
                    ended.exit = exit_of(succeeded, cancelled.then(|| stopper.exit()));
                }

                break;
            }
            Report::PlanStarted { steps: _ }
            | Report::Step(_)
            | Report::Problem(_)
            | Report::Busy
            | Report::CancelWaits => {}
        }
    }

    ended
}

fn stop(run: &Running, press: u8, look: &mut Look, out: &mut Out<'_>) {
    if let Some(command) = interrupt::command(press) {
        run.service.send(command);
    }

    if let Some(note) = interrupt::note(press) {
        out.write_all(look.note(note).as_bytes()).ok();
        out.flush().ok();
    }
}

const fn exit_of(succeeded: bool, stopped: Option<Exit>) -> Exit {
    match (succeeded, stopped) {
        (true, _) => Exit::Success,
        (false, Some(signal)) => signal,
        (false, None) => Exit::Failure,
    }
}
