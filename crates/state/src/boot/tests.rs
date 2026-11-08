use super::BootLoader;

/// Files of the Garuda Lima image: the raw EFI variable from efivarfs and
/// `/etc/kernel/install.conf`.
const LOADER_INFO: &[u8] = include_bytes!("fixtures/LoaderInfo");
const INSTALL_CONF: &str = include_str!("fixtures/install.conf");

#[test]
fn the_image_is_found_by_its_loader_info() {
    assert_eq!(
        BootLoader::detect(Some(LOADER_INFO), false, None),
        BootLoader::SystemdBoot
    );
}

/// A GRUB left on a disk does not matter while systemd-boot started the
/// machine.
#[test]
fn the_loader_that_booted_wins_over_files() {
    assert_eq!(
        BootLoader::detect(Some(LOADER_INFO), true, None),
        BootLoader::SystemdBoot
    );
}

/// GRUB sets no `LoaderInfo`. The image's `install.conf` alone, and a
/// system with `/etc/default/grub` alone.
#[test]
fn without_the_variable_the_files_decide() {
    assert_eq!(
        BootLoader::detect(None, false, Some(INSTALL_CONF)),
        BootLoader::SystemdBoot
    );
    assert_eq!(BootLoader::detect(None, true, None), BootLoader::Grub);
    assert_eq!(BootLoader::detect(None, false, None), BootLoader::Unknown);
}

/// The image's `install.conf` with the layout systemd uses for Type 1
/// entries, which the page does not handle.
#[test]
fn another_kernel_install_layout_is_no_systemd_boot_of_the_page() {
    let bls = INSTALL_CONF.replace("layout=uki", "layout=bls");

    assert_eq!(
        BootLoader::detect(None, false, Some(&bls)),
        BootLoader::Unknown
    );
}

/// The image's variable without its attribute bytes, or cut in its text.
#[test]
fn a_broken_variable_falls_back_to_the_files() {
    assert_eq!(
        BootLoader::detect(LOADER_INFO.get(..3), true, None),
        BootLoader::Grub
    );
    assert_eq!(
        BootLoader::detect(LOADER_INFO.get(..9), false, Some(INSTALL_CONF)),
        BootLoader::SystemdBoot
    );
}
