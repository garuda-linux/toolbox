//! Choose on a desktop whose file dialog cannot open.

use egui_kittest::kittest::Queryable as _;

use super::{grub_system, menu, page};
use crate::boot::Boot;

const FAILED: &str = "The file dialog could not open";

#[test]
fn a_dialog_that_could_not_open_is_named_beside_the_file_rows() {
    let harness = page(
        grub_system(),
        Boot::showing(menu(), None, None).with_failed_chooser(),
    );

    assert!(harness.query_by_label_contains(FAILED).is_some());
}

#[test]
fn without_a_failed_dialog_no_banner_shows() {
    let harness = page(grub_system(), Boot::showing(menu(), None, None));

    assert!(harness.query_by_label_contains(FAILED).is_none());
}
