use std::ffi::OsStr;
use std::path::PathBuf;

use toolbox_plan::ConfigFilePath;
use toolbox_plan::protocol::{Mode, ReadKind};
use toolbox_system::pkexec::Refusal;
use toolbox_system::{HelperFetch as _, HelperReads as _};

use super::{HelperLauncher, Launcher as _, Way};
use crate::i18n::FALLBACK;

fn launcher(way: Way) -> HelperLauncher {
    HelperLauncher {
        way,
        language: FALLBACK,
    }
}

fn argv(launcher: &HelperLauncher) -> (String, Vec<String>) {
    let command = launcher.command();
    let program = command
        .as_std()
        .get_program()
        .to_string_lossy()
        .into_owned();

    let arguments = command
        .as_std()
        .get_args()
        .map(OsStr::to_string_lossy)
        .map(String::from)
        .collect();

    (program, arguments)
}

#[test]
fn a_real_run_goes_through_pkexec_with_the_helper_path_and_apply() {
    let launcher = launcher(Way::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    });

    let (program, arguments) = argv(&launcher);

    assert_eq!(program, "/usr/bin/pkexec");
    assert_eq!(
        arguments,
        ["/usr/lib/garuda-toolbox/garuda-toolbox-helper", "apply"]
    );
    assert_eq!(launcher.mode(), Mode::Run);
}

#[test]
fn a_dry_run_starts_the_helper_itself() {
    let launcher = launcher(Way::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    });

    let (program, arguments) = argv(&launcher);

    assert_eq!(program, "/tmp/garuda-toolbox-helper");
    assert_eq!(arguments, ["apply"]);
    assert_eq!(launcher.mode(), Mode::DryRun);
}

#[test]
fn the_environment_variable_picks_the_dry_run() {
    let dry = HelperLauncher::from_lookup(
        |key| (key == "GARUDA_TOOLBOX_HELPER").then(|| "dry-run".into()),
        FALLBACK,
    );

    let real = HelperLauncher::from_lookup(|_| None, FALLBACK);

    assert_eq!(dry.mode(), Mode::DryRun);
    assert_eq!(real.mode(), Mode::Run);
}

#[test]
fn a_log_read_asks_pkexec_for_the_helper_and_its_logs_action() {
    let pkexec = launcher(Way::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    });

    let command = pkexec.read_command(ReadKind::Journal);
    let arguments: Vec<&OsStr> = command.get_args().collect();

    assert_eq!(command.get_program(), "/usr/bin/pkexec");
    assert_eq!(
        arguments,
        [
            "/usr/lib/garuda-toolbox/garuda-toolbox-helper",
            "logs",
            "journal"
        ]
    );

    let dry = launcher(Way::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    });

    let dry_command = dry.read_command(ReadKind::Dmesg);
    let dry_arguments: Vec<&OsStr> = dry_command.get_args().collect();

    assert_eq!(dry_command.get_program(), "/tmp/garuda-toolbox-helper");
    assert_eq!(dry_arguments, ["logs", "dmesg"]);
}

#[test]
fn a_development_build_finds_pkexec_on_the_path() {
    let path = std::ffi::OsString::from("/home/u/bin:/run/wrappers/bin:/usr/bin");
    let exists =
        |candidate: &std::path::Path| candidate == std::path::Path::new("/run/wrappers/bin/pkexec");

    assert_eq!(
        super::root_tool("pkexec", "/usr/bin/pkexec", Some(&path), true, exists),
        PathBuf::from("/run/wrappers/bin/pkexec")
    );
}

#[test]
fn a_release_build_never_searches_the_path_for_pkexec() {
    let path = std::ffi::OsString::from("/home/u/bin");
    let exists = |_: &std::path::Path| true;

    assert_eq!(
        super::root_tool("pkexec", "/usr/bin/pkexec", Some(&path), false, exists),
        PathBuf::from("/usr/bin/pkexec")
    );
}

#[test]
fn a_launcher_runs_the_pkexec_it_was_given() {
    let launcher = launcher(Way::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/run/wrappers/bin/pkexec"),
    });

    let (program, _) = argv(&launcher);

    assert_eq!(program, "/run/wrappers/bin/pkexec");
}

#[test]
fn a_plain_file_is_no_setuid_root_pkexec() {
    let folder = tempfile::tempdir().expect("temporary folder");
    let fake = folder.path().join("pkexec");
    std::fs::write(&fake, "#!/bin/sh\n").expect("fake pkexec");

    assert!(!super::is_setuid_root(&fake));
    assert!(!super::is_setuid_root(&folder.path().join("missing")));
}

#[test]
fn sudo_runs_the_same_helper_after_its_option_end() {
    let launcher = through_sudo();

    let (program, arguments) = argv(&launcher);

    assert_eq!(program, "/usr/bin/sudo");
    assert_eq!(
        arguments,
        [
            "--",
            "/usr/lib/garuda-toolbox/garuda-toolbox-helper",
            "apply"
        ]
    );
    assert_eq!(launcher.mode(), Mode::Run);
}

#[test]
fn a_dry_run_has_no_sudo_way() {
    let dry = launcher(Way::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    });

    assert_eq!(dry.with_sudo(PathBuf::from("/usr/bin/sudo")), None);
}

#[test]
fn every_pkexec_command_asks_for_untranslated_words() {
    let launcher = launcher(Way::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    });

    let path = ConfigFilePath::new("/etc/pacman.conf").unwrap();
    let apply = launcher.command();
    let read = launcher.read_command(ReadKind::Journal);
    let fetch = launcher.fetch_command(&path);

    for command in [apply.as_std(), &read, &fetch] {
        let locale = command
            .get_envs()
            .find(|(key, _)| *key == "LC_ALL")
            .and_then(|(_, value)| value);

        assert_eq!(locale, Some(OsStr::new("C")), "{command:?}");
    }
}

fn through_sudo() -> HelperLauncher {
    launcher(Way::Sudo {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        sudo: PathBuf::from("/usr/bin/sudo"),
    })
}

/// pkexec's 126 and 127 mean nothing to sudo, whose refusals end with 1.
#[test]
fn sudo_is_not_read_with_the_exit_codes_of_pkexec() {
    let sudo = through_sudo();

    assert!(!sudo.may_refuse(Some(126_i32)));
    assert!(!sudo.may_refuse(Some(127_i32)));
    assert!(sudo.may_refuse(Some(1_i32)));
    assert_eq!(sudo.refusal(Some(127_i32), ""), None);
}

/// Without the words of sudo, an exit with 1 is the helper's own failure.
#[test]
fn sudo_without_its_words_refused_nothing() {
    assert_eq!(through_sudo().refusal(Some(1_i32), ""), None);
}

/// sudo of the Garuda Lima image in the C locale. The `.terminal`
/// captures are what sudo printed on the terminal script(1) gave it.
#[test]
fn real_sudo_refusals_are_told_apart() {
    for (code, words, expected) in [
        (
            include_str!("fixtures/wrong-password-three-times.code"),
            include_str!("fixtures/wrong-password-three-times.terminal"),
            Refusal::Denied,
        ),
        (
            include_str!("fixtures/not-in-sudoers.code"),
            include_str!("fixtures/not-in-sudoers.terminal"),
            Refusal::Denied,
        ),
        (
            include_str!("fixtures/no-terminal.code"),
            include_str!("fixtures/no-terminal.stderr"),
            Refusal::NoAgent,
        ),
    ] {
        let code = code.trim().parse().ok();

        assert_eq!(
            through_sudo().refusal(code, words),
            Some(expected),
            "{words}"
        );
    }
}

#[test]
fn a_dry_run_is_never_refused() {
    let dry = launcher(Way::DryRun {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
    });

    assert!(!dry.may_refuse(Some(127_i32)));
    assert_eq!(dry.refusal(Some(127_i32), ""), None);
}

#[test]
fn sudo_reads_its_words_in_the_c_locale() {
    let command = through_sudo().command();
    let locale = command
        .as_std()
        .get_envs()
        .find(|(key, _)| *key == OsStr::new("LC_ALL"))
        .and_then(|(_, value)| value);

    assert_eq!(locale, Some(OsStr::new("C")));
}

/// logind shows the reason in the language the launcher was built with,
/// not the one of the environment.
#[test]
fn the_inhibitor_speaks_the_language_it_was_given() {
    let german: fluent_templates::LanguageIdentifier = "de-DE".parse().unwrap();
    let built = HelperLauncher::from_lookup(
        |key| (key == "LANG").then(|| "en_US.UTF-8".into()),
        german.clone(),
    );

    assert_eq!(
        built.inhibitor(),
        super::Inhibitor::Logind { language: german }
    );
}
