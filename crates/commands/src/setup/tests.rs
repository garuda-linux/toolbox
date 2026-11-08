use toolbox_cli::Exit;
use toolbox_core::Snapshot;
use toolbox_services::catalogs::{Catalogs, catalog_dir};

use super::apply;

fn catalogs() -> Catalogs {
    Catalogs::load(&catalog_dir(|key| std::env::var_os(key)))
}

#[test]
fn an_id_the_assistant_lacks_is_not_available() {
    let catalogs = catalogs();
    let catalog = catalogs.setup.catalog.as_ref().unwrap();

    let refused = apply(
        catalog,
        &["no-such-choice".to_owned()],
        &Snapshot::default(),
    )
    .unwrap_err();

    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn several_choices_land_in_one_plan() {
    let catalogs = catalogs();

    let catalog = catalogs.setup.catalog.as_ref().unwrap();
    let ids: Vec<String> = catalog
        .sections()
        .iter()
        .flat_map(toolbox_catalog::ToggleSection::entries)
        .take(2)
        .map(|entry| entry.id().to_owned())
        .collect();

    let one = apply(catalog, &ids[..1], &Snapshot::default()).unwrap();
    let both = apply(catalog, &ids, &Snapshot::default()).unwrap();
    assert!(format!("{both:?}").len() > format!("{one:?}").len());
}
