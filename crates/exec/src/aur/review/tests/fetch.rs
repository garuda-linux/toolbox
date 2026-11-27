use std::fs;

use toolbox_plan::AUR_CLONE;
use toolbox_plan::protocol::Outcome;

use super::support::{
    Fetch, OCTOPI, QUESTION, asked, case, enter_at, review, review_on, reviewed, shown,
};
use crate::test_support::mode;

#[test]
fn the_fetch_runs_in_the_clone_with_dash_g_and_the_bases() {
    let case = case(Fetch::Copy);

    let (outcome, _) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    let clone = case.home.join(AUR_CLONE);
    assert!(case.log.starts_with(case.root.path()));
    let log = fs::read_to_string(&case.log).unwrap();
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        [clone.to_str().unwrap(), "-G", "--", OCTOPI]
    );
    assert_eq!(mode(&clone), 0o700);
    assert_eq!(outcome, Outcome::Cancelled);
    assert_eq!(reviewed(&case, OCTOPI), None);
}

#[test]
fn a_fetch_that_leaves_no_usable_folder_fails_by_name_and_asks_nothing() {
    let cases = [
        (Fetch::Nothing, "octopi-git: paru left no folder"),
        (Fetch::Link, "octopi-git: its folder in"),
        (Fetch::Folder, "octopi-git: its folder has no PKGBUILD"),
        (
            Fetch::PkgbuildOnly,
            "octopi-git: its folder is no git repository",
        ),
    ];

    for (how, reason) in cases {
        let case = case(how);

        let (outcome, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

        assert!(
            matches!(
                outcome,
                Outcome::Failed {
                    cause: _,
                    cleanup_failed: _
                }
            ),
            "{reason}"
        );
        assert!(
            shown(&session).contains(reason),
            "{reason}: {}",
            shown(&session)
        );
        assert!(!asked(&session), "{reason}");
    }
}

#[test]
fn a_failed_fetch_fails_the_step_and_asks_nothing() {
    let case = case(Fetch::Fail);

    let (outcome, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(matches!(
        outcome,
        Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }
    ));
    assert!(!asked(&session));
}

#[test]
fn paru_lines_reach_the_console_without_escapes_or_bidi_overrides() {
    let case = case(Fetch::Noisy);

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    let text = shown(&session);
    assert!(text.contains("downloading: octopi-git"), "{text:?}");
    assert!(!text.contains('\u{1b}'), "{text:?}");
    assert!(!text.contains('\u{202e}'), "{text:?}");
}

#[test]
fn a_fetch_printing_more_than_the_cap_fails_and_asks_nothing() {
    let case = case(Fetch::Flood);

    let (outcome, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(matches!(
        outcome,
        Outcome::Failed {
            cause: _,
            cleanup_failed: _
        }
    ));
    assert!(shown(&session).contains("paru printed more than"));
    assert!(!asked(&session));
}

#[test]
fn cancel_during_the_fetch_stops_paru() {
    let case = case(Fetch::Sleep);
    let (tx, rx) = std::sync::mpsc::channel();
    let canceller = crate::test_support::answer_after(
        tx.clone(),
        std::time::Duration::from_millis(300),
        toolbox_plan::protocol::Answer::Cancel,
    );

    let started = std::time::Instant::now();

    let (outcome, session) = review_on(&case, &[OCTOPI], vec![], None, (tx, rx));
    canceller.join().unwrap();

    assert!(started.elapsed() < std::time::Duration::from_secs(20));
    assert_eq!(outcome, Outcome::Cancelled);
    assert!(!asked(&session));
}
