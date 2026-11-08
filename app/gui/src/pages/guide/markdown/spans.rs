use egui::{FontFamily, FontId, Label, RichText, TextStyle, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{self, BODY, Weight};
use toolbox_guide::{Link, Span};

use super::Reader;
use crate::pages::guide::Jump;

/// The height of a line of running text, against its font's size. Long
/// text reads easier with more room than a control's label has.
pub(super) const LEADING: f32 = 1.5;

pub(super) fn show(ui: &mut Ui, spans: &[Span], weight: Weight, reader: &mut Reader<'_>) {
    ui.scope(|ui| {
        // A wrapped row starts as tall as a control, which would push the
        // first line away from the second. The row takes its height from
        // the ui it starts in.
        ui.spacing_mut().interact_size.y = 0.0;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for span in spans {
                one(ui, span, weight, reader);
            }
        });
    });
}

fn one(ui: &mut Ui, span: &Span, weight: Weight, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let font = if span.style.code {
        let size = TextStyle::Monospace.resolve(ui.style()).size;
        FontId::new(size, FontFamily::Monospace)
    } else if span.style.strong {
        fonts::font(ui.style(), Weight::SemiBold, BODY)
    } else {
        fonts::font(ui.style(), weight, BODY)
    };

    let colour = if span.link.is_some() {
        tokens.accent_text
    } else {
        tokens.text
    };

    // The pages name the app's ⋯ buttons. Inter has no U+22EF, so the
    // ellipsis it has stands in.
    let words = span.text.replace('⋯', "…");
    let leading = font.size * LEADING;
    let mut text = RichText::new(words)
        .font(font)
        .color(colour)
        .line_height(Some(leading));

    if span.style.emphasis {
        text = text.italics();
    }

    if span.style.code {
        text = text.background_color(tokens.field);
    }

    let Some(link) = &span.link else {
        ui.add(Label::new(text).wrap());
        return;
    };

    if ui.add(egui::Link::new(text)).clicked() {
        reader.jump = Some(match link {
            Link::Page { file, anchor } => Jump::Page {
                file: file.clone(),
                anchor: anchor.clone(),
            },
            Link::Anchor(anchor) => Jump::Anchor(anchor.clone()),
            Link::Web(address) => Jump::Web(address.clone()),
        });
    }
}
