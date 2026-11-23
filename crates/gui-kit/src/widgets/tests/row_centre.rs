use egui::{Label, Rect, Ui};
use egui_kittest::kittest::Queryable as _;

use super::super::{
    ActionRow, ButtonKind, card, card_row, pill_button, rich, split_row, switch, title_line,
};
use super::action_row_tile::{ink, row_content};
use crate::fonts::{SMALL, Weight};
use crate::testing::{harness, painted_text, textured_rects};
use crate::{icons, tokens};

/// A glyph snaps to whole pixels, so its centre can be half a point off.
const SNAP: f32 = 1.0;

/// Off by less than this, two centres count as one.
const SAME: f32 = 0.5;

/// Two centres of text count as one, as the ink of a line is not its box.
const TEXT: f32 = 1.0;

/// Letters without a descender, so the title's ink bottom is its baseline.
const TITLE: &str = "Lutris";

#[derive(Clone, Copy)]
struct Placed {
    title: Rect,
    control: Option<Rect>,
    content: Rect,
}

/// What an action row drew for `lines` lines of detail.
#[derive(Clone, Copy)]
struct Action {
    badge: Rect,
    button: Rect,
    content: Rect,
    /// The top of the title's capitals and the bottom of the last baseline.
    ink: (f32, f32),
    /// The top of the title's capitals and their baseline.
    capitals: (f32, f32),
}

fn content_of(ui: &Ui, row: Rect) -> Rect {
    let padding = tokens::current(ui.ctx()).sizes.row_padding.y;
    row.shrink2(egui::vec2(0.0, padding))
}

/// A title and `lines` lines of description, short enough to never wrap.
fn words(ui: &mut Ui, lines: u8) -> Rect {
    let title = title_line(ui, true, |ui| ui.label("Lutris").rect);
    let ink = tokens::current(ui.ctx()).secondary;
    for line in 0..lines {
        let text = rich(ui, &format!("Line {line}"), Weight::Regular, SMALL, ink);
        ui.add(Label::new(text));
    }

    title
}

fn split(lines: u8, with_switch: bool) -> Placed {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        move |ui, placed: &mut Placed| {
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    let width = if with_switch { 60.0 } else { 0.0 };
                    split_row(
                        ui,
                        width,
                        |ui| words(ui, lines),
                        |ui| with_switch.then(|| switch(ui, true, "Lutris").rect),
                    )
                });

                let (title, control) = drawn.inner;
                *placed = Placed {
                    title,
                    control,
                    content: content_of(ui, drawn.response.rect),
                };
            });
        },
        Placed {
            title: Rect::NOTHING,
            control: None,
            content: Rect::NOTHING,
        },
    );

    harness.run();
    *harness.state()
}

#[test]
fn a_control_is_centred_on_the_whole_row_whatever_its_lines() {
    for lines in 0..=3 {
        let placed = split(lines, true);
        let control = placed.control.expect("a switch");
        let off = control.center().y - placed.content.center().y;

        assert!(
            off.abs() < SAME,
            "{lines} lines: the switch sits {off} off the row's middle"
        );
    }
}

#[test]
fn a_control_does_not_move_the_title() {
    for lines in 0..=3 {
        let with = split(lines, true);
        let without = split(lines, false);
        let with = with.title.top() - with.content.top();
        let without = without.title.top() - without.content.top();

        assert!(
            (with - without).abs() < SAME,
            "{lines} lines: the title starts {with} down beside a switch, {without} without"
        );
    }
}

fn action(lines: u8) -> Action {
    let detail = (0..lines)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    let shown = detail.clone();
    let mut harness = harness(800.0, 400.0).build_ui_state(
        move |ui, placed: &mut Action| placed.content = row_content(ui, &shown),
        Action {
            badge: Rect::NOTHING,
            button: Rect::NOTHING,
            content: Rect::NOTHING,
            ink: (0.0, 0.0),
            capitals: (0.0, 0.0),
        },
    );

    harness.run();
    let title = painted_text(&harness, TITLE).expect("the title is painted");
    let last = if lines == 0 { TITLE } else { detail.as_str() };
    let last = painted_text(&harness, last).expect("the last line is painted");
    let badge = textured_rects(&harness)
        .first()
        .copied()
        .expect("the icon is painted");

    let button = harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Run now")
        .rect();

    let mut placed = *harness.state();
    placed.badge = badge;
    placed.button = button;
    placed.ink = (ink(&title).0, ink(&last).1);
    placed.capitals = ink(&title);
    placed
}

/// Two buttons stacked, taller than the tile and a one line title.
fn tall_actions() -> (Rect, Rect) {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        |ui, (content, actions): &mut (Rect, Rect)| {
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    ActionRow::new(icons::CHECKMARK, "Update the system").show(ui, |ui| {
                        *actions = ui
                            .vertical(|ui| {
                                _ = pill_button(ui, "Run now", ButtonKind::Soft);
                                _ = pill_button(ui, "Later", ButtonKind::Soft);
                            })
                            .response
                            .rect;
                    });
                });

                *content = content_of(ui, drawn.response.rect);
            });
        },
        (Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    harness.run();
    *harness.state()
}

#[test]
fn actions_taller_than_the_tile_sit_in_the_middle_of_the_row() {
    let (content, actions) = tall_actions();

    assert!(
        (actions.center().y - content.center().y).abs() < SAME,
        "actions at {actions:?}, row content {content:?}"
    );
}

/// As in KDE's form delegates, the icon and the text block (the title's
/// capitals to the last baseline) centre on the row, and the button does
/// too while the detail is short enough to keep it beside the words.
#[test]
fn the_icon_and_the_text_block_centre_on_the_row() {
    for lines in 0..=3 {
        let placed = action(lines);
        let content = placed.content;
        let (top, bottom) = placed.ink;

        let icon = (placed.badge.center().y - content.center().y).abs();
        assert!(
            icon < SNAP,
            "{lines} lines: the icon sits {icon} off the row's middle, at {:?}",
            placed.badge
        );

        let text = (top.midpoint(bottom) - content.center().y).abs();
        assert!(
            text < TEXT,
            "{lines} lines: the text block sits {text} off the row's middle, from {top} to {bottom} in {content:?}"
        );

        if lines < 3 {
            assert!(
                (placed.button.center().y - content.center().y).abs() < SAME,
                "{lines} lines: button at {:?}, row content {content:?}",
                placed.button
            );
        } else {
            // Title and three lines of description are taller than the row's
            // share, so the actions centre on the title's capitals.
            let (cap_top, baseline) = placed.capitals;
            assert!(
                (placed.button.center().y - f32::midpoint(cap_top, baseline)).abs() < TEXT,
                "{lines} lines: button at {:?}, title capitals {cap_top}..{baseline}",
                placed.button
            );
        }
    }
}
