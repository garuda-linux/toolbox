//! The files behind the System Settings page: `/etc/shells` (read only),
//! NetworkManager's drop-ins and garuda-update's config. Every write goes
//! through the `secure` module, so a symlink planted in one of these
//! folders leads nowhere.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::procedure::SHELLS;
use toolbox_plan::{
    DNS_CONFIG, DnsProvider, GARUDA_UPDATE_CONFIG, GarudaUpdateFlag, NETWORKMANAGER_CONF_D,
    OLD_WIFI_BACKEND_CONFIG, ShellPath, WIFI_BACKEND_CONFIG, WifiBackend,
};

use crate::secure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileChange {
    Written,
    Removed,
    Unchanged,
}

/// # Errors
///
/// When the list cannot be read.
pub fn shell_is_listed(root: &Path, shell: &ShellPath) -> io::Result<bool> {
    let list = std::fs::read_to_string(root.join(SHELLS))?;

    Ok(list
        .lines()
        .map(str::trim)
        .any(|line| line == shell.as_str()))
}

/// The folder must exist: without it NetworkManager is not installed and
/// the file would do nothing.
///
/// # Errors
///
/// When the folder is missing or reached through a symlink, or the write
/// or removal fails.
pub fn write_dns_config(root: &Path, provider: DnsProvider) -> io::Result<FileChange> {
    let dir = secure::open_dir(root, Path::new(NETWORKMANAGER_CONF_D))?;

    match provider.config() {
        Some(content) => {
            secure::replace_system_file(&dir, DNS_CONFIG, content.as_bytes(), None)?;
            Ok(FileChange::Written)
        }
        None => Ok(removed(secure::remove_file(&dir, DNS_CONFIG)?)),
    }
}

/// # Errors
///
/// When the folder is missing or reached through a symlink, or the write
/// or removal fails.
pub fn write_wifi_backend(root: &Path, backend: WifiBackend) -> io::Result<FileChange> {
    let dir = secure::open_dir(root, Path::new(NETWORKMANAGER_CONF_D))?;

    if let Some(content) = backend.config() {
        secure::replace_system_file(&dir, WIFI_BACKEND_CONFIG, content.as_bytes(), None)?;
        return Ok(FileChange::Written);
    }

    let current = secure::remove_file(&dir, WIFI_BACKEND_CONFIG)?;
    let old = secure::remove_file(&dir, OLD_WIFI_BACKEND_CONFIG)?;

    Ok(removed(current || old))
}

/// Creates the folder and the file when they are missing.
///
/// # Errors
///
/// When a folder on the way is a symlink, the file is not UTF-8, or the
/// write fails.
pub fn set_garuda_update_flag(
    root: &Path,
    flag: GarudaUpdateFlag,
    on: bool,
) -> io::Result<FileChange> {
    let (folder, name) = secure::split(GARUDA_UPDATE_CONFIG)?;
    let dir = secure::create_dirs(root, folder)?;
    let current = secure::read_text(&dir, name)?.unwrap_or_default();
    let wanted = flag.set(&current, on);

    super::rewritten(&dir, name, &current, &wanted, None)
}

const fn removed(any: bool) -> FileChange {
    if any {
        FileChange::Removed
    } else {
        FileChange::Unchanged
    }
}
