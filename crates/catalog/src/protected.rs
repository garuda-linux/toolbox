//! The protected catalog, `protected.toml`: packages no page and no
//! command removes, each with the reason it stays.
//!
//! ```toml
//! [[package]]
//! name = "sudo"
//! why = "root"
//! ```

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use toolbox_plan::PackageName;
use toolbox_state::packages::{ProtectedPackages, Protection};

use crate::{CatalogError, Problem, source};

pub const PROTECTED: &str = "protected.toml";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProtectedCatalog {
    packages: ProtectedPackages,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    #[serde(default)]
    package: Vec<RawPackage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackage {
    name: String,
    why: Protection,
}

impl ProtectedCatalog {
    /// Reads and validates `protected.toml` in the catalog folder `dir`.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Read`] when the file cannot be read, otherwise the
    /// errors of [`ProtectedCatalog::parse`].
    pub fn load(dir: &Path) -> Result<Self, CatalogError> {
        let (text, source_name) = source::read(dir, PROTECTED)?;

        Self::parse(&text, &source_name)
    }

    /// # Errors
    ///
    /// [`CatalogError::Syntax`] for invalid TOML, unknown fields or an
    /// unknown reason, [`CatalogError::Invalid`] for a bad or repeated
    /// package name.
    pub fn parse(text: &str, source_name: &str) -> Result<Self, CatalogError> {
        let raw: RawCatalog = source::parse(text, source_name)?;
        let mut packages = BTreeMap::new();

        for package in raw.package {
            let invalid = |problem| CatalogError::Invalid {
                source_name: source_name.to_owned(),
                id: package.name.clone(),
                problem,
            };

            let name = PackageName::new(&package.name)
                .map_err(Problem::BadName)
                .map_err(invalid)?;

            if packages.insert(name, package.why).is_some() {
                return Err(invalid(Problem::DuplicateId));
            }
        }

        Ok(Self {
            packages: ProtectedPackages(packages),
        })
    }

    pub const fn packages(&self) -> &ProtectedPackages {
        &self.packages
    }
}
