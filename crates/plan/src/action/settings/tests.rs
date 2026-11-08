use crate::action::testing::shown;
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{DnsProvider, GarudaUpdateFlag, RootAction, ShellPath, UserName, WifiBackend};

#[test]
fn set_shell_checks_the_list_then_runs_chsh_with_a_separator() {
    let action = RootAction::SetShell {
        user: UserName::new("nico").unwrap(),
        shell: ShellPath::new("/usr/bin/fish").unwrap(),
    };

    assert_eq!(
        action.procedure(),
        [
            Invocation::Builtin(Builtin::RequireListedShell(
                ShellPath::new("/usr/bin/fish").unwrap()
            )),
            Invocation::Run(Command::new(Program::Chsh).args([
                "-s",
                "/usr/bin/fish",
                "--",
                "nico"
            ])),
        ]
    );
    assert_eq!(
        shown(&action),
        [
            "check that /usr/bin/fish is listed in /etc/shells",
            "chsh -s /usr/bin/fish -- nico"
        ]
    );
}

#[test]
fn set_dns_writes_the_drop_in_and_reloads_network_manager() {
    let action = RootAction::SetDns {
        provider: DnsProvider::Cloudflare,
    };

    assert_eq!(
        shown(&action),
        [
            "write /etc/NetworkManager/conf.d/10-garuda-assistant-dns.conf (servers=1.1.1.1)",
            "nmcli general reload"
        ]
    );
}

#[test]
fn the_default_dns_removes_the_drop_in() {
    let action = RootAction::SetDns {
        provider: DnsProvider::Default,
    };

    assert_eq!(
        shown(&action),
        [
            "rm -f /etc/NetworkManager/conf.d/10-garuda-assistant-dns.conf",
            "nmcli general reload"
        ]
    );
}

#[test]
fn hblock_on_enables_the_timer_and_fills_the_hosts_file() {
    assert_eq!(
        shown(&RootAction::SetHblock { enabled: true }),
        ["systemctl enable --now -- hblock.timer", "hblock"]
    );
}

#[test]
fn hblock_off_disables_the_timer_and_empties_the_block_list() {
    assert_eq!(
        shown(&RootAction::SetHblock { enabled: false }),
        [
            "systemctl disable --now -- hblock.timer",
            "hblock -S none -D none"
        ]
    );
}

#[test]
fn switching_to_iwd_masks_wpa_supplicant_and_always_restarts_network_manager() {
    let action = RootAction::SetWifiBackend {
        backend: WifiBackend::Iwd,
    };

    assert_eq!(
        shown(&action),
        [
            "systemctl stop -- NetworkManager.service",
            "systemctl disable --now -- wpa_supplicant.service",
            "systemctl mask -- wpa_supplicant.service",
            "write /etc/NetworkManager/conf.d/20_wifi_backend_toolbox.conf (wifi.backend=iwd)",
            "systemctl daemon-reload",
            "systemctl start -- NetworkManager.service  # always, even after a failure"
        ]
    );
}

#[test]
fn switching_back_unmasks_wpa_supplicant_and_removes_both_drop_ins() {
    let action = RootAction::SetWifiBackend {
        backend: WifiBackend::WpaSupplicant,
    };

    assert_eq!(
        shown(&action),
        [
            "systemctl stop -- NetworkManager.service",
            "rm -f /etc/NetworkManager/conf.d/20_wifi_backend_toolbox.conf /etc/NetworkManager/conf.d/20_wifi_backend_rani.conf",
            "systemctl unmask -- wpa_supplicant.service",
            "systemctl enable --now -- wpa_supplicant.service",
            "systemctl daemon-reload",
            "systemctl start -- NetworkManager.service  # always, even after a failure"
        ]
    );
}

#[test]
fn only_the_last_wifi_step_always_runs() {
    for backend in WifiBackend::ALL {
        let procedure = RootAction::SetWifiBackend { backend }.procedure();
        let always: Vec<bool> = procedure
            .iter()
            .map(|invocation| matches!(invocation, Invocation::Always(_)))
            .collect();

        assert_eq!(always.iter().filter(|always| **always).count(), 1);
        assert_eq!(always.last(), Some(&true));
    }
}

#[test]
fn a_garuda_update_switch_is_one_builtin_edit() {
    assert_eq!(
        shown(&RootAction::SetGarudaUpdateFlag {
            flag: GarudaUpdateFlag::UpdateAur,
            enabled: true,
        }),
        ["set UPDATE_AUR=1 in /etc/garuda/garuda-update/config"]
    );
    assert_eq!(
        shown(&RootAction::SetGarudaUpdateFlag {
            flag: GarudaUpdateFlag::SkipMirrorlist,
            enabled: false,
        }),
        ["comment out SKIP_MIRRORLIST in /etc/garuda/garuda-update/config"]
    );
}

#[test]
fn settings_actions_round_trip_through_json() {
    let actions = [
        RootAction::SetShell {
            user: UserName::new("nico").unwrap(),
            shell: ShellPath::new("/usr/bin/zsh").unwrap(),
        },
        RootAction::SetDns {
            provider: DnsProvider::Quad9,
        },
        RootAction::SetHblock { enabled: true },
        RootAction::SetWifiBackend {
            backend: WifiBackend::WpaSupplicant,
        },
        RootAction::SetGarudaUpdateFlag {
            flag: GarudaUpdateFlag::NoCheckSpace,
            enabled: false,
        },
    ];

    for action in actions {
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
    }
}

#[test]
fn a_shell_outside_the_rules_is_refused_while_decoding() {
    let json = r#"{"kind":"set-shell","user":"nico","shell":"/tmp/../bin/evil"}"#;
    serde_json::from_str::<RootAction>(json).unwrap_err();

    let invalid_user = r#"{"kind":"set-shell","user":"-nico","shell":"/usr/bin/zsh"}"#;
    serde_json::from_str::<RootAction>(invalid_user).unwrap_err();
}
