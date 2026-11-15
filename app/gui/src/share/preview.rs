//! The text an upload sends, shown before it goes: how many values were
//! replaced, and a search that marks and jumps to each match, so a name
//! the redaction missed is easy to find.
//!
//! A report holds up to megabytes, so the text shows in rows as the
//! Diagnostics report does, and its rows and matches are found again only
//! when the text, the search or the width changes.

mod found;

use egui::accesskit::Role;
use egui::{Frame, Id, Label, Margin, ScrollArea, Sense, TextStyle, Ui};
use toolbox_core::features::diagnostics::Redacted;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::theme::CONTROL_RADIUS;
use toolbox_gui_kit::widgets::{ButtonKind, SearchTexts, highlighted, pill_button, search_field};

use self::found::Found;
use super::rows::{character_count, piece_text, row_top};
use super::view::{ShareView, Subject};
use crate::i18n;

/// The box's least height, for a small window.
const LEAST_HEIGHT: f32 = 160.0;

/// The share of the window's height the box takes, leaving room for the
/// words, the search and the buttons around it.
const WINDOW_SHARE: f32 = 0.5;

const SEARCH_WIDTH: f32 = 280.0;

/// Room around the text inside its box, as a text box has.
const PADDING_X: i8 = 14;
const PADDING_Y: i8 = 10;

#[derive(Clone, Debug, Default)]
struct Search {
    query: String,
    /// The match shown, counted from 0.
    current: usize,
    found: Found,
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

    let id = search_id(view.subject);

    // Taken out and put back, as the rows of a long text are not worth a
    // copy every frame.
    let mut search =
        ctx.data_mut(|data| std::mem::take(data.get_temp_mut_or_default::<Search>(id)));

    let font = TextStyle::Monospace.resolve(ui.style());
    let glyph = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'M'));
    let room = 2.0_f32.mul_add(-f32::from(PADDING_X), ui.available_width());
    let columns = Some(character_count(room / glyph.max(1.0)));
    let text = shareable.text();
    search.found.update(text, &search.query, columns);

    let jump = search_row(ui, &mut search);

    // A typed query is searched before its first match is brought in.
    search.found.update(text, &search.query, columns);
    let shown = search.found.row_of(search.current).filter(|_| jump);

    let name = view.text(&ctx, "share-upload-text");
    let height = (ctx.content_rect().height() * WINDOW_SHARE).max(LEAST_HEIGHT);
    text_rows(ui, (view.subject, &name), &search, (height, shown), text);
    ctx.data_mut(|data| data.insert_temp(id, search));
}

/// The search ends with its question, so the next one starts blank.
pub(super) fn forget(ctx: &egui::Context, subject: Subject) {
    ctx.data_mut(|data| data.remove::<Search>(search_id(subject)));
}

fn search_id(subject: Subject) -> Id {
    Id::unique("toolbox_gui::share::preview").with(subject.as_str())
}

/// The rows in view in a filled box, `height` at most, the matches marked.
/// `shown` is the row brought to the middle this frame. Screen readers get
/// the whole text, as from a read-only text box.
fn text_rows(
    ui: &mut Ui,
    (subject, name): (Subject, &str),
    search: &Search,
    (height, shown): (f32, Option<usize>),
    text: &str,
) {
    let tokens = ui.tokens();
    let font = TextStyle::Monospace.resolve(ui.style());
    let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font));
    let found = &search.found;

    let framed = Frame::new()
        .fill(tokens.field)
        .corner_radius(CONTROL_RADIUS)
        .inner_margin(Margin::symmetric(PADDING_X, PADDING_Y))
        .show(ui, |ui| {
            // show_rows strides by the row height plus this spacing, and
            // row_top counts the row height alone.
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut area = ScrollArea::vertical()
                .id_salt(("share-preview", subject.as_str()))
                .auto_shrink([false, true])
                .max_height(height);

            if let Some(row) = shown {
                // An offset past the last row would spring back over frames.
                let last = row_top(found.rows().len(), row_height) - height;
                let top = row_top(row, row_height) - height / 2.0;
                area = area.vertical_scroll_offset(top.min(last).max(0.0));
            }

            area.show_rows(ui, row_height, found.rows().len(), |ui, rows| {
                for row in found.rows().get(rows).unwrap_or_default() {
                    let piece = piece_text(found.line(row.line), *row);
                    let job = highlighted(ui, &piece, &search.query, font.clone(), tokens.text);
                    ui.add(Label::new(job).extend());
                }
            });
        });

    let named = Id::unique("toolbox_gui::share::preview_text").with(subject.as_str());
    let response = ui.interact(framed.response.rect, named, Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(Role::MultilineTextInput, true, name));
    let value = text.to_owned();
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_value(value);
        node.set_read_only();
    });
}

/// The search field, its count and Next. Returns whether the box should
/// scroll to the current match this frame. The count of a query typed
/// this frame shows from the next one.
fn search_row(ui: &mut Ui, search: &mut Search) -> bool {
    let ctx = ui.ctx().clone();
    let name = i18n::text(&ctx, "share-upload-search");
    let hint = i18n::text(&ctx, "share-upload-search-hint");
    let clear = i18n::text(&ctx, "share-upload-search-clear");
    let next = i18n::text(&ctx, "share-upload-search-next");
    let found = search.found.count();

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
        typed || pressed
    })
    .inner
}
