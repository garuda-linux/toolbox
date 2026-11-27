use toolbox_plan::protocol::Outcome;

use super::support::{
    Fetch, OCTOPI, QUESTION, asked, case, case_with, commit, enter_at, review, reviewed, shown,
    yes_at,
};
use crate::test_support::assert_failed;

#[test]
fn the_first_review_shows_the_whole_pkgbuild_under_its_heading() {
    let case = case(Fetch::Copy);

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(shown(&session).contains("octopi-git: all files, the first review"));
    assert!(shown(&session).contains("+pkgname=octopi-git"));
    assert!(shown(&session).contains("+pkgver=1"));
}

#[test]
fn a_second_review_shows_only_the_diff_since_the_yes() {
    let case = case(Fetch::Copy);
    review(&case, &[OCTOPI], vec![yes_at(QUESTION)], None);
    commit(
        &case.sources.join(OCTOPI),
        &[(
            "PKGBUILD",
            "pkgname=octopi-git\npkgver=1\necho added-line\n",
        )],
    );

    let (outcome, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(shown(&session).contains("octopi-git: changes since the last review"));
    assert!(shown(&session).contains("+echo added-line"));
    assert!(!shown(&session).contains("+pkgname=octopi-git"));
    assert_eq!(outcome, Outcome::Cancelled);
}

#[test]
fn an_unchanged_base_says_no_change_and_is_still_asked() {
    let case = case(Fetch::Copy);
    review(&case, &[OCTOPI], vec![yes_at(QUESTION)], None);

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(shown(&session).contains("octopi-git: no change since the last review"));
    assert!(asked(&session));
}

#[test]
fn escape_sequences_and_bidi_overrides_do_not_reach_the_console() {
    let case = case_with(
        Fetch::Copy,
        "pkgname=octopi-git\n# \x1b[31mred\x1b[0m \u{202e}evil\n",
    );

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(shown(&session).contains("red"));
    assert!(!shown(&session).contains('\u{1b}'));
    assert!(!shown(&session).contains('\u{202e}'));
}

#[test]
fn a_binary_attribute_cannot_hide_the_pkgbuild_text() {
    let case = case(Fetch::Copy);
    commit(
        &case.sources.join(OCTOPI),
        &[(".gitattributes", "PKGBUILD -diff\n")],
    );

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(shown(&session).contains("+pkgver=1"));
    assert!(!shown(&session).contains("Binary files"));
}

#[test]
fn a_diff_over_the_cap_fails_and_asks_nothing() {
    let big = "echo line\n".repeat(30_000);
    let case = case_with(Fetch::Copy, &big);

    let (outcome, session) = review(&case, &[OCTOPI], vec![yes_at(QUESTION)], None);

    assert_failed(outcome);
    assert!(shown(&session).contains("octopi-git: the changes are too large"));
    assert!(!asked(&session));
    assert_eq!(reviewed(&case, OCTOPI), None);
}
