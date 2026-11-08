use egui::{Label, Rect, Ui};
use egui_kittest::kittest::Queryable as _;

use super::super::action_row::capitals_top;
use super::super::{
    ActionRow, ButtonKind, Tone, card, card_row, pill_button, rich, split_row, switch, title_line,
};
use crate::fonts::{SMALL, Weight};
use crate::icons;
use crate::testing::{harness, painted_text};
use crate::tokens::{self, set_page_hue};

/// Off by less than this, two centres count as one.
const SAME: f32 = 0.5;

#[derive(Clone, Copy)]
struct Placed {
    title: Rect,
    control: Option<Rect>,
    content: Rect,
}

fn content_of(ui: &Ui, row: Rect) -> Rect {
    let padding = tokens::current(ui.ctx()).sizes.row_padding.y;
    row.shrink2(egui::vec2(0.0, padding))
}

/// A title and `lines` lines of description, short enough to never wrap.
fn words(ui: &mut Ui, lines: u8) -> Rect {
    let title = title_line(ui, true, |ui| ui.label("Bottles").rect);
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
                        |ui| with_switch.then(|| switch(ui, true, "Bottles", true).rect),
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

/// The title, button and row content of an action row with `lines` lines
/// of detail, the capitals' offset below the top of the title text and
/// the detail's last baseline, read from letters that do not descend.
fn action(lines: u8) -> (Rect, Rect, Rect, f32, Option<f32>) {
    let detail = (0..lines)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    let shown = detail.clone();
    let mut harness = harness(800.0, 400.0).build_ui_state(
        move |ui, (content, capitals): &mut (Rect, f32)| {
            set_page_hue(ui.ctx(), tokens::Hue::Green);
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    ActionRow::new(icons::CHECKMARK, Tone::Neutral, "Update the system")
                        .detail(&shown)
                        .show(ui, |ui| {
                            _ = pill_button(ui, "Run now", ButtonKind::Soft);
                        });
                });

                *content = content_of(ui, drawn.response.rect);
                *capitals = capitals_top(ui);
            });
        },
        (Rect::NOTHING, 0.0),
    );

    harness.run();
    let shape = painted_text(&harness, "Update the system").expect("the title is painted");
    let title = shape.galley.rect.translate(shape.pos.to_vec2());
    let button = harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Run now")
        .rect();

    let baseline = painted_text(&harness, &detail)
        .map(|painted| painted.galley.mesh_bounds.bottom() + painted.pos.y);

    let (content, capitals) = *harness.state();
    (title, button, content, capitals, baseline)
}

/// The tile's top is checked against the title's letters in
/// `action_row_tile`. Words lower than the tile end on a baseline, which
/// keeps as much room to the row's edge as the capitals do.
#[test]
fn an_action_rows_capitals_have_the_room_of_its_last_baseline_and_actions_sit_in_the_middle() {
    let padding = tokens::Sizes::DEFAULT.row_padding.y;
    for lines in 0..=3 {
        let (title, button, content, capitals, baseline) = action(lines);
        let above = title.top() + capitals - (content.top() - padding);
        let expected = match baseline {
            Some(baseline) if lines >= 2 => content.bottom() + padding - baseline,
            _ => padding,
        };

        assert!(
            (above - expected).abs() <= 1.0,
            "{lines} lines: the capitals start {above} under the row's top, {expected} wanted"
        );
        assert!(
            (button.center().y - content.center().y).abs() < SAME,
            "{lines} lines: button at {button:?}, row content {content:?}"
        );
    }
}
