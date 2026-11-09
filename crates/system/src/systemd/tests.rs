use toolbox_plan::UnitName;
use toolbox_state::services::{ActiveState, FileState, UnitInfo};
use zbus::zvariant::Type as _;

use super::{
    LIST_UNIT_FILES, LIST_UNITS, ListUnitsRow, SUBSCRIBE, SYSTEMD_DESTINATION, UnitFileRow,
    unit_infos,
};

/// `busctl --json=short call ... ListUnits` and `ListUnitFiles` in the
/// Lima VM: `{"type": <signature>, "data": [<the one array>]}`.
const LIST_UNITS_JSON: &str = include_str!("fixtures/list-units.json");
const LIST_UNIT_FILES_JSON: &str = include_str!("fixtures/list-unit-files.json");

fn reply(json: &str) -> serde_json::Value {
    serde_json::from_str(json).unwrap()
}

fn captured_units() -> Vec<ListUnitsRow> {
    serde_json::from_value(reply(LIST_UNITS_JSON)["data"][0].take()).unwrap()
}

fn captured_files() -> Vec<UnitFileRow> {
    serde_json::from_value(reply(LIST_UNIT_FILES_JSON)["data"][0].take()).unwrap()
}

fn captured_infos() -> Vec<UnitInfo> {
    unit_infos(captured_units(), captured_files())
}

fn captured_unit(name: &str) -> ListUnitsRow {
    captured_units()
        .into_iter()
        .find(|row| row.0 == name)
        .unwrap()
}

fn captured_file(name: &str) -> UnitFileRow {
    captured_files()
        .into_iter()
        .find(|(path, _)| path.ends_with(&format!("/{name}")))
        .unwrap()
}

fn info(
    name: &str,
    description: &str,
    active: ActiveState,
    sub: &str,
    file: FileState,
) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: description.to_owned(),
        active,
        sub_state: sub.to_owned(),
        file_state: file,
    }
}

fn found<'a>(infos: &'a [UnitInfo], name: &str) -> Option<&'a UnitInfo> {
    infos.iter().find(|info| info.name.as_str() == name)
}

fn names(infos: &[UnitInfo]) -> Vec<&str> {
    infos.iter().map(|info| info.name.as_str()).collect()
}

#[test]
fn the_rows_match_the_manager_signatures() {
    assert_eq!(Vec::<ListUnitsRow>::SIGNATURE.to_string(), "a(ssssssouso)");
    assert_eq!(Vec::<UnitFileRow>::SIGNATURE.to_string(), "a(ss)");
    assert_eq!(reply(LIST_UNITS_JSON)["type"], "a(ssssssouso)");
    assert_eq!(reply(LIST_UNIT_FILES_JSON)["type"], "a(ss)");
}

#[test]
fn the_methods_address_the_systemd_manager() {
    assert_eq!(SYSTEMD_DESTINATION, "org.freedesktop.systemd1");
    assert_eq!(
        [LIST_UNITS.name, LIST_UNIT_FILES.name, SUBSCRIBE.name],
        ["ListUnits", "ListUnitFiles", "Subscribe"]
    );
}

#[test]
fn loaded_units_take_their_file_state_by_file_name() {
    let infos = captured_infos();

    assert_eq!(
        found(&infos, "sshd.service"),
        Some(&info(
            "sshd.service",
            "OpenSSH Daemon",
            ActiveState::Active,
            "running",
            FileState::Enabled
        ))
    );
    assert_eq!(
        found(&infos, "sshd-unix-local.socket"),
        Some(&info(
            "sshd-unix-local.socket",
            "OpenSSH Server Socket (systemd-ssh-generator, AF_UNIX Local)",
            ActiveState::Active,
            "listening",
            FileState::Generated
        ))
    );
}

#[test]
fn unit_files_that_never_loaded_are_listed_as_dead() {
    assert_eq!(
        found(&captured_infos(), "iptables.service"),
        Some(&info(
            "iptables.service",
            "",
            ActiveState::Inactive,
            "dead",
            FileState::Disabled
        ))
    );
}

/// Only the template `getty@.service` has a file.
#[test]
fn a_unit_without_a_listed_file_has_an_unknown_file_state() {
    assert_eq!(
        found(&captured_infos(), "getty@tty1.service"),
        Some(&info(
            "getty@tty1.service",
            "Getty on tty1",
            ActiveState::Active,
            "running",
            FileState::Unknown
        ))
    );
}

#[test]
fn only_services_and_sockets_are_listed() {
    let infos = captured_infos();

    assert!(
        names(&infos)
            .iter()
            .all(|name| name.ends_with(".service") || name.ends_with(".socket")),
        "{:?}",
        names(&infos)
    );
    assert!(found(&infos, "systemd-storage-fs.socket").is_some());
    assert!(found(&infos, "user-1000.slice").is_none());
}

/// A template cannot start without an instance, its instances can.
#[test]
fn templates_are_left_out_but_their_instances_stay() {
    let infos = captured_infos();

    assert!(!names(&infos).iter().any(|name| name.contains("@.")));
    assert!(found(&infos, "getty@tty1.service").is_some());
    assert!(found(&infos, "serial-getty@ttyS0.service").is_some());
}

/// The Lima lists with a made-up `-bad.service` row and a unit file whose
/// path holds a space.
#[test]
fn units_that_were_not_found_or_have_odd_names_are_skipped() {
    let mut units = captured_units();
    let mut files = captured_files();
    let mut bad = captured_unit("sshd.service");

    bad.0 = "-bad.service".to_owned();
    units.push(bad);
    files.push((
        "/etc/systemd/system/with space.service".to_owned(),
        "enabled".to_owned(),
    ));

    let infos = unit_infos(units, files);

    assert!(found(&infos, "sshd-keygen.service").is_none());
    assert!(found(&infos, "rc-local.service").is_none());
    assert!(found(&infos, "-bad.service").is_none());
    assert!(!names(&infos).iter().any(|name| name.contains(' ')));
}

/// An alias names a unit that is listed under its own name already.
#[test]
fn aliases_are_not_listed_twice() {
    let infos = captured_infos();

    assert!(found(&infos, "dbus-org.freedesktop.network1.service").is_none());
    assert!(found(&infos, "dbus-org.freedesktop.home1.service").is_none());
    assert_eq!(
        found(&infos, "systemd-networkd.service").map(|info| info.file_state),
        Some(FileState::Enabled)
    );
}

#[test]
fn masked_failed_and_transient_units_keep_their_states() {
    let infos = captured_infos();

    assert_eq!(
        [
            "toolbox-fixture-failed.service",
            "toolbox-fixture-masked.service",
            "systemd-firstboot.service",
            "toolbox-fixture-runtime.service",
        ]
        .map(|name| found(&infos, name).cloned()),
        [
            Some(info(
                "toolbox-fixture-failed.service",
                "[systemd-run] /usr/bin/false",
                ActiveState::Failed,
                "failed",
                FileState::Transient
            )),
            Some(info(
                "toolbox-fixture-masked.service",
                "",
                ActiveState::Inactive,
                "dead",
                FileState::Masked
            )),
            Some(info(
                "systemd-firstboot.service",
                "systemd-firstboot.service",
                ActiveState::Inactive,
                "dead",
                FileState::Masked
            )),
            Some(info(
                "toolbox-fixture-runtime.service",
                "toolbox fixture",
                ActiveState::Inactive,
                "dead",
                FileState::Enabled
            )),
        ]
    );
}

/// The Lima sshd row twice, the second one changed, and its file once
/// more as masked in `/etc`.
#[test]
fn duplicates_are_listed_once_with_the_first_answer() {
    let mut other = captured_unit("sshd.service");

    other.1 = "Other".to_owned();
    other.3 = "inactive".to_owned();
    other.4 = "dead".to_owned();

    let units = vec![captured_unit("sshd.service"), other];
    let files = vec![
        (
            "/etc/systemd/system/sshd.service".to_owned(),
            "masked".to_owned(),
        ),
        captured_file("sshd.service"),
    ];

    assert_eq!(
        unit_infos(units, files),
        [info(
            "sshd.service",
            "OpenSSH Daemon",
            ActiveState::Active,
            "running",
            FileState::Masked
        )]
    );
}

/// The Lima sshd file with copies of it in `/run` and `/etc`.
#[test]
fn the_file_state_follows_the_systemd_precedence() {
    let (usr, enabled) = captured_file("sshd.service");
    let copy = |folder: &str, state: &str| {
        (
            usr.replace("/usr/lib/systemd/system/", folder),
            state.to_owned(),
        )
    };

    let dead_sshd = |file| {
        [info(
            "sshd.service",
            "",
            ActiveState::Inactive,
            "dead",
            file,
        )]
    };

    assert_eq!(usr, "/usr/lib/systemd/system/sshd.service");
    assert_eq!(
        unit_infos(
            Vec::new(),
            vec![
                (usr.clone(), enabled.clone()),
                copy("/run/systemd/system/", "disabled"),
                copy("/etc/systemd/system/", "masked"),
            ]
        ),
        dead_sshd(FileState::Masked)
    );
    assert_eq!(
        unit_infos(
            Vec::new(),
            vec![
                (usr.clone(), enabled),
                copy("/run/systemd/system/", "masked"),
            ]
        ),
        dead_sshd(FileState::Masked)
    );
}

#[test]
fn unknown_folders_rank_after_the_known_ones() {
    assert_eq!(super::precedence("/opt/units/a.service"), 3);
}

/// A package's unit file sets its description, so it is outside text.
#[test]
fn a_unit_description_loses_a_bidi_override_and_a_zero_width_joiner() {
    let mut units = captured_units();
    let service = units
        .iter_mut()
        .find(|row| row.0.ends_with(".service") && !row.1.is_empty())
        .unwrap();

    let name = service.0.clone();
    let description = service.1.clone();

    // Changed: the first service's description gains both characters.
    service.1 = format!("\u{202e}{description}\u{200d}");

    let infos = unit_infos(units, captured_files());

    assert_eq!(
        found(&infos, &name).map(|info| info.description.as_str()),
        Some(description.as_str())
    );
}
