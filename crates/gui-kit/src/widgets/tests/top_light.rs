use egui::{CornerRadius, Id, Margin, Rect, Theme, Ui};
use egui_kittest::Harness;

use super::super::{MetricKind, MetricText, ModalWrapper, card, link_tile, metric};
use crate::frost::Glass;
use crate::icons;
use crate::testing::{harness, rims_in, top_lights};
use crate::theme::CARD_RADIUS;

fn shown(theme: Theme, surface: fn(&mut Ui) -> Rect) -> Harness<'static, Rect> {
    let mut harness = harness(500.0, 300.0).with_theme(theme).build_ui_state(
        move |ui, rect: &mut Rect| *rect = surface(ui),
        Rect::NOTHING,
    );

    harness.run();
    harness
}

/// Taller than both its corners, so they keep their whole radius.
fn plain_card(ui: &mut Ui) -> Rect {
    card(ui, |ui| {
        ui.label("Kernel");
        ui.add_space(60.0);
    })
    .response
    .rect
}

#[test]
fn a_dark_card_has_a_light_along_its_top_that_fades_down_its_corners() {
    let harness = shown(Theme::Dark, plain_card);
    let rect = *harness.state();
    let found = top_lights(&harness);

    let [light] = found.as_slice() else {
        panic!("one light, found {}", found.len());
    };

    let bounds = light.calc_bounds();
    assert!(
        (bounds.left() - rect.left()).abs() <= 1.5,
        "{bounds:?} on {rect:?}"
    );
    assert!(
        (bounds.right() - rect.right()).abs() <= 1.5,
        "{bounds:?} on {rect:?}"
    );
    assert!(bounds.top() >= rect.top() - 1.0, "it stays on the edge");
    assert!(
        bounds.bottom() > rect.top() + f32::from(CARD_RADIUS) - 2.0,
        "it runs down the corners: {bounds:?}"
    );

    let low = rect.top() + f32::from(CARD_RADIUS) - 1.0;
    for vertex in light.vertices.iter().filter(|vertex| vertex.pos.y > low) {
        assert!(
            vertex.color.a() < 40,
            "faded out down the corner: {vertex:?}"
        );
    }
}

#[test]
fn a_light_card_keeps_its_rim_and_no_light() {
    let harness = shown(Theme::Light, plain_card);
    let outline = crate::tokens::current(&harness.ctx).outline;

    assert_eq!(top_lights(&harness).len(), 0);
    assert_ne!(rims_in(&harness, outline).len(), 0);
}

#[test]
fn a_dark_metric_has_the_light() {
    let harness = shown(Theme::Dark, |ui| {
        let text = MetricText {
            name: "Updates pending",
            value: Some("2"),
            detail: "",
        };

        metric(ui, MetricKind::Secondary, text, true).rect
    });

    assert_eq!(top_lights(&harness).len(), 1);
}

#[test]
fn a_dark_menu_has_the_light() {
    let harness = shown(Theme::Dark, |ui| {
        Glass::raised(ui, CornerRadius::same(16), Margin::same(6))
            .show(ui, |ui| ui.label("Copy"))
            .response
            .rect
    });

    assert_eq!(top_lights(&harness).len(), 1);
}

#[test]
fn a_dark_dialog_has_the_light() {
    let harness = shown(Theme::Dark, |ui| {
        ModalWrapper::new(Id::unique("lit"), "Remove all changes").show(ui.ctx(), |ui| {
            ui.label("Remove all 3 queued changes?");
        });

        Rect::NOTHING
    });

    assert_eq!(top_lights(&harness).len(), 1);
}

fn a_link_tile(ui: &mut Ui) -> Rect {
    link_tile(ui, icons::GLOBE, "Forum", "Ask the community", 300.0).rect
}

#[test]
fn a_dark_link_tile_has_the_light_along_its_top() {
    let harness = shown(Theme::Dark, a_link_tile);
    let rect = *harness.state();
    let found = top_lights(&harness);

    let [light] = found.as_slice() else {
        panic!("one light, found {}", found.len());
    };

    let bounds = light.calc_bounds();
    assert!(
        (bounds.left() - rect.left()).abs() <= 1.5 && (bounds.right() - rect.right()).abs() <= 1.5,
        "{bounds:?} on {rect:?}"
    );
}

#[test]
fn a_light_link_tile_has_no_light() {
    assert_eq!(top_lights(&shown(Theme::Light, a_link_tile)).len(), 0);
}
