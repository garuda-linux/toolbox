use egui::epaint::RectShape;
use egui::{Color32, CornerRadius, Id, Margin, Rect, Shape, Theme, Ui};
use egui_kittest::Harness;

use super::super::{MetricKind, MetricText, ModalWrapper, card, link_tile, metric};
use crate::frost::Glass;
use crate::icons;
use crate::testing::{harness, rims_in, top_lights};

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
fn a_dark_card_keeps_its_rim_and_no_light() {
    let harness = shown(Theme::Dark, plain_card);
    let outline = crate::tokens::current(&harness.ctx).card_outline;

    assert_eq!(top_lights(&harness).len(), 0);
    assert_ne!(rims_in(&harness, outline).len(), 0);
}

#[test]
fn a_light_card_keeps_its_rim_and_no_light() {
    let harness = shown(Theme::Light, plain_card);
    let outline = crate::tokens::current(&harness.ctx).card_outline;

    assert_eq!(top_lights(&harness).len(), 0);
    assert_ne!(rims_in(&harness, outline).len(), 0);
}

/// The blur widths and rectangles of the shadows the last frame painted in `colour`.
fn shadows_in(harness: &Harness<'_, Rect>, colour: Color32) -> Vec<(Rect, f32)> {
    harness
        .output()
        .shapes
        .iter()
        .flat_map(|clipped| rects_within(&clipped.shape))
        .filter(|rect| rect.fill == colour && rect.blur_width > 0.0)
        .map(|rect| (rect.rect, rect.blur_width))
        .collect()
}

/// The rectangles of `shape`, looking inside the groups a shadow is painted in.
pub(super) fn rects_within(shape: &Shape) -> Vec<RectShape> {
    if let Shape::Rect(rect) = shape {
        return vec![rect.clone()];
    }

    if let Shape::Vec(shapes) = shape {
        return shapes.iter().flat_map(rects_within).collect();
    }

    Vec::new()
}

/// Darkly's group box casts the control shadow, blur 4 px as Qt draws it
/// and 8 px in egui (twice Qt's radius), 1 px down (darklyhelper.cpp:1229
/// and :592), and nothing else.
#[test]
fn a_card_casts_darklys_group_box_shadow_one_pixel_down() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = shown(theme, plain_card);
        let card = *harness.state();
        let shadow = crate::tokens::current(&harness.ctx).control_shadow;
        let shadows = shadows_in(&harness, shadow);

        assert_eq!(shadows.len(), 1, "{theme:?}: one shadow");
        let (rect, blur) = shadows[0];
        assert!((blur - 8.0).abs() < f32::EPSILON, "{theme:?}: blur {blur}");
        assert!(
            (rect.center().y - card.center().y - 1.0).abs() < 0.01,
            "{theme:?}: the shadow is not one pixel down"
        );
    }
}

#[test]
fn a_dark_metric_has_no_light() {
    let harness = shown(Theme::Dark, |ui| {
        let text = MetricText {
            name: "Updates pending",
            value: Some("2"),
            detail: "",
        };

        metric(ui, MetricKind::Secondary, text, true).rect
    });

    assert_eq!(top_lights(&harness).len(), 0);
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
fn a_dark_link_tile_is_a_frame_without_a_light() {
    let harness = shown(Theme::Dark, a_link_tile);
    let outline = crate::tokens::current(&harness.ctx).card_outline;

    assert_eq!(top_lights(&harness).len(), 0);
    assert_ne!(rims_in(&harness, outline).len(), 0);
}

#[test]
fn a_light_link_tile_has_no_light() {
    assert_eq!(top_lights(&shown(Theme::Light, a_link_tile)).len(), 0);
}
