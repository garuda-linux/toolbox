use egui::{Label, Rect, Ui};
use egui_kittest::kittest::Queryable as _;

use super::super::card::DIVIDER;
use super::super::{
    ButtonKind, ListItem, RowHeights, RowList, Select, card, card_row, icon_button, number_field,
    pill_button, rich, row_height, segmented, split_row, switch, text_block_height, title_line,
};
use crate::fonts::{BODY, SMALL, Weight};
use crate::icons;
use crate::testing::{harness, painted_text};
use crate::theme::PILL_HEIGHT;

/// The room a focus ring takes outside its control: its gap and stroke.
const RING: f32 = 4.0;

type Control = fn(&mut Ui) -> Rect;

/// What a control took and the row around it.
#[derive(Clone, Copy)]
struct Placed {
    control: Rect,
    row: Rect,
}

/// A group of the Groups list, and whether it has a Remove button.
struct Group(&'static str, bool);

impl ListItem for Group {
    fn key(&self) -> &str {
        self.0
    }

    fn name(&self) -> &str {
        self.0
    }
}

fn placed(in_row: bool, control: Control) -> Placed {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        move |ui, placed: &mut Placed| {
            if in_row {
                card(ui, |ui| {
                    let drawn = card_row(ui, true, control);
                    placed.control = drawn.inner;
                    placed.row = drawn.response.rect;
                });
            } else {
                placed.control = control(ui);
            }
        },
        Placed {
            control: Rect::NOTHING,
            row: Rect::NOTHING,
        },
    );

    harness.run();
    *harness.state()
}

fn button(ui: &mut Ui) -> Rect {
    pill_button(ui, "Remove", ButtonKind::Soft).rect
}

fn dropdown(ui: &mut Ui) -> Rect {
    let before = ui.cursor().min;
    Select::new("Language", &["English", "Deutsch"])
        .selected(Some(0))
        .show(ui);
    Rect::from_min_max(before, ui.min_rect().max)
}

fn choice(ui: &mut Ui) -> Rect {
    let before = ui.cursor().min;
    segmented(ui, "Theme", &[(0_u8, "Dark"), (1_u8, "Light")], 0_u8);
    Rect::from_min_max(before, ui.min_rect().max)
}

fn number(ui: &mut Ui) -> Rect {
    let mut value = 5;
    number_field(ui, &mut value, 1..=10, 80.0, "Count").rect
}

fn kebab(ui: &mut Ui) -> Rect {
    icon_button(ui, icons::OVERFLOW_MENU, "More").rect
}

#[test]
fn a_control_in_a_row_is_as_tall_as_outside_one() {
    let controls: [(&str, Control); 5] = [
        ("button", button),
        ("dropdown", dropdown),
        ("segmented", choice),
        ("number", number),
        ("icon button", kebab),
    ];

    for (name, control) in controls {
        let inside = placed(true, control).control.height();
        let outside = placed(false, control).control.height();
        assert!(
            (inside - outside).abs() < 0.5,
            "{name}: {inside} tall in a row, {outside} outside"
        );
    }
}

#[test]
fn a_pill_shaped_control_keeps_the_pill_height_everywhere() {
    let controls: [(&str, Control); 3] = [
        ("button", button),
        ("dropdown", dropdown),
        ("segmented", choice),
    ];

    for in_row in [false, true] {
        for (name, control) in controls {
            let height = placed(in_row, control).control.height();
            assert!(
                (height - PILL_HEIGHT).abs() < 0.5,
                "{name}, in a row {in_row}: {height} tall"
            );
        }
    }
}

/// The title line's rect and the row's in a card's row, beside `control`.
fn titled(control: Control) -> (Rect, Rect) {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        move |ui, placed: &mut (Rect, Rect)| {
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    split_row(
                        ui,
                        160.0,
                        |ui| title_line(ui, false, |ui| ui.label("Only when I ask")).rect,
                        control,
                    )
                    .0
                });

                *placed = (drawn.inner, drawn.response.rect);
            });
        },
        (Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    *harness.state()
}

fn nothing(_ui: &mut Ui) -> Rect {
    Rect::NOTHING
}

fn toggle(ui: &mut Ui) -> Rect {
    switch(ui, true, "Probe", true).rect
}

#[test]
fn a_title_line_in_a_row_is_as_tall_as_a_control_with_or_without_one() {
    let rows: [(&str, Control); 3] = [
        ("none", nothing),
        ("a switch", toggle),
        ("a button", button),
    ];

    let padding = crate::tokens::Sizes::default().row_padding.y;

    for (name, control) in rows {
        let (title, row) = titled(control);
        let line = padding.mul_add(-2.0, row.height());
        assert!(
            (line - PILL_HEIGHT).abs() < 0.5,
            "beside {name}: a {line} tall title line"
        );
        assert!(
            (row.center().y - title.center().y).abs() < 0.5,
            "beside {name}: row at {row:?}, title at {title:?}"
        );
    }
}

#[test]
fn a_focus_ring_in_a_row_stays_inside_the_row() {
    let drawn = placed(true, button);
    assert!(
        drawn.row.contains_rect(drawn.control.expand(RING)),
        "ring around {:?} reaches past {:?}",
        drawn.control,
        drawn.row
    );
}

/// A row of the Groups list: a title, a detail, and a button for a
/// group that has one.
fn group_row(ui: &mut Ui, group: &Group) {
    split_row(
        ui,
        160.0,
        |ui| {
            title_line(ui, false, |ui| {
                let title = rich(ui, group.0, Weight::SemiBold, BODY, egui::Color32::WHITE);
                ui.add(Label::new(title).truncate());
            });

            let detail = rich(ui, "GID 1000", Weight::Regular, SMALL, egui::Color32::GRAY);
            ui.add(Label::new(detail).truncate());
        },
        |ui| {
            if group.1 {
                pill_button(ui, &format!("Remove {}", group.0), ButtonKind::Soft);
            }
        },
    );
}

#[test]
fn text_in_a_row_with_a_control_sits_where_it_does_without_one() {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        |ui, height: &mut f32| {
            let items = [Group("wheel", false), Group("gamers", true)];
            *height = row_height(ui, text_block_height(ui, 1));
            let heights = RowHeights::Fixed(*height);
            RowList::new("groups", "Groups", heights).show(ui, &items, |ui, group, _| {
                group_row(ui, group);
            });
        },
        0.0,
    );

    harness.run();
    let top = |text: &str| {
        painted_text(&harness, text)
            .map_or_else(|| panic!("{text} was not painted"), |shape| shape.pos.y)
    };

    // Both rows are as tall, so a title one row height and the divider
    // below the other has the same place in its row.
    let lower = top("gamers") - top("wheel") - *harness.state() - DIVIDER;
    assert!(
        lower.abs() < 0.5,
        "the title beside a button sits {lower} lower"
    );
}

#[test]
fn a_one_line_row_with_a_control_stays_centred() {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        |ui, placed: &mut (Rect, Rect, Rect)| {
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    split_row(
                        ui,
                        160.0,
                        |ui| title_line(ui, false, |ui| ui.label("Only when I ask").rect),
                        button,
                    )
                });

                let (title, control) = drawn.inner;
                *placed = (drawn.response.rect, title, control);
            });
        },
        (Rect::NOTHING, Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    let (row, title, control) = *harness.state();
    assert!(
        (title.center().y - control.center().y).abs() < 0.5,
        "title at {title:?}, control at {control:?}"
    );
    assert!(
        (row.center().y - title.center().y).abs() < 0.5,
        "row at {row:?}, title at {title:?}"
    );
}

#[test]
fn a_segment_in_a_row_holds_its_whole_label_in_its_middle() {
    let mut harness = harness(800.0, 300.0).build_ui(|ui| {
        card(ui, |ui| {
            card_row(ui, true, choice);
        });
    });

    harness.run();
    let segment = harness
        .get_by_role_and_label(egui::accesskit::Role::RadioButton, "Dark")
        .rect();

    let label = painted_text(&harness, "Dark").expect("the label is painted");

    assert!(
        segment.height() >= label.galley.size().y,
        "a {} tall segment holds a {} tall label",
        segment.height(),
        label.galley.size().y
    );
}
