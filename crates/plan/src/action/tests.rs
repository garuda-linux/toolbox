use super::testing::{shown, shown_user, units};
use super::{RemoteAction, RootAction, TargetList, Upgrader, UserAction};
use crate::procedure::{Builtin, Invocation};
use crate::{GroupName, Mirrorlist, PackageName, UserName};

fn packages(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(names.iter().map(|name| PackageName::new(*name).unwrap())).unwrap()
}

#[test]
fn target_list_sorts_and_removes_duplicates() {
    let list = packages(&["vlc", "mpv", "vlc"]);
    let names: Vec<&str> = list.iter().map(PackageName::as_str).collect();
    assert_eq!(names, ["mpv", "vlc"]);
}

#[test]
fn target_list_refuses_to_be_empty() {
    TargetList::<PackageName>::new([]).unwrap_err();
    serde_json::from_str::<TargetList<PackageName>>("[]").unwrap_err();
}

#[test]
fn install_packages_runs_pacman_needed_without_questions() {
    let action = RootAction::InstallPackages {
        packages: packages(&["mpv", "vlc"]),
    };

    assert_eq!(
        shown(&action),
        ["pacman -S --needed --noconfirm -- mpv vlc"]
    );
}

#[test]
fn remove_packages_removes_unneeded_dependencies_too() {
    let action = RootAction::RemovePackages {
        packages: packages(&["vlc"]),
    };

    assert_eq!(shown(&action), ["pacman -Rns --noconfirm -- vlc"]);
}

#[test]
fn upgrade_with_garuda_update_tells_it_who_is_asking() {
    let action = RootAction::UpgradeSystem {
        upgrader: Upgrader::GarudaUpdate,
    };

    assert_eq!(
        shown(&action),
        ["GARUDA_UPDATE_TOOLBOX=1 garuda-update --skip-mirrorlist"]
    );
    let plain = RootAction::UpgradeSystem {
        upgrader: Upgrader::Pacman,
    };

    assert_eq!(shown(&plain), ["pacman -Syu --noconfirm"]);
}

#[test]
fn maintenance_actions_have_fixed_commands() {
    assert_eq!(shown(&RootAction::CleanPackageCache), ["paccache -ruk0"]);
    assert_eq!(
        shown(&RootAction::RefreshKeyring),
        ["pacman-key --refresh-keys"]
    );
}

#[test]
fn garuda_update_remote_fixes_run_in_toolbox_mode() {
    let expected = [
        (RemoteAction::Fix, "fix"),
        (RemoteAction::Keyring, "keyring"),
        (RemoteAction::FullFix, "fullfix"),
        (RemoteAction::ResetAudio, "reset-audio"),
        (RemoteAction::ResetSnapper, "reset-snapper"),
        (RemoteAction::Reinstall, "reinstall"),
    ];

    for (action, argument) in expected {
        assert_eq!(
            shown(&RootAction::GarudaUpdateRemote { action }),
            [format!(
                "GARUDA_UPDATE_TOOLBOX=1 garuda-update remote {argument}"
            )]
        );
    }
}

#[test]
fn remote_actions_are_kebab_case_on_the_wire() {
    let action = RootAction::GarudaUpdateRemote {
        action: RemoteAction::ResetAudio,
    };

    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"garuda-update-remote","action":"reset-audio"}"#
    );
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
    serde_json::from_str::<RootAction>(r#"{"kind":"garuda-update-remote","action":"--help"}"#)
        .unwrap_err();
}

#[test]
fn garuda_health_fixes_as_root() {
    assert_eq!(shown(&RootAction::GarudaHealthFix), ["garuda-health --fix"]);
}

#[test]
fn the_chroot_enters_the_installed_system() {
    assert_eq!(shown(&RootAction::ChrootInstallation), ["garuda-chroot -a"]);
    assert_eq!(
        serde_json::to_string(&RootAction::ChrootInstallation).unwrap(),
        r#"{"kind":"chroot-installation"}"#
    );
}

#[test]
fn the_mirrorlist_is_written_by_a_builtin() {
    let mirrorlist =
        Mirrorlist::new("Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n").unwrap();

    let action = RootAction::InstallMirrorlist {
        mirrorlist: mirrorlist.clone(),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::InstallMirrorlist(mirrorlist))]
    );
    assert_eq!(
        shown(&action),
        [
            "write /etc/pacman.d/chaotic-mirrorlist (1 server), the old one kept as chaotic-mirrorlist.bak"
        ]
    );
}

#[test]
fn an_invalid_mirrorlist_rejects_the_action() {
    let json = r#"{"kind":"install-mirrorlist","mirrorlist":"Include = /etc/shadow"}"#;
    serde_json::from_str::<RootAction>(json).unwrap_err();
}

#[test]
fn remove_orphans_takes_its_targets_from_a_query() {
    let procedure = RootAction::RemoveOrphans.procedure();
    assert_eq!(procedure.len(), 1);
    assert_eq!(procedure[0].to_string(), "pacman -Rns -- $(pacman -Qtdq)");
}

#[test]
fn removing_the_database_lock_is_a_checked_builtin() {
    assert_eq!(
        RootAction::RemoveDatabaseLock.procedure(),
        [Invocation::Builtin(Builtin::RemoveDatabaseLock)]
    );
}

#[test]
fn units_are_enabled_and_started_together() {
    let enable = RootAction::EnableUnits {
        units: units(&["docker.socket", "libvirtd.service"]),
    };

    assert_eq!(
        shown(&enable),
        ["systemctl enable --now -- docker.socket libvirtd.service"]
    );
    let disable = RootAction::DisableUnits {
        units: units(&["cups.socket"]),
    };

    assert_eq!(shown(&disable), ["systemctl disable --now -- cups.socket"]);
}

#[test]
fn group_membership_uses_gpasswd() {
    let user = UserName::new("nico").unwrap();
    let group = GroupName::new("docker").unwrap();
    let add = RootAction::AddToGroup {
        user: user.clone(),
        group: group.clone(),
    };

    assert_eq!(shown(&add), ["gpasswd -a nico docker"]);
    assert_eq!(
        shown(&RootAction::RemoveFromGroup { user, group }),
        ["gpasswd -d nico docker"]
    );
}

#[test]
fn the_aur_upgrade_leaves_every_question_to_the_user() {
    assert_eq!(
        shown_user(&UserAction::UpgradeAur),
        ["paru --sudo pkexec -Sua"]
    );
}

#[test]
fn the_aur_upgrade_round_trips_through_json() {
    let json = serde_json::to_string(&UserAction::UpgradeAur).unwrap();

    assert_eq!(json, r#"{"kind":"upgrade-aur"}"#);
    assert_eq!(
        serde_json::from_str::<UserAction>(&json).unwrap(),
        UserAction::UpgradeAur
    );
}

#[test]
fn user_units_go_through_the_user_manager() {
    let action = UserAction::EnableUserUnits {
        units: units(&["psd.service"]),
    };

    assert_eq!(
        shown_user(&action),
        ["systemctl --user enable --now -- psd.service"]
    );
}

#[test]
fn garuda_health_checks_as_the_user() {
    assert_eq!(shown_user(&UserAction::GarudaHealth), ["garuda-health"]);
}

#[test]
fn actions_round_trip_through_json() {
    let action = RootAction::InstallPackages {
        packages: packages(&["mpv"]),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, r#"{"kind":"install-packages","packages":["mpv"]}"#);
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
}

#[test]
fn an_action_with_an_invalid_name_does_not_deserialize() {
    let json = r#"{"kind":"install-packages","packages":["--overwrite=*"]}"#;
    serde_json::from_str::<RootAction>(json).unwrap_err();
}

#[test]
fn a_locale_switch_is_one_typed_edit_of_locale_gen() {
    let locale = crate::LocaleEntry::parse("de_DE.UTF-8 UTF-8").unwrap();
    let on = RootAction::SetLocaleEnabled {
        locale: locale.clone(),
        enabled: true,
    };

    let off = RootAction::SetLocaleEnabled {
        locale: locale.clone(),
        enabled: false,
    };

    assert_eq!(
        on.procedure(),
        [Invocation::Builtin(Builtin::EditLocaleGen {
            entry: locale,
            on: true
        })]
    );
    assert_eq!(
        shown(&on),
        ["enable `de_DE.UTF-8 UTF-8` in /etc/locale.gen"]
    );
    assert_eq!(
        shown(&off),
        ["comment out `de_DE.UTF-8 UTF-8` in /etc/locale.gen"]
    );
}

#[test]
fn generating_locales_runs_locale_gen_without_arguments() {
    assert_eq!(shown(&RootAction::GenerateLocales), ["locale-gen"]);
}

#[test]
fn the_system_locale_is_a_checked_edit_of_locale_conf() {
    let action = RootAction::SetSystemLocale {
        locale: crate::LocaleName::new("de_DE.UTF-8").unwrap(),
    };

    assert_eq!(
        shown(&action),
        ["set LANG=de_DE.UTF-8 in /etc/locale.conf  # only when /etc/locale.gen has it enabled"]
    );
    assert!(matches!(
        action.procedure().as_slice(),
        [Invocation::Builtin(Builtin::SetSystemLocale(_))]
    ));
}

#[test]
fn a_dkms_module_is_installed_for_one_kernel() {
    let action = RootAction::InstallDkmsModule {
        build: crate::DkmsBuild {
            module: crate::DkmsModuleName::new("vboxhost").unwrap(),
            version: crate::DkmsVersion::new("7.1.6_OSE").unwrap(),
            kernel: crate::KernelRelease::new("6.12.9-1-cachyos-lts").unwrap(),
        },
    };

    assert_eq!(
        shown(&action),
        ["dkms install vboxhost/7.1.6_OSE -k 6.12.9-1-cachyos-lts"]
    );
}

#[test]
fn locale_and_dkms_actions_refuse_option_like_values_on_the_wire() {
    for json in [
        r#"{"kind":"set-system-locale","locale":"--help"}"#,
        r#"{"kind":"set-locale-enabled","locale":{"locale":"de_DE.UTF-8","charset":"UTF-8 x"},"enabled":true}"#,
        r#"{"kind":"install-dkms-module","build":{"module":"-k","version":"1","kernel":"6.1"}}"#,
        r#"{"kind":"install-dkms-module","build":{"module":"nvidia","version":"1/2","kernel":"6.1"}}"#,
        r#"{"kind":"install-dkms-module","build":{"module":"nvidia","version":"1","kernel":"../6.1"}}"#,
    ] {
        serde_json::from_str::<RootAction>(json).unwrap_err();
    }

    let action: RootAction = serde_json::from_str(
        r#"{"kind":"install-dkms-module","build":{"module":"nvidia","version":"550.90.07","kernel":"6.14.0-4-arch1-1"}}"#,
    )
    .unwrap();

    assert_eq!(
        shown(&action),
        ["dkms install nvidia/550.90.07 -k 6.14.0-4-arch1-1"]
    );
}
