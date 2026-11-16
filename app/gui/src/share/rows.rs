//! The row viewer of a long text, which the Diagnostics report and the
//! upload preview share. Every row is one line tall, so only the rows in
//! view are laid out, however long the text.

use egui::FontId;
use egui::text::{LayoutJob, TextFormat};
use toolbox_gui_kit::palette::Palette;
use toolbox_gui_kit::terminal_colors::{Ansi, resolve};
use toolbox_pty::Run;
use toolbox_services::diagnostics::view::{self, Row};

/// The whole characters that fit `room`, at least one.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a row holds a few hundred characters at most"
)]
pub const fn character_count(room: f32) -> usize {
    let whole = room as usize;
    if whole == 0 { 1 } else { whole }
}

/// How far down row `row` starts, `row_height` apart.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "no text has rows past f32's exact integers"
)]
pub const fn row_top(row: usize, row_height: f32) -> f32 {
    row as f32 * row_height
}

/// The piece of one line `row` holds as one job, each run in its colour.
pub fn layout(
    runs: &[Run],
    row: Row,
    font: &FontId,
    (ansi, palette): (&Ansi, &Palette),
) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (text, run) in view::piece(runs, row.start, row.end) {
        let color = resolve(run.style.fg, ansi, palette);
        job.append(&text, 0.0, TextFormat::simple(font.clone(), color));
    }

    job
}

pub fn piece_text(runs: &[Run], row: Row) -> String {
    view::piece(runs, row.start, row.end)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}
