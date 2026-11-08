use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{BootLoader, KernelCmdline, SdBootKernel};
use toolbox_core::features::boot::Flag;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::kernels::KernelReleases;
use toolbox_plan::sdboot::{
    LoaderEntry, LoaderSetting, LoaderSettings, LoaderTimeout, SdBootMenu, sections,
};
use toolbox_plan::{KernelRelease, LoaderDefault, PackageName};
use toolbox_system::{Host, Menu, MenuRead};

use super::super::tests::{Shown, page};
use crate::boot::Boot;
use crate::command::{BootCommand, Command};

/// Files of the Garuda Lima image.
const LOADER_CONF: &str =
    include_str!("../../../../../../crates/plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str =
    include_str!("../../../../../../crates/state/src/boot/fixtures/kernel-cmdline");
/// `/proc/cmdline` of the Garuda live ISO.
const LIVE_CMDLINE: &str =
    include_str!("../../../../../../crates/system/src/host/fixtures/live/proc-cmdline");
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

fn image(cmdline: KernelCmdline) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(SdBootKernel {
        cmdline,
        theme: None,
    });
    snapshot
}

fn shown(menu: Menu) -> Shown {
    page(
        image(KernelCmdline::read(KERNEL_CMDLINE, false)),
        Boot::showing(menu, None, None),
    )
}

fn edit(change: Edit) -> Command {
    Command::from(BootCommand::Edit(change))
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
fn the_root_parameters_stay_and_the_others_can_go() {
    let mut harness = shown(image_menu(LOADER_CONF));

    for kept in [
        "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6",
        "rootflags=subvol=@",
        "rw",
    ] {
        assert!(
            harness.query_by_label(&format!("Remove {kept}")).is_none(),
            "{kept}"
        );
        assert!(harness.query_by_label(kept).is_some(), "{kept}");
    }

    harness.get_by_label("Remove loglevel=3").scroll_to_me();
    harness.run();
    harness.get_by_label("Remove loglevel=3").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [edit(Edit::RemoveParameter("loglevel=3".to_owned()))]
    );
}

#[test]
fn the_switches_change_the_kernel_command_line() {
    let mut harness = shown(image_menu(LOADER_CONF));

    harness
        .get_by_role_and_label(Role::CheckBox, "Hide most boot messages (quiet)")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [edit(Edit::Flag(Flag::Quiet, false))]);
}

/// The live ISO's `initrd=\…` words cannot be written back word for word.
#[test]
fn a_line_the_page_cannot_write_is_shown_with_where_to_edit_it() {
    let harness = page(
        image(KernelCmdline::read(LIVE_CMDLINE, false)),
        Boot::showing(image_menu(LOADER_CONF), None, None),
    );

    assert!(
        harness
            .query_by_label_contains("Edit /etc/kernel/cmdline on the Config Files page")
            .is_some()
    );
    assert!(harness.query_by_label("New kernel parameter").is_none());
}

#[test]
fn the_running_command_line_says_the_file_comes_with_the_first_change() {
    let harness = page(
        image(KernelCmdline::read(KERNEL_CMDLINE, true)),
        Boot::showing(image_menu(LOADER_CONF), None, None),
    );

    assert!(
        harness
            .query_by_label_contains("The first change creates the file")
            .is_some()
    );
}

#[test]
fn the_splash_brings_the_plymouth_theme_and_nothing_of_grub() {
    let without = shown(image_menu(LOADER_CONF));

    assert!(
        without
            .query_by_role_and_label(Role::ComboBox, "Boot splash theme")
            .is_none()
    );

    let line = format!("{} splash", KERNEL_CMDLINE.trim_end());
    let with = page(
        image(KernelCmdline::read(&line, false)),
        Boot::showing(image_menu(LOADER_CONF), None, None),
    );

    assert!(
        with.query_by_role_and_label(Role::ComboBox, "Boot splash theme")
            .is_some()
    );
    assert!(with.query_by_label_contains("GRUB theme").is_none());
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

/// As on NixOS, which writes its own entries.
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

/// The releases of the Lima image's kernels: linux-lts has no UKI there.
fn with_kernels(mut snapshot: Snapshot) -> Snapshot {
    snapshot.insert(KernelReleases(
        [
            ("linux-garuda", "7.3.1-1-garuda"),
            ("linux-lts", "6.18.60-1-lts"),
        ]
        .into_iter()
        .map(|(kernel, release)| {
            (
                PackageName::new(kernel).unwrap(),
                KernelRelease::new(release).unwrap(),
            )
        })
        .collect(),
    ));
    snapshot
}

fn proposed(conf: &str, kernel: &str) -> Shown {
    page(
        with_kernels(image(KernelCmdline::read(KERNEL_CMDLINE, false))),
        Boot::showing(
            image_menu(conf),
            None,
            Some(PackageName::new(kernel).unwrap()),
        ),
    )
}

#[test]
fn a_proposed_kernel_is_queued_by_its_flavour() {
    let mut harness = proposed(LOADER_CONF, "linux-garuda");

    harness.get_by_label("Queue as default").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            loader(LoaderSetting::Default(Some(
                LoaderDefault::new("garuda-*-garuda.efi").unwrap()
            ))),
            Command::from(BootCommand::DropProposal),
        ]
    );
}

#[test]
fn a_kernel_without_an_image_is_not_offered() {
    let harness = proposed(LOADER_CONF, "linux-lts");

    assert!(
        harness
            .query_by_label_contains("systemd-boot has no image of the kernel linux-lts yet")
            .is_some()
    );
    assert!(harness.query_by_label("Queue as default").is_none());
}

/// The image's `loader.conf` with the kernel already its default.
#[test]
fn the_kernel_that_is_the_default_already_says_so() {
    let conf = format!("default garuda-*-garuda.efi\n{LOADER_CONF}");
    let harness = proposed(&conf, "linux-garuda");

    assert!(harness.query_by_label("Queue as default").is_none());
}
