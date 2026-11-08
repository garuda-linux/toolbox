//! The switches of `/etc/garuda/garuda-update/config`, a shell file that
//! garuda-update sources. A switch is on when its last uncommented
//! assignment is `KEY=1`. Writing one touches only the lines of its key,
//! so comments and other settings stay byte for byte.
//! The probe and the helper both use this module, so what one writes the
//! other reads back the same way.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::lines::{ending, push_line};

/// Relative to `/`, like every path the helper writes below its root.
pub const GARUDA_UPDATE_CONFIG: &str = "etc/garuda/garuda-update/config";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GarudaUpdateFlag {
    SkipMirrorlist,
    UpdateAur,
    PacmanNoconfirm,
    NoCheckSpace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Line {
    Assignment {
        on: bool,
    },
    /// `# KEY=…`, the shape the shipped file uses for an unset switch.
    Commented,
    Unrelated,
}

impl GarudaUpdateFlag {
    pub const ALL: [Self; 4] = [
        Self::SkipMirrorlist,
        Self::UpdateAur,
        Self::PacmanNoconfirm,
        Self::NoCheckSpace,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SkipMirrorlist => "skip-mirrorlist",
            Self::UpdateAur => "update-aur",
            Self::PacmanNoconfirm => "pacman-noconfirm",
            Self::NoCheckSpace => "no-check-space",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::SkipMirrorlist => "SKIP_MIRRORLIST",
            Self::UpdateAur => "UPDATE_AUR",
            Self::PacmanNoconfirm => "PACMAN_NOCONFIRM",
            Self::NoCheckSpace => "NO_CHECK_SPACE",
        }
    }

    /// Whether the switch is on in `config`. The shell keeps the last
    /// assignment, so the last one counts.
    pub fn is_on(self, config: &str) -> bool {
        config
            .split_inclusive('\n')
            .rev()
            .find_map(|line| match classify(line, self.key()) {
                Line::Assignment { on } => Some(on),
                Line::Commented | Line::Unrelated => None,
            })
            .unwrap_or(false)
    }

    pub fn enabled_in(config: &str) -> BTreeSet<Self> {
        Self::ALL
            .into_iter()
            .filter(|flag| flag.is_on(config))
            .collect()
    }

    /// `config` with the switch set. On turns every assignment of the key
    /// into `KEY=1`, or else revives the first commented one, or else
    /// appends the line. Off comments every assignment out as `# KEY=1`.
    pub fn set(self, config: &str, on: bool) -> String {
        let key = self.key();
        let wanted = if on {
            format!("{key}=1")
        } else {
            format!("# {key}=1")
        };

        let assigned = config
            .split_inclusive('\n')
            .any(|line| matches!(classify(line, key), Line::Assignment { on: _ }));

        let mut placed = assigned;
        let mut result = String::with_capacity(config.len() + wanted.len() + 1);

        for line in config.split_inclusive('\n') {
            let replace = match classify(line, key) {
                Line::Assignment { on: _ } => true,
                Line::Commented => on && !placed,
                Line::Unrelated => false,
            };

            if replace {
                placed = true;
                result.push_str(&wanted);
                result.push_str(ending(line));
            } else {
                result.push_str(line);
            }
        }

        if on && !placed {
            push_line(&mut result, &wanted);
        }

        result
    }
}

fn classify(line: &str, key: &str) -> Line {
    let trimmed = line.trim();

    if let Some(value) = assignment(trimmed, key) {
        return Line::Assignment { on: is_on(value) };
    }

    match trimmed.strip_prefix('#') {
        Some(comment) if assignment(comment.trim_start(), key).is_some() => Line::Commented,
        Some(_) | None => Line::Unrelated,
    }
}

fn assignment<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.strip_prefix(key)?.strip_prefix('=')
}

/// `1`, quoted or not, with an optional trailing comment.
fn is_on(value: &str) -> bool {
    let value = value.split('#').next().unwrap_or_default().trim();

    value.trim_matches(['"', '\'']) == "1"
}
