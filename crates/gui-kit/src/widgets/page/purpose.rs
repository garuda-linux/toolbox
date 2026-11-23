use egui::{Label, Ui};

use crate::fonts::{BODY, Weight};
use crate::tokens;
use crate::widgets::rich;

/// The purpose of the page's header, which [`page_layout_shaped`] draws
/// at the top of the body.
///
/// [`page_layout_shaped`]: super::page_layout_shaped
pub(in crate::widgets) fn stash_purpose(ctx: &egui::Context, purpose: &str) {
    ctx.data_mut(|data| data.insert_temp(purpose_id(), purpose.to_owned()));
}

pub(super) fn take_purpose(ctx: &egui::Context) -> Option<String> {
    ctx.data_mut(|data| data.remove_temp::<String>(purpose_id()))
}

/// `body`, preceded by the purpose the header stashed this frame.
pub(super) fn with_purpose<P>(
    purpose: Option<String>,
    body: impl FnOnce(&mut P, &mut Ui),
) -> impl FnOnce(&mut P, &mut Ui) {
    move |shown: &mut P, inner: &mut Ui| {
        if let Some(text) = purpose.as_deref().filter(|text| !text.is_empty()) {
            purpose_line(inner, text);
        }

        body(shown, inner);
    }
}

fn purpose_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::page::purpose")
}

fn purpose_line(ui: &mut Ui, text: &str) {
    let tokens = tokens::current(ui.ctx());
    ui.add(Label::new(rich(ui, text, Weight::Regular, BODY, tokens.secondary)).wrap());
    ui.add_space(tokens.sizes.heading_gap);
}
