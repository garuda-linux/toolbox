//! The built helper, driven over stdin and stdout like the app does.

mod common;
mod handmade;
mod oneshot;
mod person;

use std::error::Error;
use std::fs::OpenOptions;
use std::os::fd::OwnedFd;
use std::process::{Command, Stdio};

use common::{TestResult, events, helper, outcomes, request, run_helper};
use handmade::request_line;
use oneshot::output;
use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
use toolbox_plan::protocol::{Event, Mode, Outcome, PROTOCOL_VERSION, ProgramAnswer};
use toolbox_plan::{
    ConfigFilePath, ConfigText, ConfigWrite, DkmsBuild, DkmsModuleName, DkmsVersion, Fingerprint,
    KernelRelease, LocaleEntry, LocaleName, RootAction, StepId,
};

const SECRET: &str = "correct horse battery staple";

/// A one-step request that changes nothing on a system without orphans.
fn orphans(mode: Mode) -> Result<String, Box<dyn Error>> {
    request(mode, vec![RootAction::RemoveOrphans])
}

#[test]
fn a_dry_run_reports_without_root() -> TestResult {
    let output = helper(&["apply"], &orphans(Mode::DryRun)?)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        events(&output)?,
        [
            Event::Accepted,
            Event::Started {
                step: StepId::new(1)
            },
            Event::Finished {
                step: StepId::new(1),
                outcome: Outcome::Skipped
            },
            Event::Done,
        ]
    );
    Ok(())
}

#[test]
fn a_real_run_without_root_is_rejected() -> TestResult {
    let output = run_helper(
        person::command(&["apply"], true),
        &orphans(Mode::Run)?,
        None,
    )?;

    let first = events(&output)?.into_iter().next();

    assert!(!output.status.success());
    assert!(
        matches!(&first, Some(Event::Rejected { reason }) if reason.contains("root")),
        "{}",
        output.stderr
    );
    Ok(())
}

#[test]
fn an_invalid_package_name_is_rejected() -> TestResult {
    let line = request_line("run", r#"{"kind":"install-packages","packages":["-Syu"]}"#);

    let output = helper(&["apply"], &line)?;
    let first = events(&output)?.into_iter().next();

    assert!(!output.status.success());
    assert!(
        matches!(&first, Some(Event::Rejected { reason }) if reason.contains("package name")),
        "{}",
        output.stderr
    );
    Ok(())
}

/// The first refused name is quoted with at most 64 characters, and its
/// control characters are spelled out.
#[test]
fn a_refused_name_is_cut_to_64_characters_with_its_controls_spelled_out() -> TestResult {
    let long = request_line(
        "dry-run",
        &format!(
            r#"{{"kind":"install-packages","packages":["-{}"]}}"#,
            "a".repeat(200)
        ),
    );

    let controls = request_line(
        "dry-run",
        r#"{"kind":"install-packages","packages":["-\u0007x"]}"#,
    );

    let reason = |line: &str| -> Result<String, Box<dyn Error>> {
        match events(&helper(&["apply"], line)?)?.into_iter().next() {
            Some(Event::Rejected { reason }) => Ok(reason),
            other => Err(format!("not rejected: {other:?}").into()),
        }
    };

    let cut = reason(&long)?;
    assert!(cut.contains(&format!("-{}…", "a".repeat(63))), "{cut}");
    assert!(!cut.contains(&"a".repeat(64)), "{cut}");

    let spelled = reason(&controls)?;
    assert!(spelled.contains(r"\u{7}x"), "{spelled}");
    assert!(!spelled.contains('\u{7}'), "{spelled:?}");
    Ok(())
}

/// The helper reads a request from the app only. A terminal on stdin is
/// refused before any request is read.
#[test]
fn a_terminal_on_stdin_is_refused_before_a_request_is_read() -> TestResult {
    let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY)?;

    grantpt(&master)?;
    unlockpt(&master)?;

    let slave = ptsname(&master, Vec::new())?;
    let terminal = OpenOptions::new()
        .read(true)
        .write(true)
        .open(slave.to_str()?)?;

    let output = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"))
        .arg("apply")
        .stdin(Stdio::from(OwnedFd::from(terminal)))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("not from a terminal"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    Ok(())
}

/// The version is read before the rest of the request, so a request of
/// another version is refused whatever its actions are.
#[test]
fn a_request_of_another_protocol_version_is_refused() -> TestResult {
    let line = format!(
        "{{\"version\":{},\"mode\":\"dry-run\",\"steps\":[]}}\n",
        PROTOCOL_VERSION + 1
    );

    let output = helper(&["apply"], &line)?;

    assert!(
        matches!(events(&output)?.into_iter().next(), Some(Event::Rejected { reason }) if reason.contains("does not match")),
        "{}",
        output.stdout
    );
    Ok(())
}

#[test]
fn an_argument_that_is_no_utf8_prints_usage() -> TestResult {
    use std::os::unix::ffi::OsStrExt as _;

    let output = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"))
        .arg("fetch")
        .arg(std::ffi::OsStr::from_bytes(b"/etc/\xff"))
        .output()?;

    assert_eq!(output.status.code(), Some(2_i32));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
    Ok(())
}

#[test]
fn unknown_subcommands_print_usage() -> TestResult {
    let output = helper(&["shell"], "")?;

    assert_eq!(output.status.code(), Some(2_i32));
    assert!(output.stderr.contains("usage"));
    Ok(())
}

/// The locale and DKMS actions cross the wire to the real helper, which
/// accepts them and, in a dry run, touches nothing.
#[test]
fn locale_and_dkms_actions_are_accepted_in_a_dry_run() -> TestResult {
    let actions = vec![
        RootAction::SetLocaleEnabled {
            locale: LocaleEntry::parse("de_DE.UTF-8 UTF-8").ok_or("no entry")?,
            enabled: toolbox_plan::Toggle::On,
        },
        RootAction::GenerateLocales,
        RootAction::SetSystemLocale {
            locale: LocaleName::new("de_DE.UTF-8")?,
        },
        RootAction::InstallDkmsModule {
            build: DkmsBuild {
                module: DkmsModuleName::new("vboxhost")?,
                version: DkmsVersion::new("7.1.6_OSE")?,
                kernel: KernelRelease::new("6.12.9-1-lts")?,
            },
        },
    ];

    let output = helper(&["apply"], &request(Mode::DryRun, actions)?)?;
    let skipped = outcomes(&output)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(skipped, vec![Outcome::Skipped; 4]);
    Ok(())
}

#[test]
fn an_option_like_kernel_release_is_rejected() -> TestResult {
    let line = request_line(
        "dry-run",
        r#"{"kind":"install-dkms-module","build":{"module":"nvidia","version":"550","kernel":"--all"}}"#,
    );

    let output = helper(&["apply"], &line)?;
    let first = events(&output)?.into_iter().next();

    assert!(!output.status.success());
    assert!(
        matches!(&first, Some(Event::Rejected { reason }) if reason.contains("kernel release")),
        "{}",
        output.stdout
    );
    Ok(())
}

fn set_password() -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::SetPassword {
        user: toolbox_plan::UserName::new("ada")?,
        password: Some(toolbox_plan::Password::new(SECRET.to_owned())?),
    })
}

#[test]
fn a_dry_run_of_a_password_change_needs_and_prints_no_password() -> TestResult {
    let output = helper(&["apply"], &request(Mode::DryRun, vec![set_password()?])?)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(
        events(&output)?.get(2),
        Some(&Event::Finished {
            step: StepId::new(1),
            outcome: Outcome::Skipped
        })
    );
    assert!(!output.stdout.contains(SECRET) && !output.stderr.contains(SECRET));
    Ok(())
}

#[test]
fn a_rejected_run_does_not_repeat_the_secret_line() -> TestResult {
    let input = format!(
        "{}secret 1 {SECRET}\n",
        request(Mode::Run, vec![set_password()?])?
    );

    let output = run_helper(person::command(&["apply"], true), &input, None)?;

    assert!(!output.status.success());
    assert!(!output.stdout.contains(SECRET), "{}", output.stdout);
    assert!(!output.stderr.contains(SECRET), "{}", output.stderr);
    Ok(())
}

#[test]
fn read_knows_only_its_fixed_kinds() -> TestResult {
    for args in [
        vec!["read"],
        vec!["read", "/etc/shadow"],
        vec!["read", "shadow"],
        vec!["read", "shadow-locks", "extra"],
        vec!["logs"],
        vec!["logs", "dmesg", "--follow"],
        vec!["logs", "journal", "-u", "sshd"],
        vec!["logs", "journalctl"],
        vec!["logs", "/var/log/pacman.log"],
        // The password-less command reads no log and the log command no
        // file: polkit tells the two apart by this argument alone.
        vec!["read", "dmesg"],
        vec!["read", "journal"],
        vec!["logs", "shadow-locks"],
        // `read` and `logs` take a kind, never a path, and the menu is no
        // kind of `logs`.
        vec!["read", "/etc/fstab"],
        vec!["logs", "/etc/fstab"],
        vec!["read", "/boot/grub/grub.cfg"],
        vec!["logs", "grub-menu"],
        vec!["read", "grub-menu-x"],
    ] {
        let output = output(&args)?;

        assert_eq!(output.status.code(), Some(2_i32), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
    }

    Ok(())
}

#[test]
fn read_without_root_answers_nothing() -> TestResult {
    let output = person::output(&["read", "shadow-locks"], true)?;

    // NixOS and Arch both keep /etc/shadow from other users.
    assert_eq!(output.status.code(), Some(1_i32));
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot answer shadow-locks"));
    Ok(())
}

/// Started from a terminal or a script, stdin is no socket, so the helper
/// cannot tell who asks and answers nothing.
#[test]
fn shadow_locks_without_a_socket_are_refused() -> TestResult {
    let output = output(&["read", "shadow-locks"])?;

    assert_eq!(output.status.code(), Some(2_i32));
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert!(String::from_utf8_lossy(&output.stderr).contains("answers only the app"));
    Ok(())
}

/// Whatever the machine: the kernel log may be open, kept from users or
/// the program may be missing (NixOS has no `/usr/bin/dmesg`). Each is one
/// answer line, never a crash or free text.
#[test]
fn the_log_reads_answer_one_line_on_every_machine() -> TestResult {
    for kind in ["dmesg", "journal"] {
        let output = output(&["logs", kind])?;

        let answer = String::from_utf8(output.stdout)?;

        assert_eq!(output.status.code(), Some(0_i32), "{kind}");
        assert_eq!(answer.matches('\n').count(), 1, "{kind}");

        match ProgramAnswer::parse(answer.trim_end())? {
            ProgramAnswer::Missing => {}
            ProgramAnswer::Ran {
                output: printed,
                errors,
                code,
                truncated: _,
            } => assert!(
                code == Some(0_i32) || !errors.is_empty() || !printed.is_empty(),
                "{kind} failed without saying why"
            ),
        }
    }

    Ok(())
}

/// The catalog shows these files but marks them read-only, so even a dry
/// run of a write to one is refused before any step.
#[test]
fn a_write_to_a_read_only_config_file_is_rejected() -> TestResult {
    for path in [
        "/etc/passwd",
        "/etc/sudoers",
        "/etc/pam.d/su",
        "/etc/systemd/system/x.service",
    ] {
        let action = RootAction::WriteConfigFile {
            write: ConfigWrite {
                path: ConfigFilePath::new(path)?,
                content: ConfigText::new("x\n")?,
                original: Fingerprint::of(b""),
            },
        };

        let output = helper(&["apply"], &request(Mode::DryRun, vec![action])?)?;
        let first = events(&output)?.into_iter().next();

        assert!(!output.status.success(), "{path}");
        assert!(
            matches!(&first, Some(Event::Rejected { reason }) if reason.contains("never written")),
            "{path}: {}",
            output.stdout
        );
    }

    Ok(())
}
