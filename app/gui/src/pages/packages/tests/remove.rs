//! The question before a removal names the packages that need the one
//! removed, as pacman refuses while they stay.

use egui_kittest::kittest::Queryable as _;

use super::super::PackagesPage;
use super::fixture;

#[test]
fn removing_a_package_others_need_names_them() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();
    let gimps = harness.query_all_by_label("gimp").count();

    super::open_row(&mut harness, "gegl");
    harness.get_by_label("Remove gegl").click();
    harness.run();

    assert!(
        harness
            .query_by_label("1 installed package needs it. pacman removes nothing while it stays:")
            .is_some()
    );
    assert_eq!(harness.query_all_by_label("gimp").count(), gimps + 1);
}

#[test]
fn removing_a_package_nothing_needs_names_no_user() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    super::open_row(&mut harness, "gimp");
    harness.get_by_label("Remove gimp").click();
    harness.run();

    assert!(
        harness
            .query_by_label_contains("installed package needs it")
            .is_none()
    );
}
