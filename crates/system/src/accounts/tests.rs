use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use toolbox_plan::{GroupName, ShellPath, UserName};
use toolbox_state::parts::CurrentShells;

use super::{AccountSource as _, EtcAccounts, accounts, available_shells, login_shells};
use crate::ProbeError;

const PASSWD: &str = "# local accounts\n\
                      root:x:0:0::/root:/bin/bash\n\
                      \n\
                      nico:x:1000:1000:Nico:/home/nico:/bin/fish\n\
                      other:x:1001:984::/home/other:/bin/sh\n\
                      broken line\n";

const GROUP: &str = "# local groups\n\
                     root:x:0:root\n\
                     wheel:x:998:other,nico\n\
                     \n\
                     nico:x:1000:\n\
                     docker:x:984:other\n\
                     audio:x:995: nico , foo\n\
                     lp:x:991:nicolas\n\
                     bad name:x:12:nico\n\
                     truncated\n";

const SHELLS: &str = "# Pathnames of valid login shells.\n\
                      /bin/sh\n\
                      /usr/bin/bash\n\
                      /usr/bin/fish\n\
                      /usr/bin/bash\n\
                      not a path\n";

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn groups(names: &[&str]) -> BTreeSet<GroupName> {
    names
        .iter()
        .map(|name| GroupName::new(*name).unwrap())
        .collect()
}

#[test]
fn members_and_the_primary_group_are_listed() {
    assert_eq!(
        accounts(PASSWD, GROUP).groups_of(&user("nico")),
        groups(&["audio", "nico", "wheel"])
    );
}

#[test]
fn the_primary_group_counts_without_a_member_entry() {
    assert_eq!(
        accounts(PASSWD, GROUP).groups_of(&user("other")),
        groups(&["docker", "wheel"])
    );
}

#[test]
fn a_user_missing_from_passwd_keeps_the_member_groups() {
    assert_eq!(
        accounts(PASSWD, GROUP).groups_of(&user("nicolas")),
        groups(&["lp"])
    );
}

#[test]
fn a_name_is_not_matched_by_prefix() {
    assert!(
        !accounts(PASSWD, GROUP)
            .groups_of(&user("nic"))
            .contains(&GroupName::new("lp").unwrap())
    );
}

#[test]
fn the_adapter_reads_both_files_below_its_root() {
    let etc = tempfile::tempdir().unwrap();
    fs::write(etc.path().join("passwd"), PASSWD).unwrap();
    fs::write(etc.path().join("group"), GROUP).unwrap();

    let accounts = EtcAccounts::new(etc.path());

    assert_eq!(
        accounts.accounts().map(|all| all.groups_of(&user("nico"))),
        Ok(groups(&["audio", "nico", "wheel"]))
    );
}

#[test]
fn missing_files_are_unavailable() {
    let etc = tempfile::tempdir().unwrap();
    let accounts = EtcAccounts::new(etc.path());

    assert_eq!(accounts.accounts(), Err(ProbeError::Unavailable));

    fs::write(etc.path().join("passwd"), PASSWD).unwrap();

    assert_eq!(accounts.accounts(), Err(ProbeError::Unavailable));
}

#[test]
fn an_unreadable_file_is_a_failure() {
    let etc = tempfile::tempdir().unwrap();
    fs::create_dir_all(etc.path().join("passwd")).unwrap();
    fs::write(etc.path().join("group"), GROUP).unwrap();

    let accounts = EtcAccounts::new(etc.path());

    assert!(matches!(accounts.accounts(), Err(ProbeError::Failed(_))));
}
#[test]
fn login_shells_come_from_the_last_passwd_field() {
    let shells = login_shells(PASSWD);

    assert_eq!(
        shells.get(&user("nico")),
        Some(&ShellPath::new("/bin/fish").unwrap())
    );
    assert_eq!(
        shells.get(&user("root")),
        Some(&ShellPath::new("/bin/bash").unwrap())
    );
    assert_eq!(shells.len(), 3);
}

#[test]
fn an_account_without_a_shell_is_left_out() {
    let shells = login_shells("svc:x:5:5::/:\nbad:x:6:6::/:relative\n");

    assert_eq!(shells, BTreeMap::new());
}

#[test]
fn available_shells_keep_their_order_without_duplicates() {
    let shells: Vec<String> = available_shells(SHELLS)
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(shells, ["/bin/sh", "/usr/bin/bash", "/usr/bin/fish"]);
}

#[test]
fn the_adapter_reads_passwd_and_shells_below_its_root() {
    let etc = tempfile::tempdir().unwrap();
    let accounts = EtcAccounts::new(etc.path());

    assert_eq!(accounts.available_shells(), Err(ProbeError::Unavailable));
    assert_eq!(accounts.login_shells(), Err(ProbeError::Unavailable));

    fs::write(etc.path().join("passwd"), PASSWD).unwrap();
    fs::write(etc.path().join("shells"), SHELLS).unwrap();

    assert_eq!(
        accounts.available_shells().map(|shells| shells.0.len()),
        Ok(3)
    );
    assert_eq!(
        accounts.login_shells(),
        Ok(CurrentShells(login_shells(PASSWD)))
    );
}

#[test]
fn accounts_hold_every_valid_user_and_group_sorted_by_name() {
    let all = accounts(PASSWD, GROUP);
    let users: Vec<(&str, u32, u32, &str, &str, &str)> = all
        .users
        .iter()
        .map(|user| {
            (
                user.name.as_str(),
                user.uid,
                user.gid,
                user.full_name(),
                user.home.as_str(),
                user.shell.as_str(),
            )
        })
        .collect();

    assert_eq!(
        users,
        [
            ("nico", 1000, 1000, "Nico", "/home/nico", "/bin/fish"),
            ("other", 1001, 984, "", "/home/other", "/bin/sh"),
            ("root", 0, 0, "", "/root", "/bin/bash"),
        ]
    );

    let group_names: Vec<(&str, u32)> = all
        .groups
        .iter()
        .map(|group| (group.name.as_str(), group.gid))
        .collect();

    assert_eq!(
        group_names,
        [
            ("audio", 995),
            ("docker", 984),
            ("lp", 991),
            ("nico", 1000),
            ("root", 0),
            ("wheel", 998)
        ]
    );
    assert_eq!(
        all.members_of(&GroupName::new("audio").unwrap()),
        BTreeSet::from([user("foo"), user("nico")]),
        "blanks around member names are dropped"
    );
    assert_eq!(
        all.members_of(&GroupName::new("docker").unwrap()),
        BTreeSet::from([user("other")])
    );
}

#[test]
fn entries_with_odd_ids_or_repeated_names_are_left_out() {
    let all = accounts(
        "ada:x:abc:1::/home/ada:/bin/sh\n\
         Ada:x:1002:1::/home/Ada:/bin/sh\n\
         bob:x:1003:1:Bob,Room 2:/home/bob\n\
         bob:x:0:0:second bob:/root:/bin/sh\n",
        "team:x:x:bob\nteam2:x:1005:bob,,Bob\n",
    );

    assert_eq!(all.users.len(), 1);
    assert_eq!(all.users.first().map(|user| user.uid), Some(1003));
    assert_eq!(
        all.users
            .first()
            .map(toolbox_state::accounts::User::full_name),
        Some("Bob")
    );
    assert_eq!(all.users.first().map(|user| user.shell.as_str()), Some(""));
    assert_eq!(all.groups.len(), 1);
    assert_eq!(
        all.groups.first().map(|group| group.members.clone()),
        Some(BTreeSet::from([user("bob")]))
    );
}
