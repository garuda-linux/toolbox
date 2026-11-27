//! The AUR's copy as the search reads it, for the catalog tests.

use toolbox_state::aur::{AurIndex, AurPackage};

use super::super::AurCatalog;
use crate::test_support::package;

pub(super) fn aur(name: &str, description: &str, popularity: f64) -> AurPackage {
    AurPackage {
        name: package(name),
        version: toolbox_text::plain("1.0-1"),
        description: toolbox_text::plain(description),
        url: None,
        votes: 1,
        popularity,
        out_of_date: None,
        maintainer: None,
        first_submitted: 0,
        last_modified: 0,
        depends: Vec::new(),
        make_depends: Vec::new(),
        opt_depends: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        licenses: Vec::new(),
        keywords: Vec::new(),
    }
}

pub(super) fn catalog() -> AurCatalog {
    let mut tagged = aur("vscodium-bin", "Free build of the editor", 9.0);
    tagged.keywords = vec![toolbox_text::plain("IDE")];

    AurCatalog::new(AurIndex {
        packages: vec![
            aur("yay-bin", "Yet another yogurt, prebuilt", 4.0),
            aur("paru", "Feature packed AUR helper", 20.0),
            aur("paru-bin", "Feature packed AUR helper, prebuilt", 30.0),
            aur("aurutils", "Helper tools for paru users", 50.0),
            tagged,
        ],
        downloaded: 1_700_000_000,
        stale: false,
    })
}

pub(super) fn names(catalog: &AurCatalog, places: &[usize]) -> Vec<String> {
    places
        .iter()
        .map(|&place| catalog.entries()[place].name.to_string())
        .collect()
}

#[test]
fn without_a_search_the_most_popular_package_comes_first() {
    let catalog = catalog();

    assert_eq!(
        names(&catalog, &catalog.view("")),
        ["aurutils", "paru-bin", "paru", "vscodium-bin", "yay-bin"]
    );
}

#[test]
fn the_exact_name_comes_first_then_names_that_start_with_the_search() {
    let catalog = catalog();

    assert_eq!(
        names(&catalog, &catalog.view("PARU")),
        ["paru", "paru-bin", "aurutils"]
    );
}

#[test]
fn the_search_finds_keywords_and_descriptions() {
    let catalog = catalog();

    assert_eq!(names(&catalog, &catalog.view("ide")), ["vscodium-bin"]);
    assert_eq!(names(&catalog, &catalog.view("yogurt")), ["yay-bin"]);
    assert_eq!(catalog.view("nothing like it"), Vec::<usize>::new());
}

#[test]
fn a_package_is_found_by_name() {
    let catalog = catalog();

    assert_eq!(
        catalog
            .entry(&package("paru"))
            .map(|entry| entry.popularity),
        Some(20.0_f64)
    );
    assert_eq!(catalog.entry(&package("pacman")), None);
    assert_eq!(catalog.downloaded(), 1_700_000_000);
}
