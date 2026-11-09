use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use toolbox_plan::{GroupName, ShellPath, UserName};
use toolbox_state::parts::CurrentShells;

use super::{AccountSource as _, EtcAccounts, accounts, available_shells, login_shells};
use crate::ProbeError;

/// The Lima VM after `guest` and `nopass` were added. The login name of
/// the VM's own account was replaced with `user`.
const PASSWD: &str = include_str!("fixtures/passwd");
const GROUP: &str = include_str!("fixtures/group");
/// The same VM before `guest` was added.
const IMAGE_PASSWD: &str = include_str!("fixtures/image/passwd");
const SHELLS: &str = include_str!("fixtures/shells");

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn group(name: &str) -> GroupName {
    GroupName::new(name).unwrap()
}

fn groups(names: &[&str]) -> BTreeSet<GroupName> {
    names.iter().copied().map(group).collect()
}

fn line<'a>(file: &'a str, name: &str) -> &'a str {
    file.lines()
        .find(|line| line.split(':').next() == Some(name))
        .unwrap()
}

#[test]
fn members_and_the_primary_group_are_listed() {
    assert_eq!(
        accounts(PASSWD, GROUP).groups_of(&user("guest")),
        groups(&["audio", "guest", "video"])
    );
}

#[test]
fn the_primary_group_counts_without_a_member_entry() {
    assert_eq!(
        accounts(PASSWD, GROUP).groups_of(&user("user")),
        groups(&["user"])
    );
}

/// The group file after `guest` was added, the passwd file from before.
#[test]
fn a_user_missing_from_passwd_keeps_the_member_groups() {
    assert_eq!(
        accounts(IMAGE_PASSWD, GROUP).groups_of(&user("guest")),
        groups(&["audio", "video"])
    );
}

#[test]
fn a_name_is_not_matched_by_prefix() {
    assert!(
        !accounts(PASSWD, GROUP)
            .groups_of(&user("gues"))
            .contains(&group("audio"))
    );
}

#[test]
fn the_adapter_reads_both_files_below_its_root() {
    let etc = tempfile::tempdir().unwrap();
    fs::write(etc.path().join("passwd"), PASSWD).unwrap();
    fs::write(etc.path().join("group"), GROUP).unwrap();

    let accounts = EtcAccounts::new(etc.path());

    assert_eq!(
        accounts.accounts().map(|all| all.groups_of(&user("guest"))),
        Ok(groups(&["audio", "guest", "video"]))
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
        shells.get(&user("user")),
        Some(&ShellPath::new("/bin/bash").unwrap())
    );
    assert_eq!(
        shells.get(&user("root")),
        Some(&ShellPath::new("/usr/bin/bash").unwrap())
    );
    assert_eq!(
        shells.get(&user("git")),
        Some(&ShellPath::new("/usr/bin/git-shell").unwrap())
    );
    assert_eq!(shells.len(), PASSWD.lines().count());
}

/// The root line with its shell field emptied and made relative.
#[test]
fn an_account_without_a_shell_is_left_out() {
    let root = line(PASSWD, "root");
    let passwd = format!(
        "{}\n{}\n",
        root.replacen(":/usr/bin/bash", ":", 1),
        root.replacen(":/usr/bin/bash", ":relative", 1)
    );

    assert_eq!(login_shells(&passwd), BTreeMap::new());
}

/// The VM's list with `/usr/bin/bash` repeated and a line that is no path
/// appended.
#[test]
fn available_shells_keep_their_order_without_duplicates() {
    let shells: Vec<String> = available_shells(&format!("{SHELLS}/usr/bin/bash\nnot a path\n"))
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        shells,
        [
            "/bin/sh",
            "/bin/bash",
            "/bin/rbash",
            "/usr/bin/sh",
            "/usr/bin/bash",
            "/usr/bin/rbash",
            "/usr/bin/systemd-home-fallback-shell",
        ]
    );
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
        Ok(7)
    );
    assert_eq!(
        accounts.login_shells(),
        Ok(CurrentShells(login_shells(PASSWD)))
    );
}

#[test]
fn a_full_name_loses_a_bidi_override_and_a_zero_width_joiner() {
    // The captured passwd with the GECOS of dbus spoofed.
    let passwd = PASSWD.replacen(
        ":System Message Bus:",
        ":System\u{202e} Message\u{200d} Bus:",
        1,
    );

    let all = accounts(&passwd, GROUP);
    let dbus = all.users.iter().find(|user| user.name.as_str() == "dbus");

    assert_eq!(
        dbus.map(|user| user.gecos.as_str()),
        Some("System Message Bus")
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
        .filter(|(name, ..)| ["guest", "nobody", "root", "user"].contains(name))
        .collect();

    assert_eq!(all.users.len(), PASSWD.lines().count());
    assert_eq!(
        all.users.first().map(|user| user.name.as_str()),
        Some("_talkd")
    );
    assert_eq!(
        users,
        [
            ("guest", 1001, 1001, "", "/home/guest", "/usr/bin/bash"),
            (
                "nobody",
                0xFFFE,
                0xFFFE,
                "Kernel Overflow User",
                "/",
                "/usr/bin/nologin"
            ),
            ("root", 0, 0, "", "/root", "/usr/bin/bash"),
            ("user", 1000, 1000, "", "/home/user.guest", "/bin/bash"),
        ]
    );

    let gids: BTreeMap<&str, u32> = all
        .groups
        .iter()
        .map(|group| (group.name.as_str(), group.gid))
        .collect();

    assert_eq!(all.groups.len(), GROUP.lines().count());
    assert_eq!(gids.get("audio"), Some(&995));
    assert_eq!(gids.get("wheel"), Some(&998));
    assert_eq!(gids.get("nobody"), Some(&0xFFFE));
    assert_eq!(
        all.members_of(&group("video")),
        BTreeSet::from([user("guest")])
    );
    assert!(all.members_of(&group("wheel")).is_empty());
}

/// The audio line with blanks and a second member added around `guest`.
#[test]
fn blanks_around_member_names_are_dropped() {
    let blanks = GROUP.replacen("audio:x:995:guest", "audio:x:995: guest , user", 1);

    assert_eq!(
        accounts(PASSWD, &blanks).members_of(&group("audio")),
        BTreeSet::from([user("guest"), user("user")])
    );
}

/// The guest lines of both files, edited: an id that is no number, a name
/// in capitals, a full name with a room and no shell field, the name
/// repeated with root's ids, and a member list with an empty and a capital
/// entry.
#[test]
fn entries_with_odd_ids_or_repeated_names_are_left_out() {
    let guest = line(PASSWD, "guest");
    let passwd = [
        guest.replacen("guest:x:1001", "nopass:x:abc", 1),
        guest.replacen("guest:", "Guest:", 1),
        guest.replacen(
            "::/home/guest:/usr/bin/bash",
            ":Guest,Room 2:/home/guest",
            1,
        ),
        guest.replacen(":1001:1001:", ":0:0:", 1),
    ]
    .join("\n");

    let group = [
        line(GROUP, "audio").replacen(":995:", ":x:", 1),
        line(GROUP, "video").replacen(":guest", ":guest,,Guest", 1),
    ]
    .join("\n");

    let all = accounts(&passwd, &group);

    assert_eq!(all.users.len(), 1);
    assert_eq!(all.users.first().map(|user| user.uid), Some(1001));
    assert_eq!(
        all.users
            .first()
            .map(toolbox_state::accounts::User::full_name),
        Some("Guest")
    );
    assert_eq!(all.users.first().map(|user| user.shell.as_str()), Some(""));
    assert_eq!(all.groups.len(), 1);
    assert_eq!(
        all.groups.first().map(|entry| entry.members.clone()),
        Some(BTreeSet::from([user("guest")]))
    );
}
