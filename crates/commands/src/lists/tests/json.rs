use toolbox_plan::{GroupName, HomeDirectory, LocaleName, ShellPath, UnitName, UserName};

use crate::lists::{GroupRow, LocaleRow, ServiceRow, UserRow};

/// The unit, group and locale ids are plain strings in JSON, as before.
#[test]
fn service_group_and_locale_rows_as_json_keep_their_ids_as_strings() {
    let service = ServiceRow {
        name: UnitName::new("sshd.service").unwrap(),
        active: "active",
        state: "running".to_owned(),
        at_boot: true,
        file_state: "enabled",
        description: toolbox_text::plain("OpenSSH"),
    };

    let group = GroupRow {
        name: GroupName::new("wheel").unwrap(),
        gid: 998,
        members: vec!["ada".to_owned()],
        system: true,
    };

    let locale = LocaleRow {
        locale: LocaleName::new("de_DE.UTF-8").unwrap(),
        charset: "UTF-8".to_owned(),
        name: None,
        enabled: true,
        system: false,
    };

    assert_eq!(
        serde_json::to_string(&service).unwrap(),
        r#"{"id":"sshd.service","active":"active","sub_state":"running","at_boot":true,"file_state":"enabled","description":"OpenSSH"}"#
    );
    assert_eq!(
        serde_json::to_string(&group).unwrap(),
        r#"{"id":"wheel","gid":998,"members":["ada"],"system":true}"#
    );
    assert_eq!(
        serde_json::to_string(&locale).unwrap(),
        r#"{"id":"de_DE.UTF-8","charset":"UTF-8","name":null,"enabled":true,"system":false}"#
    );
}

/// Home and shell are one string each, checked or not, as before the check.
#[test]
fn a_user_row_as_json_keeps_home_and_shell_as_strings() {
    let checked = UserRow {
        name: UserName::new("ada").unwrap(),
        uid: 1001,
        full_name: toolbox_text::plain("Ada Lovelace"),
        home: Ok(HomeDirectory::new("/home/ada").unwrap()),
        shell: Ok(ShellPath::new("/usr/bin/fish").unwrap()),
    };

    let outside = UserRow {
        name: UserName::new("grace").unwrap(),
        uid: 0,
        full_name: toolbox_text::plain(""),
        home: Err(toolbox_text::plain("/root")),
        shell: Err(toolbox_text::plain("")),
    };

    assert_eq!(
        serde_json::to_string(&checked).unwrap(),
        r#"{"id":"ada","uid":1001,"name":"Ada Lovelace","home":"/home/ada","shell":"/usr/bin/fish"}"#
    );
    assert_eq!(
        serde_json::to_string(&outside).unwrap(),
        r#"{"id":"grace","uid":0,"name":"","home":"/root","shell":""}"#
    );
}
