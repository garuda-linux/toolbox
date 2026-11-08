use std::collections::BTreeSet;

use toolbox_plan::UnitName;

use super::{
    ENABLED, FailedUnitSource as _, MASKED, Systemctl, UnitSource as _, list_args, parse_failed,
    parse_listed,
};
use crate::ProbeError;
use crate::testing::{fake_program, on_path};

fn units(names: &[&str]) -> BTreeSet<UnitName> {
    names
        .iter()
        .map(|name| UnitName::new(*name).unwrap())
        .collect()
}

#[test]
fn the_first_column_names_the_unit() {
    let output = "sshd.service                 enabled enabled\n\
                  getty@.service               enabled enabled\n\
                  fstrim.timer enabled\n";

    assert_eq!(
        parse_listed(output, ENABLED),
        units(&["fstrim.timer", "getty@.service", "sshd.service"])
    );
}

#[test]
fn odd_lines_are_skipped() {
    let output = "\n\
                  bluetooth.service enabled-runtime disabled\n\
                  not a unit\n\
                  -bad.service enabled enabled\n\
                  noextension enabled enabled\n\
                  lonely.service\n\
                  3 unit files listed.\n\
                  cups.socket enabled enabled\n";

    assert_eq!(parse_listed(output, ENABLED), units(&["cups.socket"]));
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

#[test]
fn masked_units_are_read_the_same_way() {
    let output = "cups.service masked enabled\n\
                  tracker.service masked-runtime enabled\n\
                  sshd.service enabled enabled\n";

    assert_eq!(parse_listed(output, MASKED), units(&["cups.service"]));
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
    let (_folder, program) = fake_program("cups.service masked enabled\n", "", 0);
    let systemctl = Systemctl::with_program(program);

    assert_eq!(
        systemctl.masked_units().map(|masked| masked.0),
        Ok(units(&["cups.service"]))
    );
    assert_eq!(
        systemctl.masked_user_units().map(|masked| masked.0),
        Ok(units(&["cups.service"]))
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

#[test]
fn a_failed_unit_is_named_by_its_first_column() {
    let output = "bluetooth.service loaded failed failed Bluetooth service\n\
                  \n\
                  2 loaded units listed.\n\
                  cups.service loaded active running CUPS\n";

    assert_eq!(parse_failed(output), units(&["bluetooth.service"]));
}

#[test]
fn the_failed_units_come_from_systemctl() {
    let (_folder, program) = fake_program("smb.service loaded failed failed Samba\n", "", 0);

    assert_eq!(
        Systemctl::with_program(program).failed_units(),
        Ok(vec![UnitName::new("smb.service").unwrap()])
    );
}
