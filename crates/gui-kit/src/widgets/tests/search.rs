use egui::accesskit::Role;
use egui::{Align2, Rect, TextEdit};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{SearchTexts, request_search_focus, search_field, text_field};
use crate::testing::{harness, lines_in};
use crate::theme::PILL_HEIGHT;

const TEXTS: SearchTexts<'static> = SearchTexts {
    name: "Search services",
    hint: "Name or description",
    clear: "Clear the search",
};

fn field(text: &str) -> Harness<'static, String> {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        |ui, text: &mut String| {
            search_field(ui, text, TEXTS, 300.0);
        },
        text.to_owned(),
    );

    harness.run();
    harness
}

#[test]
fn an_empty_field_has_no_clear_button() {
    let harness = field("");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Clear the search")
            .is_none()
    );
}

#[test]
fn clear_empties_the_field_and_puts_the_focus_back() {
    let mut harness = field("ssh");

    harness
        .get_by_role_and_label(Role::Button, "Clear the search")
        .click();
    harness.run();

    assert_eq!(harness.state(), "");
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Search services")
            .is_focused()
    );
}

#[test]
fn a_focus_request_focuses_the_field() {
    let mut harness = field("");

    request_search_focus(&harness.ctx);
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Search services")
            .is_focused()
    );
}

#[test]
fn a_focus_request_no_field_took_lapses() {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        |ui, (shown, text): &mut (bool, String)| {
            if *shown {
                search_field(ui, text, TEXTS, 300.0);
            }
        },
        (false, String::new()),
    );

    harness.run();
    request_search_focus(&harness.ctx);
    harness.run_steps(5);
    harness.state_mut().0 = true;
    harness.run();

    assert!(
        !harness
            .get_by_role_and_label(Role::TextInput, "Search services")
            .is_focused()
    );
}

#[test]
fn the_caret_of_a_focused_empty_field_leaves_the_hint_readable() {
    let mut harness = field("");

    request_search_focus(&harness.ctx);
    harness.run();

    let caret_colour = harness.ctx.global_style().visuals.text_cursor.stroke.color;
    let caret = lines_in(&harness, caret_colour)
        .into_iter()
        .find(|[top, bottom]| (top.x - bottom.x).abs() < f32::EPSILON)
        .expect("a focused field shows its caret");

    let hint = crate::testing::painted_text(&harness, TEXTS.hint).expect("the hint shows");
    let hint_left = hint.pos.x + hint.galley.rect.left();
    let caret_width = harness.ctx.global_style().visuals.text_cursor.stroke.width;

    assert!(
        hint_left >= caret[0].x + caret_width,
        "the hint starts at {hint_left}, the caret at {}",
        caret[0].x
    );
}

fn text_field_shown(focused: bool) -> Harness<'static, String> {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        move |ui, text: &mut String| {
            let response = text_field(
                ui,
                TextEdit::singleline(text)
                    .desired_width(200.0)
                    .align(Align2::LEFT_CENTER),
            );

            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_label("Name"));
            if focused {
                response.request_focus();
            }
        },
        "Hello".to_owned(),
    );

    harness.run();
    harness
}

fn node(harness: &Harness<'_, String>) -> Rect {
    harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .rect()
}

#[test]
fn a_field_is_as_tall_as_the_pills() {
    let harness = text_field_shown(false);

    assert!((node(&harness).height() - PILL_HEIGHT).abs() < 0.5);
}
