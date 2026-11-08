#[cfg(test)]
mod tests;

use super::network::{start_network_manager, stop_network_manager};
use super::{run, systemctl};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{
    DnsProvider, GarudaUpdateFlag, LocaleEntry, LocaleName, ShellPath, UserName, WifiBackend,
};

const WPA_SUPPLICANT_UNIT: &str = "wpa_supplicant.service";
const HBLOCK_TIMER: &str = "hblock.timer";

/// chsh as root accepts any program, so the list check comes first. The
/// `--` keeps the user name from being read as an option.
pub fn set_shell(user: &UserName, shell: &ShellPath) -> Vec<Invocation> {
    vec![
        Invocation::Builtin(Builtin::RequireListedShell(shell.clone())),
        run(Command::new(Program::Chsh).args(["-s", shell.as_str(), "--", user.as_str()])),
    ]
}

/// NetworkManager reads its drop-ins only on start or reload.
pub fn set_dns(provider: DnsProvider) -> Vec<Invocation> {
    vec![
        Invocation::Builtin(Builtin::WriteDnsConfig(provider)),
        run(Command::new(Program::Nmcli).args(["general", "reload"])),
    ]
}

/// Off empties the block list in `/etc/hosts` again: `-S none -D none`
/// means no sources and no extra denied domains.
pub fn set_hblock(enabled: bool) -> Vec<Invocation> {
    if enabled {
        vec![
            run(systemctl(["enable", "--now", "--", HBLOCK_TIMER])),
            run(Command::new(Program::Hblock)),
        ]
    } else {
        vec![
            run(systemctl(["disable", "--now", "--", HBLOCK_TIMER])),
            run(Command::new(Program::Hblock).args(["-S", "none", "-D", "none"])),
        ]
    }
}

/// NetworkManager is stopped while its backend changes.
pub fn set_wifi_backend(backend: WifiBackend) -> Vec<Invocation> {
    let mut steps = vec![stop_network_manager()];

    match backend {
        WifiBackend::Iwd => steps.extend([
            run(systemctl(["disable", "--now", "--", WPA_SUPPLICANT_UNIT])),
            run(systemctl(["mask", "--", WPA_SUPPLICANT_UNIT])),
            Invocation::Builtin(Builtin::WriteWifiBackend(backend)),
        ]),
        WifiBackend::WpaSupplicant => steps.extend([
            Invocation::Builtin(Builtin::WriteWifiBackend(backend)),
            run(systemctl(["unmask", "--", WPA_SUPPLICANT_UNIT])),
            run(systemctl(["enable", "--now", "--", WPA_SUPPLICANT_UNIT])),
        ]),
    }

    steps.extend([run(systemctl(["daemon-reload"])), start_network_manager()]);
    steps
}

pub fn set_garuda_update_flag(flag: GarudaUpdateFlag, enabled: bool) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::SetGarudaUpdateFlag {
        flag,
        on: enabled,
    })]
}

pub fn set_locale_enabled(locale: &LocaleEntry, enabled: bool) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::EditLocaleGen {
        entry: locale.clone(),
        on: enabled,
    })]
}

/// The builtin checks `/etc/locale.gen` itself, so the check and the write
/// see the same file.
pub fn set_system_locale(locale: &LocaleName) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::SetSystemLocale(
        locale.clone(),
    ))]
}
