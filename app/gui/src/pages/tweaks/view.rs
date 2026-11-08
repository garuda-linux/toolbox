//! System Tweaks' system-wide choices as plain data: what each shows (the
//! queued wish, else what the system has), whether a probe delivered it
//! and whether a wish waits.

#[cfg(test)]
mod tests;

use toolbox_core::domains::{
    AvailableShells, CurrentDns, CurrentShells, CurrentWifiBackend, Dns, GarudaUpdateConfig,
    GarudaUpdateFlags, Hblock, HblockState, LoginShells, WifiBackends,
};
use toolbox_core::{ChangeSet, Single, Snapshot, Toggle};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick<T> {
    pub options: Vec<T>,
    /// The option shown, `None` when the value is none of them.
    pub shown: Option<usize>,
    pub known: bool,
    pub queued: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flip {
    pub known: bool,
    pub on: bool,
    pub queued: bool,
}

#[derive(Debug)]
pub struct Choices {
    pub shell: Pick<ShellPath>,
    pub dns: Pick<DnsProvider>,
    pub wifi: Pick<WifiBackend>,
    pub hblock: Flip,
    /// The garuda-update options, `None` without its configuration.
    pub flags: Option<Vec<(GarudaUpdateFlag, Flip)>>,
}

impl Choices {
    /// `user` is the account the app runs for, whose login shell shows.
    pub fn of(snapshot: &Snapshot, changes: &ChangeSet, user: Option<&UserName>) -> Self {
        let available = snapshot
            .part::<AvailableShells>()
            .map(|shells| shells.0.clone())
            .unwrap_or_default();

        let current_shell = user.and_then(|user| snapshot.part::<CurrentShells>()?.0.get(user));
        let config = snapshot.part::<GarudaUpdateConfig>();

        Self {
            shell: pick(
                available,
                current_shell,
                user.and_then(|user| changes.wish::<LoginShells>(user)),
            ),
            dns: pick(
                DnsProvider::ALL.to_vec(),
                snapshot.part::<CurrentDns>().map(|dns| &dns.0),
                changes.wish::<Dns>(&Single),
            ),
            wifi: pick(
                WifiBackend::ALL.to_vec(),
                snapshot.part::<CurrentWifiBackend>().map(|wifi| &wifi.0),
                changes.wish::<WifiBackends>(&Single),
            ),
            hblock: flip(
                snapshot.part::<HblockState>().map(|state| state.0),
                changes.wish::<Hblock>(&Single),
            ),
            flags: config.map(|config| {
                GarudaUpdateFlag::ALL
                    .into_iter()
                    .map(|flag| {
                        let wish = changes.wish::<GarudaUpdateFlags>(&flag);
                        (flag, flip(Some(config.0.contains(&flag)), wish))
                    })
                    .collect()
            }),
        }
    }
}

fn pick<T>(options: Vec<T>, current: Option<&T>, queued: Option<&T>) -> Pick<T>
where
    T: PartialEq,
{
    let shown = queued.or(current);

    Pick {
        shown: options.iter().position(|option| Some(option) == shown),
        options,
        known: current.is_some(),
        queued: queued.is_some(),
    }
}

fn flip(system: Option<bool>, wish: Option<&Toggle>) -> Flip {
    Flip {
        known: system.is_some(),
        on: wish.map_or_else(|| system.unwrap_or_default(), |wish| wish.is_on()),
        queued: wish.is_some(),
    }
}
