use toolbox_cli::Exit;
use toolbox_core::Snapshot;
use toolbox_services::catalogs::Catalogs;

use super::apply;

#[test]
fn an_id_the_assistant_lacks_is_not_available() {
    let catalogs = Catalogs::from_env();
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
    let catalogs = Catalogs::from_env();

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
    let text = |plan: &toolbox_plan::Plan| -> String {
        plan.steps()
            .iter()
            .flat_map(|step| step.action.command_lines())
            .collect::<Vec<_>>()
            .join("\n")
    };

    assert_eq!(both.steps().len(), 1);
    assert!(text(&one).contains("strawberry"), "{}", text(&one));
    assert!(!text(&one).contains("lollypop"), "{}", text(&one));
    assert!(text(&both).contains("strawberry"), "{}", text(&both));
    assert!(text(&both).contains("lollypop"), "{}", text(&both));
}
