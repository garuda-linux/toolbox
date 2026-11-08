use super::{Fold, NARROW};

const WIDE: f32 = NARROW + 100.0;
const SLIM: f32 = NARROW - 100.0;

#[test]
fn a_window_that_turns_narrow_folds_the_sidebar_and_unfolds_it_when_wide_again() {
    let mut fold = Fold::default();

    assert!(fold.fit(WIDE, true));
    assert!(!fold.fit(SLIM, true));
    assert!(!fold.fit(SLIM, false), "it stays folded");
    assert!(fold.fit(WIDE, false), "unfolded, as it folded itself");
}

#[test]
fn a_narrow_start_folds_too() {
    assert!(!Fold::default().fit(SLIM, true));
}

#[test]
fn a_sidebar_the_user_folded_stays_folded_when_the_window_widens() {
    let mut fold = Fold::default();

    assert!(!fold.fit(WIDE, false));
    assert!(!fold.fit(SLIM, false));
    assert!(!fold.fit(WIDE, false));
}

#[test]
fn the_user_may_unfold_it_in_a_narrow_window() {
    let mut fold = Fold::default();
    assert!(!fold.fit(SLIM, true));

    assert!(fold.fit(SLIM, true), "the user unfolded it");
    assert!(fold.fit(WIDE, true));
    assert!(!fold.fit(SLIM, true), "narrow once more, it folds again");
}
