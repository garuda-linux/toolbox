use std::sync::Arc;

use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::packages::list::{Catalog, Filter};
use toolbox_core::features::packages::{InstallReason, LocalNode, Origin, PackageIndex};

use super::{entry, fixture_with, listed, node};
use crate::pages::packages::PackagesPage;

fn catalog() -> Arc<Catalog> {
    let extra = || Origin::Repository("extra".to_owned());
    Arc::new(Catalog::new(PackageIndex {
        entries: vec![
            entry("gimp", Some(InstallReason::Explicit), extra()),
            entry("leftover", Some(InstallReason::Dependency), extra()),
        ],
        local: vec![
            LocalNode {
                optional: vec!["leftover".to_owned()],
                ..node("gimp", InstallReason::Explicit, &[])
            },
            node("leftover", InstallReason::Dependency, &[]),
        ],
    }))
}

#[test]
fn the_optional_chip_lists_what_is_only_optional_and_names_who_lists_it() {
    let page = PackagesPage {
        filter: Filter::Optional,
        ..PackagesPage::default()
    };

    let mut harness = fixture_with(catalog(), None).harness(page);
    harness.run();

    assert!(listed(&harness, "leftover"));
    assert!(!listed(&harness, "gimp"));
    assert!(harness.query_by_label("Optional for gimp").is_some());
    assert!(harness.query_by_label("Optional 1").is_some(), "the chip");

    harness.state_mut().page.filter = Filter::Orphans;
    harness.run();
    assert!(!listed(&harness, "leftover"));
}
