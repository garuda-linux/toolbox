use super::Budget;

#[test]
fn past_the_limit_the_picture_drawn_longest_ago_leaves() {
    let mut budget = Budget::new(2);

    assert_eq!(budget.drawn("a"), None);
    assert_eq!(budget.drawn("b"), None);
    assert_eq!(budget.drawn("c").as_deref(), Some("a"));
}

#[test]
fn a_picture_drawn_again_counts_as_new() {
    let mut budget = Budget::new(2);
    assert_eq!(budget.drawn("a"), None);
    assert_eq!(budget.drawn("b"), None);

    assert_eq!(budget.drawn("a"), None, "on screen again");
    assert_eq!(budget.drawn("c").as_deref(), Some("b"));
}

#[test]
fn every_frame_draws_the_same_rows_without_letting_any_go() {
    let mut budget = Budget::new(3);
    for _ in 0..10_u8 {
        for uri in ["a", "b", "c"] {
            assert_eq!(budget.drawn(uri), None);
        }
    }
}
