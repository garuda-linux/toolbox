use egui::{Label, Rect, Ui, vec2};

use super::super::list::text_line_height;
use super::super::{
    ButtonKind, card, card_row, pill_button, rich, split_row, text_block_height, title_line,
};
use crate::fonts::{BODY, SMALL, Weight};
use crate::testing::harness;
use crate::tokens;

/// Where one card row's title, description and button went, and the
/// row's content height with the height `text_block_height` gives it.
#[derive(Clone, Copy)]
struct Placed {
    title: Rect,
    description: Rect,
    button: Option<Rect>,
    row: Rect,
    estimate: f32,
}

fn words(ui: &mut Ui) -> (Rect, Rect) {
    let tokens = tokens::current(ui.ctx());
    let words = rich(ui, "Login shell", Weight::SemiBold, BODY, tokens.text);
    let line = title_line(ui, true, |ui| ui.label(words).rect);

    // egui gives the label the line's height and centres the text in it.
    let text = text_line_height(ui, Weight::SemiBold, BODY);
    let title = Rect::from_center_size(line.center(), vec2(line.width(), text));
    let description = rich(
        ui,
        "The shell new terminals start",
        Weight::Regular,
        SMALL,
        tokens.secondary,
    );

    (title, ui.add(Label::new(description).wrap()).rect)
}

fn row(with_button: bool) -> Placed {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        move |ui, placed: &mut Placed| {
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    placed.estimate = text_block_height(ui, 1);
                    let width = if with_button { 120.0 } else { 0.0 };
                    split_row(ui, width, words, |ui| {
                        with_button.then(|| pill_button(ui, "Change", ButtonKind::Soft).rect)
                    })
                });

                let ((title, description), button) = drawn.inner;
                placed.title = title;
                placed.description = description;
                placed.button = button;
                placed.row = drawn.response.rect;
            });
        },
        Placed {
            title: Rect::NOTHING,
            description: Rect::NOTHING,
            button: None,
            row: Rect::NOTHING,
            estimate: 0.0,
        },
    );

    harness.run();
    *harness.state()
}

#[test]
fn the_description_starts_a_tight_gap_under_the_title_text() {
    let placed = row(true);
    let gap = placed.description.top() - placed.title.bottom();
    let tight = tokens::Sizes::DEFAULT.tight_gap;

    assert!(
        (gap - tight).abs() <= 0.5,
        "gap {gap}, title {:?}, description {:?}",
        placed.title,
        placed.description
    );
}

#[test]
fn the_button_is_centred_on_the_row() {
    let placed = row(true);
    let button = placed.button.expect("a button");

    assert!(
        (placed.row.center().y - button.center().y).abs() <= 0.5,
        "row {:?}, button {button:?}",
        placed.row
    );
}

#[test]
fn a_row_without_a_control_starts_its_title_at_the_same_height() {
    let with = row(true);
    let without = row(false);
    let with = with.title.top() - with.row.top();
    let without = without.title.top() - without.row.top();

    assert!((with - without).abs() <= 0.5, "{with} and {without}");
}

#[test]
fn the_button_stays_inside_the_row() {
    let placed = row(true);
    let button = placed.button.expect("a button");
    let padding = tokens::Sizes::DEFAULT.row_padding.y;

    assert!(
        button.top() >= placed.row.top() + padding - 0.5
            && button.bottom() <= placed.row.bottom() - padding + 0.5,
        "button {button:?}, row {:?}",
        placed.row
    );
}

#[test]
fn a_fixed_row_height_matches_the_drawn_row() {
    for with_button in [true, false] {
        let placed = row(with_button);
        let padding = tokens::Sizes::DEFAULT.row_padding.y;
        let content = 2.0_f32.mul_add(-padding, placed.row.height());

        assert!(
            (content - placed.estimate).abs() <= 0.5,
            "button {with_button}: drawn {content}, estimated {}",
            placed.estimate
        );
    }
}
