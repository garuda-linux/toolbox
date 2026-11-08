//! The room above a package row's capitals against the room under its
//! last baseline, in the repositories' list and the AUR's.

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_aur::AurSource as _;
use toolbox_core::features::packages::aur::AurCatalog;
use toolbox_core::features::packages::list::Filter;
use toolbox_gui_kit::testing::{capitals_top, last_baseline_within};
use toolbox_gui_kit::tokens;

use super::super::PackagesPage;
use super::{catalog, fixture};
use crate::packages::{Packages, Read, SampleAur};

/// Off by at most this, two gaps count as one.
const SAME: f32 = 1.0;

fn assert_even<State>(harness: &Harness<'_, State>, package: &str) {
    let row = harness
        .get_by_role_and_label(Role::ListItem, package)
        .rect();

    let capitals = capitals_top(harness, package).expect("the name is painted");
    let (baseline, last) = last_baseline_within(harness, row).expect("the text is painted");
    let above = capitals - row.top();
    let below = row.bottom() - baseline;
    let padding = tokens::Sizes::DEFAULT.row_padding.y;

    assert!(
        (above - below).abs() <= SAME,
        "{package}: the capitals start {above} under the row's top, the last baseline ends \
         {below} above its bottom"
    );
    assert!(
        (row.bottom() - last - padding).abs() <= SAME,
        "{package}: the last line ends {} above the row's bottom",
        row.bottom() - last
    );
}

#[test]
fn a_package_rows_capitals_have_as_much_room_above_as_its_last_baseline_below() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    for package in ["gegl", "gimp"] {
        assert_even(&harness, package);
    }
}

#[test]
fn an_aur_rows_capitals_have_as_much_room_above_as_its_last_baseline_below() {
    let mut fixture = fixture(None);
    fixture.settings.packages.show_aur = true;
    let aur = Arc::new(AurCatalog::new(SampleAur.index().unwrap()));
    fixture.packages = Packages::showing(Read::Known(catalog()), None).with_aur(Read::Known(aur));

    let page = PackagesPage {
        filter: Filter::Aur,
        aur_asked: true,
        ..PackagesPage::default()
    };

    let mut harness = fixture.harness(page);
    harness.run();
    assert_even(&harness, "paru-bin");
}
