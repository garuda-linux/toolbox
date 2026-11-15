use super::Found;

const TEXT: &str = "abcdefghij\nxx foo\n";

#[test]
fn a_match_maps_to_the_row_its_line_wraps_into() {
    let mut found = Found::default();

    found.update(TEXT, "foo", Some(4));
    assert_eq!((found.count(), found.row_of(0)), (1, Some(3)));

    found.update(TEXT, "IJ", Some(4));
    assert_eq!((found.count(), found.row_of(0)), (1, Some(2)));
    assert_eq!(found.row_of(1), None);
}

#[test]
fn the_same_inputs_find_nothing_again() {
    let mut found = Found::default();
    found.update(TEXT, "x", Some(4));
    let rows = found.rows().as_ptr();

    found.update(TEXT, "x", Some(4));
    assert_eq!(found.rows().as_ptr(), rows);

    // Another query keeps the rows and finds only the matches.
    found.update(TEXT, "foo", Some(4));
    assert_eq!((found.rows().as_ptr(), found.count()), (rows, 1));

    found.update(TEXT, "foo", Some(8));
    assert_eq!(found.rows().len(), 3);
}
