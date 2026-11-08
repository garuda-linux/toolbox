mod menu;

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::PlymouthTheme;
use toolbox_state::boot::{PlymouthState, VirtualMachine};

use super::{BootSource as _, SystemBoot};
use crate::ProbeError;
use crate::testing::{fake_program, write};

const DEFAULT_GRUB: &str = include_str!("../../../plan/src/grub/fixtures/installed/default-grub");

const PLYMOUTHD_CONF: &str = include_str!("fixtures/plymouthd.conf");
const PLYMOUTHD_DEFAULTS: &str = include_str!("fixtures/plymouthd.defaults");
/// `ls -1 ./*/` in the VM's theme folder.
const THEME_FILES: &str = include_str!("fixtures/plymouth-theme-files");

const DETECT_VIRT: &str = include_str!("fixtures/detect-virt.stdout");
/// The laptop answered `none` and exited with 1.
const BARE_METAL: &str = include_str!("fixtures/bare-metal/detect-virt");

fn theme(name: &str) -> PlymouthTheme {
    PlymouthTheme::new(name).unwrap()
}

/// The VM's theme folder: every listed file, empty, as only the names
/// count.
fn install_themes(root: &Path) {
    let themes = root.join("usr/share/plymouth/themes");
    let mut folder = themes.clone();

    for line in THEME_FILES.lines().filter(|line| !line.is_empty()) {
        if let Some(name) = line
            .strip_prefix("./")
            .and_then(|line| line.strip_suffix("/:"))
        {
            folder = themes.join(name);
            fs::create_dir_all(&folder).unwrap();
        } else {
            fs::write(folder.join(line), "").unwrap();
        }
    }
}

fn plymouth_system() -> TempDir {
    let root = tempfile::tempdir().unwrap();

    install_themes(root.path());
    write(root.path(), "etc/plymouth/plymouthd.conf", PLYMOUTHD_CONF);
    write(
        root.path(),
        "usr/share/plymouth/plymouthd.defaults",
        PLYMOUTHD_DEFAULTS,
    );
    root
}

#[test]
fn the_grub_file_is_read_into_its_state() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "etc/default/grub", DEFAULT_GRUB);

    let state = SystemBoot::new(root.path(), "/nonexistent")
        .grub()
        .unwrap()
        .0;

    assert_eq!(state.default, "0");
    assert_eq!(state.timeout, Some(5));
    assert_eq!(
        state.parameters.map(|parameters| parameters.to_string()),
        Some("quiet loglevel=3".to_owned())
    );
}

#[test]
fn a_system_without_the_grub_file_has_no_grub() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemBoot::new(root.path(), "/nonexistent").grub(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn plymouth_themes_are_the_folders_with_a_theme_file_of_their_name() {
    let root = plymouth_system();
    let themes: Vec<PlymouthTheme> = [
        "bgrt",
        "details",
        "dr460nized",
        "fade-in",
        "glow",
        "script",
        "solar",
        "spinfinity",
        "spinner",
        "text",
        "tribar",
    ]
    .map(theme)
    .into();

    assert_eq!(
        SystemBoot::new(root.path(), "/nonexistent").plymouth(),
        Ok(PlymouthState {
            themes,
            current: Some(theme("dr460nized")),
        })
    );
}

/// The VM's themes plus a folder without a theme file, a plain file and a
/// theme whose name is an option, with the configured theme written with
/// blanks around `=`.
#[test]
fn odd_entries_of_the_theme_folder_are_no_themes() {
    let root = plymouth_system();
    let themes = root.path().join("usr/share/plymouth/themes");

    fs::create_dir_all(themes.join("empty")).unwrap();
    fs::write(themes.join("README"), "").unwrap();
    fs::create_dir_all(themes.join("-R")).unwrap();
    fs::write(themes.join("-R/-R.plymouth"), "").unwrap();
    write(
        root.path(),
        "etc/plymouth/plymouthd.conf",
        PLYMOUTHD_CONF.replacen("Theme=dr460nized", "Theme = spinner", 1),
    );

    let state = SystemBoot::new(root.path(), "/nonexistent")
        .plymouth()
        .unwrap();

    assert_eq!(state.themes.len(), 11);
    assert_eq!(state.current, Some(theme("spinner")));
}

/// What `plymouth-set-default-theme` does: a configured theme that is
/// not installed does not count, the distribution's default does. The
/// test removes the `dr460nized` folder the VM's config names.
#[test]
fn the_distributions_theme_counts_when_the_configured_one_is_gone() {
    let root = plymouth_system();
    fs::remove_dir_all(root.path().join("usr/share/plymouth/themes/dr460nized")).unwrap();

    let boot = SystemBoot::new(root.path(), "/nonexistent");

    assert_eq!(boot.plymouth().unwrap().current, Some(theme("bgrt")));

    fs::remove_file(root.path().join("usr/share/plymouth/plymouthd.defaults")).unwrap();
    assert_eq!(boot.plymouth().unwrap().current, None);
}

#[test]
fn without_the_theme_folder_plymouth_is_not_installed() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemBoot::new(root.path(), "/nonexistent").plymouth(),
        Err(ProbeError::Unavailable)
    );
}

/// The last program answers like the laptop but exits with 0.
#[test]
fn a_virtual_machine_is_what_detect_virt_names_with_exit_zero() {
    let root = tempfile::tempdir().unwrap();
    let (_kvm_folder, kvm) = fake_program(DETECT_VIRT, "", 0);
    let (_none_folder, none) = fake_program(BARE_METAL, "", 1);
    let (_odd_folder, odd) = fake_program(BARE_METAL, "", 0);

    assert_eq!(
        SystemBoot::new(root.path(), kvm).virtual_machine(),
        Ok(VirtualMachine(true))
    );
    assert_eq!(
        SystemBoot::new(root.path(), none).virtual_machine(),
        Ok(VirtualMachine(false))
    );
    assert_eq!(
        SystemBoot::new(root.path(), odd).virtual_machine(),
        Ok(VirtualMachine(false))
    );
    assert_eq!(
        SystemBoot::new(root.path(), "/nonexistent/systemd-detect-virt").virtual_machine(),
        Err(ProbeError::Unavailable)
    );
}
