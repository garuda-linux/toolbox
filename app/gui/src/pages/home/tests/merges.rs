use std::path::PathBuf;
use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::SnapshotResults;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::merges::choice;
use toolbox_gui_kit::Loadable;
use toolbox_plan::{ConfigText, MergeCopy, Resolution};
use toolbox_system::MergePair;
use toolbox_system::testing::packages::PACMAN_CONF;

use super::{Shown, page, status};
use crate::command::{Command, HomeCommand};
use crate::testing::PageFixture;

const COPY: &str = "/etc/pacman.conf.pacnew";

/// The Lima VM's `/etc/pacman.conf` with `Color` commented out, so its
/// `.pacnew`, the capture itself, switches it on again.
fn current() -> String {
    PACMAN_CONF.replacen("\nColor\n", "\n#Color\n", 1)
}

fn pair(current: Option<String>) -> MergePair {
    MergePair {
        copy: MergeCopy::new(COPY).unwrap(),
        copied: Ok(ConfigText::new(PACMAN_CONF).unwrap()),
        current: Ok(current.map(|text| ConfigText::new(text).unwrap())),
    }
}

fn waiting(pairs: Vec<MergePair>) -> PageFixture {
    let mut status = status();
    status.merges = Ok(vec![PathBuf::from(COPY)]);

    let mut fixture = PageFixture {
        status: Some(Loadable::Loaded(Arc::new(status))),
        ..PageFixture::default()
    };

    fixture.services.home.merges_mut().set_pairs(pairs);
    fixture
}

fn opened(pairs: Vec<MergePair>) -> Shown {
    let mut harness = page(waiting(pairs));

    harness
        .get_by_role_and_label(Role::Button, "Review…")
        .click();
    harness.run();
    harness
}

fn merge(resolution: Resolution, current: Option<&str>) -> Command {
    Command::from(HomeCommand::Merge {
        copy: MergeCopy::new(COPY).unwrap(),
        choice: Some(choice(resolution, current, PACMAN_CONF)),
    })
}

#[test]
fn review_reads_the_files_and_opens_the_dialog() {
    let harness = opened(vec![pair(Some(current()))]);

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(HomeCommand::ReadMerges)]
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Configuration files to merge")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Tab, "/etc/pacman.conf")
            .is_some()
    );
}

#[test]
fn the_whole_new_file_is_queued_with_the_marks_of_both() {
    let mut harness = opened(vec![pair(Some(current()))]);
    harness.state().fixture.sent();

    harness
        .get_by_role_and_label(Role::Button, "Use new file")
        .click();
    harness.run();

    let current = current();
    assert_eq!(
        harness.state().fixture.sent(),
        [merge(Resolution::TakeCopy, Some(&current))]
    );
}

#[test]
fn a_merge_needs_a_pick_that_takes_a_change() {
    let mut harness = opened(vec![pair(Some(current()))]);

    assert!(
        harness
            .get_by_role_and_label(Role::Button, "Queue the merge")
            .accesskit_node()
            .is_disabled()
    );

    harness
        .get_by_role_and_label(Role::RadioButton, "Take new")
        .click();
    harness.run();
    harness.state().fixture.sent();

    harness
        .get_by_role_and_label(Role::Button, "Queue the merge")
        .click();
    harness.run();

    let current = current();
    assert_eq!(
        harness.state().fixture.sent(),
        [merge(Resolution::TakeCopy, Some(&current))],
        "taking the one change takes the whole copy"
    );
}

#[test]
fn a_copy_without_its_file_offers_to_take_its_place() {
    let mut harness = opened(vec![pair(None)]);
    harness.state().fixture.sent();

    assert!(harness.query_by_label_contains("is gone").is_some());

    harness
        .get_by_role_and_label(Role::Button, "Use new file")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [merge(Resolution::TakeCopy, None)]
    );
}

#[test]
fn a_copy_equal_to_the_file_can_only_go() {
    let harness = opened(vec![pair(Some(PACMAN_CONF.to_owned()))]);

    assert!(harness.query_by_label_contains("the same as").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Use new file")
            .is_none()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Remove the copy")
            .is_some()
    );
}

#[test]
fn the_dialog_in_dark_and_light() {
    let mut harness = opened(vec![pair(Some(current()))]);

    harness
        .get_by_role_and_label(Role::RadioButton, "Take new")
        .click();
    harness.run();

    let mut results = SnapshotResults::new();
    results.add(harness.try_snapshot("home/merges_dark"));

    harness.ctx.set_theme(Theme::Light);
    harness.run();
    results.add(harness.try_snapshot("home/merges_light"));
}

#[test]
fn the_card_offers_review_and_meld() {
    let mut harness = page(waiting(vec![pair(Some(current()))]));
    harness
        .get_by_role_and_label(Role::Button, "Review…")
        .scroll_to_me();
    harness.run();

    harness.snapshot("home/merges_card_dark");
    assert!(
        harness
            .query_by_label("More actions for Configuration files to merge")
            .is_some()
    );
}

#[test]
fn closing_the_dialog_stops_its_read() {
    let mut harness = opened(vec![pair(Some(current()))]);
    harness.state().fixture.sent();

    harness.query_all_by_label("Close").last().unwrap().click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(HomeCommand::CloseMerges)]
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Configuration files to merge")
            .is_none()
    );
}
