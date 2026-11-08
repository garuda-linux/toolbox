//! The skel catalog, `skel.toml`: application configs the Maintenance page
//! resets by copying Garuda's defaults from `/etc/skel` into the home.
//!
//! ```toml
//! [[config]]
//! id = "bash"
//! name = "Bash"
//! files = ["/etc/skel/.bashrc_garuda", "/etc/skel/.config/autostart/bashrc-setup.desktop"]
//! ```

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::{CatalogError, Problem, source};

pub const SKEL_ROOT: &str = "/etc/skel";

const SKEL: &str = "skel.toml";

/// The validated entries of `skel.toml`, in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkelCatalog {
    entries: Vec<SkelEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkelEntry {
    id: String,
    name: String,
    files: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    #[serde(default)]
    config: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: String,
    name: String,
    files: Vec<String>,
}

impl SkelCatalog {
    /// Reads and validates `skel.toml` in the catalog folder `dir`.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Read`] when the file cannot be read, otherwise the
    /// errors of [`SkelCatalog::parse`].
    pub fn load(dir: &Path) -> Result<Self, CatalogError> {
        let (text, source_name) = source::read(dir, SKEL)?;

        Self::parse(&text, &source_name)
    }

    /// Parses and validates the text of a skel catalog. `source_name` names
    /// the file in errors.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Syntax`] for invalid TOML, unknown or missing fields,
    /// [`CatalogError::Invalid`] for the first entry that breaks a rule.
    pub fn parse(text: &str, source_name: &str) -> Result<Self, CatalogError> {
        let raw: RawCatalog = source::parse(text, source_name)?;
        let mut seen = BTreeSet::new();
        let mut entries = Vec::with_capacity(raw.config.len());

        for entry in raw.config {
            let invalid = |problem| CatalogError::Invalid {
                source_name: source_name.to_owned(),
                id: entry.id.clone(),
                problem,
            };

            source::claim_id(&entry.id, &mut seen).map_err(invalid)?;

            if source::is_blank(&entry.name) {
                return Err(invalid(Problem::EmptyName));
            }

            if entry.files.is_empty() {
                return Err(invalid(Problem::NoFiles));
            }

            let files = entry
                .files
                .iter()
                .map(String::as_str)
                .map(below_skel)
                .collect::<Result<Vec<_>, _>>()
                .map_err(invalid)?;

            entries.push(SkelEntry {
                id: entry.id,
                name: entry.name,
                files,
            });
        }

        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[SkelEntry] {
        &self.entries
    }

    pub fn entry(&self, id: &str) -> Option<&SkelEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }
}

impl SkelEntry {
    /// The stable id, lowercase words joined by dashes.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The English name, translated in the UI.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The files and folders, relative to [`SKEL_ROOT`] and to the home.
    /// Each has at least one component and none is `..`.
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}

fn below_skel(file: &str) -> Result<PathBuf, Problem> {
    let outside = || Problem::OutsideSkel(file.to_owned());
    let rest = file
        .strip_prefix(SKEL_ROOT)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(outside)?;

    let mut relative = PathBuf::new();

    for component in Path::new(rest).components() {
        match component {
            Component::Normal(word) => relative.push(word),
            Component::CurDir => {}
            Component::ParentDir => return Err(Problem::ParentComponent(file.to_owned())),
            Component::RootDir | Component::Prefix(_) => return Err(outside()),
        }
    }

    if relative.as_os_str().is_empty() {
        return Err(outside());
    }

    Ok(relative)
}
