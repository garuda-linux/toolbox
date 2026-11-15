//! The cursor shows only while a program runs and reads what is typed.

use egui_kittest::Harness;
use toolbox_terminal::{Terminal, TerminalSize};

use super::super::TerminalView;
use crate::palette;
use crate::testing::{filled_rects, harness};

fn shown(running: Option<bool>) -> Harness<'static, Terminal> {
    let mut terminal = Terminal::new(TerminalSize {
        columns: 40,
        rows: 4,
    });

    terminal.feed(b"Done\r\n");
    let mut harness = harness(400.0, 120.0).build_ui_state(
        move |ui, terminal: &mut Terminal| {
            let view = TerminalView::new(terminal, "Console");
            let view = match running {
                Some(running) => view.running(running),
                None => view,
            };

            view.show(ui);
        },
        terminal,
    );

    harness.run();
    harness
}

fn has_block_cursor(harness: &Harness<'_, Terminal>) -> bool {
    let text = palette::current(&harness.ctx).text;
    !filled_rects(harness, text).is_empty()
}

#[test]
fn the_cursor_shows_while_a_program_runs_as_it_did_before() {
    assert!(has_block_cursor(&shown(None)));
    assert!(has_block_cursor(&shown(Some(true))));
}

#[test]
fn without_a_program_the_cursor_hides() {
    assert!(!has_block_cursor(&shown(Some(false))));
}
