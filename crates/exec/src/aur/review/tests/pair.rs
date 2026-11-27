use toolbox_plan::protocol::Outcome;

use super::support::{
    Case, Fetch, OCTOPI, PKGBUILD, QUESTION, case_with_bases, enter_at, head, review, reviewed,
    shown, yes_at,
};

const ALPHA: &str = "alpha-git";

/// Two bases, sorted by name: `alpha-git` shows before `octopi-git`.
pub fn pair(how: Fetch, octopi: &str) -> Case {
    case_with_bases(
        how,
        &[(ALPHA, "pkgname=alpha-git\npkgver=1\n"), (OCTOPI, octopi)],
    )
}

#[test]
fn two_bases_show_both_headings_in_order_under_one_question() {
    let case = pair(Fetch::Copy, PKGBUILD);

    let (_, session) = review(&case, &[ALPHA, OCTOPI], vec![enter_at(QUESTION)], None);

    let text = shown(&session);
    let first = text.find("alpha-git: all files, the first review");
    let second = text.find("octopi-git: all files, the first review");
    assert!(
        first.is_some_and(|first| second.is_some_and(|second| first < second)),
        "{text}"
    );
    assert!(text.contains("+pkgname=alpha-git"), "{text}");
    assert!(text.contains("+pkgname=octopi-git"), "{text}");
    assert_eq!(text.matches(QUESTION).count(), 1, "{text}");
}

#[test]
fn a_yes_marks_both_shown_commits() {
    let case = pair(Fetch::Copy, PKGBUILD);

    let (outcome, _) = review(&case, &[ALPHA, OCTOPI], vec![yes_at(QUESTION)], None);

    assert_eq!(outcome, Outcome::Succeeded);
    assert_eq!(reviewed(&case, ALPHA), Some(head(&case, ALPHA)));
    assert_eq!(reviewed(&case, OCTOPI), Some(head(&case, OCTOPI)));
}

#[test]
fn a_base_over_the_cap_among_two_fails_the_step_and_marks_neither() {
    let big = "echo line\n".repeat(30_000);
    let case = pair(Fetch::Copy, &big);

    let (outcome, session) = review(&case, &[ALPHA, OCTOPI], vec![yes_at(QUESTION)], None);

    crate::test_support::assert_failed(outcome);
    let text = shown(&session);
    assert!(
        text.contains("octopi-git: the changes are too large"),
        "{text}"
    );
    assert!(!text.contains(QUESTION), "{text}");
    assert_eq!(reviewed(&case, ALPHA), None);
    assert_eq!(reviewed(&case, OCTOPI), None);
}
