use toolbox_plan::UnitName;
use toolbox_state::services::{ActiveState, FileState, UnitInfo};
use zbus::OwnedObjectPath;
use zbus::zvariant::Type as _;

use super::{
    LIST_UNIT_FILES, LIST_UNITS, ListUnitsRow, SUBSCRIBE, SYSTEMD_DESTINATION, UnitFileRow,
    unit_infos,
};

fn path(text: &str) -> OwnedObjectPath {
    OwnedObjectPath::try_from(text).unwrap()
}

fn row(name: &str, description: &str, load: &str, active: &str, sub: &str) -> ListUnitsRow {
    (
        name.to_owned(),
        description.to_owned(),
        load.to_owned(),
        active.to_owned(),
        sub.to_owned(),
        String::new(),
        path("/org/freedesktop/systemd1/unit/sshd_2eservice"),
        0_u32,
        String::new(),
        path("/"),
    )
}

fn file(path: &str, state: &str) -> UnitFileRow {
    (path.to_owned(), state.to_owned())
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

fn names(infos: &[UnitInfo]) -> Vec<&str> {
    infos.iter().map(|info| info.name.as_str()).collect()
}

#[test]
fn the_rows_match_the_manager_signatures() {
    assert_eq!(Vec::<ListUnitsRow>::SIGNATURE.to_string(), "a(ssssssouso)");
    assert_eq!(Vec::<UnitFileRow>::SIGNATURE.to_string(), "a(ss)");
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
    let units = vec![
        row(
            "sshd.service",
            "OpenSSH Daemon",
            "loaded",
            "active",
            "running",
        ),
        row(
            "cups.socket",
            "CUPS Scheduler",
            "loaded",
            "active",
            "listening",
        ),
    ];

    let files = vec![
        file("/usr/lib/systemd/system/sshd.service", "enabled"),
        file("/usr/lib/systemd/system/cups.socket", "disabled"),
    ];

    assert_eq!(
        unit_infos(units, files),
        [
            info(
                "cups.socket",
                "CUPS Scheduler",
                ActiveState::Active,
                "listening",
                FileState::Disabled
            ),
            info(
                "sshd.service",
                "OpenSSH Daemon",
                ActiveState::Active,
                "running",
                FileState::Enabled
            ),
        ]
    );
}

#[test]
fn unit_files_that_never_loaded_are_listed_as_dead() {
    let files = vec![file(
        "/usr/lib/systemd/system/bluetooth.service",
        "disabled",
    )];

    assert_eq!(
        unit_infos(Vec::new(), files),
        [info(
            "bluetooth.service",
            "",
            ActiveState::Inactive,
            "dead",
            FileState::Disabled
        )]
    );
}

#[test]
fn a_unit_without_a_listed_file_has_an_unknown_file_state() {
    let units = vec![row(
        "getty@tty1.service",
        "Getty on tty1",
        "loaded",
        "active",
        "running",
    )];

    assert_eq!(
        unit_infos(units, Vec::new()),
        [info(
            "getty@tty1.service",
            "Getty on tty1",
            ActiveState::Active,
            "running",
            FileState::Unknown
        )]
    );
}

#[test]
fn only_services_and_sockets_are_listed() {
    let units = vec![
        row(
            "fstrim.timer",
            "Discard unused blocks",
            "loaded",
            "active",
            "waiting",
        ),
        row("home.mount", "/home", "loaded", "active", "mounted"),
        row(
            "sshd.service",
            "OpenSSH Daemon",
            "loaded",
            "active",
            "running",
        ),
    ];

    let files = vec![
        file("/usr/lib/systemd/system/fstrim.timer", "enabled"),
        file("/usr/lib/systemd/system/dbus.socket", "static"),
    ];

    assert_eq!(
        names(&unit_infos(units, files)),
        ["dbus.socket", "sshd.service"]
    );
}

/// A template cannot start without an instance, its instances can.
#[test]
fn templates_are_left_out_but_their_instances_stay() {
    let units = vec![row(
        "getty@tty1.service",
        "Getty on tty1",
        "loaded",
        "active",
        "running",
    )];

    let files = vec![
        file("/usr/lib/systemd/system/getty@.service", "enabled"),
        file("/usr/lib/systemd/system/systemd-fsck@.service", "static"),
    ];

    assert_eq!(names(&unit_infos(units, files)), ["getty@tty1.service"]);
}

#[test]
fn units_that_were_not_found_or_have_odd_names_are_skipped() {
    let units = vec![
        row(
            "ghost.service",
            "ghost.service",
            "not-found",
            "inactive",
            "dead",
        ),
        row("-bad.service", "", "loaded", "active", "running"),
        row(
            "sshd.service",
            "OpenSSH Daemon",
            "loaded",
            "active",
            "running",
        ),
    ];

    let files = vec![file("/etc/systemd/system/with space.service", "enabled")];

    assert_eq!(names(&unit_infos(units, files)), ["sshd.service"]);
}

/// An alias names a unit that is listed under its own name already.
#[test]
fn aliases_are_not_listed_twice() {
    let units = vec![row(
        "systemd-networkd.service",
        "Network Configuration",
        "loaded",
        "active",
        "running",
    )];

    let files = vec![
        file(
            "/usr/lib/systemd/system/systemd-networkd.service",
            "enabled",
        ),
        file(
            "/etc/systemd/system/dbus-org.freedesktop.network1.service",
            "alias",
        ),
    ];

    assert_eq!(
        names(&unit_infos(units, files)),
        ["systemd-networkd.service"]
    );
}

#[test]
fn masked_and_failed_units_keep_their_states() {
    let units = vec![
        row(
            "bluetooth.service",
            "Bluetooth service",
            "loaded",
            "failed",
            "failed",
        ),
        row("cups.service", "cups.service", "masked", "inactive", "dead"),
    ];

    let files = vec![
        file(
            "/usr/lib/systemd/system/bluetooth.service",
            "enabled-runtime",
        ),
        file("/etc/systemd/system/cups.service", "masked"),
    ];

    assert_eq!(
        unit_infos(units, files),
        [
            info(
                "bluetooth.service",
                "Bluetooth service",
                ActiveState::Failed,
                "failed",
                FileState::Enabled
            ),
            info(
                "cups.service",
                "cups.service",
                ActiveState::Inactive,
                "dead",
                FileState::Masked
            ),
        ]
    );
}

#[test]
fn duplicates_are_listed_once_with_the_first_answer() {
    let units = vec![
        row(
            "sshd.service",
            "OpenSSH Daemon",
            "loaded",
            "active",
            "running",
        ),
        row("sshd.service", "Other", "loaded", "inactive", "dead"),
    ];

    let files = vec![
        file("/etc/systemd/system/sshd.service", "masked"),
        file("/usr/lib/systemd/system/sshd.service", "enabled"),
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

#[test]
fn the_file_state_follows_the_systemd_precedence() {
    let files = vec![
        file("/usr/lib/systemd/system/sshd.service", "enabled"),
        file("/run/systemd/system/sshd.service", "disabled"),
        file("/etc/systemd/system/sshd.service", "masked"),
    ];

    assert_eq!(
        unit_infos(Vec::new(), files),
        [info(
            "sshd.service",
            "",
            ActiveState::Inactive,
            "dead",
            FileState::Masked
        )]
    );

    let run_files = vec![
        file("/usr/lib/systemd/system/cups.service", "enabled"),
        file("/run/systemd/system/cups.service", "masked"),
    ];

    assert_eq!(
        unit_infos(Vec::new(), run_files),
        [info(
            "cups.service",
            "",
            ActiveState::Inactive,
            "dead",
            FileState::Masked
        )]
    );
}

#[test]
fn unknown_folders_rank_after_the_known_ones() {
    assert_eq!(super::precedence("/opt/units/a.service"), 3);
}
