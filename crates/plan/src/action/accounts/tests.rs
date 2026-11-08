mod homes;

use std::collections::BTreeSet;

use crate::action::testing::shown;
use crate::procedure::{Builtin, Invocation};
use crate::protocol::{Mode, Request, RequestError, encode};
use crate::{
    FullName, GroupName, HomeFolder, NewHome, NewUser, NewUserParts, Password, RootAction,
    ShellPath, Step, StepId, UserName,
};

const SECRET: &str = "correct horse battery staple";

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn group(name: &str) -> GroupName {
    GroupName::new(name).unwrap()
}

fn set_password() -> RootAction {
    RootAction::SetPassword {
        user: user("nico"),
        password: Some(Password::new(SECRET.to_owned()).unwrap()),
    }
}

fn checked(parts: NewUserParts) -> NewUser {
    NewUser::new(parts).unwrap()
}

fn plain() -> NewUserParts {
    NewUserParts {
        name: user("ada"),
        full_name: FullName::default(),
        shell: None,
        groups: BTreeSet::new(),
        home: NewHome::Standard,
        uid: None,
        system: false,
    }
}

fn new_user() -> NewUserParts {
    NewUserParts {
        name: user("ada"),
        full_name: FullName::new("Ada Lovelace").unwrap(),
        shell: Some(ShellPath::new("/usr/bin/fish").unwrap()),
        groups: BTreeSet::from([group("wheel"), group("audio")]),
        home: NewHome::Standard,
        uid: None,
        system: false,
    }
}

#[test]
fn a_new_user_gets_a_home_a_checked_shell_and_sorted_groups() {
    assert_eq!(
        shown(&RootAction::CreateUser {
            user: checked(new_user())
        }),
        [
            "check that /usr/bin/fish is listed in /etc/shells",
            "useradd -m -c 'Ada Lovelace' -s /usr/bin/fish -G audio,wheel -- ada",
        ]
    );
}

#[test]
fn a_plain_new_user_takes_the_defaults_of_useradd() {
    assert_eq!(
        shown(&RootAction::CreateUser {
            user: checked(plain())
        }),
        ["useradd -m -- ada"]
    );
}

#[test]
fn the_password_is_fed_to_chpasswd_and_never_an_argument() {
    let action = set_password();
    let procedure = action.procedure();

    assert_eq!(
        shown(&action),
        ["chpasswd  # the secret arrives on its standard input"]
    );

    let [Invocation::Feed { command, input }] = procedure.as_slice() else {
        panic!("one feed expected");
    };

    assert_eq!(command.arguments(), Vec::<String>::new());
    assert_eq!(
        input.as_ref().unwrap().expose(),
        format!("nico:{SECRET}\n").as_bytes()
    );
}

#[test]
fn the_password_shows_in_no_text_form_of_the_action() {
    let action = set_password();
    let request = Request::new(
        Mode::Run,
        vec![Step {
            id: StepId::new(1),
            action: action.clone(),
        }],
    );

    let texts = [
        format!("{action:?}"),
        format!("{action:#?}"),
        format!("{:?}", action.procedure()),
        shown(&action).join("\n"),
        format!("{request:?}"),
        serde_json::to_string(&action).unwrap(),
        encode(&request).unwrap(),
    ];

    for text in texts {
        assert!(!text.contains(SECRET), "{text}");
        assert!(!text.contains("battery"), "{text}");
    }

    assert_eq!(
        serde_json::to_string(&action).unwrap(),
        "{\"kind\":\"set-password\",\"user\":\"nico\"}"
    );
}

#[test]
fn a_decoded_password_action_waits_for_its_secret_line() {
    let line = encode(&Request::new(
        Mode::Run,
        vec![
            Step {
                id: StepId::new(1),
                action: RootAction::CreateUser {
                    user: checked(new_user()),
                },
            },
            Step {
                id: StepId::new(2),
                action: set_password(),
            },
        ],
    ))
    .unwrap();

    let mut request = Request::parse(line.trim_end()).unwrap();
    let waiting = request.steps.last().unwrap().action.clone();

    assert!(waiting.needs_secret());
    assert_eq!(waiting.secret(), None);
    assert_eq!(request.secret_steps(), [StepId::new(2)]);
    assert!(matches!(
        waiting.procedure().as_slice(),
        [Invocation::Feed {
            command: _,
            input: None
        }]
    ));

    let secret = Password::new(SECRET.to_owned()).unwrap();

    assert_eq!(
        request.attach_secret(StepId::new(1), secret.clone()),
        Err(RequestError::MissingSecret(StepId::new(1)))
    );
    assert_eq!(
        request.attach_secret(StepId::new(9), secret.clone()),
        Err(RequestError::MissingSecret(StepId::new(9)))
    );
    request.attach_secret(StepId::new(2), secret).unwrap();
    assert_eq!(request.steps.last().unwrap().action, set_password());
}

#[test]
fn a_dry_run_asks_for_no_secret() {
    let request = Request::new(
        Mode::DryRun,
        vec![Step {
            id: StepId::new(1),
            action: set_password(),
        }],
    );

    assert_eq!(request.secret_steps(), Vec::new());
}

#[test]
fn removing_a_user_keeps_the_home_unless_asked() {
    let guard = Invocation::Builtin(Builtin::RequireOrdinaryUser(user("ada")));
    let keep = RootAction::DeleteUser {
        user: user("ada"),
        home: HomeFolder::Keep,
    };

    let delete = RootAction::DeleteUser {
        user: user("ada"),
        home: HomeFolder::Delete,
    };

    assert_eq!(keep.procedure().first(), Some(&guard));
    assert_eq!(
        shown(&keep),
        [
            "check that ada is an ordinary account (id 1000 to 60000)",
            "userdel -- ada"
        ]
    );
    assert_eq!(
        shown(&delete),
        [
            "check that ada is an ordinary account (id 1000 to 60000)",
            "userdel -r -- ada"
        ]
    );
}

#[test]
fn locking_is_guarded_and_unlocking_is_not() {
    assert_eq!(
        shown(&RootAction::SetUserLocked {
            user: user("ada"),
            locked: true
        }),
        [
            "check that ada is an ordinary account (id 1000 to 60000)",
            "usermod -L -- ada"
        ]
    );
    assert_eq!(
        shown(&RootAction::SetUserLocked {
            user: user("ada"),
            locked: false
        }),
        ["usermod -U -- ada"]
    );
}

#[test]
fn the_full_name_goes_through_chfn() {
    assert_eq!(
        shown(&RootAction::SetFullName {
            user: user("ada"),
            full_name: FullName::new("Ada L.").unwrap()
        }),
        ["chfn -f 'Ada L.' -- ada"]
    );
    assert_eq!(
        shown(&RootAction::SetFullName {
            user: user("ada"),
            full_name: FullName::default()
        }),
        ["chfn -f '' -- ada"]
    );
}

#[test]
fn groups_are_added_and_removed_after_a_double_dash() {
    assert_eq!(
        shown(&RootAction::CreateGroup {
            group: group("makers")
        }),
        ["groupadd -- makers"]
    );
    assert_eq!(
        shown(&RootAction::DeleteGroup {
            group: group("makers")
        }),
        [
            "check that makers is not a system group",
            "groupdel -- makers"
        ]
    );
}

#[test]
fn account_actions_refuse_bad_names_when_decoded() {
    for line in [
        r#"{"kind":"delete-user","user":"-r","home":"keep"}"#,
        r#"{"kind":"delete-user","user":"a b","home":"keep"}"#,
        r#"{"kind":"create-group","group":"--gid=0"}"#,
        r#"{"kind":"set-full-name","user":"ada","full_name":"x:0:0"}"#,
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"","shell":"sh","groups":[],"home":{"kind":"standard"},"uid":null,"system":false}}"#,
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"","shell":null,"groups":["a,b"],"home":{"kind":"standard"},"uid":null,"system":false}}"#,
        r#"{"kind":"delete-user","user":"ada","home":"shred"}"#,
    ] {
        serde_json::from_str::<RootAction>(line).unwrap_err();
    }

    serde_json::from_str::<RootAction>(
        r#"{"kind":"create-user","user":{"name":"ada","full_name":"Ada","shell":null,"groups":["wheel"],"home":{"kind":"standard"},"uid":null,"system":false}}"#,
    )
    .unwrap();
}

#[test]
fn a_password_field_in_a_request_is_ignored() {
    let action: RootAction =
        serde_json::from_str(r#"{"kind":"set-password","user":"nico","password":"x"}"#).unwrap();

    assert_eq!(action.secret(), None);
}

#[test]
fn only_a_password_step_takes_its_secret() {
    let mut action = set_password();
    let secret = Password::new("another secret here".to_owned()).unwrap();

    assert!(action.needs_secret());
    assert!(action.attach_secret(secret));
    assert!(!RootAction::RemoveOrphans.needs_secret());
    assert!(!RootAction::RemoveOrphans.attach_secret(Password::new("x".to_owned()).unwrap()));
    assert_eq!(RootAction::RemoveOrphans.secret(), None);
}
