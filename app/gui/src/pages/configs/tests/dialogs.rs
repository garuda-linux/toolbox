use std::fmt::Write as _;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::configs::Refusal;
use toolbox_core::features::configs::session::{Changes, Problem, ProblemCode, Session};
use toolbox_core::features::configs::viewing::Viewing;
use toolbox_gui_kit::testing::{fill_under, painted_text_within};

use super::{listed, page, reviewing, reviewing_of, sent, shows, viewing};
use crate::command::{AppCommand, Command, ConfigsCommand};
use crate::page::PageId;
use crate::refused::Refused;

/// How far a dialog that fills the window keeps its buttons from the
/// window's bottom: the window margin and the card padding, with room.
const FILLED_BOTTOM: f32 = 48.0;

/// A dialog as tall as a few lines of text keeps far more room below
/// its buttons, centred in the 900 point high window.
const GROWN_BOTTOM: f32 = 150.0;

#[test]
fn the_review_shows_the_changes_and_saves_on_request() {
    let configs = listed().with(reviewing(), Viewing::default());
    let mut harness = page(configs);

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
    let mut harness = page(configs);

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
    let mut harness = page(configs);

    assert!(shows(&harness, "The text of /etc/sudoers"));
    assert!(shows(&harness, "Copy all"));

    harness.query_all_by_label("Close").last().unwrap().click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(ConfigsCommand::CloseView)]);
}

#[test]
fn a_refused_click_says_why_until_dismissed() {
    let mut configs = listed();
    configs.set_refusal(Some(Refused {
        refusal: Refusal::Busy,
        subject: "grub".to_owned(),
    }));

    let mut harness = page(configs);

    assert!(shows(
        &harness,
        "Another task runs in the Console. Try again when it has finished."
    ));

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(AppCommand::DismissRefusal(PageId::Configs))]
    );
}

#[test]
fn the_review_names_the_changed_lines_in_words() {
    let configs = listed().with(reviewing(), Viewing::default());
    let harness = page(configs);

    assert!(
        harness.query_all_by_label_contains("@@").next().is_none(),
        "no raw hunk header"
    );
    assert!(shows(&harness, "Line 1"));
}

#[test]
fn the_review_numbers_each_line_and_tints_the_changed_ones() {
    let harness = page(listed().with(reviewing(), Viewing::default()));
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);

    for (line, text, wash) in [
        ("+GRUB_TIMEOUT=2", "GRUB_TIMEOUT=2", tokens.success),
        ("-GRUB_TIMEOUT=5", "GRUB_TIMEOUT=5", tokens.error),
    ] {
        let row = harness.get_by_label(line).rect();
        let painted = |shown: &str| {
            let shape = painted_text_within(&harness, shown, row)
                .unwrap_or_else(|| panic!("{shown} is not inside the row of {line}"));

            shape.galley.rect.translate(shape.pos.to_vec2())
        };

        let (number, content) = (painted("1"), painted(text));
        assert!(
            number.right() < content.left(),
            "{line}: the number comes first"
        );
        assert_eq!(
            fill_under(&harness, row.center()),
            Some(tokens.banner_wash(wash)),
            "{line}"
        );
    }
}

#[test]
fn discard_asks_first_and_keeps_the_edits_on_no() {
    let configs = listed().with(reviewing(), Viewing::default());
    let mut harness = page(configs);

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
    let harness = page(configs);

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

/// The right edges of the title bar's `close` and of the last `footer` button.
fn right_edges(harness: &super::Shown, close: &str, footer: &str) -> (f32, f32) {
    let title = harness.query_all_by_label(close).next().unwrap().rect();
    let button = harness.query_all_by_label(footer).last().unwrap().rect();
    (title.right(), button.right())
}

/// `count` numbered lines, as a long file or its diff.
fn numbered(mark: &str, count: u32) -> String {
    (0..count).fold(String::new(), |mut text, number| {
        writeln!(text, "{mark}line {number}").unwrap();
        text
    })
}

fn long_review() -> Session {
    let (original, edited) = (numbered("", 300), numbered("new ", 300));
    let plain = format!(
        "@@ -1,300 +1,300 @@\n{}{}",
        numbered("-", 300),
        numbered("+new ", 300)
    );

    super::reviewing_with(
        super::path("/etc/default/grub").into(),
        Changes {
            original: super::text(&original),
            edited: super::text(&edited),
            plain,
            added: 300,
            removed: 300,
        },
    )
}

#[test]
fn a_short_review_is_as_tall_as_its_diff() {
    let harness = page(listed().with(reviewing(), Viewing::default()));

    let below = button_room_below(&harness, "Save");

    assert!(below > GROWN_BOTTOM, "Save sits {below} above the bottom");
}

#[test]
fn a_long_review_grows_to_the_window_and_keeps_its_buttons_in_it() {
    let harness = page(listed().with(long_review(), Viewing::default()));

    let below = button_room_below(&harness, "Save");

    assert!(
        (0.0..FILLED_BOTTOM).contains(&below),
        "Save sits {below} above the bottom"
    );
}

#[test]
fn a_short_file_is_shown_in_a_dialog_as_tall_as_its_text() {
    let harness = page(listed().with(Session::default(), viewing()));

    let below = button_room_below(&harness, "Close");

    assert!(below > GROWN_BOTTOM, "Close sits {below} above the bottom");
}

#[test]
fn a_long_file_grows_to_the_window_and_keeps_close_in_it() {
    let long = numbered("", 300);
    let harness = page(listed().with(Session::default(), super::viewing_of(&long)));

    let below = button_room_below(&harness, "Close");

    assert!(
        (0.0..FILLED_BOTTOM).contains(&below),
        "Close sits {below} above the bottom"
    );
}

#[test]
fn a_file_being_read_or_not_read_shows_in_a_small_dialog() {
    let mut failed = Viewing::default();
    let number = failed.start(super::path("/etc/sudoers")).unwrap();
    failed.read(number, Err(Problem::new(ProblemCode::Missing, "")));
    let mut reading = Viewing::default();
    reading.start(super::path("/etc/sudoers")).unwrap();

    let shown = page(listed().with(Session::default(), viewing()));
    let (shown_right, _) = right_edges(&shown, "Close", "Close");

    for (phase, viewing) in [("failed", failed), ("reading", reading)] {
        let harness = page(listed().with(Session::default(), viewing));
        let (title, button) = right_edges(&harness, "Close", "Close");

        assert!(
            title < shown_right - 50.0,
            "{phase}: the dialog ends at {title}, as wide as a shown file at {shown_right}"
        );
        assert!(
            (title - button).abs() < 1.0,
            "{phase}: Close ends at {button}, the dialog at {title}"
        );
    }
}

#[test]
fn the_buttons_end_at_the_dialogs_right_edge() {
    let viewed = page(listed().with(Session::default(), viewing()));
    let reviewed = page(listed().with(reviewing(), Viewing::default()));

    for (dialog, (title, button)) in [
        ("view", right_edges(&viewed, "Close", "Close")),
        (
            "review",
            right_edges(&reviewed, "Close and keep the edits", "Save"),
        ),
    ] {
        assert!(
            (title - button).abs() < 1.0,
            "{dialog}: the last button ends at {button}, the dialog at {title}"
        );
    }
}

/// The dialog's padding is the same on every side, so its centre is the
/// middle between its title's left and Close's right, and between the
/// title's top and the last button's bottom.
fn assert_centred(harness: &super::Shown, title: &str, close: &str, last: &str) {
    let heading = harness.get_by_role_and_label(Role::Heading, title).rect();
    let close = harness.query_all_by_label(close).next().unwrap().rect();
    let button = harness.query_all_by_label(last).last().unwrap().rect();
    let centre = egui::pos2(
        f32::midpoint(heading.left(), close.right()),
        f32::midpoint(heading.top().min(close.top()), button.bottom()),
    );

    let off = centre - harness.ctx.content_rect().center();

    assert!(
        off.x.abs() <= 1.0 && off.y.abs() <= 1.0,
        "{title}: the dialog's centre is {off:?} from the window's centre"
    );
}

#[test]
fn a_file_read_while_its_dialog_shows_stays_centred_as_the_dialog_grows() {
    let mut reading = Viewing::default();
    reading.start(super::path("/etc/sudoers")).unwrap();
    let mut harness = page(listed().with(Session::default(), reading));

    harness.state_mut().fixture.services.configs = listed().with(Session::default(), viewing());
    harness.step();

    assert_centred(&harness, "The text of /etc/sudoers", "Close", "Close");
}

#[test]
fn the_review_sits_in_the_windows_centre() {
    let harness = page(listed().with(reviewing(), Viewing::default()));

    assert_centred(
        &harness,
        "Save the changes to /etc/default/grub?",
        "Close and keep the edits",
        "Save",
    );
}
