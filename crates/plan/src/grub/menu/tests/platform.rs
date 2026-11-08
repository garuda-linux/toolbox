use super::super::{GrubMenu, GrubPlatform};
use super::{INSTALLED, LIMA, PC_MEMTEST, brief, found, line, memtests};

/// GRUB on a BIOS machine skips the `efi` branches, so the firmware entry
/// is missing and memtest86+ comes from the `pc` branch.
#[test]
fn a_bios_machine_counts_only_the_pc_branches() {
    let menu = GrubMenu::for_platform(INSTALLED, GrubPlatform::Pc);
    let tops: Vec<_> = brief(&menu)
        .into_iter()
        .filter(|(position, _, _)| !position.contains('>'))
        .collect();

    assert_eq!(
        tops,
        [
            line("0", "Garuda Linux", false),
            line("1", "Advanced options for Garuda Linux", true),
            line("2", "Garuda Linux snapshots", true),
            line("3", "Memory Tester (memtest86+)", false),
            line("4", "Shutdown", false),
            line("5", "Restart", false),
        ]
    );
}

/// A position names what GRUB itself shows at that place.
#[test]
fn a_position_finds_the_entry_of_the_running_platform() {
    let efi = GrubMenu::for_platform(INSTALLED, GrubPlatform::Efi);
    let pc = GrubMenu::for_platform(INSTALLED, GrubPlatform::Pc);

    assert_eq!(found(&efi, "5").as_deref(), Some("Shutdown"));
    assert_eq!(found(&efi, "6").as_deref(), Some("Restart"));
    assert_eq!(found(&efi, "7"), None);
    assert_eq!(found(&pc, "4").as_deref(), Some("Shutdown"));
    assert_eq!(found(&pc, "uefi-firmware"), None);

    for platform in [GrubPlatform::Efi, GrubPlatform::Pc] {
        assert_eq!(
            memtests(&GrubMenu::for_platform(LIMA, platform)),
            1,
            "{platform:?}"
        );
    }

    assert_eq!(
        found(
            &GrubMenu::for_platform(LIMA, GrubPlatform::Efi),
            "uefi-firmware"
        )
        .as_deref(),
        Some("UEFI Firmware Settings")
    );
}

/// The test turns the `pc` check of memtest86+ into `!= "efi"`, which
/// leaves the menu as it was.
#[test]
fn a_negated_platform_check_is_read() {
    let negated = INSTALLED.replacen(
        PC_MEMTEST,
        "if [ \"${grub_platform}\" != \"efi\" ]; then",
        1,
    );

    assert_ne!(negated, INSTALLED);

    for platform in [GrubPlatform::Efi, GrubPlatform::Pc] {
        assert_eq!(
            GrubMenu::for_platform(&negated, platform),
            GrubMenu::for_platform(INSTALLED, platform),
            "{platform:?}"
        );
    }
}

/// The test joins the two memtest86+ scripts into one `if` with an
/// `else`, and makes the `else` an `elif` with the `efi` check.
#[test]
fn else_and_elif_take_the_other_platform() {
    let between = "fi\n### END /etc/grub.d/60_memtest86+ ###\n\n\
### BEGIN /etc/grub.d/60_memtest86+-efi ###\n\
if [ \"${grub_platform}\" == \"efi\" ]; then\n";

    let otherwise = INSTALLED.replacen(between, "else\n", 1);
    let elif = INSTALLED.replacen(
        between,
        "elif [ \"${grub_platform}\" == \"efi\" ]; then\n",
        1,
    );

    assert_ne!(otherwise, INSTALLED);
    assert_ne!(elif, INSTALLED);

    for platform in [GrubPlatform::Efi, GrubPlatform::Pc] {
        let expected = GrubMenu::for_platform(INSTALLED, platform);

        assert_eq!(GrubMenu::for_platform(&otherwise, platform), expected);
        assert_eq!(GrubMenu::for_platform(&elif, platform), expected);
    }
}

/// `/sys/firmware/efi` exists only when the firmware started the system
/// through UEFI.
#[test]
fn the_firmware_folder_picks_the_platform() {
    assert_eq!(GrubPlatform::from_efi_firmware(true), GrubPlatform::Efi);
    assert_eq!(GrubPlatform::from_efi_firmware(false), GrubPlatform::Pc);
}
