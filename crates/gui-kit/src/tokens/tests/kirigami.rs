//! The room between things follows Kirigami's units (`FormCard`'s paddings
//! and spacing, `ScrollablePage`'s gridUnit margin), so a page reads like a
//! KDE page at any font size.

use egui::{Margin, vec2};

use crate::theme::{GRID_UNIT, LARGE_SPACING, SMALL_SPACING};
use crate::tokens::Sizes;

#[test]
fn a_form_row_pads_by_large_plus_small_spacing_on_both_axes() {
    let formcard = LARGE_SPACING + SMALL_SPACING;

    assert_eq!(Sizes::DEFAULT.row_padding, vec2(formcard, formcard));
}

#[test]
fn a_page_keeps_one_grid_unit_around_its_content() {
    assert_eq!(Sizes::DEFAULT.page_padding, Margin::same(18));
    assert!((GRID_UNIT - 18.0).abs() < f32::EPSILON);
}

#[test]
fn a_form_header_sits_large_plus_small_above_and_small_below() {
    let sizes = Sizes::DEFAULT;

    assert!((sizes.section_gap - (LARGE_SPACING + SMALL_SPACING)).abs() < f32::EPSILON);
    assert!((sizes.header_gap - SMALL_SPACING).abs() < f32::EPSILON);
}

#[test]
fn a_card_row_keeps_formcards_inner_padding() {
    let formcard = LARGE_SPACING + SMALL_SPACING;

    let padding = Sizes::DEFAULT.card_padding;

    assert_eq!(f32::from(padding.left), formcard);
    assert_eq!(f32::from(padding.right), formcard);
    assert_eq!(f32::from(padding.top), formcard);
    assert_eq!(f32::from(padding.bottom), formcard);
}

#[test]
fn a_label_sits_large_spacing_from_its_control() {
    assert!((Sizes::DEFAULT.text_to_control - LARGE_SPACING).abs() < f32::EPSILON);
}
