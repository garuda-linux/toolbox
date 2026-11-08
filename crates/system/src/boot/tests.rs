use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use toolbox_plan::protocol::ReadKind;
use toolbox_plan::{GrubMenu, PackageName, PlymouthTheme};
use toolbox_state::boot::{PlymouthState, VirtualMachine};

use super::{BootMenuSource as _, BootSource as _, Menu, MenuRead, SystemBoot, SystemBootMenu};
use crate::pkexec::Refusal;
use crate::testing::{fake_program, running_as_root, write};
use crate::{HelperReads, ProbeError};

const GRUB_CFG: &str = "\
password_pbkdf2 admin grub.pbkdf2.sha512.10000.ABCDEF
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
	linux /vmlinuz-linux-zen root=UUID=1 rw
}
submenu 'Advanced options' $menuentry_id_option 'gnulinux-advanced-1' {
	menuentry 'Garuda Linux, with Linux linux-lts' $menuentry_id_option 'gnulinux-lts-1' {
	}
}
";

/// Stands in for `pkexec <helper> read grub-menu` with a fixed program.
struct FakeHelper(std::path::PathBuf);

impl HelperReads for FakeHelper {
    fn read_command(&self, kind: ReadKind) -> Command {
        assert_eq!(kind, ReadKind::GrubMenu);
        Command::new(&self.0)
    }
}

/// An open password dialog that nobody answers.
struct SlowHelper;

impl HelperReads for SlowHelper {
    fn read_command(&self, _kind: ReadKind) -> Command {
        let mut command = Command::new("/bin/sh");

        command.args(["-c", "sleep 30"]);
        command
    }
}

fn no_helper() -> Arc<dyn HelperReads> {
    Arc::new(FakeHelper("/nonexistent/helper".into()))
}

fn theme(name: &str) -> PlymouthTheme {
    PlymouthTheme::new(name).unwrap()
}

fn unreadable_menu(root: &Path) {
    let config = root.join("boot/grub/grub.cfg");

    write(root, "boot/grub/grub.cfg", GRUB_CFG);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
}

fn install_theme(root: &Path, name: &str) {
    let folder = root.join("usr/share/plymouth/themes").join(name);

    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join(format!("{name}.plymouth")),
        "[Plymouth Theme]\n",
    )
    .unwrap();
}

#[test]
fn the_grub_file_is_read_into_its_state() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "etc/default/grub",
        "GRUB_DEFAULT=saved\nGRUB_TIMEOUT=3\nGRUB_CMDLINE_LINUX_DEFAULT='quiet splash'\n",
    );

    let state = SystemBoot::new(root.path(), "/nonexistent")
        .grub()
        .unwrap()
        .0;

    assert_eq!(state.default, "saved");
    assert_eq!(state.timeout, Some(3));
    assert!(state.parameters.unwrap().contains("splash"));
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
    let root = tempfile::tempdir().unwrap();
    install_theme(root.path(), "spinner");
    install_theme(root.path(), "bgrt");
    fs::create_dir_all(root.path().join("usr/share/plymouth/themes/empty")).unwrap();
    write(root.path(), "usr/share/plymouth/themes/README", "no theme");
    install_theme(root.path(), "-R");
    write(
        root.path(),
        "etc/plymouth/plymouthd.conf",
        "# Administrator customizations\n[Daemon]\n#Theme=fade-in\nTheme = spinner\n",
    );
    write(
        root.path(),
        "usr/share/plymouth/plymouthd.defaults",
        "[Daemon]\nTheme=bgrt\n",
    );

    assert_eq!(
        SystemBoot::new(root.path(), "/nonexistent").plymouth(),
        Ok(PlymouthState {
            themes: vec![theme("bgrt"), theme("spinner")],
            current: Some(theme("spinner")),
        })
    );
}

/// What `plymouth-set-default-theme` does: a configured theme that is
/// not installed does not count, the distribution's default does.
#[test]
fn the_distributions_theme_counts_when_the_configured_one_is_gone() {
    let root = tempfile::tempdir().unwrap();
    install_theme(root.path(), "bgrt");
    write(
        root.path(),
        "etc/plymouth/plymouthd.conf",
        "[Daemon]\nTheme=removed-theme\n",
    );
    write(
        root.path(),
        "usr/share/plymouth/plymouthd.defaults",
        "[Daemon]\nTheme=bgrt\n",
    );

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

#[test]
fn a_virtual_machine_is_what_detect_virt_names_with_exit_zero() {
    let root = tempfile::tempdir().unwrap();
    let (_kvm_folder, kvm) = fake_program("kvm\n", "", 0);
    let (_none_folder, none) = fake_program("none\n", "", 1);
    let (_odd_folder, odd) = fake_program("none\n", "", 0);

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

#[test]
fn a_readable_menu_is_parsed_without_the_helper() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "boot/grub/grub.cfg", GRUB_CFG);

    let MenuRead::Known(menu) = SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default())
    else {
        panic!("the menu is readable");
    };

    assert_eq!(
        menu.entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        [
            "gnulinux-simple-1",
            "gnulinux-advanced-1",
            "gnulinux-advanced-1>gnulinux-lts-1"
        ]
    );
}

/// A title in Latin-1 from an old `40_custom` must not hide the menu.
#[test]
fn a_stray_byte_in_the_menu_hides_no_entry() {
    let root = tempfile::tempdir().unwrap();

    write(
        root.path(),
        "boot/grub/grub.cfg",
        [b"# caf\xe9\n".as_slice(), GRUB_CFG.as_bytes()].concat(),
    );

    let MenuRead::Known(menu) = SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default())
    else {
        panic!("the menu is readable");
    };

    assert_eq!(menu.entries.len(), 3);
}

#[test]
fn no_grub_config_is_an_answer_of_its_own() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default()),
        MenuRead::Missing
    );
}

/// As a user that may not read `grub.cfg` (root reads everything, so the
/// test then has nothing to show).
#[test]
fn an_unreadable_menu_is_asked_of_the_helper() {
    let root = tempfile::tempdir().unwrap();
    unreadable_menu(root.path());

    if running_as_root() {
        return;
    }

    let answer = r#"{"entries":[{"id":"from-helper","title":"From the helper","position":"0","submenu":false,"parent":null}]}"#;
    let (_folder, program) = fake_program(answer, "", 0);
    let helper: Arc<dyn HelperReads> = Arc::new(FakeHelper(program));

    let MenuRead::Known(menu) = SystemBootMenu::new(root.path(), helper).menu(&Arc::default())
    else {
        panic!("the helper answered a menu");
    };

    assert_eq!(menu.entries[0].title, "From the helper");

    for (stdout, code) in [
        ("not json", 0),
        (r#"{"entries":[{"id":"a$b"}]}"#, 0),
        ("", 1),
    ] {
        let (_other, refused) = fake_program(stdout, "", code);
        let refusing: Arc<dyn HelperReads> = Arc::new(FakeHelper(refused));

        assert!(matches!(
            SystemBootMenu::new(root.path(), refusing).menu(&Arc::default()),
            MenuRead::Unknown(_)
        ));
    }

    for (stderr, code, refusal) in [
        ("", 126, Refusal::Dismissed),
        ("Not authorized", 127, Refusal::Denied),
        ("No authentication agent found.", 127, Refusal::NoAgent),
        (
            "Cannot run program /helper: No such file or directory",
            127,
            Refusal::HelperMissing,
        ),
    ] {
        let (_other, refused) = fake_program("", stderr, code);
        let refusing: Arc<dyn HelperReads> = Arc::new(FakeHelper(refused));

        assert_eq!(
            SystemBootMenu::new(root.path(), refusing).menu(&Arc::default()),
            MenuRead::Refused(refusal),
            "{stderr}"
        );
    }

    assert!(matches!(
        SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default()),
        MenuRead::Unknown(_)
    ));
}

#[test]
fn a_set_flag_ends_the_wait_for_the_helper_as_cancelled() {
    let root = tempfile::tempdir().unwrap();
    unreadable_menu(root.path());

    if running_as_root() {
        return;
    }

    let started = std::time::Instant::now();

    assert_eq!(
        SystemBootMenu::new(root.path(), Arc::new(SlowHelper))
            .menu(&Arc::new(std::sync::atomic::AtomicBool::new(true))),
        MenuRead::Cancelled
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(20));
}

#[test]
fn the_menu_states_have_codes_and_no_entries_unless_known() {
    for (menu, code) in [
        (Menu::NotAsked, "not-asked"),
        (Menu::Loading, "loading"),
        (Menu::Read(MenuRead::Missing), "missing"),
        (
            Menu::Read(MenuRead::Unknown("broken".to_owned())),
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

    let known = Menu::Read(MenuRead::Known(GrubMenu::from_config(GRUB_CFG)));
    assert_eq!(known.as_str(), "known");
}

#[test]
fn a_kernel_names_its_menu_entry_once_the_menu_is_read() {
    let kernel = |name: &str| PackageName::new(name).unwrap();
    let known = Menu::Read(MenuRead::Known(GrubMenu::from_config(GRUB_CFG)));

    assert_eq!(Menu::NotAsked.kernel_entry(&kernel("linux-zen")), "");
    assert_eq!(known.kernel_entry(&kernel("no-such-kernel")), "");

    let with_zen = GRUB_CFG.replace(
        "\t}\n}\n",
        "\t}\n\tmenuentry 'Garuda Linux, with Linux linux-zen' $menuentry_id_option 'gnulinux-linux-zen-advanced-1' {\n\t}\n}\n",
    );

    let zen_known = Menu::Read(MenuRead::Known(GrubMenu::from_config(&with_zen)));

    assert_eq!(
        zen_known.kernel_entry(&kernel("linux-zen")),
        "gnulinux-advanced-1>gnulinux-linux-zen-advanced-1"
    );
}
