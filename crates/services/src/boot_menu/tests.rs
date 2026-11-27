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
fn the_menu_states_have_no_entries_unless_known() {
    for menu in [
        Menu::NotAsked,
        Menu::Loading,
        Menu::Read(MenuRead::Missing),
        Menu::Read(MenuRead::Unknown(HelperError::Unreadable)),
        Menu::Read(MenuRead::Refused(Refusal::Dismissed)),
        Menu::Read(MenuRead::Refused(Refusal::HelperMissing)),
    ] {
        assert_eq!(menu.known(), None);
    }

    assert!(known(GRUB_CFG).known().is_some());
}

#[test]
fn a_kernel_names_its_menu_entry_once_the_menu_is_read() {
    let kernel = |name: &str| PackageName::new(name).unwrap();
    let installed = known(GRUB_CFG);
    let lima = known(LIMA_GRUB_CFG);

    let uuid = "cd2fff21-4bb7-40f3-a158-406f545e11e6";

    assert_eq!(Menu::NotAsked.kernel_entry(&kernel("linux-garuda")), None);
    assert_eq!(installed.kernel_entry(&kernel("linux-lts")), None);
    assert_eq!(
        installed.kernel_entry(&kernel("linux-garuda")),
        Some(
            format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}").as_str()
        )
    );
    assert_eq!(
        lima.kernel_entry(&kernel("linux-lts")),
        Some(format!("gnulinux-advanced-{uuid}>gnulinux-linux-lts-advanced-{uuid}").as_str())
    );
}
