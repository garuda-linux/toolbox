use egui::accesskit::Role;
use egui::{CornerRadius, Frame, Margin, Sense, Ui, WidgetInfo};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::theme::CARD_RADIUS;
use toolbox_gui_kit::widgets::{empty_state, focus_ring, rich};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_guide::Hit;

use super::Jump;
use crate::i18n;

pub(super) fn show(ui: &mut Ui, hits: &[Hit], query: &str) -> Option<Jump> {
    let ctx = ui.ctx().clone();
    if hits.is_empty() {
        let title = i18n::text_with(&ctx, "guide-no-hits", &[("search", query)]);
        let body = i18n::text(&ctx, "guide-no-hits-body");
        _ = empty_state(ui, icons::SYSTEM_SEARCH, &title, &body);
        return None;
    }

    let tokens = ui.tokens();
    let count = hits.len().to_string();
    let found = i18n::text_with(&ctx, "guide-hits", &[("count", &count)]);
    ui.label(rich(ui, &found, Weight::Regular, SMALL, tokens.secondary));
    ui.add_space(tokens.sizes.tight_gap);

    let mut jump = None;
    for (at, hit) in hits.iter().enumerate() {
        ui.push_id(at, |ui| {
            if place(ui, hit) {
                jump = Some(Jump::Page {
                    file: hit.file.clone(),
                    anchor: hit.anchor.clone(),
                });
            }
        });
        ui.add_space(tokens.sizes.tight_gap);
    }

    jump
}

/// Returns whether it was clicked.
fn place(ui: &mut Ui, hit: &Hit) -> bool {
    let tokens = ui.tokens();
    let title = hit.heading.as_ref().map_or_else(
        || hit.page_title.clone(),
        |heading| format!("{} › {heading}", hit.page_title),
    );

    let shown = Frame::new()
        .fill(tokens.card)
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(rich(ui, &title, Weight::SemiBold, BODY, tokens.accent_text));
            ui.label(rich(
                ui,
                &hit.snippet,
                Weight::Regular,
                SMALL,
                tokens.secondary,
            ));
        });

    let response = ui.interact(
        shown.response.rect,
        ui.scope_id().with("hit"),
        Sense::click(),
    );

    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), &title));
    if response.hovered() {
        ui.painter().rect_stroke(
            shown.response.rect,
            CornerRadius::same(CARD_RADIUS),
            egui::Stroke::new(1.0, tokens.outline),
            egui::StrokeKind::Inside,
        );
    }

    focus_ring(ui, &response, CARD_RADIUS);
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}
