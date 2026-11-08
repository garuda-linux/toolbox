//! The catalogs the pages show, loaded once at start. A broken one only
//! empties its page, with the reason: the app runs on.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use toolbox_catalog::{
    APPS, COMPONENTS, Check, ColourSchemeCatalog, GAMING, LoadProblem, ProtectedCatalog, SETTINGS,
    SETUP, SkelCatalog, ToggleCatalog, ToggleEntry,
};
use toolbox_core::features::discover::Apps;
use toolbox_plan::UserConfigEdit;

/// The variable that names the catalog folder, for tests and odd installs.
const DIR_VARIABLE: &str = "GARUDA_TOOLBOX_CATALOG_DIR";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CatalogId {
    Components,
    Gaming,
    /// System Tweaks' toggles.
    Settings,
    Setup,
}

/// One toggle catalog: its page shows it and the snapshot reads the config
/// files it edits.
#[derive(Debug)]
pub struct Toggles {
    pub catalog: Result<ToggleCatalog, LoadProblem>,
    pub icon_dir: PathBuf,
}

/// The Discover categories, one catalog each. A broken one is left out
/// with its reason, the others still show.
#[derive(Debug, Default)]
pub struct AppCatalogs {
    pub apps: Apps,
    /// Why some categories could not be loaded, `None` when all did.
    pub problem: Option<LoadProblem>,
}

/// Every catalog of the pages. The default is empty, for tests that show
/// no catalog page.
#[derive(Debug, Default)]
pub struct Catalogs {
    pub components: Toggles,
    pub gaming: Toggles,
    pub settings: Toggles,
    pub setup: Toggles,
    pub apps: AppCatalogs,
    /// What Maintenance resets from `/etc/skel`. `None` hides the reset.
    pub skel: Option<SkelCatalog>,
    /// The app's own colour schemes. Empty when the file is unusable, so
    /// the window keeps the desktop's colours.
    pub colour_schemes: ColourSchemeCatalog,
    /// The packages no removal takes. Empty when the file is unusable,
    /// which the log says.
    pub protected: ProtectedCatalog,
}

impl Default for Toggles {
    fn default() -> Self {
        Self {
            catalog: Ok(ToggleCatalog::default()),
            icon_dir: PathBuf::new(),
        }
    }
}

impl Catalogs {
    pub fn load(dir: &Path) -> Self {
        Self {
            components: Toggles::load(dir, CatalogId::Components.file()),
            gaming: Toggles::load(dir, CatalogId::Gaming.file()),
            settings: Toggles::load(dir, CatalogId::Settings.file()),
            setup: Toggles::load(dir, CatalogId::Setup.file()),
            apps: AppCatalogs::load(dir),
            skel: SkelCatalog::load(dir)
                .inspect_err(|error| tracing::warn!("the settings reset is unavailable: {error}"))
                .ok(),
            colour_schemes: ColourSchemeCatalog::load(dir)
                .inspect_err(|error| tracing::warn!("the colour schemes are unavailable: {error}"))
                .unwrap_or_default(),
            protected: ProtectedCatalog::load(dir)
                .inspect_err(|error| {
                    tracing::error!("no package is protected from removal: {error}");
                })
                .unwrap_or_default(),
        }
    }

    pub const fn get(&self, id: CatalogId) -> &Toggles {
        match id {
            CatalogId::Components => &self.components,
            CatalogId::Gaming => &self.gaming,
            CatalogId::Settings => &self.settings,
            CatalogId::Setup => &self.setup,
        }
    }

    /// The config file edits the catalogs offer, for the snapshot to read.
    pub fn config_edits(&self) -> Vec<UserConfigEdit> {
        self.loaded()
            .flat_map(|catalog| catalog.config_edits().cloned())
            .collect()
    }

    /// Every name an icon theme may have a picture under for an entry of
    /// the catalogs, each once.
    pub fn app_icon_names(&self) -> Vec<String> {
        let names: BTreeSet<String> = self
            .loaded()
            .flat_map(ToggleCatalog::entries)
            .flat_map(icon_names)
            .collect();

        names.into_iter().collect()
    }

    fn loaded(&self) -> impl Iterator<Item = &ToggleCatalog> {
        [&self.components, &self.gaming, &self.settings, &self.setup]
            .into_iter()
            .filter_map(|toggles| toggles.catalog.as_ref().ok())
    }
}

impl CatalogId {
    pub const fn file(self) -> &'static str {
        match self {
            Self::Components => COMPONENTS,
            Self::Gaming => GAMING,
            Self::Settings => SETTINGS,
            Self::Setup => SETUP,
        }
    }

    /// Stable code, part of the catalog texts' ids.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Gaming => "gaming",
            Self::Settings => "settings",
            Self::Setup => "setup",
        }
    }
}

impl Toggles {
    pub fn load(dir: &Path, file: &str) -> Self {
        Self {
            catalog: ToggleCatalog::load(dir, file)
                .inspect_err(|error| tracing::warn!("{file} is unavailable: {error}"))
                .map_err(|error| LoadProblem::from(&error)),
            icon_dir: ToggleCatalog::icon_dir(dir, file),
        }
    }
}

impl AppCatalogs {
    pub fn load(dir: &Path) -> Self {
        let mut problems = Vec::new();
        let catalogs = APPS
            .iter()
            .filter_map(|file| {
                ToggleCatalog::load(dir, file)
                    .inspect_err(|error| tracing::warn!("{file} is unavailable: {error}"))
                    .map_err(|error| problems.push(LoadProblem::from(&error)))
                    .ok()
            })
            .collect();

        Self {
            apps: Apps::new(catalogs),
            problem: LoadProblem::joined(problems),
        }
    }
}

/// The folder with the catalogs: the variable when set, the source tree in
/// development builds or when the installed copy is missing, the installed
/// copy otherwise.
pub fn catalog_dir(lookup: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    let installed = Path::new(toolbox_catalog::INSTALLED_DIR);
    let dir = lookup(DIR_VARIABLE).map_or_else(
        || {
            if cfg!(debug_assertions) || !installed.is_dir() {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
            } else {
                installed.to_path_buf()
            }
        },
        PathBuf::from,
    );

    dir.canonicalize().unwrap_or(dir)
}

/// The names an icon theme may have `entry`'s picture under: the file name
/// of its bundled picture without the ending, then the package it checks.
pub fn icon_names(entry: &ToggleEntry) -> Vec<String> {
    let stem = entry
        .icon()
        .and_then(|file| Path::new(file).file_stem())
        .and_then(|stem| stem.to_str())
        .map(str::to_owned);

    let package = match entry.check() {
        Check::Package(name) => Some(name.as_str().to_owned()),
        Check::Service(_) | Check::UserService(_) | Check::Group(_) | Check::UserConfig(_) => None,
    };

    let mut names: Vec<String> = stem.into_iter().chain(package).collect();
    names.dedup();
    names
}
