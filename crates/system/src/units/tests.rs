use std::collections::BTreeSet;

use toolbox_plan::UnitName;

use super::{
    FailedUnitSource as _, Systemctl, UnitSource as _, list_args, parse_failed, parse_listed,
};
use crate::ProbeError;
use crate::test_support::{fake_program, on_path};

/// The listings of the Lima VM, each from the argv of [`list_args`] with
/// its `--state`, or from `list-units --state=failed`.
const LISTED_ENABLED: &str = include_str!("fixtures/enabled");
const LISTED_ENABLED_RUNTIME: &str = include_str!("fixtures/enabled-runtime");
const LISTED_MASKED: &str = include_str!("fixtures/masked");
const LISTED_USER_ENABLED: &str = include_str!("fixtures/user-enabled");
const LISTED_USER_MASKED: &str = include_str!("fixtures/user-masked");
const LISTED_FAILED: &str = include_str!("fixtures/failed");
/// `list-unit-files --state=enabled,masked` of the Lima image, as the probe runs it.
const IMAGE_ENABLED_MASKED: &str = include_str!("fixtures/image/enabled-masked");
const IMAGE_USER_ENABLED_MASKED: &str = include_str!("fixtures/image/user-enabled-masked");

fn units(names: &[&str]) -> BTreeSet<UnitName> {
    names
        .iter()
        .map(|name| UnitName::new(*name).unwrap())
        .collect()
}

fn user_enabled() -> BTreeSet<UnitName> {
    units(&[
        "dbus-broker.service",
        "p11-kit-server.service",
        "systemd-tmpfiles-setup.service",
        "gssuserproxy.socket",
        "p11-kit-server.socket",
        "ssh-agent.socket",
        "systemd-storage-fs.socket",
        "machines.target",
        "systemd-tmpfiles-clean.timer",
    ])
}

#[test]
fn the_first_column_names_the_unit() {
    let listed = parse_listed(LISTED_ENABLED);

    assert_eq!(listed.enabled.len(), 51);
    assert!(listed.enabled.is_superset(&units(&[
        "getty@.service",
        "sshd.service",
        "systemd-userdbd.socket",
        "remote-fs.target",
    ])));
    assert_eq!(parse_listed(LISTED_USER_ENABLED).enabled, user_enabled());
}

/// The Lima `enabled-runtime` listing and made-up broken lines in front of
/// the user listing.
#[test]
fn odd_lines_are_skipped() {
    let output = format!(
        "\n{LISTED_ENABLED_RUNTIME}\
         not a unit\n\
         -bad.service enabled enabled\n\
         noextension enabled enabled\n\
         lonely.service\n\
         3 unit files listed.\n\
         {LISTED_USER_ENABLED}"
    );

    assert_eq!(parse_listed(&output).enabled, user_enabled());
}

/// One `--state=enabled,masked` run of the Lima image: each row keeps its
/// own state.
#[test]
fn one_listing_splits_enabled_from_masked() {
    let listed = parse_listed(IMAGE_ENABLED_MASKED);

    assert_eq!(listed.masked, units(&["systemd-firstboot.service"]));
    assert!(listed.enabled.is_superset(&units(&["sshd.service"])));
    assert!(
        listed
            .enabled
            .is_disjoint(&units(&["systemd-firstboot.service"]))
    );
}

/// The user scope of the Lima image masks nothing.
#[test]
fn the_user_listing_of_the_image_is_all_enabled() {
    let listed = parse_listed(IMAGE_USER_ENABLED_MASKED);

    assert!(listed.masked.is_empty());
    assert_eq!(
        listed.enabled.len(),
        IMAGE_USER_ENABLED_MASKED.lines().count()
    );
}

#[test]
fn a_missing_program_is_unavailable() {
    let systemctl = Systemctl::with_program("toolbox-system-no-such-program");

    assert_eq!(systemctl.system_unit_files(), Err(ProbeError::Unavailable));
    assert_eq!(systemctl.user_unit_files(), Err(ProbeError::Unavailable));
}

#[test]
fn a_failing_program_is_a_failure() {
    let (_folder, program) = fake_program("", "Failed to connect to bus", 1);
    let systemctl = Systemctl::with_program(program);

    assert!(matches!(
        systemctl.system_unit_files(),
        Err(error) if error.is_failure()
    ));
    assert!(matches!(
        systemctl.user_unit_files(),
        Err(error) if error.is_failure()
    ));
}

/// A sandbox without a system or user manager may still fail, but only a
/// missing systemctl is "not available".
#[test]
fn real_systemctl_answers_or_is_unavailable() {
    let systemctl = Systemctl::new();
    let system = systemctl.system_unit_files();
    let user = systemctl.user_unit_files();

    if on_path("systemctl") {
        assert_ne!(system, Err(ProbeError::Unavailable));
        assert_ne!(user, Err(ProbeError::Unavailable));
    } else {
        assert_eq!(system, Err(ProbeError::Unavailable));
        assert_eq!(user, Err(ProbeError::Unavailable));
    }
}

/// No `masked-runtime` unit was captured. The Lima `enabled-runtime`
/// listing stands in with its state renamed.
#[test]
fn masked_units_are_read_the_same_way() {
    let output = format!(
        "{LISTED_MASKED}{}{LISTED_ENABLED}",
        LISTED_ENABLED_RUNTIME.replace("enabled-runtime", "masked-runtime ")
    );

    assert_eq!(
        parse_listed(&output).masked,
        units(&[
            "systemd-firstboot.service",
            "toolbox-fixture-masked.service"
        ])
    );
    assert_eq!(
        parse_listed(LISTED_USER_MASKED).masked,
        units(&["toolbox-fixture-user.service"])
    );
}

#[test]
fn unit_files_need_a_working_systemctl_too() {
    let (_folder, program) = fake_program("", "Failed to connect to bus", 1);
    let missing = Systemctl::with_program("toolbox-system-no-such-program");
    let failing = Systemctl::with_program(program);

    assert_eq!(missing.system_unit_files(), Err(ProbeError::Unavailable));
    assert_eq!(missing.user_unit_files(), Err(ProbeError::Unavailable));
    assert!(matches!(failing.system_unit_files(), Err(error) if error.is_failure()));
    assert!(matches!(
        failing.user_unit_files(),
        Err(error) if error.is_failure()
    ));
}

#[test]
fn a_fake_systemctl_gives_both_states_per_scope() {
    let (_folder, program) = fake_program(LISTED_USER_MASKED, "", 0);
    let systemctl = Systemctl::with_program(program);

    let system = systemctl.system_unit_files().unwrap();
    assert_eq!(system.masked.0, units(&["toolbox-fixture-user.service"]));
    assert!(system.enabled.0.is_empty());

    let user = systemctl.user_unit_files().unwrap();
    assert_eq!(user.masked.0, units(&["toolbox-fixture-user.service"]));
}

#[test]
fn the_listing_asks_systemctl_once_per_scope_for_both_states() {
    assert_eq!(
        list_args(&[]),
        [
            "list-unit-files",
            "--state=enabled,masked",
            "--no-legend",
            "--no-pager",
            "--plain"
        ]
    );
    assert_eq!(
        list_args(&["--user"]),
        [
            "--user",
            "list-unit-files",
            "--state=enabled,masked",
            "--no-legend",
            "--no-pager",
            "--plain"
        ]
    );
}

/// The Lima listing with a made-up blank line and summary appended.
#[test]
fn a_failed_unit_is_named_by_its_first_column() {
    let output = format!("{LISTED_FAILED}\n2 loaded units listed.\n");

    assert_eq!(
        parse_failed(&output),
        units(&["cloud-init-main.service", "toolbox-fixture-failed.service"])
    );
}

#[test]
fn the_failed_units_come_from_systemctl() {
    let (_folder, program) = fake_program(LISTED_FAILED, "", 0);

    assert_eq!(
        Systemctl::with_program(program).failed_units(),
        Ok(vec![
            UnitName::new("cloud-init-main.service").unwrap(),
            UnitName::new("toolbox-fixture-failed.service").unwrap(),
        ])
    );
}

/// systemctl prints nothing and exits 1 when no unit has either state (systemd
/// 261.3 for `--state=masked-runtime`), which is an empty listing and not a
/// failure.
#[test]
fn a_listing_without_a_matching_unit_is_empty() {
    let (_folder, program) = fake_program("", "", 1);
    let systemctl = Systemctl::with_program(program);

    let system = systemctl.system_unit_files().unwrap();
    assert!(system.enabled.0.is_empty());
    assert!(system.masked.0.is_empty());
}

/// The user scope without its bus exits 1 and says why on stderr.
#[test]
fn a_failure_that_names_its_reason_stays_one() {
    let (_folder, program) = fake_program("", "Failed to connect to user scope bus", 1);

    assert!(matches!(
        Systemctl::with_program(program).user_unit_files(),
        Err(error) if error.is_failure()
    ));
}
