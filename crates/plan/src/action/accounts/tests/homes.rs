use std::collections::BTreeSet;

use super::{checked, group, new_user, plain, shown, user};
use crate::procedure::{Builtin, Invocation};
use crate::{
    HomeChange, HomeDirectory, InvalidNewUser, NewHome, NewUser, NewUserParts, RootAction, UserId,
};

fn home(path: &str) -> HomeDirectory {
    HomeDirectory::new(path).unwrap()
}

fn change(path: &str, move_files: bool) -> RootAction {
    RootAction::SetHomeFolder {
        user: user("ada"),
        change: HomeChange {
            home: home(path),
            move_files,
        },
    }
}

#[test]
fn a_new_user_can_get_its_home_at_another_path() {
    let user = checked(NewUserParts {
        home: NewHome::At(home("/srv/people/ada")),
        ..plain()
    });

    assert_eq!(
        shown(&RootAction::CreateUser { user }),
        [
            "check the folders above /srv/people/ada",
            "useradd -m -d /srv/people/ada -- ada"
        ]
    );
}

#[test]
fn a_new_user_without_a_home_is_told_so_to_useradd() {
    let user = checked(NewUserParts {
        home: NewHome::Absent,
        ..plain()
    });

    assert_eq!(
        shown(&RootAction::CreateUser { user }),
        ["useradd -M -- ada"]
    );
}

#[test]
fn a_new_user_can_ask_for_its_user_id() {
    let user = checked(NewUserParts {
        uid: Some(UserId::new(1500).unwrap()),
        ..plain()
    });

    assert_eq!(
        shown(&RootAction::CreateUser { user }),
        ["useradd -m -u 1500 -- ada"]
    );
}

#[test]
fn a_system_account_takes_every_option_in_a_fixed_order() {
    let user = checked(NewUserParts {
        home: NewHome::At(home("/var/lib/My Service")),
        uid: Some(UserId::new(450).unwrap()),
        system: true,
        groups: BTreeSet::from([group("audio")]),
        ..new_user()
    });

    assert_eq!(
        shown(&RootAction::CreateUser { user }),
        [
            "check the folders above /var/lib/My Service",
            "check that /usr/bin/fish is listed in /etc/shells",
            "useradd -r -m -d '/var/lib/My Service' -u 450 -c 'Ada Lovelace' -s /usr/bin/fish -G audio -- ada",
        ]
    );
}

#[test]
fn a_system_account_without_a_home() {
    let user = checked(NewUserParts {
        home: NewHome::Absent,
        system: true,
        ..plain()
    });

    assert_eq!(
        shown(&RootAction::CreateUser { user }),
        ["useradd -r -M -- ada"]
    );
}

#[test]
fn the_home_of_an_account_changes_with_or_without_its_files() {
    let guard = Invocation::Builtin(Builtin::RequireOrdinaryUser(user("ada")));
    let moved = change("/srv/people/ada", true);

    assert_eq!(moved.procedure().first(), Some(&guard));
    assert_eq!(
        shown(&moved),
        [
            "check that ada is an ordinary account (id 1000 to 60000)",
            "check that /srv/people/ada does not exist yet",
            "usermod -d /srv/people/ada -m -- ada"
        ]
    );
    assert_eq!(
        shown(&change("/home/Ada L", false)),
        [
            "check that ada is an ordinary account (id 1000 to 60000)",
            "usermod -d '/home/Ada L' -- ada"
        ]
    );
}

#[test]
fn a_user_id_outside_the_range_of_its_kind_is_refused_when_decoded() {
    let request = |uid: &str, system: bool| {
        format!(
            r#"{{"kind":"create-user","user":{{"name":"ada","full_name":"","shell":null,"groups":[],"home":{{"kind":"standard"}},"uid":{uid},"system":{system}}}}}"#
        )
    };

    serde_json::from_str::<RootAction>(&request("1500", false)).unwrap();
    serde_json::from_str::<RootAction>(&request("450", true)).unwrap();
    serde_json::from_str::<RootAction>(&request("null", true)).unwrap();

    for (uid, system) in [
        ("450", false),
        ("1500", true),
        ("0", false),
        ("0", true),
        ("60001", false),
        ("-1", false),
        ("\"1500\"", false),
    ] {
        serde_json::from_str::<RootAction>(&request(uid, system)).unwrap_err();
    }
}

#[test]
fn home_actions_refuse_bad_paths_and_missing_fields_when_decoded() {
    for line in [
        r#"{"kind":"set-home-folder","user":"ada","change":{"home":"/home/../etc","move_files":true}}"#,
        r#"{"kind":"set-home-folder","user":"ada","change":{"home":"srv","move_files":true}}"#,
        r#"{"kind":"set-home-folder","user":"ada","change":{"home":"-m","move_files":false}}"#,
        r#"{"kind":"set-home-folder","user":"ada","change":{"home":"/a:b","move_files":false}}"#,
        r#"{"kind":"set-home-folder","user":"ada","change":{"home":"/srv/ada"}}"#,
        r#"{"kind":"set-home-folder","user":"-d","change":{"home":"/srv/ada","move_files":false}}"#,
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"","shell":null,"groups":[]}}"#,
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"","shell":null,"groups":[],"home":{"kind":"at","path":"/home/a\nb"},"uid":null,"system":false}}"#,
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"","shell":null,"groups":[],"home":{"kind":"at","path":"../x"},"uid":null,"system":false}}"#,
    ] {
        serde_json::from_str::<RootAction>(line).unwrap_err();
    }

    assert_eq!(
        serde_json::from_str::<RootAction>(
            r#"{"kind":"set-home-folder","user":"ada","change":{"home":"/srv/ada","move_files":true}}"#,
        )
        .unwrap(),
        change("/srv/ada", true)
    );
}

#[test]
fn a_user_id_outside_the_range_of_its_kind_makes_no_new_user() {
    let person = UserId::new(1500).unwrap();
    let service = UserId::new(450).unwrap();

    assert_eq!(
        NewUser::new(NewUserParts {
            uid: Some(person),
            system: true,
            ..plain()
        }),
        Err(InvalidNewUser::IdOfAPerson(person))
    );
    assert_eq!(
        NewUser::new(NewUserParts {
            uid: Some(service),
            ..plain()
        }),
        Err(InvalidNewUser::IdOfTheSystem(service))
    );
    assert_eq!(NewUser::new(plain()).map(NewUser::into_parts), Ok(plain()));
}
