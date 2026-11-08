use egui::accesskit::Role;
use egui::{Id, Key};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{Asked, ButtonKind, DialogWidth, Question, question};
use crate::testing::harness;

/// What the question under test answered, and whether it is still asked.
#[derive(Debug)]
struct Asking {
    open: bool,
    answer: Asked,
}

fn asking() -> Harness<'static, Asking> {
    let state = Asking {
        open: true,
        answer: Asked::default(),
    };

    let mut harness = harness(800.0, 600.0).build_ui_state(
        |ui, asking: &mut Asking| {
            if !asking.open {
                return;
            }

            let texts = Question {
                title: "Remove all changes",
                close: "Close",
                body: "Remove all 3 queued changes?",
                cancel: "Keep",
                confirm: "Remove all",
                kind: ButtonKind::Danger,
                width: DialogWidth::Small,
            };

            let answer = question(ui.ctx(), Id::unique("question"), texts);
            if answer.done {
                asking.answer = answer;
                asking.open = false;
            }
        },
        state,
    );

    harness.run();
    harness
}

#[test]
fn the_question_shows_its_title_and_body() {
    let harness = asking();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Remove all changes")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Remove all 3 queued changes?")
            .is_some()
    );
}

#[test]
fn confirm_confirms_and_ends_the_question() {
    let mut harness = asking();
    harness
        .get_by_role_and_label(Role::Button, "Remove all")
        .click();
    harness.run();

    assert_eq!(
        harness.state().answer,
        Asked {
            confirmed: true,
            done: true
        }
    );
}

#[test]
fn cancel_ends_the_question_without_confirming() {
    let mut harness = asking();
    harness.get_by_role_and_label(Role::Button, "Keep").click();
    harness.run();

    assert_eq!(
        harness.state().answer,
        Asked {
            confirmed: false,
            done: true
        }
    );
}

#[test]
fn escape_ends_the_question_without_confirming() {
    let mut harness = asking();
    harness.key_press(Key::Escape);
    harness.run();

    assert!(!harness.state().open);
    assert!(!harness.state().answer.confirmed);
}

#[test]
fn an_unanswered_question_stays() {
    let mut harness = asking();
    harness.run();

    assert!(harness.state().open);
}
