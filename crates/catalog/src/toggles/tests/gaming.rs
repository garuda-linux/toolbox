use toolbox_plan::PackageName;

use crate::{Check, GAMING, ToggleCatalog, ToggleSection, source_dir};

#[test]
fn the_shipped_gaming_catalog_has_its_seven_sections() {
    let catalog = ToggleCatalog::load(&source_dir(), GAMING).unwrap();
    let sections: Vec<&str> = catalog.sections().iter().map(ToggleSection::id).collect();

    assert_eq!(
        sections,
        [
            "launchers",
            "compatibility",
            "tools",
            "misc",
            "controllers",
            "games",
            "emulators"
        ]
    );
    assert_eq!(catalog.entries().count(), 156);

    let gamemode = catalog.entry("gamemode").unwrap();

    assert_eq!(
        gamemode.also(),
        [PackageName::new("lib32-gamemode").unwrap()]
    );
    assert_eq!(
        catalog.entry("luxtorpeda-git").unwrap().check(),
        &Check::Package(PackageName::new("luxtorpeda-git").unwrap())
    );
    assert_eq!(catalog.entry("steam").unwrap().icon(), Some("steam.png"));
    assert_eq!(
        catalog.entry("0ad").unwrap().url(),
        Some("https://play0ad.com/")
    );

    let linked = catalog
        .entries()
        .filter(|entry| entry.url().is_some())
        .count();

    assert_eq!(linked, 71);
}

#[test]
fn every_gaming_icon_is_in_its_folder() {
    let catalog = ToggleCatalog::load(&source_dir(), GAMING).unwrap();
    let folder = ToggleCatalog::icon_dir(&source_dir(), GAMING);
    let missing: Vec<&str> = catalog
        .entries()
        .filter_map(|entry| entry.icon())
        .filter(|icon| !folder.join(icon).is_file())
        .collect();

    assert_eq!(missing, Vec::<&str>::new());
}
