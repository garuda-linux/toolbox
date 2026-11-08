use super::{KEEP, remember};

#[test]
fn the_newest_comes_first() {
    let mut recent = vec![2_u8, 1];
    remember(&mut recent, 3);

    assert_eq!(recent, [3, 2, 1]);
}

#[test]
fn a_second_visit_moves_it_to_the_front() {
    let mut recent = vec![3_u8, 2, 1];
    remember(&mut recent, 1);

    assert_eq!(recent, [1, 3, 2]);
}

#[test]
fn only_the_last_few_are_kept() {
    let mut recent = Vec::new();
    for visit in 0_u8..10 {
        remember(&mut recent, visit);
    }

    assert_eq!(recent.len(), KEEP);
    assert_eq!(recent, [9, 8, 7]);
}
