//! The procedures of the Network page: kernel modules as root, ping and
//! traceroute as the user.

#[cfg(test)]
mod tests;

use super::{run, systemctl};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{HopLimit, HostName, KernelModule, PingCount, PingTimeout};

const NETWORK_MANAGER: &str = "NetworkManager.service";

/// The wireless stack every Wi-Fi driver hooks into.
const CFG80211: &str = "cfg80211";

const WPA_SUPPLICANT_PROCESS: &str = "wpa_supplicant";

/// `wpa_supplicant` ends so it starts again with the new device. Every
/// part must succeed, so the step fails on a system without
/// NetworkManager or without a running `wpa_supplicant`.
pub fn load_module(module: &KernelModule) -> Vec<Invocation> {
    vec![
        stop_network_manager(),
        run(modprobe().args(["--", CFG80211])),
        run(modprobe().args(["--", module.as_str()])),
        run(Command::new(Program::Pkill).args(["-x", "--", WPA_SUPPLICANT_PROCESS])),
        start_network_manager(),
    ]
}

pub fn unload_module(module: &KernelModule) -> Vec<Invocation> {
    vec![
        stop_network_manager(),
        run(modprobe().args(["-r", "--", module.as_str()])),
        start_network_manager(),
    ]
}

pub fn set_module_blacklisted(module: &KernelModule, blacklisted: bool) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::EditModprobeBlacklist {
        module: module.clone(),
        on: blacklisted,
    })]
}

/// `-W` is the wait for each answer. The host follows a `--`, and its
/// rule keeps a leading `-` out anyway.
pub fn ping(host: &HostName, count: PingCount, timeout: PingTimeout) -> Vec<Invocation> {
    vec![run(Command::new(Program::Ping).args([
        "-c".to_owned(),
        count.to_string(),
        "-W".to_owned(),
        timeout.to_string(),
        "--".to_owned(),
        host.to_string(),
    ]))]
}

pub fn traceroute(host: &HostName, max_hops: HopLimit) -> Vec<Invocation> {
    vec![run(Command::new(Program::Traceroute).args([
        "-m".to_owned(),
        max_hops.to_string(),
        "--".to_owned(),
        host.to_string(),
    ]))]
}

const fn modprobe() -> Command {
    Command::new(Program::Modprobe)
}

pub fn stop_network_manager() -> Invocation {
    run(systemctl(["stop", "--", NETWORK_MANAGER]))
}

/// Runs whatever happened before, so a failed or cancelled step never
/// leaves the machine without network management.
pub fn start_network_manager() -> Invocation {
    Invocation::Always(systemctl(["start", "--", NETWORK_MANAGER]))
}
