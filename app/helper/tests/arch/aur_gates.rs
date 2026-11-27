//! The gates of a request with the real helper and real pacman: a step behind
//! a gate waits for its `Proceed`, and a cancel or a closed stdin at the gate
//! runs nothing after it.

use std::error::Error;

use toolbox_plan::protocol::{Answer, Event, Outcome, PROTOCOL_VERSION, encode};
use toolbox_plan::{RootAction, StepId};

use super::aur::piped;
use super::{in_test_image, installed, packages, reset};
use crate::common::{Run, TestResult, events, outcomes};

/// The request of `actions` as steps `1..`, with a gate before each step
/// number in `gates`.
fn gated(actions: &[RootAction], gates: &[u32]) -> Result<String, Box<dyn Error>> {
    let steps = (1_u32..)
        .zip(actions)
        .map(|(id, action)| {
            Ok::<_, Box<dyn Error>>(format!(
                "{{\"id\":{id},\"action\":{}}}",
                encode(action)?.trim_end()
            ))
        })
        .collect::<Result<Vec<String>, _>>()?;

    let gates = gates
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");

    Ok(format!(
        "{{\"version\":{PROTOCOL_VERSION},\"mode\":\"run\",\"steps\":[{}],\"gates\":[{gates}]}}\n",
        steps.join(",")
    ))
}

/// Installs `toolbox-test-a`, removes it behind a gate on step 2, then
/// cleans the cache as step 3.
fn install_then_gated_remove() -> Result<Vec<RootAction>, Box<dyn Error>> {
    Ok(vec![
        RootAction::InstallPackages {
            packages: packages(&["toolbox-test-a"])?,
        },
        RootAction::RemovePackages {
            packages: packages(&["toolbox-test-a"])?,
        },
        RootAction::CleanPackageCache,
    ])
}

fn started(run: &Run, step: u32) -> Result<bool, Box<dyn Error>> {
    Ok(events(run)?.contains(&Event::Started {
        step: StepId::new(step),
    }))
}

#[test]
fn a_step_behind_a_gate_runs_after_the_app_proceeds() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    let text = gated(&install_then_gated_remove()?, &[2])?;
    let run = piped(&text, |step| Some(Answer::Proceed { step }))?;

    let all = events(&run)?;
    let at = |wanted: Event| all.iter().position(|event| *event == wanted);
    let waiting = at(Event::Waiting {
        step: StepId::new(2),
    });

    let second = at(Event::Started {
        step: StepId::new(2),
    });

    assert!(
        matches!((waiting, second), (Some(wait), Some(begun)) if wait < begun),
        "{}",
        run.stdout
    );
    assert_eq!(
        outcomes(&run)?,
        [Outcome::Succeeded, Outcome::Succeeded, Outcome::Succeeded]
    );
    assert!(!installed("toolbox-test-a"));
    Ok(())
}

#[test]
fn a_cancel_at_a_gate_runs_nothing_after_it() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    let text = gated(&install_then_gated_remove()?, &[2])?;
    let run = piped(&text, |_step| Some(Answer::Cancel))?;

    assert_eq!(outcomes(&run)?, [Outcome::Succeeded, Outcome::Cancelled]);
    assert!(!started(&run, 3)?, "{}", run.stdout);
    assert!(installed("toolbox-test-a"));
    Ok(())
}

/// A closed stdin is what an app that went away leaves behind: the same
/// nothing runs after the gate.
#[test]
fn a_closed_stdin_at_a_gate_runs_nothing_after_it() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    reset()?;
    let text = gated(&install_then_gated_remove()?, &[2])?;
    let run = piped(&text, |_step| None)?;

    assert!(!started(&run, 3)?, "{}", run.stdout);
    assert!(installed("toolbox-test-a"));
    Ok(())
}
