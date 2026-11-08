//! The config file catalog, `config-files.toml`: the files the Config
//! Files page offers, by section.
//!
//! ```toml
//! [[section]]
//! id = "bootloader"
//! title = "Bootloader"
//! icon = "system-reboot"
//!
//! [[section.entry]]
//! id = "grub-default"
//! name = "GRUB default"
//! path = "/etc/default/grub"
//! description = "GRUB configuration variables"
//! url = "https://wiki.archlinux.org/title/GRUB"
//! ```
//!
//! A `path` that starts with `~/` is a file of the user, any other must be
//! a root-owned file below the folders the helper edits. `directory =
//! true` makes an entry list the files of a folder. `read_only = true`
//! shows a root-owned entry without an edit: it hands out root or the
//! accounts, so the helper's write table must leave it out.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;
use toolbox_plan::{
    ConfigEntryLocation, ConfigFilePath, HomePath, WRITABLE_CONFIG_FILES, WRITABLE_CONFIG_FOLDERS,
};

use crate::source::RawCatalog;
use crate::{CatalogError, Problem, is_web_url, source};

pub const CONFIG_FILES: &str = "config-files.toml";

const HOME_PREFIX: &str = "~/";

/// The validated sections of `config-files.toml`, in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigFileCatalog {
    sections: Vec<ConfigSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigSection {
    id: String,
    title: String,
    icon: String,
    entries: Vec<ConfigEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigEntry {
    id: String,
    name: String,
    description: Option<String>,
    location: ConfigEntryLocation,
    directory: bool,
    read_only: bool,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: String,
    name: String,
    path: String,
    description: Option<String>,
    url: Option<String>,
    #[serde(default)]
    directory: bool,
    #[serde(default)]
    read_only: bool,
}

impl ConfigFileCatalog {
    /// Reads and validates `config-files.toml` in the catalog folder `dir`.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Read`] when the file cannot be read, otherwise the
    /// errors of [`ConfigFileCatalog::parse`].
    pub fn load(dir: &Path) -> Result<Self, CatalogError> {
        let (text, source_name) = source::read(dir, CONFIG_FILES)?;

        Self::parse(&text, &source_name)
    }

    /// Parses and validates the text of the catalog. `source_name` names
    /// the file in errors.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Syntax`] for invalid TOML, unknown or missing fields,
    /// [`CatalogError::Invalid`] for the first section or entry that breaks
    /// a rule.
    pub fn parse(text: &str, source_name: &str) -> Result<Self, CatalogError> {
        let raw: RawCatalog<RawEntry> = source::parse(text, source_name)?;
        let mut seen = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut sections = Vec::with_capacity(raw.section.len());

        for section in raw.section {
            let invalid = |id: &str, problem| CatalogError::Invalid {
                source_name: source_name.to_owned(),
                id: id.to_owned(),
                problem,
            };

            source::claim_id(&section.id, &mut seen)
                .and_then(|()| {
                    source::check_section(&section.title, &section.icon, section.entry.len())
                })
                .map_err(|problem| invalid(&section.id, problem))?;

            let mut entries = Vec::with_capacity(section.entry.len());

            for entry in section.entry {
                let id = entry.id.clone();

                source::claim_id(&id, &mut seen).map_err(|problem| invalid(&id, problem))?;

                if !paths.insert(entry.path.clone()) {
                    return Err(invalid(&id, Problem::DuplicatePath(entry.path)));
                }

                entries.push(ConfigEntry::new(entry).map_err(|problem| invalid(&id, problem))?);
            }

            sections.push(ConfigSection {
                id: section.id,
                title: section.title,
                icon: section.icon,
                entries,
            });
        }

        Ok(Self { sections })
    }

    pub fn sections(&self) -> &[ConfigSection] {
        &self.sections
    }

    /// Lines of the write table that no root-owned entry needs. `parse` does not
    /// check them, `cargo xtask catalog-validate` does.
    pub fn unused_writable(&self) -> Vec<&'static str> {
        let roots: BTreeSet<&str> = self
            .entries()
            .filter(|entry| !entry.is_read_only())
            .filter_map(|entry| match entry.location() {
                ConfigEntryLocation::Root(path) => Some(path.as_str()),
                ConfigEntryLocation::Home(_) => None,
            })
            .collect();

        WRITABLE_CONFIG_FILES
            .into_iter()
            .chain(WRITABLE_CONFIG_FOLDERS)
            .filter(|line| !roots.contains(line))
            .collect()
    }

    pub fn entry(&self, id: &str) -> Option<&ConfigEntry> {
        self.entries().find(|entry| entry.id == id)
    }

    fn entries(&self) -> impl Iterator<Item = &ConfigEntry> {
        self.sections.iter().flat_map(ConfigSection::entries)
    }
}

impl ConfigSection {
    /// The stable id, lowercase words joined by dashes.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The English heading, translated in the UI.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// A freedesktop icon name.
    pub fn icon(&self) -> &str {
        &self.icon
    }

    pub fn entries(&self) -> &[ConfigEntry] {
        &self.entries
    }
}

impl ConfigEntry {
    fn new(raw: RawEntry) -> Result<Self, Problem> {
        if source::is_blank(&raw.name) {
            return Err(Problem::EmptyName);
        }

        if raw.description.as_deref().is_some_and(source::is_blank) {
            return Err(Problem::EmptyDescription);
        }

        if let Some(url) = raw.url.as_ref().filter(|url| !is_web_url(url)) {
            return Err(Problem::BadUrl(url.clone()));
        }

        let location = match raw.path.strip_prefix(HOME_PREFIX) {
            Some(below_home) => HomePath::new(below_home)
                .map(ConfigEntryLocation::Home)
                .map_err(Problem::BadHomePath)?,
            None => ConfigFilePath::new(raw.path)
                .map(ConfigEntryLocation::Root)
                .map_err(|error| Problem::BadConfigPath(error.to_string()))?,
        };

        match &location {
            ConfigEntryLocation::Home(_) if raw.read_only => return Err(Problem::ReadOnlyHome),
            ConfigEntryLocation::Home(_) => {}
            ConfigEntryLocation::Root(path) if raw.read_only => check_read_only(path)?,
            ConfigEntryLocation::Root(path) => {
                let table: &[&str] = if raw.directory {
                    &WRITABLE_CONFIG_FOLDERS
                } else {
                    &WRITABLE_CONFIG_FILES
                };

                if !table.contains(&path.as_str()) {
                    return Err(Problem::NotWritable(path.to_string()));
                }
            }
        }

        Ok(Self {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            location,
            directory: raw.directory,
            read_only: raw.read_only,
            url: raw.url,
        })
    }

    /// The stable id, lowercase words joined by dashes.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The English name, translated in the UI.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What the file is for, in English.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub const fn location(&self) -> &ConfigEntryLocation {
        &self.location
    }

    pub const fn is_directory(&self) -> bool {
        self.directory
    }

    /// Whether the page shows the entry without an edit: it hands out root
    /// or the accounts, so the helper's write table leaves it out.
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// The page about the file, an `https` link.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

/// A read-only entry is neither a line of the write table nor a file in
/// one of its folders, or the helper would write what the page only shows.
fn check_read_only(path: &ConfigFilePath) -> Result<(), Problem> {
    let text = path.as_str();

    if path.is_writable() || WRITABLE_CONFIG_FOLDERS.contains(&text) {
        return Err(Problem::ReadOnlyWritable(text.to_owned()));
    }

    Ok(())
}
