use egui::accesskit::{Live, Role};
use egui::{
    CornerRadius, Frame, Label, Margin, Response, Sense, TextStyle, TextWrapMode, Ui, WidgetText,
    pos2, vec2,
};

use super::buttons::{pill_button, rimmed_pill_button};
use super::icon_tile::TileSize;
use super::status_tile::paint_status_tile;
use super::{ButtonKind, Tone, rich, split_row};
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::theme::{CARD_RADIUS, PILL_HEIGHT};
use crate::tokens::{self, Tokens};

impl Tone {
    pub(super) fn banner(self, tokens: &Tokens) -> (egui::Color32, egui::Color32, Icon) {
        let (tile, icon) = self.status_mark(tokens);
        (tile.fill, tile.ink, icon)
    }

    fn wash(self, tokens: &Tokens) -> egui::Color32 {
        match self {
            Self::Accent => tokens.accent_soft,
            Self::Neutral => tokens.field,
            Self::Warning => tokens.banner_wash(tokens.warning),
            Self::Success => tokens.banner_wash(tokens.success),
            Self::Error => tokens.banner_wash(tokens.error),
        }
    }
}

/// right, in their order. Returns the index of the one clicked.
pub fn banner(ui: &mut Ui, tone: Tone, text: &str, actions: &[&str]) -> Option<usize> {
    let tokens = tokens::current(ui.ctx());
    let wash = tone.wash(&tokens);

    Frame::new()
        .fill(wash)
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let gap = ui.spacing().item_spacing.x;
            let button_width: f32 = actions
                .iter()
                .map(|action| super::buttons::pill_width(ui, action) + gap)
                .sum();

            split_row(
                ui,
                button_width,
                |ui| {
                    let said = icon_and_text(ui, tone, text);
                    announce(ui, &said, tone, text);
                },
                |ui| {
                    // A banner without buttons keeps their height, so one line
                    // reads the same on every page.
                    if actions.is_empty() {
                        ui.allocate_space(vec2(0.0, PILL_HEIGHT));
                    }

                    // Only the neutral wash is a Soft button's own fill.
                    let rimmed = tone == Tone::Neutral;
                    let mut clicked = None;
                    for (index, action) in actions.iter().enumerate() {
                        let response = if rimmed {
                            rimmed_pill_button(ui, action, ButtonKind::Soft)
                        } else {
                            pill_button(ui, action, ButtonKind::Soft)
                        };

                        if response.clicked() {
                            clicked = Some(index);
                        }
                    }

                    clicked
                },
            )
            .1
        })
        .inner
}

/// The mark of `tone` centred on the whole of `text`, however many lines
/// it wraps to. A layout would centre it on the first line only, as it
/// places the mark before the text is measured. Returns the text's
/// response.
fn icon_and_text(ui: &mut Ui, tone: Tone, text: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    let edge = TileSize::Line.edge(ui);
    let gap = tokens.sizes.picture_gap;
    let wrap = (ui.available_width() - edge - gap).max(0.0);
    let words = rich(ui, text, Weight::Medium, BODY, tokens.text);
    let galley =
        WidgetText::from(words).into_galley(ui, Some(TextWrapMode::Wrap), wrap, TextStyle::Body);

    let height = galley.size().y.max(edge);
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let centre = row.center().y;
    paint_status_tile(
        ui,
        pos2(row.left() + edge / 2.0, centre),
        TileSize::Line,
        tone,
    );

    let at = pos2(row.left() + edge + gap, centre - galley.size().y / 2.0);
    ui.put(
        egui::Rect::from_min_size(at, galley.size()),
        Label::new(galley),
    )
}

/// A screen reader reads a banner when it comes or changes, an error one
/// at once, as an alert.
pub(super) fn announce(ui: &Ui, said: &Response, tone: Tone, text: &str) {
    let alert = tone == Tone::Error;
    ui.ctx().accesskit_node_builder(said.id, |node| {
        if alert {
            node.set_role(Role::Alert);
            node.set_label(text);
            node.set_live(Live::Assertive);
        } else {
            node.set_live(Live::Polite);
        }
    });
}
