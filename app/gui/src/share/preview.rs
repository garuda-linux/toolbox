//! The text an upload sends, shown before it goes: how many values were
//! replaced, and a search that marks and jumps to each match, so a name
//! the redaction missed is easy to find.

use egui::{Id, Label, TextEdit, Ui};
use toolbox_core::features::diagnostics::Redacted;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, SearchTexts, highlighted, matches, pill_button, search_field, text_box_at,
};

use super::view::ShareView;
use crate::i18n;

/// The box's least height, for a small window.
const LEAST_HEIGHT: f32 = 160.0;

/// The share of the window's height the box takes, leaving room for the
/// words, the search and the buttons around it.
const WINDOW_SHARE: f32 = 0.5;

const SEARCH_WIDTH: f32 = 280.0;

#[derive(Clone, Debug, Default)]
struct Search {
    query: String,
    /// The match shown, counted from 0.
    current: usize,
}

pub(super) fn show(ui: &mut Ui, view: &ShareView<'_>, shareable: &Redacted) {
    let ctx = ui.ctx().clone();
    ui.add(Label::new(view.text(&ctx, "share-upload-where")).wrap());

    let replaced = match shareable.total() {
        0 => i18n::text(&ctx, "share-upload-replaced-none"),
        count => i18n::text_with(
            &ctx,
            "share-upload-replaced",
            &[("count", &count.to_string())],
        ),
    };

    ui.add(Label::new(replaced).wrap());
    ui.add(Label::new(view.text(&ctx, "share-upload-check")).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);

    let id = Id::unique("toolbox_gui::share::preview").with(view.subject.as_str());
    let mut search: Search = ctx.data(|data| data.get_temp(id).unwrap_or_default());
    let text = shareable.text();
    let found = matches(text, &search.query);
    let jump = search_row(ui, &mut search, found.len());

    let shown = found
        .get(search.current)
        .filter(|_| jump)
        .map(|range| text.get(..range.start).unwrap_or_default().chars().count());

    let name = view.text(&ctx, "share-upload-text");
    let query = search.query.clone();
    let mut layouter = move |inner: &Ui, buffer: &dyn egui::TextBuffer, width: f32| {
        let font = egui::TextStyle::Monospace.resolve(inner.style());
        let mut job = highlighted(inner, buffer.as_str(), &query, font, inner.tokens().text);
        job.wrap.max_width = width;
        inner.fonts_mut(|fonts| fonts.layout_job(job))
    };

    let height = (ctx.content_rect().height() * WINDOW_SHARE).max(LEAST_HEIGHT);
    let mut read_only = text;
    text_box_at(
        ui,
        TextEdit::multiline(&mut read_only)
            .font(egui::TextStyle::Monospace)
            .layouter(&mut layouter),
        &name,
        height,
        shown,
    );

    ctx.data_mut(|data| data.insert_temp(id, search));
}

/// The search field, its count and Next. Returns whether the box should
/// scroll to the current match this frame.
fn search_row(ui: &mut Ui, search: &mut Search, found: usize) -> bool {
    let ctx = ui.ctx().clone();
    let name = i18n::text(&ctx, "share-upload-search");
    let hint = i18n::text(&ctx, "share-upload-search-hint");
    let clear = i18n::text(&ctx, "share-upload-search-clear");
    let next = i18n::text(&ctx, "share-upload-search-next");

    ui.horizontal(|ui| {
        let texts = SearchTexts {
            name: &name,
            hint: &hint,
            clear: &clear,
        };

        let typed = search_field(ui, &mut search.query, texts, SEARCH_WIDTH).changed();
        ui.add_space(ui.tokens().sizes.control_gap);

        if typed {
            search.current = 0;
        }

        if !search.query.trim().is_empty() {
            let place = if found == 0 { 0 } else { search.current + 1 };
            let counted = i18n::text_with(
                &ctx,
                "share-upload-search-count",
                &[
                    ("current", &place.to_string()),
                    ("count", &found.to_string()),
                ],
            );

            ui.label(counted);
        }

        let pressed = ui
            .add_enabled_ui(found > 1, |ui| pill_button(ui, &next, ButtonKind::Soft))
            .inner
            .clicked();

        if pressed {
            search.current = (search.current + 1) % found;
        }

        search.current = search.current.min(found.saturating_sub(1));
        (typed && found > 0) || pressed
    })
    .inner
}
