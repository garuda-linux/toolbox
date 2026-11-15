use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::ComponentsPage;
use crate::catalogs::Catalogs;
use crate::testing::{PLAN_RUNNING, PageFixture};

fn shipped() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    PageFixture {
        catalogs: Catalogs::load(&dir),
        ..PageFixture::default()
    }
}

/// The rows on screen are packages, so their Install buttons rest.
#[test]
fn while_a_plan_runs_the_entries_wait_and_a_banner_says_why() {
    for busy in [true, false] {
        let mut harness = PageFixture { busy, ..shipped() }.harness(ComponentsPage::default());
        harness.run();

        let disabled: Vec<bool> = harness
            .query_all_by_role(Role::Button)
            .filter(|button| {
                button
                    .accesskit_node()
                    .label()
                    .is_some_and(|label| label.starts_with("Install "))
            })
            .map(|button| button.accesskit_node().is_disabled())
            .collect();

        assert!(!disabled.is_empty(), "no Install button on screen");
        if busy {
            assert!(
                disabled.iter().all(|resting| *resting),
                "busy: {disabled:?}"
            );
        } else {
            assert!(
                disabled.iter().any(|resting| !resting),
                "idle: {disabled:?}"
            );
        }

        assert_eq!(
            harness.query_by_label(PLAN_RUNNING).is_some(),
            busy,
            "the banner"
        );
    }
}
