//! A number the page is about, with its name and a line about it. The
//! primary one takes the accent tint and its number in full accent, the
//! others stay neutral.

use egui::{CursorIcon, Frame, Margin, Response, Role, Sense, Ui, UiBuilder, WidgetInfo};

use super::card::light_rim;
use super::{focus_ring, rich, text_line_height};
use crate::fonts::{SECTION, SMALL, TITLE, Weight};
use crate::theme::CARD_RADIUS;
use crate::tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricKind {
    /// The one number the page is about.
    Primary,
    /// A number beside it that supports it.
    Secondary,
}

#[derive(Clone, Copy, Debug)]
pub struct MetricText<'a> {
    pub name: &'a str,
    /// `None` while not known: a dash in the secondary colour.
    pub value: Option<&'a str>,
    pub detail: &'a str,
}

/// A card with the name, value and detail of `text`.
///
/// A button named by the name and the value, or the detail when the value
/// is not known. A click leads to what the number counts. With `leads`
/// false there is nothing to lead to, so it is a plain figure: no focus
/// stop, no hand cursor. Every metric stands as tall as a primary one, so
/// a row of them lines up.
pub fn metric(ui: &mut Ui, kind: MetricKind, text: MetricText<'_>, leads: bool) -> Response {
    let MetricText {
        name,
        value,
        detail,
    } = text;

    let tokens = tokens::current(ui.ctx());

    // An unknown figure is no highlight: it takes the neutral card, and
    // its reason stands where the number would.
    let (fill, number, size, muted) = match (kind, value.is_some()) {
        (MetricKind::Primary, true) => (tokens.accent_tint, tokens.on_tint, TITLE, tokens.text),
        (MetricKind::Primary, false) => (tokens.card, tokens.text, TITLE, tokens.secondary),
        (MetricKind::Secondary, _) => (tokens.card, tokens.text, SECTION, tokens.secondary),
    };

    let spoken = value.unwrap_or(detail);

    let height = primary_height(ui);
    let rim = light_rim(ui, tokens.outline);

    let sense = if leads {
        Sense::click()
    } else {
        Sense::hover()
    };

    let response = ui
        .scope_builder(UiBuilder::new().sense(sense), |ui| {
            Frame::new()
                .fill(fill)
                .stroke(rim)
                .corner_radius(CARD_RADIUS)
                .inner_margin(Margin::same(18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(height);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
                        ui.label(rich(ui, name, Weight::Medium, SMALL, muted));

                        value_line(
                            ui,
                            value.map_or((detail, false), |value| (value, true)),
                            (size, number),
                        );
                        let below = if value.is_some() { detail } else { "" };
                        ui.label(rich(ui, below, Weight::Regular, SMALL, muted));
                    });
                });
        })
        .response;

    let role = if leads { Role::Button } else { Role::Label };
    response.widget_info(|| WidgetInfo::labeled(role, true, format!("{name}: {spoken}")));
    if !leads {
        return response;
    }

    focus_ring(ui, &response, CARD_RADIUS);
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    response
}

/// The value, or the reason it is not known in its place. Every value line
/// takes the primary's height, so the details of a row of figures share one
/// baseline.
fn value_line(ui: &mut Ui, (text, known): (&str, bool), (size, colour): (f32, egui::Color32)) {
    let line = text_line_height(ui, Weight::Bold, TITLE);
    let secondary = tokens::current(ui.ctx()).secondary;
    let layout = egui::Layout::bottom_up(egui::Align::Min);
    ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), line), layout, |ui| {
        ui.set_min_height(line);
        if known {
            ui.label(rich(ui, text, Weight::Bold, size, colour));
        } else {
            ui.label(rich(ui, text, Weight::Medium, SMALL, secondary));
        }
    });
}

/// The three lines of a primary metric with their gaps, which every
/// metric takes.
fn primary_height(ui: &Ui) -> f32 {
    let tokens = tokens::current(ui.ctx());
    let lines = text_line_height(ui, Weight::Medium, SMALL)
        + text_line_height(ui, Weight::Bold, TITLE)
        + text_line_height(ui, Weight::Regular, SMALL);

    2.0_f32.mul_add(tokens.sizes.tight_gap, lines)
}
