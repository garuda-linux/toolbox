use std::collections::BTreeSet;

use toolbox_plan::UnitName;

use super::{
    ENABLED, FailedUnitSource as _, MASKED, Systemctl, UnitSource as _, list_args, parse_failed,
    parse_listed,
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
    let enabled = parse_listed(LISTED_ENABLED, ENABLED);

    assert_eq!(enabled.len(), 51);
    assert!(enabled.is_superset(&units(&[
        "getty@.service",
        "sshd.service",
        "systemd-userdbd.socket",
        "remote-fs.target",
    ])));
    assert_eq!(parse_listed(LISTED_USER_ENABLED, ENABLED), user_enabled());
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

    assert_eq!(parse_listed(&output, ENABLED), user_enabled());
}

#[test]
fn a_missing_program_is_unavailable() {
    let systemctl = Systemctl::with_program("toolbox-system-no-such-program");

    assert_eq!(systemctl.enabled_units(), Err(ProbeError::Unavailable));
    assert_eq!(systemctl.enabled_user_units(), Err(ProbeError::Unavailable));
}

#[test]
fn a_failing_program_is_a_failure() {
    let systemctl = Systemctl::with_program("false");

    assert!(matches!(
        systemctl.enabled_units(),
        Err(ProbeError::Failed(_))
    ));
    assert!(matches!(
        systemctl.enabled_user_units(),
        Err(ProbeError::Failed(_))
    ));
}

/// A sandbox without a system or user manager may still fail, but only a
/// missing systemctl is "not available".
#[test]
fn real_systemctl_answers_or_is_unavailable() {
    let systemctl = Systemctl::new();
    let system = systemctl.enabled_units();
    let user = systemctl.enabled_user_units();

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
        parse_listed(&output, MASKED),
        units(&[
            "systemd-firstboot.service",
            "toolbox-fixture-masked.service"
        ])
    );
    assert_eq!(
        parse_listed(LISTED_USER_MASKED, MASKED),
        units(&["toolbox-fixture-user.service"])
    );
}

#[test]
fn masks_need_a_working_systemctl_too() {
    let missing = Systemctl::with_program("toolbox-system-no-such-program");
    let failing = Systemctl::with_program("false");

    assert_eq!(missing.masked_units(), Err(ProbeError::Unavailable));
    assert_eq!(missing.masked_user_units(), Err(ProbeError::Unavailable));
    assert!(matches!(failing.masked_units(), Err(ProbeError::Failed(_))));
    assert!(matches!(
        failing.masked_user_units(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn a_fake_systemctl_is_asked_for_masked_units() {
    let (_folder, program) = fake_program(LISTED_USER_MASKED, "", 0);
    let systemctl = Systemctl::with_program(program);
    let masked = units(&["toolbox-fixture-user.service"]);

    assert_eq!(
        systemctl.masked_units().map(|listed| listed.0),
        Ok(masked.clone())
    );
    assert_eq!(
        systemctl.masked_user_units().map(|listed| listed.0),
        Ok(masked)
    );
}

#[test]
fn the_listing_asks_systemctl_for_one_state_only() {
    assert_eq!(
        list_args(&["--user"], &format!("--state={MASKED}")),
        [
            "--user",
            "list-unit-files",
            "--state=masked",
            "--no-legend",
            "--no-pager",
            "--plain"
        ]
    );
    assert_eq!(
        list_args(&[], &format!("--state={ENABLED}"))
            .first()
            .copied(),
        Some("list-unit-files")
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
