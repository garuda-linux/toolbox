//! A page with the command palette over it, for the pages whose own
//! Escape must leave an overlay its key.

use egui::accesskit::Role;
use egui::{Key, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::widgets::BodyShape;

use crate::pages::{PageContext, PageView};
use crate::shell::Palette;
use crate::testing::PageState;

/// A page with the command palette drawn after it, in the shell's order.
#[derive(Default)]
pub struct UnderPalette<P> {
    pub page: P,
    pub palette: Palette,
}

impl<P> PageView for UnderPalette<P>
where
    P: PageView,
{
    fn shape(&self) -> BodyShape {
        self.page.shape()
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        self.page.header(ui, context)
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        self.page.ui(ui, context);
        self.palette.show(ui.ctx(), context, context.commands);
    }
}

fn palette_shows<S>(harness: &Harness<'_, S>) -> bool {
    harness
        .query_by_role_and_label(Role::TextInput, "Command Palette")
        .is_some()
}

/// Opens the palette over the page, presses Escape once and asserts that
/// the palette closed. The caller checks what the page kept.
pub fn escape_under_palette<P>(harness: &mut Harness<'static, PageState<UnderPalette<P>>>)
where
    P: PageView,
{
    harness.state_mut().page.palette.open();
    harness.run();
    assert!(palette_shows(harness));

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(!palette_shows(harness), "Escape closes the palette");
}
