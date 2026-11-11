//! The report with the programs' colours, in the theme's ANSI palette as
//! the Console has it. A report holds up to megabytes, so only the lines
//! in view are laid out, and long lines scroll sideways instead of
//! wrapping, as in a terminal.

use egui::text::{LayoutJob, TextFormat};
use egui::{FontId, Label, ScrollArea, TextStyle, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::palette::{self, Palette};
use toolbox_gui_kit::terminal_colors::{Ansi, resolve};
use toolbox_pty::Run;

use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::pages::PageContext;
use crate::share::view::padded_card;

/// The view never shrinks below this, in points, even at the bottom of a
/// short window.
const MIN_HEIGHT: f32 = 240.0;

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let lines = context.service::<Diagnostics>().lines();
    let palette = palette::current(ui.ctx());
    let ansi = Ansi::of(&palette, ui.visuals().dark_mode);
    let font = TextStyle::Monospace.resolve(ui.style());
    let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font));

    // The box takes exactly the room left in view, so the page around it
    // never scrolls and the report's own bar is the only one.
    let sizes = ui.tokens().sizes;
    let around = 2.0_f32.mul_add(
        f32::from(sizes.card_padding.top.max(sizes.card_padding.bottom)),
        sizes.card_gap,
    );

    let height = (ui.clip_rect().bottom() - ui.cursor().top() - around).max(MIN_HEIGHT);
    let label = i18n::text(ui.ctx(), "diagnostics-report-label");

    padded_card(ui, |ui| {
        // Long lines run past the edge, a bar at rest says so.
        toolbox_gui_kit::theme::show_scroll_bars(ui.style_mut());
        ScrollArea::both()
            .id_salt("diagnostics-report")
            .auto_shrink([false, true])
            .max_height(height)
            .show_rows(ui, row_height, lines.len(), |ui, rows| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for line in lines.get(rows).unwrap_or_default() {
                    let job = layout(line, &font, &ansi, &palette);
                    ui.add(Label::new(job).extend());
                }
            });
    })
    .response
    .widget_info(|| egui::WidgetInfo::labeled(egui::accesskit::Role::Document, true, &*label));
}

/// One line's runs as one job, each run in its colour.
fn layout(line: &[Run], font: &FontId, ansi: &Ansi, palette: &Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    for run in line {
        let color = resolve(run.style.fg, ansi, palette);
        job.append(&run.text, 0.0, TextFormat::simple(font.clone(), color));
    }

    job
}
