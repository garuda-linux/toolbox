use std::path::{Path, PathBuf};

use toolbox_catalog::{COMPONENTS, GAMING, SETTINGS, SETUP, ToggleCatalog};
use toolbox_gui_kit::icons;

fn source_tree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}

#[test]
fn every_section_icon_is_bundled() {
    for file in [COMPONENTS, GAMING, SETTINGS, SETUP] {
        let catalog = ToggleCatalog::load(&source_tree(), file).unwrap();

        for section in catalog.sections() {
            assert!(
                icons::named(section.icon()).is_some(),
                "{file}: {} is not bundled",
                section.icon()
            );
        }
    }
}
