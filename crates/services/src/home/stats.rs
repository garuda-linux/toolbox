//! Home's stat tiles: the updates, the free disk space, garuda-health's
//! findings and the last update, each fine, worth a look or not known.

#[cfg(test)]
mod tests;

use toolbox_core::features::status::disk_low;
use toolbox_system::{Facts, HealthLevel};

use super::{Check, Overview};

const UNKNOWN: (Figure, Standing) = (Figure::Unknown, Standing::Unknown);

/// One tile, in the order Home shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Updates,
    Disk,
    Health,
    LastUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Fine,
    Attention,
    /// Not read yet, or the check gave no answer.
    Unknown,
}

/// The value a tile shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Figure {
    Unknown,
    /// Pending updates of the repositories and the AUR together.
    Updates(usize),
    /// Bytes a user may still write on the root file system.
    Free(u64),
    /// The worst level garuda-health found, `None` for nothing above a
    /// note.
    Health(Option<HealthLevel>),
    /// Days since the last update.
    Days(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub stat: Stat,
    pub figure: Figure,
    pub standing: Standing,
}

impl Stat {
    pub const ALL: [Self; 4] = [Self::Updates, Self::Disk, Self::Health, Self::LastUpdate];

    /// Stable code, part of the text ids.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Updates => "updates",
            Self::Disk => "disk",
            Self::Health => "health",
            Self::LastUpdate => "last-update",
        }
    }
}

/// The four tiles. `overview` is `None` until the status was read,
/// `facts` until the facts were.
pub fn tiles(overview: Option<&Overview<'_>>, facts: Option<&Facts>) -> [Tile; 4] {
    Stat::ALL.map(|stat| {
        let (figure, standing) = match stat {
            Stat::Updates => updates(overview),
            Stat::Disk => disk(facts),
            Stat::Health => health(overview),
            Stat::LastUpdate => last_update(overview),
        };

        Tile {
            stat,
            figure,
            standing,
        }
    })
}

fn updates(overview: Option<&Overview<'_>>) -> (Figure, Standing) {
    let Some(overview) = overview.filter(|overview| overview.ran.contains(&Check::Updates)) else {
        return UNKNOWN;
    };

    let count = overview.updates + overview.aur_updates;
    (Figure::Updates(count), standing(count > 0))
}

fn disk(facts: Option<&Facts>) -> (Figure, Standing) {
    let Some(disk) = facts.and_then(|facts| facts.disk.as_ref().ok()) else {
        return UNKNOWN;
    };

    (
        Figure::Free(disk.free),
        standing(disk_low(disk.free, disk.total)),
    )
}

fn health(overview: Option<&Overview<'_>>) -> (Figure, Standing) {
    let Some(overview) = overview.filter(|overview| overview.ran.contains(&Check::Health)) else {
        return UNKNOWN;
    };

    (
        Figure::Health(overview.health),
        standing(overview.health.is_some()),
    )
}

fn last_update(overview: Option<&Overview<'_>>) -> (Figure, Standing) {
    let Some(overview) = overview else {
        return UNKNOWN;
    };

    overview.days.map_or(UNKNOWN, |days| {
        (Figure::Days(days), standing(overview.stale))
    })
}

const fn standing(attention: bool) -> Standing {
    if attention {
        Standing::Attention
    } else {
        Standing::Fine
    }
}
