use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::configs::session::Session;
use toolbox_core::features::configs::viewing::Viewing;

use super::{listed, page, reviewing, reviewing_of, sent, shows, viewing};
use crate::command::{Command, ConfigsCommand};
use crate::pages::{ConfigsRefusal, Refusals};

/// How far a dialog that fills the window keeps its buttons from the
/// window's bottom: the window margin and the card padding, with room.
const FILLED_BOTTOM: f32 = 48.0;

#[test]
fn the_review_shows_the_changes_and_saves_on_request() {
    let configs = listed().with(reviewing(), Viewing::default());
    let mut harness = page(configs, Refusals::default());

    assert!(shows(&harness, "Save the changes to /etc/default/grub?"));
    assert!(shows(&harness, "+GRUB_TIMEOUT=2"));
    assert!(shows(&harness, "-GRUB_TIMEOUT=5"));

    harness.get_by_label("Save").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(ConfigsCommand::Apply)]);
}

#[test]
fn a_closed_review_keeps_the_changes_for_later() {
    let configs = listed().with(reviewing(), Viewing::default());
    let mut harness = page(configs, Refusals::default());

    harness.get_by_label("Close and keep the edits").click();
    harness.run();

    assert!(shows(
        &harness,
        "Your changes to /etc/default/grub wait for your review."
    ));

    harness.get_by_label("Review changes").click();
    harness.run();

    assert!(shows(&harness, "Save the changes to /etc/default/grub?"));
    assert_eq!(sent(&harness), []);
}

#[test]
fn a_guarded_file_is_shown_and_closed() {
    let configs = listed().with(Session::default(), viewing());
    let mut harness = page(configs, Refusals::default());

    assert!(shows(&harness, "The text of /etc/sudoers"));
    assert!(shows(&harness, "Copy all"));

    harness.query_all_by_label("Close").last().unwrap().click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(ConfigsCommand::CloseView)]);
}

#[test]
fn a_refused_click_says_why_until_dismissed() {
    let refusals = Refusals {
        configs: Some(ConfigsRefusal {
            code: "busy",
            subject: "grub".to_owned(),
        }),
        ..Refusals::default()
    };

    let mut harness = page(listed(), refusals);

    assert!(shows(
        &harness,
        "Another task runs in the Console. Try again when it has finished."
    ));

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(ConfigsCommand::DismissRefusal)]
    );
}

#[test]
fn the_review_names_the_changed_lines_in_words() {
    let configs = listed().with(reviewing(), Viewing::default());
    let harness = page(configs, Refusals::default());

    assert!(
        harness.query_all_by_label_contains("@@").next().is_none(),
        "no raw hunk header"
    );
    assert!(shows(&harness, "Line 1"));
}

#[test]
fn discard_asks_first_and_keeps_the_edits_on_no() {
    let configs = listed().with(reviewing(), Viewing::default());
    let mut harness = page(configs, Refusals::default());

    harness.get_by_label("Discard…").click();
    harness.run();
    assert!(shows(&harness, "Discard your edits?"));
    assert_eq!(sent(&harness), []);

    harness.get_by_label("Keep editing").click();
    harness.run();
    assert_eq!(sent(&harness), []);

    harness.get_by_label("Discard…").click();
    harness.run();
    harness.get_by_label("Discard").click();
    harness.run();
    assert_eq!(sent(&harness), [Command::from(ConfigsCommand::Discard)]);
}

#[test]
fn the_review_of_a_file_of_the_user_asks_for_no_password() {
    let home = toolbox_plan::HomePath::new(".bashrc").unwrap();
    let configs = listed().with(reviewing_of(home.into()), Viewing::default());
    let harness = page(configs, Refusals::default());

    let summary = harness
        .get_by_label_contains("Lines added")
        .accesskit_node()
        .value()
        .unwrap_or_default();

    assert!(!summary.contains("administrator"), "{summary}");
    assert!(
        summary.contains("~/.local/state/garuda-toolbox/backups/.bashrc"),
        "{summary}"
    );
}

fn button_room_below(harness: &super::Shown, label: &str) -> f32 {
    let button = harness.query_all_by_label(label).last().unwrap().rect();
    harness.ctx.content_rect().bottom() - button.bottom()
}

#[test]
fn a_short_review_still_fills_the_window() {
    let harness = page(
        listed().with(reviewing(), Viewing::default()),
        Refusals::default(),
    );

    let below = button_room_below(&harness, "Save");

    assert!(below < FILLED_BOTTOM, "Save sits {below} above the bottom");
}

#[test]
fn a_shown_file_fills_the_window() {
    let harness = page(
        listed().with(Session::default(), viewing()),
        Refusals::default(),
    );

    let below = button_room_below(&harness, "Close");

    assert!(below < FILLED_BOTTOM, "Close sits {below} above the bottom");
}
