use egui::Ui;

use super::PageFixture;
use crate::pages::{PageContext, PageView};

struct Blank;

impl PageView for Blank {
    fn ui(&mut self, ui: &mut Ui, _context: &PageContext<'_>) {
        ui.label("Blank");
    }
}

#[test]
fn a_page_on_its_own_fills_the_whole_window_with_the_window_colour() {
    // A clear margin round the page turns into a light band once a dialog
    // blurs a shot of the window.
    let mut harness = PageFixture::default().harness(Blank);
    harness.run();

    let window = toolbox_gui_kit::tokens::current(&harness.ctx).window;
    let screen = harness.ctx.content_rect();
    let filled = toolbox_gui_kit::testing::filled_rects(&harness, window);

    assert!(
        filled.iter().any(|rect| rect.contains_rect(screen)),
        "{screen:?} not filled, only {filled:?}"
    );
}
