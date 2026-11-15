//! The state dot beside a row's name says only what its chip does not.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::packages::list::Filter;

use super::super::PackagesPage;
use super::fixture;

fn installed_dots(filter: Filter) -> usize {
    let page = PackagesPage {
        filter,
        ..PackagesPage::default()
    };

    let mut harness = fixture(None).harness(page);
    harness.run();
    harness
        .query_all_by_role_and_label(Role::Label, "Installed")
        .count()
}

#[test]
fn under_the_installed_chip_no_row_repeats_installed() {
    assert_eq!(installed_dots(Filter::Installed), 0);
}

#[test]
fn under_the_all_chip_an_installed_row_says_so() {
    assert!(installed_dots(Filter::All) > 0);
}
