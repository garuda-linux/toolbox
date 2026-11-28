//! The AUR row's button while the build orders of its upgrade are read.

use std::sync::Arc;

use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::Loadable;
use toolbox_plan::PackageName;
use toolbox_system::{PendingUpdate, Status};
use toolbox_text::plain;

use super::{Shown, page};
use crate::packages::Packages;
use crate::testing::PageFixture;

fn outdated() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(vec![PendingUpdate {
            name: PackageName::new("octopi-git").unwrap(),
            from: plain("1.0"),
            to: plain("1.1"),
        }]),
        merges: Ok(Vec::new()),
        last_update: Ok(None),
        health: Ok(Vec::new()),
    }
}

fn shown(reading: bool) -> Shown {
    let mut fixture = PageFixture {
        status: Some(Loadable::Loaded(Arc::new(outdated()))),
        ..PageFixture::default()
    };

    if reading {
        fixture.services.packages = Packages::default().with_upgrade_reading();
    }

    page(fixture)
}

#[test]
fn the_aur_button_reads_and_waits_while_the_build_orders_are_read() {
    let harness = shown(true);

    assert!(harness.query_by_label("Reading").is_some());
    assert!(harness.query_by_label("Update in Console").is_none());
}

#[test]
fn the_aur_button_offers_the_update_when_nothing_is_read() {
    let harness = shown(false);

    assert!(harness.query_by_label("Update in Console").is_some());
    assert!(harness.query_by_label("Reading").is_none());
}
