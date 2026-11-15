//! The report with the programs' colours, in the theme's ANSI palette as
//! the Console has it, under a header line: the report's name, when it
//! came in, its line count, then Wrap, the search and the report's
//! buttons. A report holds up to megabytes, so only the rows in view are
//! laid out. Long lines scroll sideways as in a terminal, or wrap.

use egui::text::{LayoutJob, TextFormat};
use egui::{FontId, Label, ScrollArea, TextStyle, Ui};
use toolbox_calendar::Utc;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::palette::{self, Palette};
use toolbox_gui_kit::terminal_colors::{Ansi, resolve};
use toolbox_gui_kit::widgets::{SearchTexts, chip, rich, search_field};
use toolbox_pty::Run;
use toolbox_services::diagnostics::view::{self, Row};

use super::DiagnosticsPage;
use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::pages::PageContext;
use crate::share::view::padded_card;

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
            let glyph = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'M'));
            let wrap = self.wrap;
            let columns = wrap.then(|| character_count(ui.available_width() / glyph.max(1.0)));
            let rows = self.rows(lines, columns);

            // Long lines run past the edge, a bar at rest says so.
            toolbox_gui_kit::theme::show_scroll_bars(ui.style_mut());
            let area = if wrap {
                ScrollArea::vertical()
            } else {
                ScrollArea::both()
            };

            area.id_salt("diagnostics-report")
                .auto_shrink([false, true])
                .max_height(height)
                .show_rows(ui, row_height, rows.len(), |ui, shown| {
                    ui.spacing_mut().item_spacing.y = 0.0;
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
            ui.add_space(tokens.sizes.text_to_control);

            let wrap = i18n::text(&ctx, "diagnostics-wrap");
            if chip(ui, &wrap, None, self.wrap).clicked() {
                self.wrap ^= true;
            }

            let name = i18n::text(&ctx, "diagnostics-search");
            let clear = i18n::text(&ctx, "diagnostics-search-clear");
            let texts = SearchTexts {
                name: &name,
                hint: &name,
                clear: &clear,
            };

            search_field(ui, &mut self.query, texts, SEARCH_WIDTH);
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

/// The whole characters that fit `room`, at least one.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a row holds a few hundred characters at most"
)]
const fn character_count(room: f32) -> usize {
    let whole = room as usize;
    if whole == 0 { 1 } else { whole }
}

/// The piece of one line `row` holds as one job, each run in its colour.
fn layout(runs: &[Run], row: Row, font: &FontId, (ansi, palette): (&Ansi, &Palette)) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (text, run) in view::piece(runs, row.start, row.end) {
        let color = resolve(run.style.fg, ansi, palette);
        job.append(&text, 0.0, TextFormat::simple(font.clone(), color));
    }

    job
}
