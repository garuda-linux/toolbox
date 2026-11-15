use std::path::{Path, PathBuf};

use toolbox_catalog::{COMPONENTS, GAMING, SETTINGS, SETUP, ToggleCatalog};
use toolbox_gui_kit::icons;

fn source_tree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}

#[test]
fn every_section_icon_and_entry_glyph_is_bundled() {
    for file in [COMPONENTS, GAMING, SETTINGS, SETUP] {
        let catalog = ToggleCatalog::load(&source_tree(), file).unwrap();

        for section in catalog.sections() {
            assert!(
                icons::named(section.icon()).is_some(),
                "{file}: {} is not bundled",
                section.icon()
            );
        }

        for (id, glyph) in catalog
            .entries()
            .filter_map(|entry| entry.glyph().map(|glyph| (entry.id(), glyph)))
        {
            assert!(
                icons::named(glyph).is_some(),
                "{file}: the glyph {glyph} of {id} is not bundled"
            );
        }
    }
}

#[test]
fn every_component_shows_a_glyph() {
    let catalog = ToggleCatalog::load(&source_tree(), COMPONENTS).unwrap();
    let bare: Vec<&str> = catalog
        .entries()
        .filter(|entry| entry.glyph().is_none())
        .map(toolbox_catalog::ToggleEntry::id)
        .collect();

    assert_eq!(bare, [""; 0], "components without a glyph");
}
