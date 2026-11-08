//! A terminal on a see-through ground leaves its fill to the frame around
//! it, so the blur under the frame shows, and cells that ask for a
//! colour of their own still get it solid.

use egui::Color32;
use egui_kittest::Harness;
use toolbox_terminal::{Color, Terminal, TerminalSize};

use super::super::TerminalView;
use crate::palette::{self, Palette};
use crate::terminal_colors::resolve;
use crate::testing::{filled_rects, harness};
use crate::tokens;

/// Red behind a word, then a block cursor on the prompt.
const OUTPUT: &[u8] = b"\x1b[41m failed \x1b[0m\r\n$ ";

fn shown(ground: Option<Color32>) -> Harness<'static, Terminal> {
    let mut terminal = Terminal::new(TerminalSize {
        columns: 40,
        rows: 4,
    });

    terminal.feed(OUTPUT);
    let mut harness = harness(400.0, 120.0).build_ui_state(
        move |ui, terminal: &mut Terminal| {
            let view = TerminalView::new(terminal, "Console");
            let view = match ground {
                Some(ground) => view.ground(ground),
                None => view,
            };

            view.show(ui);
        },
        terminal,
    );

    harness.run();
    harness
}

/// The red the theme fitted for the terminal, which programs print
/// errors in.
fn red(harness: &Harness<'_, Terminal>) -> Color32 {
    let tokens = tokens::current(&harness.ctx);
    let palette = Palette {
        view: tokens.terminal,
        ..palette::current(&harness.ctx)
    };

    resolve(Color::Indexed(1), &tokens.terminal_ansi, &palette)
}

#[test]
fn a_see_through_ground_leaves_the_grid_unfilled() {
    let glass = Color32::from_black_alpha(66);
    let harness = shown(Some(glass));
    let solid = tokens::current(&harness.ctx).terminal;

    assert!(
        filled_rects(&harness, solid).is_empty(),
        "the solid fill hides the blur"
    );
    assert!(
        filled_rects(&harness, glass).is_empty(),
        "the glass is laid twice"
    );
}

#[test]
fn on_a_see_through_ground_coloured_cells_and_the_cursor_stay_solid() {
    let harness = shown(Some(Color32::from_black_alpha(66)));
    let text = palette::current(&harness.ctx).text;

    assert_ne!(filled_rects(&harness, red(&harness)), Vec::new());
    assert_ne!(filled_rects(&harness, text), Vec::new(), "no block cursor");
}

#[test]
fn without_a_ground_the_grid_lies_on_the_solid_terminal() {
    let harness = shown(None);
    let solid = tokens::current(&harness.ctx).terminal;

    assert_ne!(filled_rects(&harness, solid), Vec::new());
}
