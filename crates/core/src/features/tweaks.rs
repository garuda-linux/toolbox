//! The System Tweaks page's choices: the login shell, the DNS servers,
//! the Wi-Fi backend, hblock and garuda-update's options.

#[cfg(test)]
mod tests;

use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};

use crate::domains::{Dns, GarudaUpdateFlags, Hblock, LoginShells, WifiBackends};
use crate::{ChangeSet, Single, Snapshot, Toggle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tweak {
    Shell(UserName, ShellPath),
    Dns(DnsProvider),
    Wifi(WifiBackend),
    Hblock(Toggle),
    Flag(GarudaUpdateFlag, Toggle),
}

/// Queues `tweak`. One the system already has is not queued, and takes
/// back a different one queued before.
pub fn queue(changes: &mut ChangeSet, tweak: Tweak, snapshot: &Snapshot) {
    match tweak {
        Tweak::Shell(user, shell) => changes.want::<LoginShells>(user, shell, snapshot),
        Tweak::Dns(provider) => changes.want::<Dns>(Single, provider, snapshot),
        Tweak::Wifi(backend) => changes.want::<WifiBackends>(Single, backend, snapshot),
        Tweak::Hblock(wish) => changes.want::<Hblock>(Single, wish, snapshot),
        Tweak::Flag(flag, wish) => changes.want::<GarudaUpdateFlags>(flag, wish, snapshot),
    }
}
