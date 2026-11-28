//! System Tweaks' system-wide choices as plain data: what each shows (the
//! queued wish, else what the system has), whether a probe delivered it
//! and whether a wish waits.

#[cfg(test)]
mod tests;

use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};
use toolbox_state::probes::{FailedProbes, Probe};

use crate::domains::{
    AvailableShells, CurrentDns, CurrentShells, CurrentWifiBackend, Dns, GarudaUpdateConfig,
    GarudaUpdateFlags, Hblock, HblockState, LoginShells, WifiBackends,
};
use crate::features::toggles::shown::Tally;
use crate::{ChangeSet, Single, Snapshot, Toggle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick<T> {
    pub options: Vec<T>,
    /// The option shown, `None` when the value is none of them.
    pub shown: Option<usize>,
    pub read: Read,
    pub queued: bool,
}

/// What became of the system's value of a [`Pick`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    Known,
    /// The first probe has not ended yet.
    NotYet,
    /// Its probe failed, so the value may still be there.
    Failed,
    /// The system lacks the thing.
    Absent,
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
        let shell_probes = [Probe::LoginShells, Probe::AvailableShells, Probe::User];

        Self {
            shell: pick(
                available,
                (current_shell, read(snapshot, current_shell, &shell_probes)),
                user.and_then(|user| changes.wish::<LoginShells>(user)),
            ),
            dns: {
                let current = snapshot.part::<CurrentDns>().map(|dns| &dns.0);
                pick(
                    DnsProvider::ALL.to_vec(),
                    (current, read(snapshot, current, &[Probe::Dns])),
                    changes.wish::<Dns>(&Single),
                )
            },
            wifi: {
                let current = snapshot.part::<CurrentWifiBackend>().map(|wifi| &wifi.0);
                pick(
                    WifiBackend::ALL.to_vec(),
                    (current, read(snapshot, current, &[Probe::WifiBackend])),
                    changes.wish::<WifiBackends>(&Single),
                )
            },
            hblock: flip(
                snapshot
                    .part::<HblockState>()
                    .map(|state| Toggle::from_on(state.0)),
                changes.wish::<Hblock>(&Single),
            ),
            flags: config.map(|config| {
                GarudaUpdateFlag::ALL
                    .into_iter()
                    .map(|flag| {
                        let wish = changes.wish::<GarudaUpdateFlags>(&flag);
                        let system = Toggle::from_on(config.0.contains(&flag));
                        (flag, flip(Some(system), wish))
                    })
                    .collect()
            }),
        }
    }
}

impl Choices {
    /// The Shell and network switches `shown` lets through, how many of
    /// them are on and whether their state was read. Ad blocking is the
    /// section's one switch, a picker is never on.
    pub fn shell_network_tally(&self, shown: impl Fn(&str) -> bool) -> (Tally, bool) {
        let shown = shown("tweaks-hblock");
        let tally = Tally {
            on: usize::from(shown && self.hblock.on),
            total: usize::from(shown),
        };

        (tally, self.hblock.known)
    }
}

/// The garuda-update option's title id, a literal so the catalog checks see
/// each one.
pub const fn flag_id(flag: GarudaUpdateFlag) -> &'static str {
    match flag {
        GarudaUpdateFlag::SkipMirrorlist => "tweaks-flag-skip-mirrorlist",
        GarudaUpdateFlag::UpdateAur => "tweaks-flag-update-aur",
        GarudaUpdateFlag::PacmanNoconfirm => "tweaks-flag-pacman-noconfirm",
        GarudaUpdateFlag::NoCheckSpace => "tweaks-flag-no-check-space",
    }
}

/// The garuda-update options `shown` lets through, how many of them are on
/// and whether their state was read.
pub fn garuda_update_tally(
    flags: &[(GarudaUpdateFlag, Flip)],
    shown: impl Fn(&str) -> bool,
) -> (Tally, bool) {
    let matching = flags.iter().filter(|(flag, _)| shown(flag_id(*flag)));

    let tally = matching
        .clone()
        .map(|(_, flip)| Tally {
            on: usize::from(flip.on),
            total: 1,
        })
        .sum();

    (tally, matching.into_iter().all(|(_, flip)| flip.known))
}

fn pick<T>(options: Vec<T>, (current, read): (Option<&T>, Read), queued: Option<&T>) -> Pick<T>
where
    T: PartialEq,
{
    let shown = queued.or(current);

    Pick {
        shown: options.iter().position(|option| Some(option) == shown),
        options,
        read,
        queued: queued.is_some(),
    }
}

/// A snapshot without [`FailedProbes`] was not probed yet.
fn read<T>(snapshot: &Snapshot, current: Option<&T>, probes: &[Probe]) -> Read {
    if current.is_some() {
        return Read::Known;
    }

    match snapshot.part::<FailedProbes>() {
        None => Read::NotYet,
        Some(failed) if probes.iter().any(|probe| failed.contains(*probe)) => Read::Failed,
        Some(_) => Read::Absent,
    }
}

fn flip(system: Option<Toggle>, wish: Option<&Toggle>) -> Flip {
    Flip {
        known: system.is_some(),
        on: wish.map_or_else(|| system.is_some_and(Toggle::is_on), |wish| wish.is_on()),
        queued: wish.is_some(),
    }
}
