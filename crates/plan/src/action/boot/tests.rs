use crate::action::testing::shown;
use crate::procedure::{Builtin, InitramfsRoot, Invocation, Program};
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
        [
            "plymouth-set-default-theme garuda-dr460nized",
            "dracut-rebuild, or mkinitcpio -P where dracut-rebuild is not installed",
        ]
    );
    assert!(matches!(
        action.procedure().as_slice(),
        [Invocation::Run(command), Invocation::Builtin(Builtin::RebuildInitramfs { root: InitramfsRoot::System })]
            if command.program().path() == "/usr/bin/plymouth-set-default-theme"
    ));
}

/// `-R` would run `mkinitcpio` even on a system that builds its initramfs
/// with dracut, as Garuda does.
#[test]
fn the_initramfs_tools_are_tried_dracut_first() {
    let shown: Vec<String> = crate::procedure::initramfs_rebuilds()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(shown, ["dracut-rebuild", "mkinitcpio -P"]);
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

/// The request names the settings only: the helper finds the ESP itself.
#[test]
fn loader_settings_are_one_edit_that_names_no_path() {
    let action = RootAction::SetLoaderSettings {
        settings: crate::sdboot::LoaderChange::new(vec![crate::sdboot::LoaderSetting::Timeout(
            Some(crate::sdboot::LoaderTimeout::Seconds(5)),
        )])
        .unwrap(),
    };

    assert_eq!(
        shown(&action),
        ["set timeout 5 in loader/loader.conf on the EFI system partition"]
    );

    let json = serde_json::to_string(&action).unwrap();

    assert!(!json.contains("efi/"), "{json}");
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
}

fn cmdline() -> KernelParameters {
    KernelParameters::parse(
        "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6 rootflags=subvol=@ rw quiet",
    )
    .unwrap()
}

#[test]
fn the_kernel_cmdline_is_one_write_that_names_no_path() {
    let action = RootAction::SetKernelCmdline {
        parameters: cmdline(),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::WriteKernelCmdline(cmdline()))]
    );
    assert_eq!(
        shown(&action),
        [
            "write root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6 rootflags=subvol=@ rw quiet to /etc/kernel/cmdline"
        ]
    );

    let json = serde_json::to_string(&action).unwrap();

    assert!(!json.contains("etc/"), "{json}");
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
}

#[test]
fn selecting_a_plymouth_theme_builds_nothing() {
    let action = RootAction::SelectPlymouthTheme {
        theme: PlymouthTheme::new("bgrt").unwrap(),
    };

    assert_eq!(shown(&action), ["plymouth-set-default-theme bgrt"]);
}

#[test]
fn the_ukis_are_built_after_the_initramfs_only_when_asked() {
    let ukis = "kernel-install-uki add, or kernel-install add-all where kernel-install-uki is not installed";

    assert_eq!(shown(&RootAction::RebuildUkis { initramfs: false }), [ukis]);
    assert_eq!(
        shown(&RootAction::RebuildUkis { initramfs: true }),
        [
            "dracut-rebuild, or mkinitcpio -P where dracut-rebuild is not installed",
            ukis,
        ]
    );
    assert!(matches!(
        RootAction::RebuildUkis { initramfs: true }
            .procedure()
            .as_slice(),
        [
            Invocation::Builtin(Builtin::RebuildInitramfs {
                root: InitramfsRoot::System
            }),
            Invocation::Builtin(Builtin::RebuildUkis {
                root: InitramfsRoot::System
            }),
        ]
    ));
}

/// Garuda's script passes the initramfs dracut built, plain Arch lets
/// kernel-install's own plugins build it.
#[test]
fn the_uki_tools_are_tried_garudas_first() {
    let tools: Vec<(String, &str)> = crate::procedure::uki_rebuilds()
        .iter()
        .map(|command| (command.to_string(), command.program().path()))
        .collect();

    assert_eq!(
        tools,
        [
            (
                "kernel-install-uki add".to_owned(),
                "/usr/lib/garuda/kernel-install-uki"
            ),
            (
                "kernel-install add-all".to_owned(),
                "/usr/bin/kernel-install"
            ),
        ]
    );
}

#[test]
fn hostile_sdboot_kernel_actions_do_not_decode() {
    for json in [
        r#"{"kind":"set-kernel-cmdline","parameters":["a\"; reboot #"]}"#,
        r#"{"kind":"set-kernel-cmdline","parameters":["acpi_osi=\"Windows 2015\""]}"#,
        r#"{"kind":"set-kernel-cmdline","parameters":"quiet"}"#,
        r#"{"kind":"select-plymouth-theme","theme":"--reset"}"#,
        r#"{"kind":"rebuild-ukis","initramfs":"yes"}"#,
    ] {
        assert!(serde_json::from_str::<RootAction>(json).is_err(), "{json}");
    }
}
