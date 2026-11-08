//! The locales of the system, read from the files the root helper edits:
//! glibc's list of supported locales, `/etc/locale.gen` and
//! `/etc/locale.conf`.
//! All three are world readable.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use parking_lot::Mutex;
use toolbox_plan::{LOCALE_CONF, LOCALE_GEN, LocaleEntry, system_lang};
use toolbox_state::locales::{
    CurrentSystemLocale, EnabledLocales, LocaleNames, SupportedLocales, parse_identification,
    parse_supported, source_name,
};

use crate::error::ProbeError;
use crate::files;

const SUPPORTED: &str = "usr/share/i18n/SUPPORTED";

const SOURCES: &str = "usr/share/i18n/locales";

/// The names stand in the first part of a source. Some sources go on for
/// megabytes of collation rules after it.
const SOURCE_HEAD_BYTES: u64 = 8192;

pub trait LocaleSource: Send + Sync {
    /// Every locale glibc can generate.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without glibc's list (a system that
    /// does not generate its locales), [`ProbeError::Failed`] when it
    /// cannot be read.
    fn supported(&self) -> Result<SupportedLocales, ProbeError>;

    /// The entries `/etc/locale.gen` has switched on.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the file and without glibc's
    /// list, [`ProbeError::Failed`] when it cannot be read.
    fn enabled(&self) -> Result<EnabledLocales, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the file cannot be read.
    fn system_locale(&self) -> Result<CurrentSystemLocale, ProbeError>;

    /// What the supported locales are called, from glibc's own sources.
    /// A locale whose source is missing or names no language is left out,
    /// so a system without the sources has no names and no error.
    fn names(&self) -> LocaleNames;
}

#[derive(Debug)]
pub struct EtcLocales {
    root: PathBuf,
    /// The names as last read. Every snapshot asks for them, and reading
    /// them opens one source per locale.
    names: Mutex<Option<(Read, LocaleNames)>>,
}

/// When glibc's list and its folder of sources last changed. Both belong
/// to the glibc package, so only its update changes the names, and that
/// can happen while the app runs. Generating a locale touches neither.
type Read = [Option<SystemTime>; 2];

impl EtcLocales {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/")
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            names: Mutex::new(None),
        }
    }

    fn sources_changed(&self) -> Read {
        [SUPPORTED, SOURCES].map(|path| {
            std::fs::metadata(self.root.join(path))
                .and_then(|metadata| metadata.modified())
                .ok()
        })
    }

    fn read_names(&self) -> LocaleNames {
        let sources = self.root.join(SOURCES);
        let supported = self.supported().unwrap_or_default();
        let names = supported
            .0
            .iter()
            .map(|entry| source_name(entry.locale.as_str()))
            .collect::<BTreeSet<String>>()
            .into_iter()
            .filter_map(|source| {
                let name = parse_identification(&head(&sources.join(&source))?)?;

                Some((source, name))
            })
            .collect();

        LocaleNames(names)
    }
}

impl LocaleSource for EtcLocales {
    fn supported(&self) -> Result<SupportedLocales, ProbeError> {
        let text = files::read(&self.root.join(SUPPORTED))?;

        Ok(SupportedLocales(parse_supported(&text)))
    }

    /// A missing `locale.gen` beside glibc's list means nothing is
    /// enabled: the helper creates the file with the first change.
    fn enabled(&self) -> Result<EnabledLocales, ProbeError> {
        match files::read_optional(&self.root.join(LOCALE_GEN))? {
            Some(config) => Ok(EnabledLocales(LocaleEntry::enabled_in(&config))),
            None if self.root.join(SUPPORTED).is_file() => Ok(EnabledLocales::default()),
            None => Err(ProbeError::Unavailable),
        }
    }

    fn system_locale(&self) -> Result<CurrentSystemLocale, ProbeError> {
        let config = files::read_optional(&self.root.join(LOCALE_CONF))?;

        Ok(CurrentSystemLocale(config.as_deref().and_then(system_lang)))
    }

    fn names(&self) -> LocaleNames {
        let changed = self.sources_changed();
        let mut kept = self.names.lock();

        match kept.as_ref() {
            Some((read, names)) if *read == changed => names.clone(),
            Some(_) | None => {
                let names = self.read_names();

                *kept = Some((changed, names.clone()));
                names
            }
        }
    }
}

/// The start of a locale source, `None` when it cannot be read. The
/// names are plain text there, so bytes that are no UTF-8 further down
/// (or a cut inside a character) only cost that character.
fn head(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();

    File::open(path)
        .ok()?
        .take(SOURCE_HEAD_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;

    Some(String::from_utf8_lossy(&bytes).into_owned())
}
