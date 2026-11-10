//! The Wi-Fi backend NetworkManager would pick, from its config files in
//! the order `NetworkManager.conf(5)` gives: the packages' drop-ins, the
//! per-boot ones, the main file, then the administrator's drop-ins. A
//! drop-in is masked by one of its name in a folder read later.
//!
//! Only the plain `[device]` section is read. A `[device-<name>]` section
//! holds for the devices it matches, which the page does not tell apart.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use toolbox_plan::{NETWORKMANAGER_CONF_D, WifiBackend};

use crate::error::{ProbeError, from_io};
use crate::files;

const PACKAGE_CONF_D: &str = "usr/lib/NetworkManager/conf.d";
const BOOT_CONF_D: &str = "run/NetworkManager/conf.d";
const MAIN_FILE: &str = "etc/NetworkManager/NetworkManager.conf";

const SECTION: &str = "[device]";
const KEY: &str = "wifi.backend";
const IWD: &str = "iwd";

/// # Errors
///
/// [`ProbeError::Unavailable`] without the administrator's drop-in
/// folder, [`ProbeError::Failed`] when it cannot be listed.
pub(super) fn backend(root: &Path) -> Result<WifiBackend, ProbeError> {
    let folder = root.join(NETWORKMANAGER_CONF_D);
    let own = names(&folder).map_err(|error| from_io(folder.display(), error))?;
    let boot = optional_names(&root.join(BOOT_CONF_D));
    let packaged = optional_names(&root.join(PACKAGE_CONF_D));

    let files = packaged
        .iter()
        .filter(|name| !boot.contains(*name) && !own.contains(*name))
        .map(|name| root.join(PACKAGE_CONF_D).join(name))
        .chain(
            boot.iter()
                .filter(|name| !own.contains(*name))
                .map(|name| root.join(BOOT_CONF_D).join(name)),
        )
        .chain([root.join(MAIN_FILE)])
        .chain(own.iter().map(|name| folder.join(name)));

    let iwd = files
        .filter_map(|file| read(&file))
        .filter_map(|text| backend_of(&text).map(|value| value == IWD))
        .next_back()
        .unwrap_or(false);

    Ok(if iwd {
        WifiBackend::Iwd
    } else {
        WifiBackend::WpaSupplicant
    })
}

fn backend_of(config: &str) -> Option<&str> {
    let mut inside = false;
    let mut value = None;

    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == SECTION;
            continue;
        }

        if inside
            && !line.starts_with('#')
            && let Some((key, written)) = line.split_once('=')
            && key.trim() == KEY
        {
            value = Some(written.trim());
        }
    }

    value
}

/// The `.conf` files of a folder in the order they are read. A link to
/// nowhere counts: it masks.
fn names(folder: &Path) -> std::io::Result<BTreeSet<OsString>> {
    Ok(std::fs::read_dir(folder)?
        .filter_map(Result::ok)
        .map(|entry| PathBuf::from(entry.file_name()))
        .filter(|name| {
            name.extension()
                .is_some_and(|extension| extension == "conf")
        })
        .map(PathBuf::into_os_string)
        .collect())
}

fn optional_names(folder: &Path) -> BTreeSet<OsString> {
    names(folder).unwrap_or_else(|error| {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::debug!("skipped {}: {error}", folder.display());
        }

        BTreeSet::new()
    })
}

/// A file that cannot be read is skipped: NetworkManager reads as root,
/// so an unreadable one may well select iwd, but the page cannot know.
fn read(file: &Path) -> Option<String> {
    files::read_lossy(file)
        .inspect_err(|error| {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::debug!("skipped {}: {error}", file.display());
            }
        })
        .ok()
}
