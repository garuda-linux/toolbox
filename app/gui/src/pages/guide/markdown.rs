//! A guide page's blocks drawn with the app's own look: its fonts, its
//! colours and its notes. Links and heading positions go back to the
//! reader.

mod spans;
mod table;

use egui::{
    Align, Color32, CornerRadius, Frame, Label, Layout, Margin, RichText, ScrollArea, Sense,
    TextStyle, Ui, vec2,
};
use toolbox_gui_kit::fonts::{self, BODY, SECTION, Weight};
use toolbox_gui_kit::theme::{CARD_RADIUS, RADIUS};
use toolbox_gui_kit::widgets::{
    TileSize, Tone, icon_button, icon_button_width, paint_status_tile, rich,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_guide::{Block, Callout, CalloutKind, Code, Heading, List};

use super::Jump;
use crate::i18n;

/// The room for a list item's bullet or number.
const MARKER_WIDTH: f32 = 26.0;
const PAGE_TITLE: f32 = 1.8;
const MINOR: f32 = 1.15;

#[derive(Debug)]
pub(super) struct Reader<'target> {
    pub target: Option<&'target str>,
    pub jump: Option<Jump>,
    pub headings: Vec<(String, f32)>,
    /// A code block went to the clipboard, which the page tells the user.
    pub copied: bool,
    /// The source and description of the picture clicked to see it large.
    pub enlarged: Option<(String, String)>,
    /// The spans drawn now sit in a table cell.
    pub in_table: bool,
}

impl<'target> Reader<'target> {
    pub(super) const fn new(target: Option<&'target str>) -> Self {
        Self {
            target,
            jump: None,
            headings: Vec::new(),
            copied: false,
            enlarged: None,
            in_table: false,
        }
    }
}

pub(super) fn blocks(ui: &mut Ui, blocks: &[Block], reader: &mut Reader<'_>) {
    let gap = ui.tokens().sizes.card_gap;
    for (at, block) in blocks.iter().enumerate() {
        ui.push_id(at, |ui| one(ui, block, reader));
        ui.add_space(gap);
    }
}

fn one(ui: &mut Ui, block: &Block, reader: &mut Reader<'_>) {
    match block {
        Block::Heading(heading) => title(ui, heading, reader),
        Block::Paragraph(text) => spans::show(ui, text, Weight::Regular, reader),
        Block::List(list) => items(ui, list, reader),
        Block::Table(table) => table::show(ui, table, reader),
        Block::Code(code) => code_block(ui, code, reader),
        Block::Callout(callout) => note(ui, callout, reader),
        Block::Image(image) => super::picture::show(ui, image, reader),
        Block::Rule => _ = ui.separator(),
    }
}

fn title(ui: &mut Ui, heading: &Heading, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let (weight, scale, room) = match heading.level {
        1 => (Weight::Bold, PAGE_TITLE, 0.0),
        2 => (Weight::SemiBold, SECTION, tokens.sizes.block_gap),
        _ => (Weight::SemiBold, MINOR, tokens.sizes.tight_gap),
    };

    ui.add_space(room);

    let text: String = heading
        .spans
        .iter()
        .map(|span| span.text.as_str())
        .collect();

    let shown = ui.add(Label::new(rich(ui, &text, weight, scale, tokens.text)).wrap());
    let level = usize::from(heading.level);
    ui.ctx().accesskit_node_builder(shown.id, |node| {
        node.set_role(egui::accesskit::Role::Heading);
        node.set_level(level);
        node.set_label(text.as_str());
    });

    if reader.target == Some(heading.anchor.as_str()) {
        shown.scroll_to_me(Some(Align::TOP));
    }

    if heading.level > 1 {
        reader
            .headings
            .push((heading.anchor.clone(), shown.rect.top()));
    }
}

fn items(ui: &mut Ui, list: &List, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    for (at, item) in list.items.iter().enumerate() {
        let marker = list.start.map_or_else(
            || "•".to_owned(),
            |first| {
                let number = u64::try_from(at).map_or(first, |at| first.saturating_add(at));
                format!("{number}.")
            },
        );

        ui.push_id(at, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let size = TextStyle::Body.resolve(ui.style()).size;
                let text = rich(ui, &marker, Weight::Regular, BODY, tokens.secondary)
                    .line_height(Some(size * spans::LEADING));

                ui.add_sized(vec2(MARKER_WIDTH, 0.0), Label::new(text));
                ui.vertical(|ui| {
                    for (part, block) in item.iter().enumerate() {
                        ui.push_id(part, |ui| one(ui, block, reader));
                    }
                });
            });
        });
    }
}

/// The copy button sits beside the code rather than over it, so it never
/// hides the end of a long line.
fn code_block(ui: &mut Ui, code: &Code, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let alt = i18n::text(ui.ctx(), "guide-copy-code");
    Frame::new()
        .fill(tokens.field)
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let gap = ui.spacing().item_spacing.x;
                let room = ui.available_width() - icon_button_width(ui) - gap;
                ui.allocate_ui(vec2(room.max(0.0), 0.0), |ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        let text = RichText::new(&code.text).monospace().color(tokens.text);
                        ui.add(Label::new(text).extend());
                    });
                });

                if icon_button(ui, icons::EDIT_COPY, &alt).clicked() {
                    ui.ctx().copy_text(code.text.clone());
                    reader.copied = true;
                }
            });
        });
}

/// A note or warning in the look of a banner, a quote on a quiet fill.
fn note(ui: &mut Ui, callout: &Callout, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let ctx = ui.ctx().clone();
    let (fill, mark) = match callout.kind {
        CalloutKind::Note => (tokens.accent_soft, Some((Tone::Accent, "guide-note"))),
        CalloutKind::Warning => (
            tokens.banner_wash(tokens.warning),
            Some((Tone::Warning, "guide-warning")),
        ),
        CalloutKind::Quote => (tokens.field, None),
    };

    Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                if let Some((tone, _)) = mark {
                    mark_tile(ui, tone);
                }

                ui.vertical(|ui| {
                    if let Some((_, label)) = mark {
                        let text = i18n::text(&ctx, label);
                        ui.label(rich(ui, &text, Weight::SemiBold, BODY, tokens.text));
                    }

                    for (at, block) in callout.blocks.iter().enumerate() {
                        ui.push_id(at, |ui| one(ui, block, reader));
                    }
                });
            });
        });
}

/// The status mark of `tone` beside a callout's label. It starts at the
/// top of the label's capitals, as a row's picture starts at its
/// title's, rather than at the line box above them.
fn mark_tile(ui: &mut Ui, tone: Tone) {
    let edge = TileSize::Row.edge(ui);
    let capital = ui.painter().layout_no_wrap(
        "H".to_owned(),
        fonts::font(ui.style(), Weight::SemiBold, BODY),
        Color32::PLACEHOLDER,
    );

    let top = capital.mesh_bounds.top().max(0.0);
    let (room, _) = ui.allocate_exact_size(vec2(edge, top + edge), Sense::hover());
    let centre = egui::pos2(room.center().x, room.top() + top + edge / 2.0);
    paint_status_tile(ui, centre, TileSize::Row, tone);
}
