mod ancestors;

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{GroupName, HomeDirectory, UserName};

use super::{Listed, group_id, user_id};
use crate::testing::{assert_failed, run_builtin as run};

/// `/etc/passwd` and `/etc/group` of the Lima Garuda image, the user
/// renamed to `user`.
const PASSWD: &str = include_str!("../../../../system/src/accounts/fixtures/passwd");
const GROUP: &str = include_str!("../../../../system/src/accounts/fixtures/group");

fn etc(root: &Path) {
    etc_with(root, PASSWD);
}

fn etc_with(root: &Path, passwd: &str) {
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/passwd"), passwd).unwrap();
    fs::write(root.join("etc/group"), GROUP).unwrap();
}

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn group(name: &str) -> GroupName {
    GroupName::new(name).unwrap()
}

#[test]
fn ids_come_from_the_third_field() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    assert_eq!(
        user_id(dir.path(), &user("user")).unwrap(),
        Listed::Id(1000)
    );
    assert_eq!(user_id(dir.path(), &user("root")).unwrap(), Listed::Id(0));
    assert_eq!(
        user_id(dir.path(), &user("ghost")).unwrap(),
        Listed::Missing
    );
    assert_eq!(
        group_id(dir.path(), &group("wheel")).unwrap(),
        Listed::Id(998)
    );
    assert_eq!(
        group_id(dir.path(), &group("use")).unwrap(),
        Listed::Missing
    );
}

/// Edit: the user id of `user` replaced by `abc`, as a hand-edited file
/// could have it.
#[test]
fn an_id_that_is_no_number_reads_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    etc_with(
        dir.path(),
        &PASSWD.replace("user:x:1000:1000:", "user:x:abc:1000:"),
    );

    assert_eq!(user_id(dir.path(), &user("user")).unwrap(), Listed::Missing);
}

/// Edit: a second account `toor` with user id 0 appended, the BSD habit of
/// a spare root login.
#[test]
fn every_account_with_user_id_zero_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    etc_with(
        dir.path(),
        &format!("{PASSWD}toor:x:0:0:second root:/root:/usr/bin/bash\n"),
    );

    for name in ["root", "toor"] {
        let (outcome, text) = run(dir.path(), &Builtin::RequireOrdinaryUser(user(name)));

        assert_failed(outcome);
        assert!(text.contains("user id 0"), "{text}");
    }
}

#[test]
fn system_accounts_and_nobody_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    for (name, id) in [("bin", "1"), ("nobody", "65534")] {
        let (outcome, text) = run(dir.path(), &Builtin::RequireOrdinaryUser(user(name)));

        assert_failed(outcome);
        assert!(text.contains(&format!("user id {id},")), "{text}");
        assert!(text.contains("1000 to 60000"), "{text}");
    }
}

#[test]
fn other_listed_users_pass_and_unknown_ones_fail() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    for name in ["user", "guest"] {
        let (outcome, _) = run(dir.path(), &Builtin::RequireOrdinaryUser(user(name)));

        assert_eq!(outcome, Outcome::Succeeded, "{name}");
    }

    let (outcome, text) = run(dir.path(), &Builtin::RequireOrdinaryUser(user("ghost")));

    assert_failed(outcome);
    assert!(text.contains("not listed"), "{text}");
}

#[test]
fn system_groups_stay_and_user_groups_pass() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    for name in ["root", "bin", "wheel", "audio", "nobody"] {
        let (outcome, text) = run(dir.path(), &Builtin::RequireUserGroup(group(name)));

        assert_failed(outcome);
        assert!(text.contains("1000 to 60000"), "{text}");
    }

    for name in ["user", "guest"] {
        let (outcome, _) = run(dir.path(), &Builtin::RequireUserGroup(group(name)));

        assert_eq!(outcome, Outcome::Succeeded, "{name}");
    }

    assert_failed(run(dir.path(), &Builtin::RequireUserGroup(group("ghost"))).0);
}

#[test]
fn missing_account_files_fail_the_guard() {
    let dir = tempfile::tempdir().unwrap();

    assert_failed(run(dir.path(), &Builtin::RequireOrdinaryUser(user("user"))).0);
    assert_failed(run(dir.path(), &Builtin::RequireUserGroup(group("guest"))).0);
}

/// `usermod -d <new> -m` writes the new home into `/etc/passwd` before it
/// fails on a folder that exists, so the step checks first.
#[test]
fn a_home_move_needs_a_folder_that_does_not_exist_yet() {
    let dir = tempfile::tempdir().unwrap();
    let free = Builtin::RequireFreeHome(HomeDirectory::new("/srv/people/ada").unwrap());
    let taken = Builtin::RequireFreeHome(HomeDirectory::new("/srv/people/taken").unwrap());

    fs::create_dir_all(dir.path().join("srv/people/taken")).unwrap();

    assert_eq!(run(dir.path(), &free).0, Outcome::Succeeded);

    let (outcome, text) = run(dir.path(), &taken);

    assert_failed(outcome);
    assert!(text.contains("/srv/people/taken exists already"), "{text}");
}

/// The sticky bit of `/tmp` keeps others from removing a link, never from
/// placing one before usermod moves the files.
#[test]
fn a_home_move_below_a_folder_everyone_may_write_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let home = Builtin::RequireFreeHome(HomeDirectory::new("/srv/shared/ada").unwrap());
    let shared = dir.path().join("srv/shared");

    fs::create_dir_all(&shared).unwrap();

    for (mode, refused) in [
        (0o755, false),
        (0o775, false),
        (0o777, true),
        (0o1777, true),
    ] {
        fs::set_permissions(&shared, fs::Permissions::from_mode(mode)).unwrap();

        let (outcome, text) = run(dir.path(), &home);

        if refused {
            assert_failed(outcome);
            assert!(text.contains("writable for everyone"), "{mode:o}: {text}");
        } else {
            assert_eq!(outcome, Outcome::Succeeded, "{mode:o}: {text}");
        }
    }
}
