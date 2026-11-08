use std::fs;
use std::path::Path;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{GroupName, HomeDirectory, UserName};

use super::{Listed, group_id, user_id};
use crate::testing::{assert_failed, run_builtin as run};

const PASSWD: &str = "root:x:0:0::/root:/usr/bin/bash
toor:x:0:0:second root:/root:/usr/bin/bash
bin:x:1:1::/:/usr/bin/nologin
nico:x:1000:1000:Nico:/home/nico:/usr/bin/fish
odd:x:abc:1000::/home/odd:/usr/bin/bash
";

const GROUP: &str = "root:x:0:root
wheel:x:998:nico
nico:x:1000:
makers:x:1001:nico
";

fn etc(root: &Path) {
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/passwd"), PASSWD).unwrap();
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
        user_id(dir.path(), &user("nico")).unwrap(),
        Listed::Id(1000)
    );
    assert_eq!(user_id(dir.path(), &user("root")).unwrap(), Listed::Id(0));
    assert_eq!(user_id(dir.path(), &user("odd")).unwrap(), Listed::Missing);
    assert_eq!(
        user_id(dir.path(), &user("ghost")).unwrap(),
        Listed::Missing
    );
    assert_eq!(
        group_id(dir.path(), &group("wheel")).unwrap(),
        Listed::Id(998)
    );
    assert_eq!(
        group_id(dir.path(), &group("nic")).unwrap(),
        Listed::Missing
    );
}

#[test]
fn every_account_with_user_id_zero_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    for name in ["root", "toor"] {
        let (outcome, text) = run(dir.path(), &Builtin::RequireOrdinaryUser(user(name)));

        assert_failed(outcome);
        assert!(text.contains("root account"), "{text}");
    }
}

#[test]
fn other_listed_users_pass_and_unknown_ones_fail() {
    let dir = tempfile::tempdir().unwrap();
    etc(dir.path());

    for name in ["nico", "bin"] {
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

    for name in ["root", "wheel"] {
        let (outcome, text) = run(dir.path(), &Builtin::RequireUserGroup(group(name)));

        assert_failed(outcome);
        assert!(text.contains("system group"), "{text}");
    }

    for name in ["nico", "makers"] {
        let (outcome, _) = run(dir.path(), &Builtin::RequireUserGroup(group(name)));

        assert_eq!(outcome, Outcome::Succeeded, "{name}");
    }

    assert_failed(run(dir.path(), &Builtin::RequireUserGroup(group("ghost"))).0);
}

#[test]
fn missing_account_files_fail_the_guard() {
    let dir = tempfile::tempdir().unwrap();

    assert_failed(run(dir.path(), &Builtin::RequireOrdinaryUser(user("nico"))).0);
    assert_failed(run(dir.path(), &Builtin::RequireUserGroup(group("makers"))).0);
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
