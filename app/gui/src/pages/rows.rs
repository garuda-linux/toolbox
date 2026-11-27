pub mod actions;
pub mod more;
mod picture;
#[cfg(test)]
mod tests;

use egui::Ui;
use fluent_templates::LanguageIdentifier;
pub use picture::{Initial, package_picture, picture_height, picture_or_initial, picture_slot};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{self, BODY, Weight};
use toolbox_gui_kit::tokens::{self, Tokens};
use toolbox_gui_kit::widgets::{RowState, Select, Tone, banner, switch};

use super::PageContext;
use crate::command::AppCommand;
use crate::i18n;
use crate::page::PageId;
use crate::refused::Shown;

pub const MENU_AND_SWITCH: f32 = 90.0;
pub const SWITCH_WIDTH: f32 = 40.0;
/// An app picture's edge in pixels, as the icon theme is asked for it.
pub const PICTURE_PIXELS: u16 = 32;
pub const PICTURE_SIZE: f32 = 32.0;
pub const PICTURE_RADIUS: u8 = 6;

/// What a cached row text depends on besides its page's inputs: the
/// language, the colours and the size of the text.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    language: LanguageIdentifier,
    tokens: Tokens,
    body: egui::FontId,
}

impl Look {
    pub fn of(ui: &Ui) -> Self {
        Self {
            language: i18n::language(ui.ctx()),
            tokens: tokens::current(ui.ctx()),
            body: fonts::font(ui.style(), Weight::Regular, BODY),
        }
    }
}

pub fn sort_select<T>(ui: &mut Ui, label_id: &str, orders: &[(T, &str)], order: &mut T)
where
    T: Copy + PartialEq,
{
    let ctx = ui.ctx().clone();
    let names: Vec<String> = orders
        .iter()
        .map(|(_, id)| i18n::text(&ctx, id).into_owned())
        .collect();

    let shown: Vec<&str> = names.iter().map(String::as_str).collect();
    let current = orders.iter().position(|(each, _)| each == order);
    let label = i18n::text(&ctx, label_id);
    let sort_by = i18n::text(&ctx, "common-sort-by");
    let width = Select::fitting_width(ui, shown.iter().copied());
    let picked = Select::new(&label, &shown)
        .caption(&sort_by)
        .selected(current)
        .width(width)
        .show(ui);

    if let Some((picked, _)) = picked.value.and_then(|index| orders.get(index)) {
        *order = *picked;
    }
}

/// The switch at a list row's edge, `enabled` false while the list waits for
/// a plan. Returns whether it flipped, by a click or by Space on the focused row.
pub fn row_switch(ui: &mut Ui, state: &RowState, on: bool, name: &str, enabled: bool) -> bool {
    let enabled = enabled && ui.is_enabled();
    let clicked = ui
        .enabled_ui(enabled, |ui| switch(ui, on, name).clicked())
        .inner;

    clicked || (state.toggle && enabled)
}

pub fn refusal_banner(ui: &mut Ui, tone: Tone, text: &str, dismiss_id: &str) -> bool {
    let dismiss = i18n::text(ui.ctx(), dismiss_id);
    let dismissed = banner(ui, tone, text, &[&dismiss]).value == Some(0);
    ui.add_space(ui.tokens().sizes.card_gap);
    dismissed
}

/// A banner for a read that is partly unread, with Read again when `again`
/// is set. Returns whether Read again was pressed.
pub fn read_again_banner(ui: &mut Ui, tone: Tone, text: &str, again: bool) -> bool {
    let label = i18n::text(ui.ctx(), "common-read-again");
    let action = again.then_some(&*label);
    banner(ui, tone, text, action.as_slice()).value == Some(0)
}

fn refused_banner(ui: &mut Ui, refused: Option<&impl Shown>, tone: Tone, dismiss_id: &str) -> bool {
    refused.is_some_and(|refused| {
        let text = refused.text(ui.ctx());
        refusal_banner(ui, tone, &text, dismiss_id)
    })
}

/// The error banner of a page's refused click. Its dismiss sends
/// `DismissRefusal` for `page`, which clears the refusal.
pub fn refused_page_banner(
    ui: &mut Ui,
    context: &PageContext<'_>,
    refused: Option<&impl Shown>,
    page: PageId,
    dismiss_id: &str,
) {
    if refused_banner(ui, refused, Tone::Error, dismiss_id) {
        context.commands.send(AppCommand::DismissRefusal(page));
    }
}
