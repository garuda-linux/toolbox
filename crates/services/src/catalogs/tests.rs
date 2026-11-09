use std::ffi::OsString;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toolbox_catalog::COMPONENTS;

use super::{AppCatalogs, CatalogId, Toggles, catalog_dir_in};

fn source_tree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}

fn variable(key: &str) -> Option<OsString> {
    (key == "GARUDA_TOOLBOX_CATALOG_DIR").then(|| OsString::from("/srv/catalog"))
}

#[test]
fn the_variable_names_the_folder() {
    let dir = catalog_dir_in(variable, true);

    assert_eq!(dir, PathBuf::from("/srv/catalog"));
}

#[test]
fn a_development_build_reads_the_source_tree() {
    let dir = catalog_dir_in(|_| None, true);

    assert_eq!(dir, source_tree().canonicalize().unwrap());
}

#[test]
fn a_release_ignores_the_variable() {
    let dir = catalog_dir_in(variable, false);

    assert_eq!(dir, PathBuf::from(toolbox_catalog::INSTALLED_DIR));
}

#[test]
fn a_release_never_reads_the_source_tree() {
    let dir = catalog_dir_in(|_| None, false);

    assert_eq!(dir, PathBuf::from(toolbox_catalog::INSTALLED_DIR));
}

#[test]
fn the_components_catalog_loads() {
    let toggles = Toggles::load(&source_tree(), COMPONENTS);

    let catalog = toggles.catalog.unwrap();
    assert!(catalog.entry("pipewire-support").is_some());
    assert_eq!(toggles.icon_dir, source_tree().join("icons/components"));
}

#[test]
fn a_missing_catalog_is_a_problem_not_a_stop() {
    let empty = TempDir::new().unwrap();

    let problem = Toggles::load(empty.path(), COMPONENTS).catalog.unwrap_err();

    assert_eq!(problem.code, "unreadable");
}

#[test]
fn every_catalog_names_a_file_that_loads() {
    for id in [
        CatalogId::Components,
        CatalogId::Gaming,
        CatalogId::Settings,
        CatalogId::Setup,
    ] {
        let toggles = Toggles::load(&source_tree(), id.file());

        assert!(toggles.catalog.is_ok(), "{}", id.as_str());
    }
}

#[test]
fn the_set_hands_out_each_catalog_and_its_edits() {
    let catalogs = super::Catalogs::load(&source_tree());

    let components = catalogs
        .get(CatalogId::Components)
        .catalog
        .as_ref()
        .unwrap();

    assert!(components.entry("pipewire-support").is_some());

    // System Tweaks edits the user's config files, the others none.
    assert_ne!(catalogs.config_edits(), []);
    assert!(
        catalogs
            .get(CatalogId::Settings)
            .catalog
            .as_ref()
            .unwrap()
            .entry("profile-sync-daemon")
            .is_some()
    );
}

#[test]
fn the_gaming_catalog_loads_with_its_pictures() {
    let catalogs = super::Catalogs::load(&source_tree());

    let gaming = catalogs.get(CatalogId::Gaming);
    let catalog = gaming.catalog.as_ref().unwrap();
    let bottles = catalog.entry("bottles").unwrap();
    assert!(gaming.icon_dir.join(bottles.icon().unwrap()).is_file());
}

#[test]
fn an_entry_is_looked_up_by_its_picture_then_its_package() {
    let catalogs = super::Catalogs::load(&source_tree());
    let gaming = catalogs.get(CatalogId::Gaming).catalog.as_ref().unwrap();

    let heroic = gaming.entry("heroic-games-launcher-bin").unwrap();
    assert_eq!(
        super::icon_names(heroic),
        ["heroic-icon", "heroic-games-launcher-bin"]
    );

    let names = catalogs.app_icon_names();
    assert!(names.contains(&"steam".to_owned()));
    assert!(
        names.is_sorted() && names.windows(2).all(|pair| pair.first() != pair.last()),
        "sorted, each once"
    );
}

#[test]
fn the_app_categories_load_together() {
    let apps = AppCatalogs::load(&source_tree());

    assert!(apps.problem.is_none());
    assert_eq!(apps.apps.all().next().unwrap().category.id(), "documents");
    assert!(apps.apps.all().any(|hit| hit.entry.id() == "firefox"));
}

#[test]
fn a_broken_category_leaves_the_others() {
    let dir = TempDir::new().unwrap();
    let apps = dir.path().join("apps");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::copy(
        source_tree().join("apps/science.toml"),
        apps.join("science.toml"),
    )
    .unwrap();

    let loaded = AppCatalogs::load(dir.path());

    assert_eq!(loaded.problem.unwrap().code, "unreadable");
    assert!(loaded.apps.all().all(|hit| hit.category.id() == "science"));
    assert!(loaded.apps.all().next().is_some());
}

#[test]
fn the_colour_schemes_load_and_a_missing_file_leaves_only_the_desktops() {
    let shipped = super::Catalogs::load(&source_tree());
    assert_ne!(shipped.colour_schemes.schemes(), []);

    let empty = TempDir::new().unwrap();
    let none = super::Catalogs::load(empty.path());
    assert_eq!(none.colour_schemes.schemes(), []);
}
