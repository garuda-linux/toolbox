use egui::Key;
use egui::accesskit::Role;
use egui_kittest::SnapshotResults;
use egui_kittest::kittest::Queryable as _;

#[test]
fn maintenance_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::maintenance::maintenance();
    let harness = fixture.harness();
    results.add(harness.try_snapshot("maintenance/common_dark"));

    harness
        .get_by_label("Merge configuration updates")
        .scroll_to_me();
    harness.run();
    results.add(harness.try_snapshot("maintenance/tools_dark"));

    crate::common::open_tab(harness, "Reset settings");
    harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .click();
    harness.run();
    results.add(harness.try_snapshot("maintenance/reset_dark"));

    harness.get_by_label("Reset selected…").click();
    harness.run();
    results.add(harness.try_snapshot("maintenance/question_dark"));

    harness.get_by_label("Cancel").click();
    harness.run();
    crate::common::open_tab(harness, "Repairs");
    results.add(harness.try_snapshot("maintenance/rescue_dark"));
}

#[test]
fn diagnostics_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::diagnostics::diagnostics_page(false);
    let harness = fixture.harness();
    results.add(harness.try_snapshot("diagnostics/idle_dark"));

    crate::diagnostics::collect(harness);
    results.add(harness.try_snapshot("diagnostics/report_dark"));

    harness.get_by_label("Share report…").click();
    harness.run();
    results.add(harness.try_snapshot("diagnostics/question_dark"));
}

#[test]
fn boot_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::boot::app();
    let harness = fixture.harness();
    crate::boot::open_boot(harness);
    results.add(harness.try_snapshot("boot/menu_dark"));

    harness.get_by_label("Choose GRUB theme").scroll_to_me();
    harness.run();
    results.add(harness.try_snapshot("boot/look_dark"));
}

#[test]
fn sdboot_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::sdboot::app();
    let harness = fixture.harness();
    crate::sdboot::open_boot(harness);
    results.add(harness.try_snapshot("boot/sdboot_dark"));
}

#[test]
fn locales_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::locales::languages();
    let harness = fixture.harness();
    results.add(harness.try_snapshot("locales/locales_dark"));

    harness
        .get_by_role_and_label(Role::CheckBox, "German (Germany)")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();
    results.add(harness.try_snapshot("locales/packs_dark"));
}

#[test]
fn network_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::network::network();
    let harness = fixture.harness();
    results.add(harness.try_snapshot("network/status_dark"));

    crate::common::open_tab(harness, "Drivers");
    results.add(harness.try_snapshot("network/drivers_dark"));

    crate::common::open_tab(harness, "Hardware");
    results.add(harness.try_snapshot("network/hardware_dark"));

    crate::common::open_tab(harness, "Tools");
    results.add(harness.try_snapshot("network/tools_dark"));
}

#[test]
fn users_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::users::users();
    let harness = fixture.harness();
    harness.get_by_label("ada, Ada Lovelace").click();
    harness.run();
    results.add(harness.try_snapshot("users/users_dark"));

    harness.get_by_label("Add user…").click();
    harness.run();
    results.add(harness.try_snapshot("users/add_dark"));

    harness.key_press(Key::Escape);
    harness.run();
    harness.get_by_label("Edit ada…").click();
    harness.run();
    results.add(harness.try_snapshot("users/edit_dark"));

    harness.key_press(Key::Escape);
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();
    results.add(harness.try_snapshot("users/groups_dark"));
}

#[test]
fn configs_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::configs::configs();
    let harness = fixture.harness();
    results.add(harness.try_snapshot("configs/list_dark"));

    harness.get_by_label("Show the files").click();
    harness.run();
    results.add(harness.try_snapshot("configs/folder_dark"));

    harness
        .query_all_by_label("View")
        .next()
        .unwrap()
        .scroll_to_me();
    harness.run();
    harness.query_all_by_label("View").next().unwrap().click();
    crate::system::step_until(harness, "the text", |harness| {
        harness.query_all_by_label("Copy all").next().is_some()
    });
    results.add(harness.try_snapshot("configs/view_dark"));
}

#[test]
fn boot_repair_page() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::bootrepair::app(true);
    let harness = fixture.harness();
    crate::bootrepair::open(harness);
    results.add(harness.try_snapshot("bootrepair/page_dark"));

    // The button lies under the window's bottom, where a click misses it.
    harness
        .get_by_label("Reinstall the boot loader…")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall the boot loader…").click();
    crate::system::step_until(harness, "the question", |harness| {
        harness
            .query_all_by_label_contains("is written to the EFI system partition /dev/nvme0n1p2")
            .next()
            .is_some()
    });
    crate::system::settle(harness);
    results.add(harness.try_snapshot("bootrepair/question_dark"));
}

#[test]
fn setup_assistant() {
    let mut results = SnapshotResults::new();
    let mut fixture = crate::setup::assistant();
    let harness = fixture.harness();
    results.add(harness.try_snapshot("setup/welcome_dark"));

    harness.get_by_label("Next").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Strawberry Player")
        .click();
    harness.run();
    results.add(harness.try_snapshot("setup/step_dark"));

    harness.get_by_label("Next").click();
    harness.run();
    results.add(harness.try_snapshot("setup/summary_dark"));
}
