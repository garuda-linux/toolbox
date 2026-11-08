use std::fs;
use std::path::Path;

use toolbox_plan::protocol::{MAX_READ_OUTPUT_BYTES, ProgramAnswer, ReadKind, decode};
use toolbox_plan::{ShadowLocks, UserName};

use super::answer;
use crate::testing::SHADOW;

/// The `grub.cfg` of a Garuda install in QEMU, UUIDs replaced by `x`.
const GRUB_CFG: &str = include_str!("../../../../crates/plan/src/grub/fixtures/installed/grub.cfg");

/// A GRUB password as `grub-mkpasswd-pbkdf2` makes one, which the
/// captured file has not.
const GRUB_PASSWORD: &str = "\
set superusers=\"admin\"
password_pbkdf2 admin grub.pbkdf2.sha512.10000.5A9F0C3E7B1D.99AABBCCDDEE
";

fn shadow(root: &Path, content: &[u8]) {
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/shadow"), content).unwrap();
}

#[test]
fn shadow_locks_answers_names_and_flags_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    shadow(dir.path(), SHADOW.as_bytes());

    let line = answer(dir.path(), ReadKind::ShadowLocks).unwrap();

    assert_eq!(line.matches('\n').count(), 1);

    let locks: ShadowLocks = decode(line.trim_end()).unwrap();
    let locked = |name: &str| {
        let user = UserName::new(name).unwrap();

        locks
            .locks
            .iter()
            .find(|lock| lock.user == user)
            .map(|lock| lock.locked)
    };

    assert_eq!(locks.locks.len(), SHADOW.lines().count());
    assert_eq!(locked("root"), Some(false));
    assert_eq!(locked("nopass"), Some(false));
    assert_eq!(locked("bin"), Some(true));
    assert_eq!(locked("user"), Some(true));
    assert_eq!(locked("guest"), Some(true));
}

#[test]
fn the_answer_holds_no_hash() {
    let dir = tempfile::tempdir().unwrap();
    shadow(dir.path(), SHADOW.as_bytes());

    let line = answer(dir.path(), ReadKind::ShadowLocks).unwrap();

    for part in ["$", "j9T", "xxxx", "20765", "14871"] {
        assert!(!line.contains(part), "{part} in {line}");
    }
}

#[test]
fn a_missing_or_binary_shadow_is_an_error() {
    let dir = tempfile::tempdir().unwrap();

    answer(dir.path(), ReadKind::ShadowLocks).unwrap_err();
    shadow(dir.path(), &[b'a', 0xff, 0xfe, b'\n']);
    answer(dir.path(), ReadKind::ShadowLocks).unwrap_err();
}

/// Returns once it can run: a test thread that forks while the script is
/// open for writing keeps that descriptor until its child execs, and
/// running the script meanwhile fails with "Text file busy".
fn program(root: &Path, path: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt as _;

    let file = root.join(path);

    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        format!("#!/bin/sh\n[ -n \"$TOOLBOX_WARM_UP\" ] && exit 0\n{script}\n"),
    )
    .unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();

    for _ in 0_u8..100_u8 {
        match std::process::Command::new(&file)
            .env("TOOLBOX_WARM_UP", "1")
            .output()
        {
            Err(error) if error.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(_) | Err(_) => break,
        }
    }
}

fn ran(root: &Path, kind: ReadKind) -> ProgramAnswer {
    ProgramAnswer::parse(answer(root, kind).unwrap().trim_end()).unwrap()
}

/// What the program printed and whether its start was cut.
fn printed(root: &Path, kind: ReadKind) -> (String, bool) {
    match ran(root, kind) {
        ProgramAnswer::Ran {
            output,
            errors: _,
            code: _,
            truncated,
        } => (output, truncated),
        ProgramAnswer::Missing => panic!("{} did not run", kind.as_str()),
    }
}

#[test]
fn dmesg_runs_the_fixed_program_with_colours_as_its_only_argument() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/dmesg",
        "echo \"dmesg $*|$SYSTEMD_COLORS|\"",
    );

    assert_eq!(
        ran(dir.path(), ReadKind::Dmesg),
        ProgramAnswer::Ran {
            output: "dmesg --color=always||\n".to_owned(),
            errors: String::new(),
            code: Some(0_i32),
            truncated: false,
        }
    );
}

#[test]
fn the_journal_runs_journalctl_with_its_fixed_arguments_and_sixteen_colours() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/journalctl",
        "echo \"$0|$*|$SYSTEMD_COLORS\"",
    );

    let (output, _) = printed(dir.path(), ReadKind::Journal);

    assert_eq!(
        output,
        format!(
            "{}|-e --no-pager|16\n",
            dir.path().join("usr/bin/journalctl").display()
        )
    );
}

#[test]
fn a_program_gets_a_clean_environment_and_no_input() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/dmesg",
        "read -r line || echo 'no input'; echo \"$PATH|$LANG|${HOME:-no home}\"",
    );

    let (output, _) = printed(dir.path(), ReadKind::Dmesg);

    assert_eq!(
        output,
        "no input\n/usr/local/sbin:/usr/local/bin:/usr/bin|C.UTF-8|no home\n"
    );
}

#[test]
fn a_failing_program_is_an_answer_with_its_errors_and_code() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/dmesg",
        "echo 'dmesg: read kernel buffer failed: Operation not permitted' >&2; exit 1",
    );

    assert_eq!(
        ran(dir.path(), ReadKind::Dmesg),
        ProgramAnswer::Ran {
            output: String::new(),
            errors: "dmesg: read kernel buffer failed: Operation not permitted\n".to_owned(),
            code: Some(1_i32),
            truncated: false,
        }
    );
}

#[test]
fn a_missing_program_is_answered_as_missing() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(
        answer(dir.path(), ReadKind::Journal).unwrap(),
        "{\"result\":\"missing\"}\n"
    );
}

#[test]
fn a_long_output_is_cut_to_its_last_two_mebibytes_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/journalctl",
        "i=0; while [ $i -lt 40000 ]; do echo \"$i ................................................................................\"; i=$((i+1)); done",
    );

    let line = answer(dir.path(), ReadKind::Journal).unwrap();

    assert_eq!(line.matches('\n').count(), 1, "the answer is not one line");

    let ProgramAnswer::Ran {
        output,
        errors: _,
        code: _,
        truncated,
    } = ProgramAnswer::parse(line.trim_end()).unwrap()
    else {
        panic!("journalctl did not run");
    };

    assert!(truncated);
    assert!(output.len() <= MAX_READ_OUTPUT_BYTES);
    assert!(output.len() > MAX_READ_OUTPUT_BYTES - 200);
    assert!(output.ends_with(&format!("39999 {}\n", ".".repeat(80))));
    assert!(!output.starts_with('.'), "the first line is cut");
}

fn grub_cfg(root: &Path, content: &[u8]) {
    fs::create_dir_all(root.join("boot/grub")).unwrap();
    fs::write(root.join("boot/grub/grub.cfg"), content).unwrap();
}

#[test]
fn the_grub_menu_answers_titles_and_ids_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    grub_cfg(dir.path(), GRUB_CFG.as_bytes());
    fs::create_dir_all(dir.path().join("sys/firmware/efi")).unwrap();

    let line = answer(dir.path(), ReadKind::GrubMenu).unwrap();
    let menu = toolbox_plan::GrubMenu::parse(line.trim_end()).unwrap();
    let ids: Vec<&str> = menu.entries.iter().map(|entry| entry.id.as_str()).collect();

    let masked = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
    let advanced = format!("gnulinux-advanced-{masked}");

    assert_eq!(line.matches('\n').count(), 1);
    assert_eq!(
        ids,
        [
            format!("gnulinux-simple-{masked}"),
            advanced.clone(),
            format!("{advanced}>gnulinux-linux-garuda-advanced-{masked}"),
            format!("{advanced}>gnulinux-linux-garuda-fallback-{masked}"),
            format!("{advanced}>gnulinux-linux-garuda-recovery-{masked}"),
            "uefi-firmware".to_owned(),
            "Garuda Linux snapshots".to_owned(),
            "Memory Tester (memtest86+)".to_owned(),
            "Shutdown".to_owned(),
            "Restart".to_owned(),
        ]
    );
    assert_eq!(
        menu.entries[2].title,
        "Garuda Linux, with Linux linux-garuda"
    );
}

/// Without `/sys/firmware/efi` GRUB runs the `pc` branches, so the
/// firmware entry is missing and Shutdown moves up.
#[test]
fn the_grub_menu_of_a_bios_machine_has_no_firmware_entry() {
    let dir = tempfile::tempdir().unwrap();
    grub_cfg(dir.path(), GRUB_CFG.as_bytes());

    let line = answer(dir.path(), ReadKind::GrubMenu).unwrap();
    let menu = toolbox_plan::GrubMenu::parse(line.trim_end()).unwrap();

    assert!(
        menu.entries
            .iter()
            .all(|entry| entry.id.as_str() != "uefi-firmware")
    );
    assert_eq!(menu.find("4").unwrap().title, "Shutdown");
}

/// `grub.cfg` is root's alone because it can hold a password hash and
/// the kernel command line. Neither leaves the helper.
#[test]
fn the_grub_menu_answer_holds_nothing_else_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let protected = format!("{GRUB_PASSWORD}{GRUB_CFG}");
    grub_cfg(dir.path(), protected.as_bytes());

    let line = answer(dir.path(), ReadKind::GrubMenu).unwrap();

    for part in [
        "pbkdf2",
        "5A9F0C3E",
        "99AABB",
        "superusers",
        "admin",
        "vmlinuz",
        "root=UUID",
        "rootflags",
        "vt.default_red",
        "initramfs-linux",
    ] {
        assert!(!line.contains(part), "{part} in {line}");
    }
}

#[test]
fn a_missing_or_binary_grub_config_is_an_error() {
    let dir = tempfile::tempdir().unwrap();

    answer(dir.path(), ReadKind::GrubMenu).unwrap_err();
    grub_cfg(dir.path(), &[b'm', 0xff, 0xfe, b'\n']);
    answer(dir.path(), ReadKind::GrubMenu).unwrap_err();
}

#[test]
fn a_kind_without_a_program_runs_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let error = super::run_program(dir.path(), ReadKind::ShadowLocks).unwrap_err();

    assert_eq!(error.to_string(), "this kind runs no program");
}

#[test]
fn an_unstartable_program_is_an_error_not_an_answer() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("usr/bin/dmesg");
    fs::create_dir_all(program.parent().unwrap()).unwrap();
    fs::write(&program, "#!/bin/sh\nexit 3\n").unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o644)).unwrap();

    super::run_program(dir.path(), ReadKind::Dmesg).unwrap_err();
}

#[test]
fn an_oversized_shadow_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let shadow = dir.path().join("etc/shadow");
    fs::create_dir_all(shadow.parent().unwrap()).unwrap();
    let over = usize::try_from(super::MAX_FILE_BYTES).unwrap() + 1;

    fs::write(&shadow, vec![b'x'; over]).unwrap();

    let error = answer(dir.path(), ReadKind::ShadowLocks).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}
