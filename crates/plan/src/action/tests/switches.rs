use crate::UserName;
use crate::action::{RootAction, UserAction};

#[test]
fn switches_stay_json_booleans_on_the_wire() {
    let edit = crate::UserConfigEdit::File {
        path: crate::HomePath::new(".config/app.conf").unwrap(),
        content: "on = true\n".parse().unwrap(),
    };

    let pins = [
        (
            r#"{"kind":"set-hblock","enabled":true}"#,
            RootAction::SetHblock {
                enabled: crate::Toggle::On,
            },
        ),
        (
            r#"{"kind":"set-garuda-update-flag","flag":"skip-mirrorlist","enabled":false}"#,
            RootAction::SetGarudaUpdateFlag {
                flag: crate::GarudaUpdateFlag::SkipMirrorlist,
                enabled: crate::Toggle::Off,
            },
        ),
        (
            r#"{"kind":"set-locale-enabled","locale":{"locale":"de_DE.UTF-8","charset":"UTF-8"},"enabled":true}"#,
            RootAction::SetLocaleEnabled {
                locale: crate::LocaleEntry::parse("de_DE.UTF-8 UTF-8").unwrap(),
                enabled: crate::Toggle::On,
            },
        ),
        (
            r#"{"kind":"set-user-locked","user":"ada","locked":true}"#,
            RootAction::SetUserLocked {
                user: UserName::new("ada").unwrap(),
                locked: crate::Toggle::On,
            },
        ),
        (
            r#"{"kind":"set-module-blacklisted","module":"nvidia","blacklisted":false}"#,
            RootAction::SetModuleBlacklisted {
                module: crate::KernelModule::new("nvidia").unwrap(),
                blacklisted: crate::Toggle::Off,
            },
        ),
    ];

    for (json, action) in pins {
        assert_eq!(serde_json::to_string(&action).unwrap(), json);
        assert_eq!(serde_json::from_str::<RootAction>(json).unwrap(), action);
    }

    let json = r#"{"kind":"edit-user-config","edit":{"kind":"file","path":".config/app.conf","content":"on = true\n"},"on":true}"#;
    let action = UserAction::EditUserConfig {
        edit,
        on: crate::Toggle::On,
    };

    assert_eq!(serde_json::to_string(&action).unwrap(), json);
    assert_eq!(serde_json::from_str::<UserAction>(json).unwrap(), action);
}

#[test]
fn a_loader_change_keeps_its_boolean_or_null_wire_form() {
    let json = r#"{"kind":"set-loader-settings","settings":[{"key":"editor","value":false},{"key":"auto-entries","value":null}]}"#;
    let action = RootAction::SetLoaderSettings {
        settings: crate::sdboot::LoaderChange::new(vec![
            crate::sdboot::LoaderSetting::Editor(Some(crate::Toggle::Off)),
            crate::sdboot::LoaderSetting::AutoEntries(None),
        ])
        .unwrap(),
    };

    assert_eq!(serde_json::to_string(&action).unwrap(), json);
    assert_eq!(serde_json::from_str::<RootAction>(json).unwrap(), action);
}
