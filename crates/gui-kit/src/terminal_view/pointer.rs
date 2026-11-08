//! Mouse and wheel over the terminal: selection by drag, double click
//! (word) and triple click (line), reports to programs that asked for the
//! mouse (Shift selects anyway, as in other terminals), and the wheel.

use egui::{Event, Id, Modifiers, MouseWheelUnit, PointerButton, Pos2, Response, Ui};
use toolbox_terminal::{
    CellPoint, Modifiers as KeyModifiers, MouseAction, MouseButton, MouseEvent, MouseReporting,
    Terminal, encode_alternate_scroll, encode_mouse,
};

use super::input::key_modifiers;
use super::{Grid, float, float_i32, whole_lines};

/// What the widget remembers between frames, as `egui_term` keeps its
/// state in egui's memory.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct State {
    selection: Option<(CellPoint, CellPoint)>,
    /// Wheel movement not yet a whole line.
    wheel: f32,
}

/// The selected cells, anchor first.
pub fn selection(ui: &Ui, id: Id) -> Option<(CellPoint, CellPoint)> {
    load(ui, id).selection
}

/// Selects, reports and scrolls. Returns what goes to the program.
pub fn handle(
    ui: &Ui,
    response: &Response,
    grid: &Grid,
    terminal: &mut Terminal,
    (events, modifiers): (&[Event], Modifiers),
) -> Vec<u8> {
    let id = response.id;
    let mut state = load(ui, id);
    let mode = terminal.mouse_mode();
    let report = mode.reporting != MouseReporting::Off && !modifiers.shift;
    let held = key_modifiers(modifiers);
    let mut out = Vec::new();

    if report {
        report_buttons(ui, grid, (events, held), mode, &mut out);
    } else {
        select(response, grid, terminal, &mut state);
    }

    if response.hovered() {
        let lines = wheel_lines(events, grid, &mut state);
        if lines != 0_i32 {
            scroll(ui, grid, terminal, (lines, report, held), &mut out);
        }
    }

    ui.data_mut(|data| data.insert_temp(id, state));
    out
}

fn load(ui: &Ui, id: Id) -> State {
    ui.data(|data| data.get_temp(id)).unwrap_or_default()
}

fn select(response: &Response, grid: &Grid, terminal: &Terminal, state: &mut State) {
    let Some(cell) = response
        .interact_pointer_pos()
        .map(|pos| grid.cell_point(pos))
    else {
        return;
    };

    if response.triple_clicked() {
        state.selection = Some(terminal.line_bounds(cell));
    } else if response.double_clicked() {
        state.selection = Some(terminal.word_bounds(cell));
    } else if response.drag_started_by(PointerButton::Primary) {
        // egui reports the start once the pointer moved, so the anchor is
        // where the button went down.
        let origin = response
            .ctx
            .input(|input| input.pointer.press_origin())
            .map_or(cell, |pos| grid.cell_point(pos));

        state.selection = Some((origin, cell));
    } else if response.dragged_by(PointerButton::Primary) {
        if let Some((anchor, _)) = state.selection {
            state.selection = Some((anchor, cell));
        }
    } else if response.clicked() {
        state.selection = None;
    } else {
        // Hovering keeps the selection.
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "only pointer buttons and motion are reported"
)]
fn report_buttons(
    ui: &Ui,
    grid: &Grid,
    (events, modifiers): (&[Event], KeyModifiers),
    mode: toolbox_terminal::MouseMode,
    out: &mut Vec<u8>,
) {
    let held = ui.input(|input| input.pointer.button_down(PointerButton::Primary));

    for event in events {
        let (pos, button, action) = match *event {
            Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: _,
            } => (
                pos,
                Some(mouse_button(button)),
                if pressed {
                    MouseAction::Press
                } else {
                    MouseAction::Release
                },
            ),
            Event::PointerMoved(pos) => {
                (pos, held.then_some(MouseButton::Left), MouseAction::Motion)
            }
            _ => continue,
        };

        if let Some(bytes) = grid
            .rect
            .contains(pos)
            .then(|| encode_mouse(mouse_event(grid, pos, button, action, modifiers), mode))
            .flatten()
        {
            out.extend(bytes);
        }
    }
}

fn wheel_lines(events: &[Event], grid: &Grid, state: &mut State) -> i32 {
    for event in events {
        if let Event::MouseWheel {
            unit,
            delta,
            phase: _,
            source: _,
            modifiers: _,
        } = *event
        {
            state.wheel += match unit {
                MouseWheelUnit::Line => delta.y,
                MouseWheelUnit::Point => delta.y / grid.cell.y,
                MouseWheelUnit::Page => delta.y * float(grid.rows),
            };
        }
    }

    let lines = whole_lines(state.wheel);
    state.wheel -= float_i32(lines);
    lines
}

fn scroll(
    ui: &Ui,
    grid: &Grid,
    terminal: &mut Terminal,
    (lines, report, modifiers): (i32, bool, KeyModifiers),
    out: &mut Vec<u8>,
) {
    let mode = terminal.mouse_mode();

    if report {
        let Some(pos) = ui.input(|input| input.pointer.hover_pos()) else {
            return;
        };

        let button = if lines > 0_i32 {
            MouseButton::WheelUp
        } else {
            MouseButton::WheelDown
        };

        for _ in 0..lines.unsigned_abs() {
            let event = mouse_event(grid, pos, Some(button), MouseAction::Press, modifiers);
            out.extend(encode_mouse(event, mode).unwrap_or_default());
        }
    } else if mode.alternate_scroll {
        out.extend(encode_alternate_scroll(lines));
    } else {
        terminal.scroll(lines);
    }
}

fn mouse_event(
    grid: &Grid,
    pos: Pos2,
    button: Option<MouseButton>,
    action: MouseAction,
    modifiers: KeyModifiers,
) -> MouseEvent {
    let (row, column) = grid.screen_cell(pos);

    MouseEvent {
        button,
        action,
        column,
        row,
        modifiers,
    }
}

const fn mouse_button(button: PointerButton) -> MouseButton {
    match button {
        PointerButton::Secondary => MouseButton::Right,
        PointerButton::Middle => MouseButton::Middle,
        PointerButton::Primary | PointerButton::Extra1 | PointerButton::Extra2 => MouseButton::Left,
    }
}
