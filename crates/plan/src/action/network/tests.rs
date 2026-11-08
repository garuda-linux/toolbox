use crate::action::testing::{shown, shown_user};
use crate::procedure::{Builtin, Invocation};
use crate::{HopLimit, HostName, KernelModule, PingCount, PingTimeout, RootAction, UserAction};

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

#[test]
fn loading_a_module_runs_fixed_commands_in_order() {
    let action = RootAction::LoadModule {
        module: module("iwlwifi"),
    };

    assert_eq!(
        shown(&action),
        [
            "systemctl stop -- NetworkManager.service",
            "modprobe -- cfg80211",
            "modprobe -- iwlwifi",
            "pkill -x -- wpa_supplicant",
            "systemctl start -- NetworkManager.service  # always, even after a failure",
        ]
    );
}

#[test]
fn every_part_of_a_load_must_succeed_and_network_manager_always_starts_again() {
    let procedure = RootAction::LoadModule {
        module: module("r8169"),
    }
    .procedure();

    let (last, before) = procedure.split_last().unwrap();

    assert!(matches!(last, Invocation::Always(_)));
    assert!(
        before
            .iter()
            .all(|invocation| matches!(invocation, Invocation::Run(_)))
    );
}

#[test]
fn unloading_a_module_starts_network_manager_again() {
    let action = RootAction::UnloadModule {
        module: module("mt76_usb"),
    };

    assert_eq!(
        shown(&action),
        [
            "systemctl stop -- NetworkManager.service",
            "modprobe -r -- mt76_usb",
            "systemctl start -- NetworkManager.service  # always, even after a failure",
        ]
    );
}

#[test]
fn blacklisting_edits_only_the_apps_own_file() {
    let on = RootAction::SetModuleBlacklisted {
        module: module("mt76-usb"),
        blacklisted: true,
    };

    let off = RootAction::SetModuleBlacklisted {
        module: module("wl"),
        blacklisted: false,
    };

    assert_eq!(
        on.procedure(),
        [Invocation::Builtin(Builtin::EditModprobeBlacklist {
            module: module("mt76_usb"),
            on: true,
        })]
    );
    assert_eq!(
        shown(&on),
        ["add `blacklist mt76_usb` in /etc/modprobe.d/garuda-toolbox-blacklist.conf"]
    );
    assert_eq!(
        shown(&off),
        ["remove `blacklist wl` in /etc/modprobe.d/garuda-toolbox-blacklist.conf"]
    );
}

#[test]
fn a_module_that_is_an_option_a_path_or_two_lines_is_refused_on_decode() {
    for bad in [
        r#"{"kind":"load-module","module":"-r"}"#,
        r#"{"kind":"load-module","module":"a b"}"#,
        r#"{"kind":"load-module","module":""}"#,
        r#"{"kind":"unload-module","module":"../wl"}"#,
        r#"{"kind":"unload-module","module":"--remove-holders"}"#,
        r#"{"kind":"set-module-blacklisted","module":"wl\ninstall x /bin/sh","blacklisted":true}"#,
        r#"{"kind":"set-module-blacklisted","module":"wl"}"#,
    ] {
        assert!(serde_json::from_str::<RootAction>(bad).is_err(), "{bad}");
    }
}

#[test]
fn module_actions_survive_the_wire() {
    let action = RootAction::SetModuleBlacklisted {
        module: module("b43"),
        blacklisted: true,
    };

    let json = serde_json::to_string(&action).unwrap();

    assert_eq!(
        json,
        r#"{"kind":"set-module-blacklisted","module":"b43","blacklisted":true}"#
    );
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
}

#[test]
fn ping_runs_as_the_user_with_checked_numbers_and_a_separator() {
    let action = UserAction::Ping {
        host: HostName::new("garudalinux.org").unwrap(),
        count: PingCount::new(4).unwrap(),
        timeout: PingTimeout::new(5).unwrap(),
    };

    assert_eq!(shown_user(&action), ["ping -c 4 -W 5 -- garudalinux.org"]);
}

#[test]
fn traceroute_limits_its_hops() {
    let action = UserAction::Traceroute {
        host: HostName::new("2a01:4f8::1").unwrap(),
        max_hops: HopLimit::new(30).unwrap(),
    };

    assert_eq!(shown_user(&action), ["traceroute -m 30 -- 2a01:4f8::1"]);
}

#[test]
fn a_test_with_a_bad_host_or_number_is_refused_on_decode() {
    for bad in [
        r#"{"kind":"ping","host":"a.org","count":100000,"timeout":5}"#,
        r#"{"kind":"ping","host":"-f","count":4,"timeout":5}"#,
        r#"{"kind":"ping","host":"a.org","count":4,"timeout":0}"#,
        r#"{"kind":"traceroute","host":"a.org","max_hops":0}"#,
        r#"{"kind":"traceroute","host":"a.org b.org","max_hops":3}"#,
    ] {
        assert!(serde_json::from_str::<UserAction>(bad).is_err(), "{bad}");
    }
}
