use toolbox_plan::sdboot::{LoaderEntry, SdBootMenu, bootctl_default};
use toolbox_plan::testing::grub::{
    INSTALLED_DEFAULT_GRUB, INSTALLED_GRUB_CFG, LIMA_DEFAULT_GRUB, LIMA_GRUB_CFG,
};
use toolbox_plan::testing::sdboot::{BARE_METAL_BOOTCTL_LIST, IMAGE_BOOTCTL_LIST};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState, PackageName};

use super::{grub, sdboot};
use crate::features::kernels::{Kernel, KernelReleases};
use crate::testing::features::kernels::{package, release};

const LIMA_GARUDA: &str = "gnulinux-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6>gnulinux-linux-garuda-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6";
const LIMA_LTS: &str = "gnulinux-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6>gnulinux-linux-lts-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6";
const LIMA_SIMPLE: &str = "gnulinux-simple-cd2fff21-4bb7-40f3-a158-406f545e11e6";

fn kernel(name: &str, installed: bool) -> Kernel {
    Kernel {
        name: package(name),
        headers: package(&format!("{name}-headers")),
        repo: String::new(),
        version: "1-1".to_owned(),
        description: String::new(),
        installed: installed.then(|| "1-1".to_owned()),
        headers_installed: false,
    }
}

fn names(name: &str) -> PackageName {
    package(name)
}

/// The Lima capture with `GRUB_DEFAULT` set to `value`, which the test
/// quotes. The capture's own line is `GRUB_DEFAULT=0`.
fn state(value: &str) -> GrubState {
    let text = LIMA_DEFAULT_GRUB.replacen("GRUB_DEFAULT=0", &format!("GRUB_DEFAULT='{value}'"), 1);

    GrubState::read(&text)
}

fn lima_menu() -> GrubMenu {
    GrubMenu::for_platform(LIMA_GRUB_CFG, GrubPlatform::Efi)
}

fn installed_kernels() -> Vec<Kernel> {
    vec![
        kernel("linux-lts", true),
        kernel("linux-garuda", true),
        kernel("linux-zen", false),
    ]
}

#[test]
fn an_advanced_id_names_the_kernel_of_its_entry() {
    assert_eq!(
        grub(&state(LIMA_LTS), &lima_menu(), &installed_kernels()),
        Some(names("linux-lts"))
    );
}

#[test]
fn a_position_names_the_kernel_of_the_entry_at_that_place() {
    // Submenu 1 holds the entries of linux-lts, its fallback and recovery
    // images, then linux-garuda.
    assert_eq!(
        grub(&state("1>3"), &lima_menu(), &installed_kernels()),
        Some(names("linux-garuda"))
    );
}

#[test]
fn a_title_names_the_kernel_of_its_entry() {
    assert_eq!(
        grub(
            &state("Garuda Linux, with Linux linux-garuda"),
            &lima_menu(),
            &installed_kernels()
        ),
        Some(names("linux-garuda"))
    );
}

#[test]
fn an_entry_of_a_kernel_that_is_not_installed_names_no_kernel() {
    let only_garuda = [kernel("linux-garuda", true), kernel("linux-lts", false)];

    assert_eq!(grub(&state(LIMA_LTS), &lima_menu(), &only_garuda), None);
}

#[test]
fn saved_names_no_kernel_because_grubenv_is_not_read() {
    assert_eq!(
        grub(&state("saved"), &lima_menu(), &installed_kernels()),
        None
    );
}

/// The top-level entry has no advanced id, so its kernel is the image of
/// its `linux` line: linux-lts in the Lima capture.
#[test]
fn the_top_level_entry_names_the_kernel_of_its_linux_line() {
    let simple = grub(&state(LIMA_SIMPLE), &lima_menu(), &installed_kernels());
    let by_position = grub(&state("0"), &lima_menu(), &installed_kernels());

    assert_eq!(simple, Some(names("linux-lts")));
    assert_eq!(by_position, Some(names("linux-lts")));
}

#[test]
fn the_top_level_entry_names_no_kernel_that_is_not_installed() {
    let only_garuda = [kernel("linux-garuda", true), kernel("linux-lts", false)];

    assert_eq!(grub(&state("0"), &lima_menu(), &only_garuda), None);
}

#[test]
fn the_installed_top_level_entry_names_linux_garuda() {
    let state = GrubState::read(INSTALLED_DEFAULT_GRUB);
    let menu = GrubMenu::for_platform(INSTALLED_GRUB_CFG, GrubPlatform::Efi);
    let found = grub(&state, &menu, &[kernel("linux-garuda", true)]);

    assert_eq!(found, Some(names("linux-garuda")));
}

/// Edge case: the kernels are renamed to `linux-cachyos` and
/// `linux-cachyos-rc` in this test, so `linux-cachyos` has to skip the
/// entries of `linux-cachyos-rc`.
#[test]
fn linux_cachyos_never_takes_the_entries_of_linux_cachyos_rc() {
    let text = LIMA_GRUB_CFG
        .replace("linux-garuda", "linux-cachyos")
        .replace("linux-lts", "linux-cachyos-rc");

    let menu = GrubMenu::for_platform(&text, GrubPlatform::Efi);
    let rc = LIMA_LTS.replace("linux-lts", "linux-cachyos-rc");
    let stable = LIMA_GARUDA.replace("linux-garuda", "linux-cachyos");
    let both = [
        kernel("linux-cachyos", true),
        kernel("linux-cachyos-rc", true),
    ];

    assert_eq!(
        grub(&state(&rc), &menu, &both),
        Some(names("linux-cachyos-rc"))
    );
    assert_eq!(
        grub(&state(&stable), &menu, &both),
        Some(names("linux-cachyos"))
    );
    assert_eq!(
        grub(&state(&rc), &menu, &[kernel("linux-cachyos", true)]),
        None
    );
}

#[test]
fn the_installed_capture_names_its_kernel_by_id() {
    let menu = GrubMenu::for_platform(INSTALLED_GRUB_CFG, GrubPlatform::Efi);

    // The capture's own ids, which its UUIDs are replaced in.
    let id = "gnulinux-advanced-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx>gnulinux-linux-garuda-advanced-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
    let text =
        INSTALLED_DEFAULT_GRUB.replacen("GRUB_DEFAULT=0", &format!("GRUB_DEFAULT='{id}'"), 1);

    assert_eq!(
        grub(
            &GrubState::read(&text),
            &menu,
            &[kernel("linux-garuda", true)]
        ),
        Some(names("linux-garuda"))
    );
}

fn releases() -> KernelReleases {
    KernelReleases(std::collections::BTreeMap::from([(
        package("linux-garuda"),
        release("7.3.1-1-garuda"),
    )]))
}

fn image_menu(default: Option<&str>) -> SdBootMenu {
    SdBootMenu {
        default: default.map(|id| toolbox_plan::LoaderEntryId::new(id).unwrap()),
        entries: vec![
            LoaderEntry::uki("garuda-7.3.1-1-garuda.efi", "", Some("7.3.1-1-garuda")).unwrap(),
            LoaderEntry::uki(
                "garuda-7.3.1-1-garuda-fallback.efi",
                "",
                Some("7.3.1-1-garuda"),
            )
            .unwrap(),
        ],
        ..SdBootMenu::default()
    }
}

#[test]
fn the_entry_bootctl_marks_names_the_kernel_of_its_release() {
    let menu = SdBootMenu {
        default: bootctl_default(IMAGE_BOOTCTL_LIST),
        ..image_menu(None)
    };

    assert_eq!(
        sdboot(&menu, &releases(), &[kernel("linux-garuda", true)]),
        Some(names("linux-garuda"))
    );
}

#[test]
fn a_release_no_installed_kernel_has_names_no_kernel() {
    let menu = image_menu(Some("garuda-7.3.1-1-garuda.efi"));

    assert_eq!(
        sdboot(
            &menu,
            &KernelReleases::default(),
            &[kernel("linux-garuda", true)]
        ),
        None
    );
}

#[test]
fn an_entry_without_a_uname_names_no_kernel() {
    let menu = SdBootMenu {
        default: bootctl_default(BARE_METAL_BOOTCTL_LIST),
        entries: vec![
            LoaderEntry::uki(
                "nixos-generation-235-vn3qitxasms3w4tyril3pz5323hxkh25hj66ok6l73b5lotswyzq.efi",
                "",
                None,
            )
            .unwrap(),
        ],
        ..SdBootMenu::default()
    };

    assert_eq!(
        sdboot(&menu, &releases(), &[kernel("linux-garuda", true)]),
        None
    );
}

#[test]
fn without_bootctl_the_default_is_unknown() {
    assert_eq!(
        sdboot(
            &image_menu(None),
            &releases(),
            &[kernel("linux-garuda", true)]
        ),
        None
    );
}
