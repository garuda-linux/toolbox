//! After `egui_term`, the caller owns the [`Terminal`] and the widget only draws it.

mod input;
mod paint;
mod pointer;
#[cfg(test)]
mod tests;

use egui::accesskit::{Live, Role};
use egui::{
    Color32, Event, EventFilter, FocusDirection, Modifiers, Pos2, Rect, Response, Sense, TextStyle,
    Ui, Vec2, ViewportCommand, WidgetInfo, pos2, vec2,
};
use toolbox_terminal::{CellPoint, Screen, Terminal, TerminalSize, encode_paste};

use self::input::{Input, translate};
use self::paint::{Look, paint};
use crate::palette::{self, Palette};
use crate::tokens;

/// How long, in seconds, a Ctrl+Shift+V waits for eframe's paste.
const PASTE_WAIT: f64 = 1.0;

/// The keys that stay with the program while the Console has focus.
/// Ctrl+Tab leaves instead (see [`input::Input::Leave`]).
const LOCKED: EventFilter = EventFilter {
    tab: true,
    horizontal_arrows: true,
    vertical_arrows: true,
    escape: true,
};

pub struct TerminalView<'a> {
    terminal: &'a mut Terminal,
    /// The accessible name, as Orca announces the Console.
    label: &'a str,
    /// How to leave, which the locked keys hide. Read with the name.
    hint: Option<&'a str>,
    /// What the grid lies on, [`crate::Tokens::terminal`] unless set.
    ground: Option<Color32>,
}

#[derive(Debug)]
pub struct TerminalOutput {
    pub response: Response,
    pub bytes: Vec<u8>,
    /// The size that fills the widget. The caller clamps it and resizes
    /// the terminal and the program together.
    pub fitting: TerminalSize,
}

struct Grid {
    rect: Rect,
    /// One cell, in points.
    cell: Vec2,
    columns: usize,
    rows: usize,
    /// The scrollback line of the top row, for [`CellPoint`]s.
    first_line: usize,
}

impl<'a> TerminalView<'a> {
    #[must_use = "the view does nothing until shown"]
    pub const fn new(terminal: &'a mut Terminal, label: &'a str) -> Self {
        Self {
            terminal,
            label,
            hint: None,
            ground: None,
        }
    }

    /// Tells a screen reader how to leave the terminal.
    #[must_use = "the view does nothing until shown"]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Lays the grid on `ground`. A see-through one is left to the frame
    /// around the view, which fills it once for grid and margin alike.
    #[must_use = "the view does nothing until shown"]
    pub const fn ground(mut self, ground: Color32) -> Self {
        self.ground = Some(ground);
        self
    }

    pub fn show(mut self, ui: &mut Ui) -> TerminalOutput {
        let font = TextStyle::Monospace.resolve(ui.style());
        let cell =
            ui.fonts_mut(|fonts| vec2(fonts.glyph_width(&font, 'M'), fonts.row_height(&font)));

        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let label = self.label;
        response.widget_info(|| WidgetInfo::labeled(Role::Terminal, true, label));

        let grid = Grid::of(rect, cell, self.terminal);
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }

        ui.ctx()
            .data_mut(|data| data.insert_temp(owner_id(), response.id));

        let (events, held) = ui.input(|input| (input.events.clone(), input.modifiers));
        let mut bytes = Vec::new();
        self.keys(ui, &response, (&events, held), &mut bytes);
        bytes.extend(pointer::handle(
            ui,
            &response,
            &grid,
            self.terminal,
            (&events, held),
        ));

        // The terminal sits on the darkest surface, set off from the sheet.
        // Its fitted colours mix from that solid fill even over a glass
        // ground, or black and white would turn see-through too.
        let tokens = tokens::current(ui.ctx());
        let terminal = tokens.terminal;
        let palette = Palette {
            view: terminal,
            ..palette::current(ui.ctx())
        };

        let look = Look {
            palette: &palette,
            ansi: &tokens.terminal_ansi,
            font: &font,
            ground: self.ground.unwrap_or(terminal),
        };

        let selection = pointer::selection(ui, response.id);
        let view = self.terminal.view();
        paint(&ui.painter_at(rect), &grid, &view, &look, selection);
        self.speak(ui, &response, &view);

        TerminalOutput {
            response,
            bytes,
            fitting: grid.fitting(),
        }
    }

    /// The text on screen as the node's value. It is a polite live region
    /// only while the last line asks something, such as a password or a
    /// Y/n prompt, so progress output does not flood the screen reader.
    fn speak(&self, ui: &Ui, response: &Response, view: &Screen) {
        let text = screen_text(view);
        let asks = asks(&text);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_value(text);
            node.set_live(if asks { Live::Polite } else { Live::Off });
            if let Some(hint) = self.hint {
                node.set_description(hint);
            }
        });
    }

    /// Ctrl+Shift+V asks eframe for the clipboard, which comes back as a
    /// paste event in a following frame. The first paste after the ask is
    /// the clipboard's text. Any other is a plain Ctrl+V, which the
    /// program gets as ^V. An empty clipboard sends no paste, so the ask
    /// runs out after [`PASTE_WAIT`].
    fn keys(
        &mut self,
        ui: &Ui,
        response: &Response,
        (events, held): (&[Event], Modifiers),
        bytes: &mut Vec<u8>,
    ) {
        let asked_id = response.id.with("paste-asked");
        let now = ui.input(|input| input.time);
        let mut asked = ui
            .ctx()
            .data_mut(|data| data.remove_temp::<f64>(asked_id))
            .filter(|at| now - at < PASTE_WAIT);

        if !response.has_focus() {
            return;
        }

        ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, LOCKED));

        for event in events {
            if let Event::Paste(text) = event
                && asked.take().is_some()
            {
                self.terminal.scroll_to_bottom();
                bytes.extend(encode_paste(text, self.terminal.key_mode()));
                continue;
            }

            match translate(event, held, self.terminal.key_mode()) {
                Input::Bytes(typed) => {
                    self.terminal.scroll_to_bottom();
                    bytes.extend(typed);
                }
                Input::Copy => {
                    if let Some((start, end)) = pointer::selection(ui, response.id) {
                        ui.ctx().copy_text(self.terminal.selection_text(start, end));
                    }
                }
                Input::Paste => {
                    ui.ctx().send_viewport_cmd(ViewportCommand::RequestPaste);
                    asked = Some(now);
                }
                Input::Leave { backward } => ui.memory_mut(|memory| {
                    memory.move_focus(if backward {
                        FocusDirection::Previous
                    } else {
                        FocusDirection::Next
                    });
                }),
                Input::Nothing => {}
            }
        }

        if let Some(at) = asked {
            ui.ctx().data_mut(|data| data.insert_temp(asked_id, at));
        }
    }
}

impl Grid {
    fn of(rect: Rect, cell: Vec2, terminal: &Terminal) -> Self {
        let size = terminal.size();

        Self {
            rect,
            cell,
            columns: usize::from(size.columns),
            rows: usize::from(size.rows),
            first_line: terminal
                .history_len()
                .saturating_sub(terminal.display_offset()),
        }
    }

    fn cells(&self, row: usize, column: usize, count: usize) -> Rect {
        let min = pos2(
            self.cell.x.mul_add(float(column), self.rect.left()),
            self.cell.y.mul_add(float(row), self.rect.top()),
        );

        Rect::from_min_size(min, vec2(self.cell.x * float(count), self.cell.y))
    }

    fn screen_cell(&self, pos: Pos2) -> (usize, usize) {
        let column = whole_cells((pos.x - self.rect.left()) / self.cell.x);
        let row = whole_cells((pos.y - self.rect.top()) / self.cell.y);

        (
            row.min(self.rows.saturating_sub(1)),
            column.min(self.columns.saturating_sub(1)),
        )
    }

    /// The cell under `pos`, counted over scrollback and screen.
    fn cell_point(&self, pos: Pos2) -> CellPoint {
        let (row, column) = self.screen_cell(pos);

        CellPoint {
            line: self.first_line.saturating_add(row),
            column,
        }
    }

    /// The most cells that fit, at least two columns and one row.
    fn fitting(&self) -> TerminalSize {
        let columns = whole_cells(self.rect.width() / self.cell.x).max(2);
        let rows = whole_cells(self.rect.height() / self.cell.y).max(1);

        TerminalSize {
            columns: u16::try_from(columns).unwrap_or(u16::MAX),
            rows: u16::try_from(rows).unwrap_or(u16::MAX),
        }
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "cell counts are far below where f32 loses whole numbers"
)]
const fn float(count: usize) -> f32 {
    count as f32
}

#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "wheel line counts are small"
)]
const fn float_i32(count: i32) -> f32 {
    count as f32
}

/// Whole cells in `cells`, 0 for anything below one or not a number.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "floored, non-negative and checked first, no From exists for f32"
)]
const fn whole_cells(cells: f32) -> usize {
    if cells.is_finite() && cells >= 1.0 {
        cells.floor() as usize
    } else {
        0
    }
}

/// Whole wheel lines in `lines`, towards zero.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "clamped to a small range first, no From exists for f32"
)]
const fn whole_lines(lines: f32) -> i32 {
    if lines.is_finite() {
        lines.trunc().clamp(-1000.0, 1000.0) as i32
    } else {
        0
    }
}

/// Whether the last line of `text` waits for an answer: it ends in a
/// colon, a question mark or a closing bracket of choices.
fn asks(text: &str) -> bool {
    text.lines()
        .last()
        .is_some_and(|line| line.trim_end().ends_with([':', '?', ']']))
}

/// The rows of `view` as lines, each without its trailing blanks, and no
/// blank lines after the last written one.
fn screen_text(view: &Screen) -> String {
    view.row_texts().join("\n").trim_end().to_owned()
}

/// While a terminal has the focus, every key belongs to its program, so
/// app shortcuts must leave them alone. egui drops the focus of a widget
/// that is no longer drawn, so a closed terminal owns nothing.
pub fn owns_keys(ctx: &egui::Context) -> bool {
    ctx.data(|data| data.get_temp::<egui::Id>(owner_id()))
        .is_some_and(|id| ctx.memory(|memory| memory.has_focus(id)))
}

fn owner_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::terminal_view::owner")
}
