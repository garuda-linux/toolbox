//! The report with the programs' colours, in the theme's ANSI palette as
//! the Console has it, under a header line: the report's name, when it
//! came in, its line count, then the search and the report's buttons. A
//! report holds up to megabytes, so only the rows in view are laid out.
//! Long lines wrap at the edge.

use egui::{Label, ScrollArea, TextStyle, Ui};
use toolbox_calendar::Utc;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::terminal_colors::Ansi;
use toolbox_gui_kit::widgets::{SearchTexts, padded_card, pill_width, rich, search_field};
use toolbox_gui_kit::{UiExt as _, palette};
use toolbox_pty::Run;
use toolbox_services::diagnostics::view::{self, Row};

use super::DiagnosticsPage;
use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::pages::PageContext;
use crate::share::rows::{character_count, layout};

/// The view never shrinks below this, in points, even at the bottom of a
/// short window.
const MIN_HEIGHT: f32 = 240.0;

const SEARCH_WIDTH: f32 = 200.0;

/// The rows of the last frame, kept while the report, the search and the
/// width do not change: finding them reads every line.
#[derive(Debug, Default)]
pub struct Shown {
    key: (usize, usize, String, Option<usize>),
    rows: Vec<Row>,
}

impl DiagnosticsPage {
    pub(super) fn report(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let lines = context.service::<Diagnostics>().lines();
        let palette = palette::current(ui.ctx());
        let ansi = Ansi::of(&palette, ui.visuals().dark_mode);
        let font = TextStyle::Monospace.resolve(ui.style());
        let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font));

        // The box takes exactly the room left in view, so the page around
        // it never scrolls and the report's own bar is the only one.
        let sizes = ui.tokens().sizes;
        let around = 2.0_f32.mul_add(
            f32::from(sizes.card_padding.top.max(sizes.card_padding.bottom)),
            sizes.card_gap,
        );

        let room = ui.clip_rect().bottom() - ui.cursor().top() - around;
        let label = i18n::text(ui.ctx(), "diagnostics-report-label");

        padded_card(ui, |ui| {
            let top = ui.cursor().top();
            self.header(ui, context);
            ui.add_space(sizes.caption_gap);
            let height = (room - (ui.cursor().top() - top)).max(MIN_HEIGHT);

            // A monospaced font fits this many characters in each row.
            // Lines always wrap, as in the upload preview, so the report
            // never scrolls sideways.
            let glyph = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'M'));
            let columns = character_count(ui.available_width() / glyph.max(1.0));
            let rows = self.rows(lines, Some(columns));

            toolbox_gui_kit::theme::show_scroll_bars(ui.style_mut());

            // show_rows counts the spacing of this ui between rows.
            ui.spacing_mut().item_spacing.y = 0.0;
            ScrollArea::vertical()
                .id_salt("diagnostics-report")
                .auto_shrink([false, true])
                .max_height(height)
                .show_rows(ui, row_height, rows.len(), |ui, shown| {
                    for row in rows.get(shown).unwrap_or_default() {
                        let runs = lines.get(row.line).map_or(&[][..], Vec::as_slice);
                        let job = layout(runs, *row, &font, (&ansi, &palette));
                        ui.add(Label::new(job).extend());
                    }
                });
        })
        .response
        .widget_info(|| egui::WidgetInfo::labeled(egui::accesskit::Role::Document, true, &*label));
    }

    /// The name of the report held, which the rows above may no longer
    /// run, when it came in and how many lines it has, then the controls.
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let tokens = ui.tokens();
        let diagnostics = context.service::<Diagnostics>();
        let page = diagnostics.page();
        let caption = super::name(&ctx, &page.selection);
        let meta = meta(&ctx, page.collected, diagnostics.lines().len());

        ui.horizontal_wrapped(|ui| {
            ui.label(rich(ui, &caption, Weight::SemiBold, BODY, tokens.text));
            ui.label(rich(ui, &meta, Weight::Regular, SMALL, tokens.secondary));

            // The controls move under the title together rather than one
            // by one, so the search stays beside the buttons it filters for.
            let controls = tokens.sizes.text_to_control + controls_width(ui);
            if ui.available_size_before_wrap().x < controls {
                ui.end_row();
            } else {
                ui.add_space(tokens.sizes.text_to_control);
            }

            let name = i18n::text(&ctx, "diagnostics-search");
            let clear = i18n::text(&ctx, "diagnostics-search-clear");
            let texts = SearchTexts {
                name: &name,
                hint: &name,
                clear: &clear,
            };

            search_field(ui, texts.name, &mut self.query, texts, SEARCH_WIDTH);
            self.report_buttons(ui, context);
        });
    }

    /// The rows of `lines` for the search and `columns`, found again only
    /// when one of them changed.
    fn rows(&mut self, lines: &[Vec<Run>], columns: Option<usize>) -> &[Row] {
        let key = (
            lines.as_ptr().addr(),
            lines.len(),
            self.query.clone(),
            columns,
        );

        if self.shown.key != key {
            self.shown.rows = view::rows(lines, &self.query, columns);
            self.shown.key = key;
        }

        &self.shown.rows
    }
}

/// "· 14:05 UTC · 312 lines", the time left out while unknown.
fn meta(ctx: &egui::Context, collected: Option<u64>, line_count: usize) -> String {
    let count = line_count.to_string();
    let lines = i18n::text_with(ctx, "diagnostics-lines", &[("count", &count)]);
    let Some(seconds) = collected else {
        return i18n::text_with(ctx, "diagnostics-collected-unknown", &[("lines", &lines)])
            .into_owned();
    };

    let time = Utc::from_unix(seconds);
    let clock = format!("{:02}:{:02}", time.hour, time.minute);
    i18n::text_with(
        ctx,
        "diagnostics-collected",
        &[("time", &clock), ("lines", &lines)],
    )
    .into_owned()
}

/// The search and the report's two buttons side by side.
fn controls_width(ui: &Ui) -> f32 {
    let ctx = ui.ctx();
    let gap = ui.spacing().item_spacing.x;
    ["diagnostics-copy", "diagnostics-share"]
        .iter()
        .map(|id| gap + pill_width(ui, &i18n::text(ctx, id)))
        .sum::<f32>()
        + SEARCH_WIDTH
}
