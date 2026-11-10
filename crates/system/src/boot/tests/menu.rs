use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use tempfile::TempDir;
use toolbox_plan::protocol::{ReadKind, encode};
use toolbox_plan::{GrubMenu, GrubPlatform, PackageName};

use super::super::{BootMenuSource as _, Menu, MenuRead, SystemBootMenu};
use crate::HelperReads;
use crate::pkexec::Refusal;
use crate::test_support::{fake_program, runnable_script, running_as_root, write};

/// A used Garuda Mokka install, every UUID masked.
const GRUB_CFG: &str = toolbox_plan::testing::grub::INSTALLED_GRUB_CFG;
/// The Lima image with `linux-garuda` and `linux-lts`.
const LIMA_GRUB_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;
const MASKED: &str = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";

/// Stands in for `pkexec <helper> read grub-menu` with a fixed program.
struct FakeHelper(PathBuf);

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

/// Prints a captured complaint of pkexec. The text goes through a file,
/// as some complaints hold quotes.
fn complaining(stderr: &str, code: &str) -> (TempDir, Arc<dyn HelperReads>) {
    let folder = tempfile::tempdir().unwrap();
    let program = folder.path().join("pkexec");

    fs::write(folder.path().join("stderr"), stderr).unwrap();
    runnable_script(
        &program,
        &format!(
            "cat '{}' >&2\nexit {}",
            folder.path().join("stderr").display(),
            code.trim()
        ),
    );

    (folder, Arc::new(FakeHelper(program)))
}

fn unreadable_menu(root: &Path) {
    let config = root.join("boot/grub/grub.cfg");

    write(root, "boot/grub/grub.cfg", GRUB_CFG);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
}

fn known(config: &str) -> Menu {
    Menu::Read(MenuRead::Known(GrubMenu::for_platform(
        config,
        GrubPlatform::Efi,
    )))
}

fn ids(read: MenuRead) -> Vec<String> {
    let MenuRead::Known(menu) = read else {
        panic!("the menu is readable");
    };

    menu.entries
        .into_iter()
        .map(|entry| entry.id.to_string())
        .collect()
}

#[test]
fn a_readable_menu_is_parsed_without_the_helper() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "boot/grub/grub.cfg", GRUB_CFG);

    let ids = ids(SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default()));

    assert_eq!(ids.len(), 9, "no UEFI firmware entry on BIOS");
    assert_eq!(
        ids[..3],
        [
            format!("gnulinux-simple-{MASKED}"),
            format!("gnulinux-advanced-{MASKED}"),
            format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}"),
        ]
    );
}

/// GRUB shows the `efi` branches only when UEFI started the system.
#[test]
fn the_firmware_folder_decides_the_branches_of_the_menu() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "boot/grub/grub.cfg", GRUB_CFG);
    fs::create_dir_all(root.path().join("sys/firmware/efi")).unwrap();

    let MenuRead::Known(menu) = SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default())
    else {
        panic!("the menu is readable");
    };

    assert_eq!(menu.entries.len(), 10);
    assert_eq!(menu.find("2").unwrap().id.as_str(), "uefi-firmware");
    assert_eq!(menu.find("5").unwrap().title, "Shutdown");
    assert_eq!(super::super::grub_platform(root.path()), GrubPlatform::Efi);
}

/// A title in Latin-1 from an old `40_custom` must not hide the menu. The
/// test puts such a comment before the install's menu.
#[test]
fn a_stray_byte_in_the_menu_hides_no_entry() {
    let root = tempfile::tempdir().unwrap();

    write(
        root.path(),
        "boot/grub/grub.cfg",
        [b"# caf\xe9\n".as_slice(), GRUB_CFG.as_bytes()].concat(),
    );

    assert_eq!(
        ids(SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default())).len(),
        9
    );
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
/// test then has nothing to show). The helper's answer is the Lima menu
/// in the helper's own encoding.
#[test]
fn an_unreadable_menu_is_asked_of_the_helper() {
    let root = tempfile::tempdir().unwrap();
    unreadable_menu(root.path());

    if running_as_root() {
        return;
    }

    let lima = GrubMenu::for_platform(LIMA_GRUB_CFG, GrubPlatform::Efi);
    let (_folder, program) = fake_program(&encode(&lima).unwrap(), "", 0);
    let helper: Arc<dyn HelperReads> = Arc::new(FakeHelper(program));

    assert_eq!(
        SystemBootMenu::new(root.path(), helper).menu(&Arc::default()),
        MenuRead::Known(lima)
    );

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

    assert!(matches!(
        SystemBootMenu::new(root.path(), no_helper()).menu(&Arc::default()),
        MenuRead::Unknown(_)
    ));
}

/// Plasma's agent answers a Cancel like a wrong password. Exit code 126
/// is what pkexec documents for a dismissed dialog, no capture has it.
#[test]
fn a_refusing_pkexec_is_a_refusal_of_its_own() {
    let root = tempfile::tempdir().unwrap();
    unreadable_menu(root.path());

    if running_as_root() {
        return;
    }

    for (name, stderr, code, refusal) in [
        ("documented", "", "126", Refusal::Dismissed),
        (
            "dismissed",
            include_str!("../../pkexec/fixtures/dismissed.stderr"),
            include_str!("../../pkexec/fixtures/dismissed.code"),
            Refusal::Denied,
        ),
        (
            "not-authorized",
            include_str!("../../pkexec/fixtures/not-authorized.stderr"),
            include_str!("../../pkexec/fixtures/not-authorized.code"),
            Refusal::Denied,
        ),
        (
            "no-agent",
            include_str!("../../pkexec/fixtures/no-agent.stderr"),
            include_str!("../../pkexec/fixtures/no-agent.code"),
            Refusal::NoAgent,
        ),
        (
            "missing-program",
            include_str!("../../pkexec/fixtures/missing-program.stderr"),
            include_str!("../../pkexec/fixtures/missing-program.code"),
            Refusal::HelperMissing,
        ),
    ] {
        let (_folder, refusing) = complaining(stderr, code);

        assert_eq!(
            SystemBootMenu::new(root.path(), refusing).menu(&Arc::default()),
            MenuRead::Refused(refusal),
            "{name}"
        );
    }
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
