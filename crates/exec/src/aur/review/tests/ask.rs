use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::protocol::{Answer, Outcome};

use super::support::{
    Fetch, OCTOPI, QUESTION, Then, case, commit, enter, enter_at, git, head, review, reviewed,
    shown, yes_at,
};

#[test]
fn a_yes_marks_the_shown_commit_reviewed() {
    let case = case(Fetch::Copy);

    let (outcome, session) = review(&case, &[OCTOPI], vec![yes_at(QUESTION)], None);

    assert_eq!(outcome, Outcome::Succeeded);
    assert!(shown(&session).contains("octopi-git is marked reviewed"));
    assert_eq!(reviewed(&case, OCTOPI), Some(head(&case, OCTOPI)));
}

#[test]
fn a_commit_landing_after_the_show_is_not_marked_reviewed() {
    let case = case(Fetch::Copy);
    let shown_commit = git(&case.sources.join(OCTOPI), &["rev-parse", "HEAD"]);
    let clone = case.clone_of(OCTOPI);
    let landing = Then::Act(Box::new(move || {
        commit(&clone, &[("PKGBUILD", "pkgname=octopi-git\npkgver=2\n")]);
    }));

    let (outcome, _) = review(
        &case,
        &[OCTOPI],
        vec![(QUESTION, landing), yes_at(QUESTION)],
        None,
    );

    assert_eq!(outcome, Outcome::Succeeded);
    assert_eq!(reviewed(&case, OCTOPI), Some(shown_commit));
}

/// The answer that declines, by the name of its case. `stop` is the flag
/// the runner reads, which the `stop` case sets when the question shows.
fn declining(name: &str, stop: &Arc<AtomicBool>) -> Then {
    match name {
        "enter" => Then::Send(enter()),
        "n" => Then::Send(Answer::Input {
            data: "n\r".to_owned(),
        }),
        "cancel" => Then::Send(Answer::Cancel),
        "finish" => Then::Send(Answer::Finish),
        "closed" => Then::Drop,
        _ => Then::Stop(Arc::clone(stop)),
    }
}

#[test]
fn every_answer_but_y_declines_and_marks_nothing() {
    for name in ["enter", "n", "cancel", "finish", "closed", "stop"] {
        let case = case(Fetch::Copy);
        let stop = Arc::new(AtomicBool::new(false));
        let then = declining(name, &stop);

        let (outcome, _) = review(&case, &[OCTOPI], vec![(QUESTION, then)], Some(stop));

        assert_eq!(outcome, Outcome::Cancelled, "{name}");
        assert_eq!(reviewed(&case, OCTOPI), None, "{name}");
    }
}

#[test]
fn a_y_typed_while_the_base_is_shown_is_dropped() {
    let case = case(Fetch::Copy);
    let typed_yes = Answer::Input {
        data: "y\r".to_owned(),
    };

    let (outcome, _) = review(
        &case,
        &[OCTOPI],
        vec![
            ("all files, the first review", Then::Send(typed_yes)),
            enter_at(QUESTION),
        ],
        None,
    );

    assert_eq!(outcome, Outcome::Cancelled);
    assert_eq!(reviewed(&case, OCTOPI), None);
}
