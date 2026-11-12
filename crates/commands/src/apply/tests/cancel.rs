use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_plan::protocol::{Event, Outcome, encode};
use toolbox_plan::{Plan, PlannedAction, RootAction, StepId};

use super::{Script, plan, run, terminal};
use crate::apply::carry_out;
use crate::interrupt;
use crate::out::{Out, Settings};

#[test]
fn a_cancelled_step_ends_the_command_as_cancelled() {
    let (exit, out) = run(plan(), true, &mut terminal(&[]), Outcome::Cancelled);

    assert_eq!(exit, Exit::Interrupted);
    let last = out.lines().last().unwrap_or_default();
    assert!(
        last.starts_with("Cancelled  (") && last.ends_with(" s)"),
        "{out}"
    );
}

fn lines(events: &[Event]) -> String {
    events.iter().map(|event| encode(event).unwrap()).collect()
}

/// A helper stand-in that waits for the first answer and acts as the real
/// one: a finish lets the first step end and cancels the second unrun.
fn finishing_helper() -> Arc<Script> {
    let (first, second) = (StepId::new(1), StepId::new(2));
    let start = lines(&[Event::Accepted, Event::Started { step: first }]);
    let finished = lines(&[
        Event::Finished {
            step: first,
            outcome: Outcome::Succeeded,
        },
        Event::Started { step: second },
        Event::Finished {
            step: second,
            outcome: Outcome::Cancelled,
        },
        Event::Done,
    ]);

    let failed = lines(&[
        Event::Finished {
            step: first,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ]);

    Arc::new(Script(format!(
        "read request; printf '%s' '{start}'; read answer; case \"$answer\" in *finish*) printf '%s' '{finished}' ;; *) printf '%s' '{failed}' ;; esac"
    )))
}

/// Ctrl+C until the command ends: the first press is the one that counts,
/// and a press before the run started would be missed.
#[test]
fn the_first_ctrl_c_lets_the_step_end_and_skips_the_rest() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let presses = interrupt::listen(runtime.handle());
    let ended = Arc::new(AtomicBool::new(false));

    let pressing = {
        let ended = Arc::clone(&ended);
        thread::Builder::new()
            .name("test-ctrl-c".to_owned())
            .spawn(move || {
                while !ended.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(150));
                    presses.press();
                }
            })
            .unwrap()
    };

    let plan = Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::CleanPackageCache),
    ]);

    let mut out = Vec::new();
    let exit = carry_out(
        plan,
        Confirm {
            yes: true,
            dry_run: false,
            json: false,
        },
        None,
        runtime.handle(),
        finishing_helper(),
        &mut terminal(&[]),
        &mut Out::new(&mut out, Settings::default()),
    );

    ended.store(true, Ordering::Relaxed);
    pressing.join().unwrap();
    let out = String::from_utf8(out).unwrap();

    assert_eq!(exit, Exit::Interrupted, "{out}");
    assert!(
        out.contains("warning: stopping after this step, press Ctrl+C again to stop it now"),
        "{out}"
    );
    assert!(out.contains("paccache -ruk0 was cancelled"), "{out}");
}
