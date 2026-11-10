use egui_kittest::kittest::Queryable as _;
use toolbox_system::{Menu, MenuRead};

use super::{LOADER_CONF, image_menu, shown};

#[expect(clippy::unreachable, reason = "the image is a systemd-boot capture")]
fn with_counts(skipped: usize, cut: usize) -> Menu {
    let Menu::Read(MenuRead::SdBoot(mut menu)) = image_menu(LOADER_CONF) else {
        unreachable!("the image's menu is a systemd-boot menu");
    };

    menu.skipped = skipped;
    menu.cut = cut;
    Menu::Read(MenuRead::SdBoot(menu))
}

#[test]
fn unread_and_cut_entries_are_named_over_the_menu() {
    let mut harness = shown(with_counts(2, 3));

    assert!(
        harness
            .query_by_label_contains("2 boot entries could not be read.")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("3 more entries are left out, the page shows the first 256.")
            .is_some()
    );

    harness.snapshot("boot/sdboot_counts_dark");
}

#[test]
fn a_menu_read_in_full_has_no_banner() {
    let harness = shown(with_counts(0, 0));

    assert!(
        harness
            .query_by_label_contains("could not be read")
            .is_none()
    );
}

#[test]
fn only_unread_entries_leave_the_limit_unsaid() {
    let harness = shown(with_counts(1, 0));

    assert!(
        harness
            .query_by_label_contains("One boot entry could not be read.")
            .is_some()
    );
    assert!(harness.query_by_label_contains("left out").is_none());
}
