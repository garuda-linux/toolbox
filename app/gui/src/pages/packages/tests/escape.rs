//! Escape over the open details goes to an overlay drawn over them first.

use super::super::PackagesPage;
use super::{fixture, gimp_details, name, open_row};
use crate::pages::under_palette::{UnderPalette, escape_under_palette};

#[test]
fn escape_closes_the_palette_over_the_details_and_the_details_stay() {
    let page = UnderPalette::<PackagesPage>::default();
    let mut harness = fixture(Some(gimp_details())).harness(page);
    harness.run();
    open_row(&mut harness, "gimp");
    assert_eq!(harness.state().page.page.open, Some(name("gimp")));

    escape_under_palette(&mut harness);
    assert_eq!(harness.state().page.page.open, Some(name("gimp")));
}
