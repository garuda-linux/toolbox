use toolbox_core::features::toggles::Refusal;
use toolbox_core::{ChangeSet, Snapshot};

use super::TogglePages;
use crate::catalogs::CatalogId;
use crate::command::AppCommand;
use crate::refused::Refused;

#[test]
fn each_page_keeps_its_own_refusal_until_dismissed() {
    let mut pages = TogglePages::default();
    let unmet = Refused::new(Refusal::Unmet, "printing".to_owned());
    let unknown = Refused::new(Refusal::Unknown, "nope".to_owned());
    pages.set_refusal(CatalogId::Gaming, Some(unmet));
    pages.set_refusal(CatalogId::Setup, Some(unknown.clone()));

    let handled = pages.handle(
        &AppCommand::DismissRefusal(CatalogId::Gaming),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(pages.refusal(CatalogId::Gaming), None);
    assert_eq!(pages.refusal(CatalogId::Setup), Some(&unknown));
}

/// Without a loaded catalog the click is the page's all the same, so App
/// does not run it a second time.
#[test]
fn a_click_on_an_unloaded_catalog_is_taken_and_changes_nothing() {
    let mut pages = TogglePages::default();

    let handled = pages.handle(
        &AppCommand::ToggleEntry {
            catalog: CatalogId::Gaming,
            entry: "steam".to_owned(),
            wish: toolbox_core::Toggle::On,
        },
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(handled.edit.is_none());
}
