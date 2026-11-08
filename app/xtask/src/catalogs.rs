//! Every catalog the app ships, loaded together so the tasks see the same
//! set. A new catalog file adds its line here.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;

use toolbox_catalog::{
    APPS, COMPONENTS, CONFIG_FILES, CatalogError, ColourSchemeCatalog, ConfigFileCatalog, GAMING,
    Problem, ProtectedCatalog, SETTINGS, SETUP, SkelCatalog, ToggleCatalog,
};

/// The catalog folder, relative to the repository root.
pub const SOURCE_DIR: &str = "data/catalog";

/// The page catalogs read with [`ToggleCatalog`]. The Discover catalogs
/// in [`APPS`] follow them.
pub const TOGGLE_FILES: [&str; 4] = [COMPONENTS, SETTINGS, GAMING, SETUP];

#[derive(Debug)]
pub struct Catalogs {
    pub toggles: Vec<ToggleCatalog>,
    pub config_files: ConfigFileCatalog,
}

/// # Errors
///
/// One error per catalog that cannot be read or breaks a rule, and one per
/// missing entry icon, so a single run shows everything to fix.
pub fn load(dir: &Path) -> Result<Catalogs, Vec<CatalogError>> {
    let mut errors = Vec::new();
    let skel = SkelCatalog::load(dir)
        .map_err(|error| errors.push(error))
        .ok();

    let mut loaded = Vec::with_capacity(TOGGLE_FILES.len() + APPS.len());

    for file in TOGGLE_FILES.into_iter().chain(APPS) {
        match ToggleCatalog::load(dir, file) {
            Ok(catalog) => {
                errors.extend(missing_icons(&catalog, dir, file));
                loaded.push((file, catalog));
            }
            Err(error) => errors.push(error),
        }
    }

    errors.extend(shared_app_ids(&loaded, dir));
    let toggles = loaded.into_iter().map(|(_, catalog)| catalog).collect();

    let config_files = ConfigFileCatalog::load(dir)
        .map_err(|error| errors.push(error))
        .ok();

    let colour_schemes = ColourSchemeCatalog::load(dir)
        .map_err(|error| errors.push(error))
        .ok();

    let protected = ProtectedCatalog::load(dir)
        .map_err(|error| errors.push(error))
        .ok();

    match (skel, config_files, colour_schemes, protected) {
        (Some(_), Some(config_files), Some(_), Some(_)) if errors.is_empty() => Ok(Catalogs {
            toggles,
            config_files,
        }),
        _ => Err(errors),
    }
}

/// One error per line of the helper's write table that no root-owned
/// entry of the config file catalog in `dir` needs. Only the shipped
/// catalog has to need them all, so [`load`] leaves this out.
pub fn unused_writable(dir: &Path, catalog: &ConfigFileCatalog) -> Vec<CatalogError> {
    catalog
        .unused_writable()
        .into_iter()
        .map(|line| CatalogError::Invalid {
            source_name: dir.join(CONFIG_FILES).display().to_string(),
            id: line.to_owned(),
            problem: Problem::NotWritable(line.to_owned()),
        })
        .collect()
}

/// The app shows no picture for a missing icon file, so only this check
/// notices one.
fn missing_icons(catalog: &ToggleCatalog, dir: &Path, file: &str) -> Vec<CatalogError> {
    let folder = ToggleCatalog::icon_dir(dir, file);

    catalog
        .entries()
        .filter_map(|entry| entry.icon().map(|icon| (entry.id(), icon)))
        .filter(|(_, icon)| !folder.join(icon).is_file())
        .map(|(id, icon)| CatalogError::Invalid {
            source_name: dir.join(file).display().to_string(),
            id: id.to_owned(),
            problem: Problem::MissingIcon(icon.to_owned()),
        })
        .collect()
}

/// The Discover page toggles entries by id over all categories, so an id
/// may appear in one apps catalog only.
fn shared_app_ids(loaded: &[(&str, ToggleCatalog)], dir: &Path) -> Vec<CatalogError> {
    let mut seen = BTreeSet::new();

    loaded
        .iter()
        .filter(|(file, _)| APPS.contains(file))
        .flat_map(|(file, catalog)| catalog.entries().map(move |entry| (file, entry.id())))
        .filter(|(_, id)| !seen.insert(*id))
        .map(|(file, id)| CatalogError::Invalid {
            source_name: dir.join(file).display().to_string(),
            id: id.to_owned(),
            problem: Problem::DuplicateId,
        })
        .collect()
}
