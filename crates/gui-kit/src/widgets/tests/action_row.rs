use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::{ActionRow, ButtonKind, Tone, pill_button};
use crate::icons;
use crate::testing::harness;

#[test]
fn a_row_without_actions_ends_with_its_words() {
    let mut harness = harness(400.0, 200.0).build_ui_state(
        |ui, drawn: &mut Rect| {
            *drawn = ui
                .scope(|ui| {
                    ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Apps to install: 1")
                        .detail("Strawberry Player")
                        .show_without_actions(ui);
                })
                .response
                .rect;
        },
        Rect::NOTHING,
    );

    harness.run();

    let height = harness.state().height();
    assert!(height <= 40.0, "the row is {height} px tall");
}

fn task_row(width: f32) -> egui_kittest::Harness<'static> {
    let mut harness = harness(width, 300.0).build_ui(|ui| {
        ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Update the system")
            .detail("Installs the newest versions of every package.")
            .show(ui, |ui| {
                _ = pill_button(ui, "Run now", ButtonKind::Soft);
            });
    });

    harness.run();
    harness
}

#[test]
fn the_action_sits_at_the_right_beside_the_words() {
    let harness = task_row(700.0);
    let title = harness.get_by_label("Update the system").rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Run now")
        .rect();

    assert!(
        button.right() > harness.ctx.content_rect().right() - 20.0,
        "the button is not at the right, it ends at {}",
        button.right()
    );
    assert!(
        button.top() <= title.bottom() && button.bottom() >= title.top(),
        "the button is not beside the title"
    );
}

#[test]
fn a_narrow_row_puts_its_action_under_the_text_at_the_left() {
    let harness = task_row(300.0);
    let title = harness.get_by_label("Update the system").rect();
    let detail = harness
        .get_by_label("Installs the newest versions of every package.")
        .rect();

    let button = harness
        .get_by_role_and_label(Role::Button, "Run now")
        .rect();

    assert!(
        button.top() >= detail.bottom(),
        "the button is not under the text"
    );
    assert!(
        (button.left() - title.left()).abs() < 0.5,
        "the button starts at {}, the title at {}",
        button.left(),
        title.left()
    );
}

#[test]
fn several_actions_stay_on_one_line_beside_the_words() {
    let mut harness = harness(900.0, 300.0).build_ui(|ui| {
        ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Updates available")
            .detail("Update now in the Console, or queue the update.")
            .show(ui, |ui| {
                for name in ["Update now", "Queue update", "Show packages"] {
                    _ = pill_button(ui, name, ButtonKind::Soft);
                }
            });
    });

    harness.run();

    let centre = |name| {
        harness
            .get_by_role_and_label(Role::Button, name)
            .rect()
            .center()
            .y
    };

    assert!((centre("Update now") - centre("Show packages")).abs() < 0.5);
    assert!((centre("Queue update") - centre("Show packages")).abs() < 0.5);
}

#[test]
fn actions_beside_the_words_keep_the_order_they_were_added_in() {
    let mut harness = harness(700.0, 300.0).build_ui(|ui| {
        ActionRow::new(icons::CHECKMARK, Tone::Neutral, "AUR updates available").show(ui, |ui| {
            _ = pill_button(ui, "Update in Console", ButtonKind::Soft);
            _ = pill_button(ui, "Show packages", ButtonKind::Soft);
        });
    });

    harness.run();

    let first = harness
        .get_by_role_and_label(Role::Button, "Update in Console")
        .rect();

    let second = harness
        .get_by_role_and_label(Role::Button, "Show packages")
        .rect();

    assert!(first.right() <= second.left(), "{first:?} {second:?}");
    assert!(
        second.right() > harness.ctx.content_rect().right() - 20.0,
        "the actions do not end at the right"
    );
}

#[test]
fn the_detail_starts_a_tight_gap_under_the_title() {
    let harness = task_row(700.0);
    let title = harness.get_by_label("Update the system").rect();
    let detail = harness
        .get_by_label("Installs the newest versions of every package.")
        .rect();

    let gap = detail.top() - title.bottom();
    let tight = crate::tokens::Sizes::DEFAULT.tight_gap;

    assert!((gap - tight).abs() <= 0.5, "gap {gap}");
}

fn spaced() -> (Rect, Rect, Rect, Rect) {
    let detail = "Installs the newest versions of every package, with snapshots before \
                  and checks after, and tells what changed once it is done.";

    let mut harness = harness(700.0, 300.0).build_ui(move |ui| {
        crate::tokens::set_page_hue(ui.ctx(), crate::tokens::Hue::Green);
        ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Update the system")
            .detail(detail)
            .show(ui, |ui| {
                _ = pill_button(ui, "Run now", ButtonKind::Soft);
            });
    });

    harness.run();
    let fill = crate::tokens::current(&harness.ctx)
        .tiles
        .of(crate::tokens::Hue::Green)
        .fill;

    let badge = crate::testing::filled_rect(&harness, fill).expect("the tile is painted");
    let title = harness.get_by_label("Update the system").rect();
    let detail = harness.get_by_label(detail).rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Run now")
        .rect();

    (badge, title, detail, button)
}

#[test]
fn the_words_start_one_picture_gap_right_of_the_tile() {
    let (badge, title, detail, _) = spaced();
    let wanted = badge.right() + crate::tokens::Sizes::DEFAULT.picture_gap;

    assert!(
        (title.left() - wanted).abs() < 0.5,
        "the title starts at {}, a picture gap right of the tile is {wanted}",
        title.left()
    );
    assert!(
        (detail.left() - title.left()).abs() < 0.5,
        "the detail starts at {}, the title at {}",
        detail.left(),
        title.left()
    );
}

#[test]
fn the_actions_keep_their_gap_from_the_words() {
    let (_, _, detail, button) = spaced();
    let gap = button.left() - detail.right();

    assert!(
        gap >= 14.0 - 0.5,
        "the wrapped detail ends {gap} before the button"
    );
}
