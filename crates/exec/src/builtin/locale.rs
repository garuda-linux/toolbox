//! The files behind the Locales page: `/etc/locale.gen` and
//! `/etc/locale.conf`. The edits are computed in `toolbox-plan` and written
//! through the `secure` module, so only the wanted line changes and a
//! symlink planted in `/etc` leads nowhere.

#[cfg(test)]
mod tests;

use std::io;
use std::os::fd::OwnedFd;
use std::path::Path;

use toolbox_plan::{
    LOCALE_CONF, LOCALE_GEN, LocaleEntry, LocaleName, is_builtin_locale, set_system_lang,
};

use super::FileChange;
use crate::secure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemLocaleChange {
    Changed(FileChange),
    /// `/etc/locale.gen` has no enabled locale that provides it, so the
    /// file stays as it is.
    NotEnabled,
}

/// Switches `entry` in `/etc/locale.gen` below `root` on or off, touching
/// only its line. Creates the file when it is missing.
///
/// # Errors
///
/// When `etc` is missing or reached through a symlink, the file is not
/// UTF-8, or the write fails.
pub fn edit_locale_gen(root: &Path, entry: &LocaleEntry, on: bool) -> io::Result<FileChange> {
    edit(root, LOCALE_GEN, |current| entry.set(current, on))
}

/// Sets `LANG` in `/etc/locale.conf` below `root`, unless `/etc/locale.gen`
/// has no enabled locale for it.
///
/// # Errors
///
/// When `etc` is missing or reached through a symlink, a file is not
/// UTF-8, or the write fails.
pub fn set_system_locale(root: &Path, locale: &LocaleName) -> io::Result<SystemLocaleChange> {
    if !is_builtin_locale(locale) {
        let (dir, name) = secure::open_parent(root, LOCALE_GEN)?;
        let enabled = LocaleEntry::enabled_in(&read(&dir, name)?);

        if !enabled.iter().any(|entry| entry.provides(locale)) {
            return Ok(SystemLocaleChange::NotEnabled);
        }
    }

    edit(root, LOCALE_CONF, |current| {
        set_system_lang(current, locale)
    })
    .map(SystemLocaleChange::Changed)
}

fn edit<F>(root: &Path, file: &'static str, change: F) -> io::Result<FileChange>
where
    F: FnOnce(&str) -> String,
{
    let (dir, name) = secure::open_parent(root, file)?;
    let current = read(&dir, name)?;
    let wanted = change(&current);

    super::rewritten(&dir, name, &current, &wanted, None)
}

/// A missing file reads as empty: the edit then creates it.
fn read(dir: &OwnedFd, name: &str) -> io::Result<String> {
    Ok(secure::read_text(dir, name)?.unwrap_or_default())
}
