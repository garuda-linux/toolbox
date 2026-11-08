use toolbox_catalog::ToggleCatalog;
use toolbox_cli::args::TogglePage;
use toolbox_core::Snapshot;
use toolbox_core::domains::{Accounts, Group, InstalledPackages, User};
use toolbox_core::features::locales::{SupportedLocales, parse_supported};
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::{GroupName, PackageName, UnitName, UserName};

use super::{UserRow, groups, locales, people, services, toggles, users};

const CATALOG: &str = r#"
[[section]]
id = "containers"
title = "Containers"
icon = "x"

[[section.entry]]
id = "docker"
title = "Docker"
description = "The engine."
check = { package = "docker" }

[[section.entry]]
id = "podman"
title = "Podman"
description = "The other engine."
check = { package = "podman" }
"#;

fn unit(name: &str, active: ActiveState, file: FileState) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: format!("About {name}"),
        active,
        sub_state: "running".to_owned(),
        file_state: file,
    }
}

fn accounts() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![User {
            name: UserName::new("ada").unwrap(),
            uid: 1001,
            gid: 1001,
            gecos: "Ada Lovelace,Room 1".to_owned(),
            home: "/home/ada".to_owned(),
            shell: "/usr/bin/fish".to_owned(),
        }],
        groups: vec![Group {
            name: GroupName::new("wheel").unwrap(),
            gid: 998,
            members: [UserName::new("ada").unwrap()].into(),
        }],
    });
    snapshot
}

#[test]
fn services_say_their_state_and_whether_they_start_at_boot() {
    let rows = services(&[
        unit("sshd.service", ActiveState::Active, FileState::Enabled),
        unit(
            "bluetooth.service",
            ActiveState::Failed,
            FileState::Disabled,
        ),
    ]);

    assert_eq!(rows[0].name, "bluetooth.service");
    assert_eq!(rows[0].active, "failed");
    assert!(!rows[0].at_boot);
    assert!(rows[1].at_boot);
}

#[test]
fn toggles_tell_on_off_and_unknown() {
    let catalog = ToggleCatalog::parse(CATALOG, "test.toml").unwrap();
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        std::iter::once(PackageName::new("docker").unwrap()).collect(),
    ));

    let rows = toggles(&[(TogglePage::Components, &catalog)], &snapshot, &[]);

    let docker = rows.iter().find(|row| row.id == "docker").unwrap();
    assert_eq!((docker.page, docker.state), ("components", "on"));
    assert_eq!(docker.section, "containers");
    assert_eq!(
        rows.iter().find(|row| row.id == "podman").unwrap().state,
        "off"
    );

    let unread = toggles(
        &[(TogglePage::Components, &catalog)],
        &Snapshot::default(),
        &[],
    );

    assert!(unread.iter().all(|row| row.state == "unknown"));
}

#[test]
fn users_name_their_full_name_and_shell_and_groups_their_members() {
    let snapshot = accounts();

    let people = users(&snapshot).unwrap();
    assert_eq!(people[0].full_name, "Ada Lovelace");
    assert_eq!(people[0].shell, "/usr/bin/fish");

    let teams = groups(&snapshot).unwrap();
    assert_eq!(teams[0].members, ["ada"]);
    assert!(teams[0].system);
}

#[test]
fn without_the_accounts_there_is_nothing_to_list() {
    assert!(users(&Snapshot::default()).is_none());
}

#[test]
fn a_toggle_row_names_its_page_as_the_window_does() {
    let catalog = ToggleCatalog::parse(CATALOG, "test.toml").unwrap();
    let rows = toggles(&[(TogglePage::Tweaks, &catalog)], &Snapshot::default(), &[]);

    assert!(rows.iter().all(|row| row.page == "system-settings"));
}

fn account(name: &str, uid: u32) -> UserRow {
    UserRow {
        name: name.to_owned(),
        uid,
        full_name: String::new(),
        home: format!("/home/{name}"),
        shell: "/bin/bash".to_owned(),
    }
}

#[test]
fn only_people_and_root_are_shown_unless_all_are_asked_for() {
    let rows = vec![
        account("root", 0),
        account("bin", 1),
        account("ada", 1000),
        account("last", 60_000),
        account("systemd-dynamic", 60_001),
        account("nobody", 65_000),
    ];

    let (shown, hidden) = people(rows.clone(), false);
    let names: Vec<&str> = shown.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["root", "ada", "last"]);
    assert_eq!(hidden, 3);

    let (every, none_hidden) = people(rows, true);
    assert_eq!((every.len(), none_hidden), (6, 0));
}

/// A row's JSON names it by `id`, and a service keeps systemd's own
/// words in `active` and `sub_state`, never one `state` that hides them.
#[test]
fn a_service_row_as_json_has_its_id_and_systemds_own_states() {
    let rows = services(&[unit(
        "sshd.service",
        ActiveState::Active,
        FileState::Enabled,
    )]);

    let row = serde_json::to_value(&rows[0]).unwrap();

    assert_eq!(row["id"], "sshd.service");
    assert_eq!(row["active"], "active");
    assert!(row.get("sub_state").is_some(), "{row}");
    assert!(
        row.get("name").is_none() && row.get("state").is_none(),
        "{row}"
    );
}

#[test]
fn a_switch_as_json_is_on_true_false_or_null() {
    let catalog = ToggleCatalog::parse(CATALOG, "test.toml").unwrap();
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        std::iter::once(PackageName::new("docker").unwrap()).collect(),
    ));

    let read = toggles(&[(TogglePage::Components, &catalog)], &snapshot, &[]);
    let unread = toggles(
        &[(TogglePage::Components, &catalog)],
        &Snapshot::default(),
        &[],
    );

    let on: Vec<serde_json::Value> = read
        .iter()
        .chain(&unread)
        .map(|row| serde_json::to_value(row).unwrap()["on"].clone())
        .collect();

    assert_eq!(
        on,
        [
            serde_json::Value::Bool(true),
            serde_json::Value::Bool(false),
            serde_json::Value::Null,
            serde_json::Value::Null,
        ]
    );

    let docker = serde_json::to_value(&read[0]).unwrap();
    assert_eq!(
        (&docker["id"], &docker["name"]),
        (&"docker".into(), &"Docker".into())
    );
}

/// The display name of an account is its `name`, as of every other row.
#[test]
fn a_user_row_as_json_has_its_id_and_name() {
    let people = users(&accounts()).unwrap();
    let row = serde_json::to_value(&people[0]).unwrap();

    assert_eq!(row["id"], "ada");
    assert_eq!(row["name"], "Ada Lovelace");
    assert!(row.get("full_name").is_none(), "{row}");
}

/// The id is what `locales enable` and `locales system` take.
#[test]
fn a_locale_row_as_json_has_the_name_the_commands_take() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported(include_str!(
        "../../../system/src/locales/fixtures/SUPPORTED"
    ))));

    let rows: Vec<serde_json::Value> = locales(&snapshot)
        .iter()
        .map(|row| serde_json::to_value(row).unwrap())
        .collect();

    let row = |id: &str| rows.iter().find(|row| row["id"] == id).unwrap();

    assert_eq!(row("de_DE.UTF-8")["charset"], "UTF-8");
    assert_eq!(row("de_DE@euro")["charset"], "ISO-8859-15");
}
