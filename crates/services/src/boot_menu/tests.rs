use toolbox_plan::{GrubMenu, GrubPlatform, PackageName};
use toolbox_system::MenuRead;
use toolbox_system::pkexec::{HelperError, Refusal};

use super::Menu;

/// A used Garuda Mokka install, every UUID masked.
const GRUB_CFG: &str = toolbox_plan::testing::grub::INSTALLED_GRUB_CFG;
/// The Lima image with `linux-garuda` and `linux-lts`.
const LIMA_GRUB_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;
const MASKED: &str = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";

fn known(config: &str) -> Menu {
    Menu::Read(MenuRead::Known(GrubMenu::for_platform(
        config,
        GrubPlatform::Efi,
    )))
}

#[test]
fn the_menu_states_have_codes_and_no_entries_unless_known() {
    for (menu, code) in [
        (Menu::NotAsked, "not-asked"),
        (Menu::Loading, "loading"),
        (Menu::Read(MenuRead::Missing), "missing"),
        (
            Menu::Read(MenuRead::Unknown(HelperError::Unreadable)),
            "unknown",
        ),
        (
            Menu::Read(MenuRead::Refused(Refusal::Dismissed)),
            "dismissed",
        ),
        (
            Menu::Read(MenuRead::Refused(Refusal::HelperMissing)),
            "helper-missing",
        ),
    ] {
        assert_eq!(menu.as_str(), code);
        assert_eq!(menu.known(), None);
    }

    assert_eq!(known(GRUB_CFG).as_str(), "known");
}

#[test]
fn a_kernel_names_its_menu_entry_once_the_menu_is_read() {
    let kernel = |name: &str| PackageName::new(name).unwrap();
    let installed = known(GRUB_CFG);
    let lima = known(LIMA_GRUB_CFG);

    let uuid = "cd2fff21-4bb7-40f3-a158-406f545e11e6";

    assert_eq!(Menu::NotAsked.kernel_entry(&kernel("linux-garuda")), "");
    assert_eq!(installed.kernel_entry(&kernel("linux-lts")), "");
    assert_eq!(
        installed.kernel_entry(&kernel("linux-garuda")),
        format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}")
    );
    assert_eq!(
        lima.kernel_entry(&kernel("linux-lts")),
        format!("gnulinux-advanced-{uuid}>gnulinux-linux-lts-advanced-{uuid}")
    );
}
