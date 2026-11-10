//! Toggle catalogs: on/off entries grouped in sections, each entry shown as
//! on when its `check` holds. `components.toml`, `settings.toml`,
//! `gaming.toml` and `setup.toml` share the schema.
//!
//! ```toml
//! [[section]]
//! id = "containers"
//! title = "Containers"
//! icon = "docker"
//!
//! [[section.entry]]
//! id = "enabled-docker"
//! title = "Docker service"
//! description = "Runs the background daemon …"
//! check = { service = "docker.service" }
//! requires = { all = ["docker"] }
//! ```
//!
//! Entries that edit a config file in the user's home have their own
//! checks and fields, see the `user_config` module. `also`, `desktop`,
//! `icon` and `url` are in the `extras` module, `conflicts` in its own.

mod conflicts;
mod duplicates;
mod entry;
mod extras;
mod requires;
#[cfg(test)]
mod tests;
mod user_config;

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

pub use extras::is_web_url;
use extras::{Also, RawAlso};
pub use requires::Requires;
use serde::Deserialize;
use toolbox_plan::{ConfigReload, GroupName, PackageName, UnitName, UserConfigEdit};
use user_config::{RawUserFile, RawUserIni, RawUserJson};

use crate::source::{RawCatalog, RawSection};
use crate::{CatalogError, EntryProblem, source};

pub const COMPONENTS: &str = "components.toml";
pub const SETTINGS: &str = "settings.toml";
/// One section per step of the Setup Assistant.
pub const SETUP: &str = "setup.toml";
pub const GAMING: &str = "gaming.toml";
/// The folder below the catalog folder that holds one folder of entry
/// pictures per catalog, named after the catalog file.
const ICONS: &str = "icons";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToggleCatalog {
    sections: Vec<ToggleSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleSection {
    id: String,
    title: String,
    icon: String,
    entries: Vec<ToggleEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleEntry {
    id: String,
    title: String,
    description: String,
    check: Check,
    also: Also,
    conflicts: Vec<String>,
    requires: Requires,
    restart: Vec<UnitName>,
    reload: Option<ConfigReload>,
    desktops: Vec<String>,
    icon: Option<String>,
    url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Check {
    Package(PackageName),
    Service(UnitName),
    UserService(UnitName),
    Group(GroupName),
    UserConfig(UserConfigEdit),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: String,
    title: String,
    description: String,
    check: RawCheck,
    also: Option<Vec<RawAlso>>,
    conflicts: Option<Vec<String>>,
    #[serde(default)]
    requires: Requires,
    #[serde(default, rename = "restart-user-units")]
    restart_user_units: Vec<String>,
    reload: Option<String>,
    desktop: Option<Vec<String>>,
    icon: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
enum RawCheck {
    Package(String),
    Service(String),
    UserService(String),
    Group(String),
    UserFile(RawUserFile),
    UserIni(RawUserIni),
    UserJson(RawUserJson),
}

/// The check as the catalog spells its kind, then what it watches.
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Package(package) => write!(f, "package {package}"),
            Self::Service(unit) => write!(f, "service {unit}"),
            Self::UserService(unit) => write!(f, "user-service {unit}"),
            Self::Group(group) => write!(f, "group {group}"),
            Self::UserConfig(edit) => write!(f, "user-config {edit}"),
        }
    }
}

impl ToggleCatalog {
    /// # Errors
    ///
    /// [`CatalogError::Read`] when the file cannot be read, otherwise the
    /// errors of [`ToggleCatalog::parse`].
    pub fn load(dir: &Path, file: &str) -> Result<Self, CatalogError> {
        let (text, source_name) = source::read(dir, file)?;

        Self::parse(&text, &source_name)
    }

    /// # Errors
    ///
    /// [`CatalogError::Syntax`] for invalid TOML, unknown or missing fields
    /// and a `check` without exactly one known kind,
    /// [`CatalogError::Invalid`] for the first section or entry that breaks
    /// a rule.
    pub fn parse(text: &str, source_name: &str) -> Result<Self, CatalogError> {
        let invalid = |id: String, problem| CatalogError::Invalid {
            source_name: source_name.to_owned(),
            id,
            problem,
        };

        let raw: RawCatalog<RawEntry> = source::parse(text, source_name)?;
        let mut seen = BTreeSet::new();
        let mut sections = Vec::with_capacity(raw.section.len());

        for section in raw.section {
            sections.push(
                section
                    .validate(&mut seen)
                    .map_err(|EntryProblem { id, problem }| invalid(id, problem))?,
            );
        }

        let catalog = Self { sections };

        requires::check(&catalog)
            .and_then(|()| conflicts::check(&catalog))
            .and_then(|()| duplicates::check(&catalog))
            .map_err(|EntryProblem { id, problem }| invalid(id, problem))?;

        Ok(catalog)
    }

    /// Callers pass one of the catalog file names, which end in `.toml`.
    pub fn icon_dir(dir: &Path, file: &str) -> PathBuf {
        debug_assert!(
            Path::new(file)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("toml")),
            "toggle catalog file names end in .toml: {file}"
        );

        let name = file.strip_suffix(".toml").unwrap_or(file);

        dir.join(ICONS).join(name)
    }

    pub fn sections(&self) -> &[ToggleSection] {
        &self.sections
    }

    pub fn entries(&self) -> impl Iterator<Item = &ToggleEntry> {
        self.sections.iter().flat_map(ToggleSection::entries)
    }

    pub fn entry(&self, id: &str) -> Option<&ToggleEntry> {
        self.entries().find(|entry| entry.id == id)
    }

    /// The user config edits of all entries, the files the probe reads.
    pub fn config_edits(&self) -> impl Iterator<Item = &UserConfigEdit> {
        self.entries().filter_map(|entry| match &entry.check {
            Check::UserConfig(edit) => Some(edit),
            Check::Package(_) | Check::Service(_) | Check::UserService(_) | Check::Group(_) => None,
        })
    }
}

impl ToggleSection {
    /// The stable id, lowercase words joined by dashes.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The English title, translated in the UI.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// A freedesktop icon name.
    pub fn icon(&self) -> &str {
        &self.icon
    }

    /// At least one.
    pub fn entries(&self) -> &[ToggleEntry] {
        &self.entries
    }
}

impl RawSection<RawEntry> {
    fn validate(self, seen: &mut BTreeSet<String>) -> Result<ToggleSection, EntryProblem> {
        if let Err(problem) = source::claim_id(&self.id, seen)
            .and_then(|()| source::check_section(&self.title, &self.icon, self.entry.len()))
        {
            return Err(EntryProblem::new(self.id.clone(), problem));
        }

        let entries = self
            .entry
            .into_iter()
            .map(|entry| entry.validate(seen))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ToggleSection {
            id: self.id,
            title: self.title,
            icon: self.icon,
            entries,
        })
    }
}
