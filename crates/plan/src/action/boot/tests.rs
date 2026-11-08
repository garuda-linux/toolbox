use crate::action::testing::shown;
use crate::procedure::{Builtin, Invocation, Program};
use crate::{
    DefaultEntry, GrubSetting, GrubSettings, GrubTimeout, KernelParameters, PlymouthTheme,
    RootAction,
};

fn settings() -> GrubSettings {
    GrubSettings::new(vec![
        GrubSetting::Timeout(GrubTimeout::new(10).unwrap()),
        GrubSetting::Default(DefaultEntry::Saved),
        GrubSetting::KernelParameters(KernelParameters::parse("quiet splash").unwrap()),
        GrubSetting::Theme(None),
    ])
    .unwrap()
}

#[test]
fn grub_settings_are_one_typed_edit_and_no_program() {
    let action = RootAction::SetGrubSettings {
        settings: settings(),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::EditGrubDefaults(settings()))]
    );
    assert_eq!(
        shown(&action),
        [
            "set GRUB_TIMEOUT=\"10\", GRUB_DEFAULT=\"saved\", GRUB_CMDLINE_LINUX_DEFAULT=\"quiet splash\", #GRUB_THEME in /etc/default/grub"
        ]
    );
}

#[test]
fn the_menu_is_generated_with_fixed_arguments() {
    let procedure = RootAction::GenerateGrubConfig.procedure();

    assert_eq!(
        shown(&RootAction::GenerateGrubConfig),
        ["grub-mkconfig -o /boot/grub/grub.cfg"]
    );
    assert!(matches!(
        procedure.as_slice(),
        [Invocation::Run(command)]
            if command.program() == Program::GrubMkconfig
                && command.program().path() == "/usr/bin/grub-mkconfig"
                && command.variables().is_empty()
    ));
}

#[test]
fn the_plymouth_theme_is_set_and_the_initramfs_rebuilt() {
    let action = RootAction::SetPlymouthTheme {
        theme: PlymouthTheme::new("garuda-dr460nized").unwrap(),
    };

    assert_eq!(
        shown(&action),
        ["plymouth-set-default-theme -R garuda-dr460nized"]
    );
    assert!(matches!(
        action.procedure().as_slice(),
        [Invocation::Run(command)]
            if command.program().path() == "/usr/bin/plymouth-set-default-theme"
    ));
}

#[test]
fn boot_actions_round_trip_through_json() {
    for action in [
        RootAction::SetGrubSettings {
            settings: settings(),
        },
        RootAction::GenerateGrubConfig,
        RootAction::SetPlymouthTheme {
            theme: PlymouthTheme::new("bgrt").unwrap(),
        },
    ] {
        let json = serde_json::to_string(&action).unwrap();

        assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
    }

    assert_eq!(
        serde_json::to_string(&RootAction::GenerateGrubConfig).unwrap(),
        r#"{"kind":"generate-grub-config"}"#
    );
}

#[test]
fn hostile_boot_actions_do_not_decode() {
    for json in [
        r#"{"kind":"set-plymouth-theme","theme":"-R"}"#,
        r#"{"kind":"set-plymouth-theme","theme":"--reset"}"#,
        r#"{"kind":"set-plymouth-theme","theme":"a b"}"#,
        r#"{"kind":"set-plymouth-theme","theme":"../../x"}"#,
        r#"{"kind":"set-grub-settings","settings":[]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"kernel-parameters","value":["a\"; reboot #"]}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"default","value":{"entry":"`id`"}}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"theme","value":"/x/$(id)/theme.txt"}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"background","value":"relative.png"}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"path","value":"/etc/passwd"}]}"#,
    ] {
        assert!(serde_json::from_str::<RootAction>(json).is_err(), "{json}");
    }
}

/// The output path is no part of the action, so nothing on the wire can
/// name another file.
#[test]
fn a_path_sent_with_the_generate_action_is_ignored() {
    let action: RootAction =
        serde_json::from_str(r#"{"kind":"generate-grub-config","output":"/etc/passwd"}"#).unwrap();

    assert_eq!(shown(&action), ["grub-mkconfig -o /boot/grub/grub.cfg"]);
}
