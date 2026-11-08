use super::cap_centring;
use crate::testing::deep_descent_font;

const INTER: &[u8] = include_bytes!("../../../assets/fonts/InterVariable.ttf");

#[test]
fn inter_already_centres_its_capitals_in_its_line() {
    let shift = cap_centring(INTER, 0);
    assert!(shift.abs() < 0.005, "Inter moves by {shift} of its size");
}

#[test]
fn a_deep_descent_moves_the_glyphs_down_by_half_of_what_it_adds() {
    // 1200 instead of Inter's 494 font units, at 2048 to the em.
    let added = (1200.0 - 494.0) / 2048.0;
    let shift = cap_centring(&deep_descent_font().data, 0);
    assert!(
        (shift - added / 2.0).abs() < 0.005,
        "{shift} instead of {}",
        added / 2.0
    );
}

#[test]
fn a_file_that_is_no_font_moves_nothing() {
    assert!(cap_centring(b"not a font", 0).abs() < f32::EPSILON);
}
