//! The closed dropdown as Darkly draws a non-editable combo box: a push
//! button, inset two pixels from its widget, with a control radius
//! (darklystyle.cpp:7736-7800, darklyhelper.cpp:811-850). It keeps the
//! card outline the app's push buttons add.

use egui::accesskit::Role;
use egui::{Color32, Key, Rect, Shape, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::Select;
use crate::testing::{harness, painted, rim_colours_around};
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

/// Whether the last frame drew a push button's light along a top edge.
fn button_light(harness: &Harness<'_>) -> bool {
    let light = tokens::current(&harness.ctx).button_top_light;
    painted(harness).into_iter().any(|shape| {
        matches!(shape, Shape::Mesh(mesh)
            if mesh.vertices.iter().any(|vertex| vertex.color == light))
    })
}

/// The pill is the app's push button: its fill, its card outline and,
/// on a dark window, its top light.
#[test]
fn a_closed_combo_box_at_rest_is_a_push_button() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = closed(theme);
        let tokens = tokens::current(&harness.ctx);

        assert_eq!(
            button(&harness),
            Some((tokens.input, CONTROL_RADIUS)),
            "{theme:?}"
        );
        assert_eq!(
            rim_colours_around(&harness, pill(&harness).center()),
            vec![tokens.card_outline],
            "{theme:?}: the pill keeps the card outline"
        );
        assert_eq!(button_light(&harness), theme == Theme::Dark, "{theme:?}");
    }
}

/// The focus fill and the open list's pressed fill replace the outline,
/// as they do on a push button.
#[test]
fn a_focused_or_open_combo_box_drops_its_outline() {
    let mut focused = closed(Theme::Dark);
    keyboard_focus(&mut focused);
    assert!(
        rim_colours_around(&focused, pill(&focused).center()).is_empty(),
        "the focus fill keeps an outline"
    );

    let mut open = closed(Theme::Dark);
    open.get_by_role_and_label(Role::ComboBox, LABEL).click();
    open.run();
    open.hover_at(egui::pos2(390.0, 110.0));
    open.run();
    assert!(
        rim_colours_around(&open, pill(&open).center()).is_empty(),
        "the open list keeps an outline"
    );
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
