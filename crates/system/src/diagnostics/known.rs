//! The names and serials an upload replaces.
//!
//! Read once at start, as the user: the kernel keeps most DMI serials for
//! root, and an unreadable one stays unknown.

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use toolbox_plan::UserName;
use toolbox_state::diagnostics::Known;

use crate::host::{HostSource as _, SystemHost};

const SERIALS: [&str; 4] = [
    "product_serial",
    "board_serial",
    "chassis_serial",
    "chassis_asset_tag",
];

/// What vendors write where a serial belongs. Replacing it would hide
/// words in the rest of the report and protect no one.
const FILLERS: [&str; 8] = [
    "to be filled by o.e.m.",
    "default string",
    "not specified",
    "not applicable",
    "system serial number",
    "chassis serial number",
    "base board serial number",
    "none",
];

/// Shortest serial worth replacing, so a `0` or `123` stays in the text.
const SHORTEST: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRoots {
    pub dmi: PathBuf,
    pub hostname: PathBuf,
}

impl KnownRoots {
    #[must_use = "the paths are only computed, nothing is read"]
    pub fn system() -> Self {
        Self {
            dmi: PathBuf::from("/sys/class/dmi/id"),
            hostname: PathBuf::from("/proc/sys/kernel/hostname"),
        }
    }
}

pub fn known(roots: &KnownRoots, user: Option<&UserName>, home: Option<&Path>) -> Known {
    let serials = SERIALS
        .iter()
        .filter_map(|name| first_line(&roots.dmi.join(name)))
        .filter(|serial| is_serial(serial))
        .collect();

    Known {
        user: user.map(|user| user.as_str().to_owned()),
        host: first_line(&roots.hostname),
        home: home
            .filter(|home| home.is_absolute() && home != &Path::new("/"))
            .and_then(Path::to_str)
            .map(str::to_owned),
        serials,
    }
}

/// [`known`] below `roots` for the user the app runs as.
pub fn of_process(roots: &KnownRoots) -> Known {
    let user = SystemHost::from_process().user().ok();
    let home = std::env::var_os("HOME").map(PathBuf::from);

    known(roots, user.as_ref(), home.as_deref())
}

fn first_line(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let line = text.lines().next()?.trim();

    (!line.is_empty()).then(|| line.to_owned())
}

fn is_serial(value: &str) -> bool {
    value.len() >= SHORTEST && !FILLERS.contains(&value.to_ascii_lowercase().as_str())
}
