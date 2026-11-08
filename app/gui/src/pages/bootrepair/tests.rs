mod dialog;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::bootrepair::page::{Facts, Offer, Progress};
use toolbox_core::features::bootrepair::{Firmware, RepairFacts, Target, Tool, parse_lsblk};
use toolbox_plan::{PartitionDevice, Password, RootLayout, VolumeDevice};
use toolbox_system::{DryRunRepair, RepairSource as _};

use super::BootRepairPage;
use crate::bootrepair::BootRepair;
use crate::command::{BootRepairCommand, Command};
use crate::testing::{PageFixture, PageState};

/// A bare-metal laptop: the running system on `nvme0n1` in LUKS, a free btrfs
/// Garuda on `/dev/nvme1n1p2` with the EFI system partition
/// `/dev/nvme1n1p1`.
const BARE_METAL: &str =
    include_str!("../../../../../crates/core/src/features/bootrepair/fixtures/bare-metal.json");

type Shown = Harness<'static, PageState<BootRepairPage>>;

fn sample() -> Facts {
    Facts::Known(RepairFacts {
        firmware: Firmware::Uefi,
        missing: Vec::new(),
        inventory: parse_lsblk(BARE_METAL).unwrap(),
    })
}

/// [`BARE_METAL`] as the live ISO sees it, both LUKS partitions of
/// `nvme0n1` locked: the dry run's sample, edited where it is made.
fn locked() -> BootRepair {
    let facts = Facts::Known(DryRunRepair.facts().unwrap());

    BootRepair::showing(Offer::Live, facts, Progress::None)
}

fn page(repair: BootRepair, start: BootRepairPage) -> Shown {
    let mut harness = PageFixture {
        boot_repair: repair,
        ..PageFixture::default()
    }
    .harness(start);

    harness.run();
    harness
}

fn live() -> BootRepair {
    BootRepair::showing(Offer::Live, sample(), Progress::None)
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn shows(harness: &Shown, label: &str) -> bool {
    harness.query_all_by_label_contains(label).next().is_some()
}

#[test]
fn an_installed_system_says_the_page_is_for_the_live_system() {
    let repair = BootRepair::showing(Offer::Hidden, sample(), Progress::None);
    let harness = page(repair, BootRepairPage::default());

    assert!(shows(
        &harness,
        "Boot repair is only offered on the live system"
    ));
    assert!(!shows(&harness, "Reinstall GRUB"));
}

#[test]
fn the_first_free_system_shows_with_what_was_found() {
    let harness = page(live(), BootRepairPage::default());

    for label in ["UEFI", "Not encrypted"] {
        assert!(shows(&harness, label), "{label}");
    }

    for (select, value) in [
        ("Root partition", "/dev/nvme1n1p2"),
        ("EFI system partition", "/dev/nvme1n1p1"),
        (
            "Where the system lies",
            "In the subvolume @ (Garuda's layout)",
        ),
    ] {
        let picked = harness
            .get_by_role_and_label(Role::ComboBox, select)
            .value()
            .unwrap_or_default();

        assert!(picked.starts_with(value), "{select}: {picked}");
    }

    assert!(!shows(&harness, "Test mode"));
}

#[test]
fn the_boot_menu_is_written_again_after_asking() {
    let mut harness = page(live(), BootRepairPage::default());

    harness.get_by_label("Generate boot menu…").click();
    harness.run();

    assert!(shows(&harness, "Generate the boot menu again?"));

    harness
        .get_by_role_and_label(Role::Button, "Generate boot menu")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(BootRepairCommand::Repair {
            system: VolumeDevice::new("/dev/nvme1n1p2").unwrap(),
            layout: Some(RootLayout::Subvolume),
            passphrase: None,
            target: None,
        })]
    );
}

#[test]
fn a_locked_system_is_repaired_only_with_its_passphrase() {
    let start = BootRepairPage {
        system: Some("/dev/nvme0n1p1".to_owned()),
        ..BootRepairPage::default()
    };

    let mut harness = page(locked(), start);

    assert!(shows(&harness, "Encrypted (LUKS), locked"));

    // The taller rows put the button under the window's foot.
    harness.get_by_label("Reinstall GRUB…").scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall GRUB…").click();
    harness.run();

    let confirm = |shown: &Shown| {
        shown
            .get_by_role_and_label(Role::Button, "Reinstall GRUB")
            .accesskit_node()
            .is_disabled()
    };

    assert!(confirm(&harness), "no passphrase yet");

    let field = "Passphrase of the encrypted partition";
    harness
        .get_by_role_and_label(Role::PasswordInput, field)
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::PasswordInput, field)
        .type_text("secret words");
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Reinstall GRUB")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(BootRepairCommand::Repair {
            system: VolumeDevice::new("/dev/nvme0n1p1").unwrap(),
            layout: Some(RootLayout::Subvolume),
            passphrase: Some(Password::new("secret words".to_owned()).unwrap()),
            target: Some(Target::Esp(PartitionDevice::new("/dev/nvme0n1p2").unwrap())),
        })]
    );
}

#[test]
fn the_test_mode_says_so_and_a_finished_repair_can_be_dismissed() {
    let repair = BootRepair::showing(Offer::Test, sample(), Progress::Succeeded);
    let mut harness = page(repair, BootRepairPage::default());

    assert!(shows(&harness, "Test mode"));
    assert!(shows(&harness, "The repair finished without an error"));

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(BootRepairCommand::Forget)]);
}

#[test]
fn missing_programs_are_named_and_nothing_is_offered() {
    let facts = Facts::Known(RepairFacts {
        firmware: Firmware::Bios,
        missing: vec![Tool::GrubInstall, Tool::Cryptsetup],
        inventory: parse_lsblk(r#"{"blockdevices":[]}"#).unwrap(),
    });

    let repair = BootRepair::showing(Offer::Live, facts, Progress::None);
    let harness = page(repair, BootRepairPage::default());

    assert!(shows(&harness, "grub-install, cryptsetup"));
    assert!(!shows(&harness, "Reinstall GRUB"));
}

#[test]
fn the_passphrase_field_is_named_above_it_and_the_wait_is_explained() {
    let start = BootRepairPage {
        system: Some("/dev/nvme0n1p1".to_owned()),
        ..BootRepairPage::default()
    };

    let mut harness = page(locked(), start);

    // The taller rows put the button under the window's foot.
    harness.get_by_label("Reinstall GRUB…").scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall GRUB…").click();
    harness.run();

    let field = "Passphrase of the encrypted partition";
    let named: Vec<_> = harness.query_all_by_label(field).collect();
    assert!(
        named
            .iter()
            .any(|node| node.accesskit_node().role() == Role::Label),
        "the field has only a placeholder for a name"
    );
    assert!(shows(&harness, "Enter the passphrase to go on."));
}

#[test]
fn show_console_is_offered_only_while_the_console_is_hidden() {
    let running = || BootRepair::showing(Offer::Live, sample(), Progress::Running);
    let hidden = page(running(), BootRepairPage::default());
    assert!(shows(&hidden, "Show Console"));

    let mut open = PageFixture {
        boot_repair: running(),
        console_open: true,
        ..PageFixture::default()
    }
    .harness(BootRepairPage::default());

    open.run();

    assert!(!shows(&open, "Show Console"));
}

#[test]
fn a_sections_sentence_sits_under_its_heading_like_on_maintenance() {
    let mut harness = page(live(), BootRepairPage::default());
    harness.run();

    let gaps = toolbox_gui_kit::tokens::Sizes::DEFAULT;
    let heading = harness.get_by_label("Repair").rect();
    let sentence = harness
        .get_by_label_contains("Each repair asks first")
        .rect();

    let intro = harness.get_by_label_contains("Use this page when").rect();
    let section = harness.get_by_label("Installed system").rect();

    assert!(
        sentence.top() - heading.bottom() <= 12.0,
        "{} px between the heading and its sentence",
        sentence.top() - heading.bottom()
    );
    assert!(
        section.top() - intro.bottom() >= gaps.section_gap,
        "{} px between the intro and the first section",
        section.top() - intro.bottom()
    );
}
