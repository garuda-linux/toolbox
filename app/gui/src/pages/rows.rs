pub mod actions;
pub mod more;
mod picture;
#[cfg(test)]
mod tests;

use egui::Ui;
use fluent_templates::LanguageIdentifier;
pub use picture::{
    Initial, package_picture, picture_height, picture_or_initial, picture_slot, picture_top,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{self, BODY, Weight};
use toolbox_gui_kit::tokens::{self, Tokens};
use toolbox_gui_kit::widgets::{Select, Tone, banner};

use crate::i18n;

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
    let picked = Select::new(&label, &shown)
        .caption(&sort_by)
        .selected(current)
        .width(ui.tokens().sizes.sort_width)
        .show(ui);

    if let Some((picked, _)) = picked.and_then(|index| orders.get(index)) {
        *order = *picked;
    }
}

/// Returns whether it was pressed.
pub fn refusal_banner(ui: &mut Ui, tone: Tone, text: &str, dismiss_id: &str) -> bool {
    let dismiss = i18n::text(ui.ctx(), dismiss_id);
    let dismissed = banner(ui, tone, text, &[&dismiss]) == Some(0);
    ui.add_space(ui.tokens().sizes.card_gap);
    dismissed
}
