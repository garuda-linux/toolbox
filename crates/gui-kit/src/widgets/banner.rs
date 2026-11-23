use egui::accesskit::{Live, Role};
use egui::{
    Color32, CornerRadius, Frame, Label, Margin, Response, Sense, Stroke, TextStyle, TextWrapMode,
    Ui, WidgetText, pos2, vec2,
};

use super::buttons::pill_button;
use super::icon_tile::TileSize;
use super::status_tile::paint_status_tile;
use super::{ButtonKind, Choice, Tone, rich, split_row};
use crate::fonts::{BODY, Weight};
use crate::theme::{CARD_RADIUS, PILL_HEIGHT};
use crate::tokens::{self, Tokens};

/// The share of the type colour in a banner's fill, Kirigami's `opacity: 0.20`.
const WASH: f32 = 0.2;
/// Kirigami pads a banner by `smallSpacing` from its 1 px frame.
const PADDING: i8 = 5;

/// Kirigami's `InlineMessage` look: the type colour as a 1 px frame, and
/// the card with a fifth of that colour over it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BannerLook {
    pub frame: Color32,
    pub fill: Color32,
}

impl Tone {
    pub(super) fn banner_look(self, tokens: &Tokens) -> BannerLook {
        let frame = self.banner_colour(tokens);
        BannerLook {
            frame,
            fill: tokens::mix(tokens.card, frame, WASH),
        }
    }

    /// Information takes the view's active text colour, Kirigami's
    /// `activeTextColor`. A neutral note takes it too, as KDE has no third
    /// colour for one.
    const fn banner_colour(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Accent | Self::Neutral => tokens.roles.active,
            Self::Warning => tokens.warning,
            Self::Success => tokens.success,
            Self::Error => tokens.error,
        }
    }
}

/// `text` with its status mark at the left and `actions` as Breeze buttons at
/// the right, in their order. Returns the index of the one clicked.
pub fn banner(ui: &mut Ui, tone: Tone, text: &str, actions: &[&str]) -> Choice<usize> {
    let tokens = tokens::current(ui.ctx());
    let look = tone.banner_look(&tokens);

    let framed = Frame::new()
        .fill(look.fill)
        .stroke(Stroke::new(1.0, look.frame))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::same(PADDING))
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

                    let mut clicked = None;
                    for (index, action) in actions.iter().enumerate() {
                        let response = pill_button(ui, action, ButtonKind::Soft);

                        if response.clicked() {
                            clicked = Some(index);
                        }
                    }

                    clicked
                },
            )
            .1
        });

    Choice {
        response: framed.response,
        value: framed.inner,
    }
}

/// The mark of `tone` centred on the whole of `text`, however many lines
/// it wraps to. A layout would centre it on the first line only, as it
/// places the mark before the text is measured. Returns the text's
/// response.
fn icon_and_text(ui: &mut Ui, tone: Tone, text: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    let edge = TileSize::Line.edge();
    let gap = tokens.sizes.tight_gap;
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
