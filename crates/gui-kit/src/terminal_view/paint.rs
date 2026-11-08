//! Drawing the terminal's visible rows: backgrounds, text, selection and
//! the cursor, one cell grid wide.

use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, StrokeKind, pos2, vec2};
use toolbox_terminal::{CellPoint, Color, CursorShape, Row, Run, Screen};
use unicode_width::UnicodeWidthChar as _;

use super::{Grid, float};
use crate::Palette;
use crate::terminal_colors::{Ansi, resolve};

/// How faint dim text (SGR 2) is drawn.
const DIM: f32 = 0.6;
/// How strongly the accent shows behind selected cells.
const SELECTION: f32 = 0.45;

pub struct Look<'a> {
    pub palette: &'a Palette,
    pub ansi: &'a Ansi,
    pub font: &'a FontId,
    pub ground: Color32,
}

pub fn paint(
    painter: &Painter,
    grid: &Grid,
    screen: &Screen,
    look: &Look<'_>,
    selection: Option<(CellPoint, CellPoint)>,
) {
    // A see-through ground is left to the frame around the grid, as two
    // translucent layers would darken the grid against its rim.
    if look.ground.is_opaque() {
        painter.rect_filled(grid.rect, 0.0, look.ground);
    }

    for (index, row) in screen.rows.iter().enumerate() {
        let line = grid.first_line.saturating_add(index);
        let mut column = 0_usize;

        for run in row {
            paint_run(painter, grid, look, run, (index, column));
            column = column.saturating_add(run.columns);
        }

        if let Some(span) = selection.and_then(|bounds| selected_columns(bounds, line, column)) {
            let rect = grid.cells(index, span.0, span.1.saturating_sub(span.0));
            painter.rect_filled(rect, 0.0, look.palette.accent.gamma_multiply(SELECTION));
        }
    }

    paint_cursor(painter, grid, screen, look);
}

fn paint_run(
    painter: &Painter,
    grid: &Grid,
    look: &Look<'_>,
    run: &Run,
    (row, column): (usize, usize),
) {
    let rect = grid.cells(row, column, run.columns);
    if run.style.bg != Color::Background {
        let background = resolve(run.style.bg, look.ansi, look.palette);
        painter.rect_filled(rect, 0.0, background);
    }

    let mut colour = resolve(run.style.fg, look.ansi, look.palette);
    if run.style.dim {
        colour = colour.gamma_multiply(DIM);
    }

    // Cell by cell: a whole run would be shaped as one text, and egui can
    // not turn off a font's ligatures (`==>` would become an arrow).
    // Combining marks stay with the character before them.
    let mut x = rect.left();
    for (cluster, cells) in clusters(&run.text) {
        painter.text(
            pos2(x, rect.top()),
            Align2::LEFT_TOP,
            cluster,
            look.font.clone(),
            colour,
        );
        x = grid.cell.x.mul_add(float(cells), x);
    }

    let stroke = Stroke::new(1.0, colour);
    if run.style.underline {
        let y = rect.bottom() - 1.0;
        painter.hline(rect.x_range(), y, stroke);
    }

    if run.style.strikeout {
        painter.hline(rect.x_range(), rect.center().y, stroke);
    }
}

fn paint_cursor(painter: &Painter, grid: &Grid, screen: &Screen, look: &Look<'_>) {
    let cursor = screen.cursor;
    if !cursor.visible {
        return;
    }

    let rect = grid.cells(cursor.row, cursor.column, 1);
    let text = look.palette.text;
    match cursor.shape {
        CursorShape::Block => {
            painter.rect_filled(rect, 0.0, text);
            if let Some(character) = character_at(screen.rows.get(cursor.row), cursor.column) {
                painter.text(
                    rect.min,
                    Align2::LEFT_TOP,
                    character,
                    look.font.clone(),
                    look.palette.view,
                );
            }
        }
        CursorShape::Underline => {
            painter.rect_filled(
                Rect::from_min_size(
                    pos2(rect.left(), rect.bottom() - 2.0),
                    vec2(rect.width(), 2.0),
                ),
                0.0,
                text,
            );
        }
        CursorShape::Beam => {
            painter.rect_filled(
                Rect::from_min_size(rect.min, vec2(2.0, rect.height())),
                0.0,
                text,
            );
        }
        CursorShape::HollowBlock => {
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, text), StrokeKind::Inside);
        }
    }
}

/// `text` as one cluster per cell-starting character, with the cells it
/// takes. Zero-width characters join the cluster before them.
fn clusters(text: &str) -> Vec<(String, usize)> {
    let mut clusters: Vec<(String, usize)> = Vec::new();
    for character in text.chars() {
        let cells = character.width().unwrap_or(0);
        match clusters.last_mut() {
            Some((cluster, _)) if cells == 0 => cluster.push(character),
            Some(_) | None => clusters.push((character.to_string(), cells)),
        }
    }

    clusters
}

/// The character in `column` of `row`, for drawing it over a block
/// cursor.
fn character_at(row: Option<&Row>, column: usize) -> Option<char> {
    let mut start = 0_usize;
    for run in row? {
        if column < start.saturating_add(run.columns) {
            let mut cell = start;
            for character in run.text.chars() {
                if cell >= column {
                    return Some(character);
                }

                cell = cell.saturating_add(character.width().unwrap_or(0));
            }

            return None;
        }

        start = start.saturating_add(run.columns);
    }

    None
}

/// The columns of `line` inside the selection, end exclusive.
fn selected_columns(
    (start, end): (CellPoint, CellPoint),
    line: usize,
    width: usize,
) -> Option<(usize, usize)> {
    let (first, last) = start.in_order(end);

    if line < first.line || line > last.line {
        return None;
    }

    let from = if line == first.line { first.column } else { 0 };
    let to = if line == last.line {
        last.column.saturating_add(1)
    } else {
        width.max(from.saturating_add(1))
    };

    (to > from).then_some((from, to))
}
