//! The footer's keys, a step that went away and the success page.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::ChangeSet;

use super::{page, shows, strawberry};
use crate::pages::Plans;
use crate::pages::setup::Stage;

#[test]
fn the_keys_are_the_tooltips_of_back_and_next() {
    let mut harness = page(
        strawberry(),
        Plans::default(),
        Stage::Step("audio".to_owned()),
    );

    assert!(!shows(&harness, "Alt+Right"), "a hint under the buttons");

    harness.get_by_label("Next").hover();
    harness.run_steps(30);
    assert!(harness.query_by_label("Alt+Right").is_some());

    harness.get_by_label("Back").hover();
    harness.run_steps(30);
    assert!(harness.query_by_label("Alt+Left").is_some());
}

#[test]
fn a_step_that_no_longer_lists_starts_over_at_the_welcome() {
    let harness = page(
        ChangeSet::default(),
        Plans::default(),
        Stage::Step("gone".to_owned()),
    );

    assert!(shows(&harness, "Welcome to the Setup Assistant"));
}

#[test]
fn the_success_page_names_what_the_apply_installed() {
    let mut harness = page(strawberry(), Plans::default(), Stage::Summary);

    let fixture = &mut harness.state_mut().fixture;
    fixture.changes = ChangeSet::default();
    fixture.revision += 1;
    fixture.plans = Plans {
        finished: 1,
        last_succeeded: true,
        ..Plans::default()
    };
    harness.run();

    assert!(shows(&harness, "Success!"));
    assert!(shows(&harness, "Apps installed: 1"));
    assert!(shows(&harness, "Strawberry Player"));
}

#[test]
fn the_success_page_links_to_the_forum_and_the_wikis() {
    let harness = page(ChangeSet::default(), Plans::default(), Stage::Success);

    for name in ["Garuda forum", "Garuda Linux wiki", "Arch Linux wiki"] {
        assert!(
            harness.query_by_role_and_label(Role::Link, name).is_some(),
            "{name} is no link"
        );
    }
}

#[test]
fn finish_without_picks_after_an_apply_recommends_no_restart() {
    let plans = Plans {
        finished: 1,
        last_succeeded: true,
        ..Plans::default()
    };

    let mut harness = page(strawberry(), plans, Stage::Summary);
    harness.state_mut().fixture.changes = ChangeSet::default();
    harness.run();
    assert!(shows(&harness, "A restart is recommended"));

    harness.get_by_label("Finish setup").click();
    harness.run();
    harness.state_mut().page.stage = Stage::Summary;
    harness.run();
    harness.get_by_label("Finish").click();
    harness.run();

    assert!(shows(&harness, "Success!"));
    assert!(!shows(&harness, "A restart is recommended"));
    assert!(!shows(&harness, "Apps installed"));
}
