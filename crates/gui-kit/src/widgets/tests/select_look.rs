//! The closed dropdown as Darkly draws a non-editable combo box: a push
//! button, inset two pixels from its widget, with a control radius and
//! no rim (darklystyle.cpp:7736-7800, darklyhelper.cpp:811-850).

use egui::accesskit::Role;
use egui::{Color32, Key, Rect, Shape, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::Select;
use crate::testing::{harness, rim_colours_around};
use crate::theme::CONTROL_RADIUS;
use crate::tokens::{self, darkly};

const LABEL: &str = "Sort by";

fn closed(theme: Theme) -> Harness<'static> {
    harness(400.0, 120.0).with_theme(theme).build_ui(|ui| {
        _ = Select::new(LABEL, &["Name", "Size"])
            .selected(Some(0))
            .width(160.0)
            .show(ui);
    })
}

fn pill(harness: &Harness<'_>) -> Rect {
    harness.get_by_role_and_label(Role::ComboBox, LABEL).rect()
}

/// The fill and corner radius of the button drawn over the pill, the
/// pill inset two pixels as Darkly's frame is.
fn button(harness: &Harness<'_>) -> Option<(Color32, u8)> {
    let frame = pill(harness).shrink(2.0);
    harness.output().shapes.iter().find_map(|clipped| {
        if let Shape::Rect(rect) = &clipped.shape
            && rect.blur_width == 0.0
            && rect.rect.center().distance(frame.center()) < 0.01
            && (rect.rect.width() - frame.width()).abs() < 0.01
        {
            Some((rect.fill, rect.corner_radius.nw))
        } else {
            None
        }
    })
}

#[test]
fn a_closed_combo_box_at_rest_is_a_field_button_with_no_rim() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = closed(theme);
        let tokens = tokens::current(&harness.ctx);

        assert_eq!(
            button(&harness),
            Some((tokens.field, CONTROL_RADIUS)),
            "{theme:?}"
        );
        assert!(
            rim_colours_around(&harness, pill(&harness).center()).is_empty(),
            "{theme:?}: the pill has a rim"
        );
    }
}

#[test]
fn a_closed_combo_box_under_the_pointer_takes_the_button_hover() {
    let mut harness = closed(Theme::Dark);
    harness.hover_at(pill(&harness).center());
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    assert_eq!(
        button(&harness).map(|(fill, _)| fill),
        Some(tokens.button_hover)
    );
}

fn keyboard_focus(harness: &mut Harness<'_>) {
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    harness.run();
}

#[test]
fn a_combo_box_focused_from_the_keyboard_fills_with_the_decoration_focus() {
    let mut harness = closed(Theme::Dark);
    keyboard_focus(&mut harness);

    let tokens = tokens::current(&harness.ctx);
    assert_eq!(
        button(&harness).map(|(fill, _)| fill),
        Some(tokens.decoration_focus)
    );
}

#[test]
fn a_combo_box_focused_by_a_click_shows_no_focus_fill() {
    let mut harness = closed(Theme::Dark);
    harness.get_by_role_and_label(Role::ComboBox, LABEL).click();
    harness.run();
    harness.run();

    // The click opens the list, which draws the box pressed, so only the
    // focus fill is ruled out here.
    let tokens = tokens::current(&harness.ctx);
    assert_ne!(
        button(&harness).map(|(fill, _)| fill),
        Some(tokens.decoration_focus)
    );
}

/// Focus under the pointer lightens the focus colour by 15 % and then
/// by 2 % (darklyhelper.cpp:266, 271, 812).
#[test]
fn a_focused_combo_box_under_the_pointer_lightens_its_focus_fill() {
    let mut harness = closed(Theme::Dark);
    keyboard_focus(&mut harness);
    harness.hover_at(pill(&harness).center());
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let lit = darkly::lighter(darkly::lighter(tokens.decoration_focus, 1.15), 1.02);
    assert_eq!(button(&harness).map(|(fill, _)| fill), Some(lit));
}
