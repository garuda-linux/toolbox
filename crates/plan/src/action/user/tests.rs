use super::UserAction;
use crate::action::TargetList;
use crate::action::testing::shown_user;
use crate::procedure::{Builtin, Invocation};
use crate::{ConfigReload, HomePath, PackageName, UserConfigEdit};

fn file_edit() -> UserConfigEdit {
    UserConfigEdit::File {
        path: HomePath::new(".config/app.conf").unwrap(),
        content: "on = true\n".to_owned(),
    }
}

#[test]
fn reloading_a_config_runs_its_command() {
    let procedure = UserAction::ReloadConfig {
        target: ConfigReload::Kwin,
    }
    .procedure();

    assert_eq!(procedure.len(), 1);
    assert!(procedure[0].to_string().contains("dbus-send"));
}

#[test]
fn editing_a_user_config_is_a_builtin() {
    let procedure = UserAction::EditUserConfig {
        edit: file_edit(),
        on: true,
    }
    .procedure();

    assert!(matches!(
        procedure.as_slice(),
        [Invocation::Builtin(Builtin::EditUserConfig {
            edit: _,
            on: true,
        })]
    ));
}

fn packages(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(names.iter().map(|name| PackageName::new(*name).unwrap())).unwrap()
}

#[test]
fn an_aur_install_leaves_the_review_and_every_question_to_the_user() {
    let action = UserAction::InstallAur {
        packages: packages(&["yay-bin", "paru-bin"]),
    };

    assert_eq!(
        shown_user(&action),
        ["paru --sudo pkexec -S --aur -- paru-bin yay-bin"]
    );
}

#[test]
fn an_aur_install_round_trips_through_json() {
    let action = UserAction::InstallAur {
        packages: packages(&["yay-bin"]),
    };

    let json = serde_json::to_string(&action).unwrap();

    assert_eq!(json, r#"{"kind":"install-aur","packages":["yay-bin"]}"#);
    assert_eq!(serde_json::from_str::<UserAction>(&json).unwrap(), action);
}

#[test]
fn an_aur_install_refuses_a_name_that_reads_as_an_option() {
    let json = r#"{"kind":"install-aur","packages":["--overwrite"]}"#;

    serde_json::from_str::<UserAction>(json).unwrap_err();
    serde_json::from_str::<UserAction>(r#"{"kind":"install-aur","packages":[]}"#).unwrap_err();
}
