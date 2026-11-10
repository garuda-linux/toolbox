//! Schemas, loader and validation for the TOML catalogs in
//! `/usr/share/garuda-toolbox/catalog` (`data/catalog` in the source tree).
//! Each catalog file has its own module with its schema, so a new catalog
//! adds a module and leaves the others alone.
//!
//! This crate only reads catalog files. It never probes the system.

mod apps;
mod colour_schemes;
mod config_files;
mod error;
mod skel;
mod source;
mod toggles;

pub use apps::APPS;
pub use colour_schemes::{
    COLOUR_SCHEMES, ColourScheme, ColourSchemeCatalog, SchemeColours, SchemeMode, SchemeTiles,
};
pub use config_files::{CONFIG_FILES, ConfigEntry, ConfigFileCatalog, ConfigSection};
pub use error::{CatalogError, EntryProblem, LoadProblem, Problem};
pub use skel::{SKEL_ROOT, SkelCatalog, SkelEntry};
pub use toggles::{
    COMPONENTS, Check, GAMING, Requires, SETTINGS, SETUP, ToggleCatalog, ToggleEntry,
    ToggleSection, is_web_url,
};

pub const INSTALLED_DIR: &str = "/usr/share/garuda-toolbox/catalog";

/// The catalogs as the repository ships them, which the tests load.
#[cfg(test)]
fn source_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}
