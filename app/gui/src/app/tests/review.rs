//! The Console's Continue and Cancel while a build waits for its review.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_plan::StepId;

use crate::app::{App, Setup};
use crate::apply::{PlanId, Report};
use crate::page::PageId;

const REVIEW: &str = "paru -G -- octopi-git in ~/.cache/garuda-toolbox/aur-clone, then show the build files and ask before any build";

/// An app on Home with the Console open and a plan of `steps` started.
fn running(steps: &[&str]) -> Harness<'static, App> {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();

    let app = harness.state_mut();
    app.state.page = PageId::Home;
    app.state.console_open = true;
    let listed = steps
        .iter()
        .enumerate()
        .map(|(number, line)| {
            let id = StepId::new(u32::try_from(number + 1).unwrap());
            (id, vec![(*line).to_owned()])
        })
        .collect();

    app.console.take(Report::PlanStarted {
        plan: PlanId::new(1),
        steps: listed,
    });
    app.console
        .take(Report::Step(toolbox_plan::protocol::Event::Started {
            step: StepId::new(1),
        }));
    harness.run();
    harness
}

#[test]
fn continue_shows_only_while_a_review_waits() {
    let harness = running(&[REVIEW]);
    assert!(harness.query_by_label("Continue").is_some());

    let other = running(&["pacman-key --refresh-keys"]);
    assert!(other.query_by_label("Continue").is_none());
    assert!(other.query_by_label("Cancel").is_some());
}

#[test]
fn continue_never_takes_the_focus_so_enter_cannot_approve_a_build() {
    let harness = running(&[REVIEW]);

    let continue_button = harness.query_by_label("Continue").expect("Continue");
    assert!(!continue_button.accesskit_node().is_focused());
}

#[test]
fn review_buttons_stay_while_the_review_is_the_running_step() {
    let mut harness = running(&[REVIEW]);
    harness
        .state_mut()
        .console
        .take(Report::Step(toolbox_plan::protocol::Event::Finished {
            step: StepId::new(1),
            outcome: toolbox_plan::protocol::Outcome::Succeeded,
        }));
    harness.run();

    assert!(harness.query_by_label("Continue").is_none());
}
