use std::collections::VecDeque;

use super::{Consent, Prompt, consent};
use crate::exit::Exit;

struct Scripted {
    terminal: bool,
    answers: VecDeque<&'static str>,
    asked: Vec<String>,
}

impl Scripted {
    fn new(terminal: bool, answers: &[&'static str]) -> Self {
        Self {
            terminal,
            answers: answers.iter().copied().collect(),
            asked: Vec::new(),
        }
    }
}

impl Prompt for Scripted {
    fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn ask(&mut self, question: &str) -> Option<String> {
        self.asked.push(question.to_owned());
        self.answers.pop_front().map(str::to_owned)
    }
}

#[test]
fn yes_carries_the_change_out_without_a_question() {
    let mut prompt = Scripted::new(false, &[]);

    assert_eq!(consent(&mut prompt, true, None), Consent::Given);
    assert_eq!(prompt.asked, Vec::<String>::new());
}

#[test]
fn without_a_terminal_and_without_yes_nothing_changes() {
    let mut prompt = Scripted::new(false, &["y"]);

    assert_eq!(consent(&mut prompt, false, None), Consent::NeedsYes);
    assert_eq!(prompt.asked, Vec::<String>::new());
}

#[test]
fn only_y_or_yes_goes_on() {
    for (answer, expected) in [
        ("y", Consent::Given),
        ("Yes", Consent::Given),
        (" y ", Consent::Given),
        ("", Consent::Declined),
        ("n", Consent::Declined),
        ("yep", Consent::Declined),
    ] {
        let mut prompt = Scripted::new(true, &[answer]);
        assert_eq!(consent(&mut prompt, false, None), expected, "{answer:?}");
    }
}

#[test]
fn a_closed_input_declines() {
    let mut prompt = Scripted::new(true, &[]);

    assert_eq!(consent(&mut prompt, false, None), Consent::Declined);
}

#[test]
fn a_destructive_change_wants_its_name_typed() {
    let mut prompt = Scripted::new(true, &["y"]);
    assert_eq!(
        consent(&mut prompt, false, Some("full-reset")),
        Consent::Declined
    );
    assert!(prompt.asked[0].contains("full-reset"));

    let mut typed = Scripted::new(true, &["full-reset"]);
    assert_eq!(
        consent(&mut typed, false, Some("full-reset")),
        Consent::Given
    );
}

#[test]
fn yes_also_covers_a_destructive_change() {
    let mut prompt = Scripted::new(false, &[]);

    assert_eq!(
        consent(&mut prompt, true, Some("full-reset")),
        Consent::Given
    );
}

#[test]
fn each_refusal_has_its_exit_code() {
    assert_eq!(Consent::Declined.exit(), Some(Exit::Declined));
    assert_eq!(Consent::NeedsYes.exit(), Some(Exit::NeedsYes));
    assert_eq!(Consent::Given.exit(), None);
}
