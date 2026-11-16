//! A lead row's left side: the section's glyph on the page's tile in the
//! picture slot, so the title starts where a catalog row's does, then the
//! title and its description.

use egui::{Context, Label, Ui};
use toolbox_core::features::search::Needle;
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{Tone, dim_picture, highlighted, status_dot};

use crate::i18n;
use crate::pages::rows::picture_slot;
use crate::pages::toggles::{control, paint_glyph};

/// The row of the text `id`, its tile faded when the row cannot change.
pub(super) fn lead(
    ui: &mut Ui,
    (glyph, dim): (Icon, bool),
    (id, query): (&str, &Needle),
    queued: Option<&str>,
) {
    ui.horizontal_top(|ui| {
        let at = picture_slot(ui, true);
        ui.vertical(|ui| text(ui, (id, query), queued));
        dim_picture(ui, at, dim, |ui| paint_glyph(ui, at, glyph));
    });
}

fn text(ui: &mut Ui, (id, query): (&str, &Needle), queued: Option<&str>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    toolbox_gui_kit::widgets::title_line(ui, true, |ui| {
        let title = i18n::text(&ctx, id);
        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        ui.label(highlighted(ui, &title, query.as_str(), strong, tokens.text));
        if let Some(queued) = queued {
            status_dot(ui, &i18n::text(&ctx, queued), Tone::Accent);
        }
    });

    // The cut is the catalog rows' own, and the tooltip keeps the whole.
    let whole = i18n::text(&ctx, &format!("{id}-description")).into_owned();
    let small = fonts::font(ui.style(), Weight::Regular, SMALL);
    let cut = control::cut(highlighted(
        ui,
        &whole,
        query.as_str(),
        small,
        tokens.secondary,
    ));

    ui.add(Label::new(cut).wrap()).on_hover_text(whole);
}

/// Whether `query` is in the title or description of `id`, as the user
/// reads them.
pub(in crate::pages::tweaks) fn matches(ctx: &Context, id: &str, query: &Needle) -> bool {
    query.is_empty()
        || [
            i18n::text(ctx, id),
            i18n::text(ctx, &format!("{id}-description")),
        ]
        .iter()
        .any(|text| query.is_in(text))
}
