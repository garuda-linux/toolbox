use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::ChangeSet;
use toolbox_core::domains::KernelCmdline;
use toolbox_core::features::boot::set_loader;
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::LoaderSetting;

use super::super::super::BootPage;
use super::{KERNEL_CMDLINE, LOADER_CONF, Shown, image, image_menu};
use crate::boot::Boot;
use crate::page_service::Services;
use crate::testing::PageFixture;

fn queued(default: Option<LoaderDefault>) -> Shown {
    let snapshot = image(KernelCmdline::read(KERNEL_CMDLINE, false));
    let mut changes = ChangeSet::default();
    set_loader(LoaderSetting::Default(default), &mut changes, &snapshot);

    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        changes,
        services: Services {
            boot: Boot::showing(image_menu(LOADER_CONF), None, None),
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(BootPage::default());

    harness.run();
    harness
}

/// Both rows write `default`: the queued value says which of them
/// changed.
fn marked(harness: &Shown) -> &'static str {
    let tags: Vec<_> = harness.query_all_by_label("Changes with Apply").collect();
    assert_eq!(tags.len(), 1, "one row carries the mark");

    let at = tags[0].rect().center().y;
    let entry = harness
        .get_by_role_and_label(Role::ComboBox, "Default entry")
        .rect()
        .center()
        .y;

    let remember = harness
        .get_by_role_and_label(Role::CheckBox, "Remember the last entry")
        .rect()
        .center()
        .y;

    if (at - entry).abs() < (at - remember).abs() {
        "Default entry"
    } else {
        "Remember the last entry"
    }
}

#[test]
fn a_queued_saved_default_marks_remember_alone() {
    let harness = queued(Some(LoaderDefault::saved()));

    assert_eq!(marked(&harness), "Remember the last entry");
}

#[test]
fn a_queued_entry_marks_the_default_entry_alone() {
    let pattern = LoaderDefault::new("garuda-*-garuda.efi").unwrap();

    assert_eq!(marked(&queued(Some(pattern))), "Default entry");
}
