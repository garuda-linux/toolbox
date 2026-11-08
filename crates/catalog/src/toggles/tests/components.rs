use toolbox_plan::{PackageName, UnitName};

use crate::{COMPONENTS, Check, Requires, ToggleCatalog, ToggleSection, source_dir};

#[test]
fn the_shipped_components_catalog_has_its_eight_sections() {
    let catalog = ToggleCatalog::load(&source_dir(), COMPONENTS).unwrap();
    let sections: Vec<&str> = catalog.sections().iter().map(ToggleSection::id).collect();

    assert_eq!(
        sections,
        [
            "audio",
            "virtualization",
            "containers",
            "network",
            "bluetooth",
            "printing",
            "firewall",
            "misc"
        ]
    );
    assert_eq!(catalog.entries().count(), 49);

    let compose = catalog.entry("docker-compose").unwrap();

    assert_eq!(compose.title(), "Docker Compose");
    assert_eq!(
        compose.requires(),
        &Requires::Any(vec!["docker".to_owned(), "podman".to_owned()])
    );
    assert_eq!(
        catalog.entry("enabled-networkmanager").unwrap().check(),
        &Check::Service(UnitName::new("NetworkManager.service").unwrap())
    );
    assert_eq!(
        catalog.entry("autoconnect-enabled").unwrap().requires(),
        &Requires::All(vec!["autoconnect".to_owned()])
    );
    assert_eq!(
        catalog.entry("gns").unwrap().check(),
        &Check::Package(PackageName::new("garuda-nix-subsystem").unwrap())
    );
}
