use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{BootLoader, KernelCmdline};
use toolbox_plan::sdboot::{
    LoaderEntry, LoaderSetting, LoaderSettings, LoaderTimeout, SdBootMenu, sections,
};
use toolbox_plan::{KernelParameters, LoaderDefault};
use toolbox_system::{Host, Menu, MenuRead};

use super::super::tests::{Shown, page};
use crate::boot::Boot;
use crate::command::{BootCommand, Command};

/// Files of the Garuda Lima image.
const LOADER_CONF: &str =
    include_str!("../../../../../../crates/plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str =
    include_str!("../../../../../../crates/state/src/boot/fixtures/kernel-cmdline");
const UKI: &[u8] = include_bytes!(
    "../../../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head"
);
const FALLBACK: &[u8] = include_bytes!(
    "../../../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head"
);

fn section(image: &[u8], name: &str) -> String {
    let found = sections(image)
        .into_iter()
        .find(|section| section.name == name)
        .unwrap();

    let start = usize::try_from(found.offset).unwrap();
    let end = start + usize::try_from(found.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

fn uki(file: &str, image: &[u8]) -> LoaderEntry {
    LoaderEntry::uki(
        file,
        &section(image, ".osrel"),
        Some(&section(image, ".uname")),
    )
    .unwrap()
}

fn image_menu(conf: &str) -> Menu {
    Menu::Read(MenuRead::SdBoot(SdBootMenu {
        settings: LoaderSettings::parse(conf),
        entries: vec![
            uki("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK),
            uki("garuda-7.3.1-1-garuda.efi", UKI),
        ],
    }))
}

fn image() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(KernelCmdline(
        KernelParameters::parse(KERNEL_CMDLINE).unwrap(),
    ));
    snapshot
}

fn shown(menu: Menu) -> Shown {
    page(image(), Boot::showing(menu, None, None))
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn loader(setting: LoaderSetting) -> Command {
    Command::from(BootCommand::Loader(setting))
}

fn pick(harness: &mut Shown, select: &str, choice: &str) {
    harness
        .get_by_role_and_label(Role::ComboBox, select)
        .click();
    harness.run();
    harness.get_by_label(choice).click();
    harness.run();
}

/// The image's kernel: the queued default names its flavour, so it still
/// finds the kernel after the next update.
#[test]
fn picking_the_kernel_queues_a_default_that_survives_updates() {
    let mut harness = shown(image_menu(LOADER_CONF));

    pick(&mut harness, "Default entry", "Garuda Linux");

    assert_eq!(
        sent(&harness),
        [loader(LoaderSetting::Default(Some(
            LoaderDefault::new("garuda-*-garuda.efi").unwrap()
        )))]
    );
}

#[test]
fn remembering_the_last_entry_queues_saved() {
    let mut harness = shown(image_menu(LOADER_CONF));

    harness
        .get_by_role_and_label(Role::CheckBox, "Remember the last entry")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [loader(LoaderSetting::Default(Some(LoaderDefault::saved())))]
    );
}

#[test]
fn a_switch_takes_on_off_or_systemd_boots_own_default() {
    let mut harness = shown(image_menu(LOADER_CONF));

    pick(&mut harness, "Edit entries at boot", "Off");

    assert_eq!(sent(&harness), [loader(LoaderSetting::Editor(Some(false)))]);
}

#[test]
fn the_timeout_can_wait_for_a_key() {
    let mut harness = shown(image_menu(LOADER_CONF));

    pick(&mut harness, "Menu timeout", "The menu waits for a key");

    assert_eq!(
        sent(&harness),
        [loader(LoaderSetting::Timeout(Some(
            LoaderTimeout::MenuForce
        )))]
    );
}

/// The image's `loader.conf` with its commented timeout switched on: the
/// seconds show in their own field beside the mode.
#[test]
fn a_timeout_in_seconds_shows_its_number() {
    let conf = LOADER_CONF.replace("#timeout", "timeout");
    let harness = shown(image_menu(&conf));

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Menu timeout in seconds")
            .is_some()
    );
}

#[test]
fn the_images_command_line_is_shown_whole() {
    let harness = shown(image_menu(LOADER_CONF));

    assert!(
        harness
            .query_by_label_contains("rootflags=subvol=@ rw loglevel=3 quiet")
            .is_some()
    );
}

#[test]
fn the_menu_is_read_when_the_page_shows() {
    let harness = shown(Menu::Loading);

    assert!(
        harness
            .query_by_label_contains("Reading the boot menu")
            .is_some()
    );
}

/// The command line is known without the menu, so a failed read leaves it
/// on the page.
#[test]
fn a_menu_that_could_not_be_read_says_why_and_keeps_the_command_line() {
    let harness = shown(Menu::Read(MenuRead::Unknown(
        "no ESP is mounted".to_owned(),
    )));

    assert!(
        harness
            .query_by_label_contains("The boot menu could not be read")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("rootflags=subvol=@")
            .is_some()
    );
}

/// The image's menu on a system without `/etc/kernel/cmdline`, as NixOS
/// boots: its entries carry their own options.
#[test]
fn without_kernel_cmdline_the_page_says_so_instead_of_none() {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);

    let harness = page(snapshot, Boot::showing(image_menu(LOADER_CONF), None, None));

    assert!(
        harness
            .query_by_label_contains("This system has no /etc/kernel/cmdline")
            .is_some()
    );
}
