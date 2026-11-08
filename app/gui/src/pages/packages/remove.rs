//! The question before a removal: what `pacman -Rns` takes along, the
//! dependencies nothing else needs. pacman lists the real set in the
//! Console.

use egui::{Context, Id, Label, ScrollArea};
use toolbox_core::Toggle;
use toolbox_core::features::packages::graph::takes_along;
use toolbox_core::features::packages::list::Catalog;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, confirm_buttons, rich,
};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::PackageName;

use super::PackagesPage;
use crate::command::PackagesCommand;
use crate::i18n;
use crate::pages::PageContext;

const LIST_HEIGHT: f32 = 240.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Removal {
    pub package: PackageName,
    pub along: Vec<PackageName>,
}

impl Removal {
    /// Works out what goes along once, when the question opens.
    pub fn of(package: PackageName, catalog: &Catalog) -> Self {
        let along = takes_along(std::slice::from_ref(&package), catalog.local());
        Self { package, along }
    }
}

pub fn dialog(page: &mut PackagesPage, ctx: &Context, context: &PageContext<'_>) {
    let Some(removal) = &page.removing else {
        return;
    };

    let tokens = tokens::current(ctx);
    let name = removal.package.as_str();
    let title = i18n::text_with(ctx, "packages-remove-title", &[("package", name)]);
    let close = i18n::text(ctx, "dialog-close");
    let modal = ModalWrapper::new(Id::unique("toolbox_gui::packages::remove"), &title)
        .title_bar(&close)
        .width(DialogWidth::Small)
        .show(ctx, |ui| {
            let count = removal.along.len().to_string();
            let intro = if removal.along.is_empty() {
                i18n::text(ctx, "packages-remove-alone")
            } else {
                i18n::text_with(ctx, "packages-remove-along", &[("count", &count)])
            };

            ui.add(Label::new(rich(ui, &intro, Weight::Regular, BODY, tokens.text)).wrap());

            if !removal.along.is_empty() {
                ui.add_space(ui.tokens().sizes.control_gap);
                ScrollArea::vertical()
                    .max_height(LIST_HEIGHT)
                    .show(ui, |ui| {
                        for package in &removal.along {
                            ui.label(rich(
                                ui,
                                package.as_str(),
                                Weight::Medium,
                                BODY,
                                tokens.text,
                            ));
                        }
                    });
            }

            ui.add_space(ui.tokens().sizes.control_gap);
            let note = i18n::text(ctx, "packages-remove-note");
            ui.add(Label::new(rich(ui, &note, Weight::Regular, BODY, tokens.secondary)).wrap());
            ui.add_space(ui.tokens().sizes.card_gap);

            confirm_buttons(
                ui,
                &i18n::text(ctx, "packages-cancel"),
                &i18n::text(ctx, "packages-remove"),
                ButtonKind::Danger,
            )
        });

    match modal.inner {
        Some(Confirmation::Confirmed) => {
            context.commands.send(PackagesCommand::Toggle {
                package: removal.package.clone(),
                wish: Toggle::Off,
            });
            page.removing = None;
        }
        Some(Confirmation::Cancelled) => page.removing = None,
        None if modal.closed => page.removing = None,
        None => {}
    }
}
