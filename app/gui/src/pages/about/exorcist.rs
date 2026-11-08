use egui::{Context, Id, Label};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{ButtonKind, DialogWidth, ModalWrapper, pill_button, rich};

use super::{BUGS, FORUM};
use crate::{i18n, links};

/// Whether the dialog stays open. Close, Escape and a link close it.
pub fn show(ctx: &Context) -> bool {
    let tokens = tokens::current(ctx);
    let title = i18n::text(ctx, "about-exorcist-title");
    let close = i18n::text(ctx, "dialog-close");
    let text = i18n::text(ctx, "about-exorcist-called");
    let forum = i18n::text(ctx, "about-exorcist-forum");
    let tracker = i18n::text(ctx, "about-exorcist-tracker");

    let modal = ModalWrapper::new(Id::unique("toolbox_gui::about::exorcist"), &title)
        .title_bar(&close)
        .width(DialogWidth::Small)
        .show(ctx, |ui| {
            ui.add(Label::new(rich(ui, &text, Weight::Regular, BODY, tokens.text)).wrap());
            ui.add_space(tokens.sizes.card_gap);

            let mut opened = None;
            ui.horizontal(|ui| {
                if pill_button(ui, &forum, ButtonKind::Soft).clicked() {
                    opened = Some(FORUM);
                }

                if pill_button(ui, &tracker, ButtonKind::Soft).clicked() {
                    opened = Some(BUGS);
                }
            });

            opened
        });

    if let Some(url) = modal.inner {
        links::open(ctx, url);
    }

    !modal.closed && modal.inner.is_none()
}
