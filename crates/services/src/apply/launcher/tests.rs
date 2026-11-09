use std::ffi::OsStr;
use std::path::PathBuf;

use toolbox_plan::ConfigFilePath;
use toolbox_plan::protocol::{Mode, ReadKind};
use toolbox_system::{HelperFetch as _, HelperReads as _};

use super::{HelperLauncher, Launcher as _};

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
    let launcher = HelperLauncher::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    };

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
    let launcher = HelperLauncher::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    };

    let (program, arguments) = argv(&launcher);

    assert_eq!(program, "/tmp/garuda-toolbox-helper");
    assert_eq!(arguments, ["apply"]);
    assert_eq!(launcher.mode(), Mode::DryRun);
}

#[test]
fn the_environment_variable_picks_the_dry_run() {
    let dry = HelperLauncher::from_lookup(|key| {
        (key == "GARUDA_TOOLBOX_HELPER").then(|| "dry-run".into())
    });

    let real = HelperLauncher::from_lookup(|_| None);

    assert_eq!(dry.mode(), Mode::DryRun);
    assert_eq!(real.mode(), Mode::Run);
}

#[test]
fn a_log_read_asks_pkexec_for_the_helper_and_its_logs_action() {
    let pkexec = HelperLauncher::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    };

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

    let dry = HelperLauncher::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    };

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
    let launcher = HelperLauncher::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/run/wrappers/bin/pkexec"),
    };

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
    let launcher = HelperLauncher::Sudo {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        sudo: PathBuf::from("/usr/bin/sudo"),
    };

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
    let dry = HelperLauncher::DryRun {
        helper: PathBuf::from("/tmp/garuda-toolbox-helper"),
    };

    assert_eq!(dry.with_sudo(PathBuf::from("/usr/bin/sudo")), None);
}

#[test]
fn every_pkexec_command_asks_for_untranslated_words() {
    let launcher = HelperLauncher::Pkexec {
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        pkexec: PathBuf::from("/usr/bin/pkexec"),
    };

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
