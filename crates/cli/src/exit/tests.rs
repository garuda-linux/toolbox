use std::collections::BTreeSet;

use super::Exit;

#[test]
fn the_codes_scripts_rely_on_never_change() {
    let codes: Vec<(Exit, u8)> = Exit::ALL.iter().map(|exit| (*exit, exit.code())).collect();

    assert_eq!(
        codes,
        [
            (Exit::Success, 0),
            (Exit::Failure, 1),
            (Exit::Usage, 2),
            (Exit::Declined, 3),
            (Exit::NeedsYes, 4),
            (Exit::Unavailable, 5),
            (Exit::Refused, 6),
            (Exit::PageRule, 8),
            (Exit::Attention, 9),
            (Exit::HungUp, 129),
            (Exit::Interrupted, 130),
            (Exit::Terminated, 143),
        ]
    );
}

/// Exit 7 (nothing to do) now ends with 0, and 10 (cancelled) with the signal's code.
/// Neither number is given again, so no script meets a new meaning for them.
#[test]
fn the_retired_codes_are_given_no_more() {
    assert!(Exit::ALL.iter().all(|exit| ![7, 10].contains(&exit.code())));
}

#[test]
fn a_signal_ends_with_128_and_its_number() {
    assert_eq!(Exit::HungUp.code(), 128 + 1);
    assert_eq!(Exit::Interrupted.code(), 128 + 2);
    assert_eq!(Exit::Terminated.code(), 128 + 15);
}

#[test]
fn the_table_lists_one_code_per_line() {
    let table = Exit::table();

    assert!(
        table.starts_with("Exit codes:\n  0    success      done\n  1    failure      "),
        "{table}"
    );
    assert!(
        table.ends_with("\n  143  terminated   SIGTERM stopped it, as Ctrl+C does"),
        "{table}"
    );
}

#[test]
fn every_meaning_is_told_apart() {
    let meanings: BTreeSet<&str> = Exit::ALL.iter().map(|exit| exit.meaning()).collect();

    assert_eq!(meanings.len(), Exit::ALL.len());
}

/// A script that matches the reason reads the same word in every
/// version, as with the code.
#[test]
fn every_code_has_a_stable_reason() {
    let reasons: Vec<(u8, &str)> = Exit::ALL
        .iter()
        .map(|exit| (exit.code(), exit.reason()))
        .collect();

    assert_eq!(
        reasons,
        [
            (0, "success"),
            (1, "failure"),
            (2, "usage"),
            (3, "declined"),
            (4, "needs-yes"),
            (5, "unavailable"),
            (6, "denied"),
            (8, "page-rule"),
            (9, "attention"),
            (129, "hung-up"),
            (130, "interrupted"),
            (143, "terminated"),
        ]
    );
}

#[test]
fn the_reasons_are_kebab_case_and_told_apart() {
    let reasons: BTreeSet<&str> = Exit::ALL.iter().map(|exit| exit.reason()).collect();

    assert_eq!(reasons.len(), Exit::ALL.len());
    assert!(reasons.iter().all(|reason| {
        !reason.starts_with('-')
            && !reason.ends_with('-')
            && reason
                .chars()
                .all(|letter| letter.is_ascii_lowercase() || letter == '-')
    }));
}
