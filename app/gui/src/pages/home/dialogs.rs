//! Home's dialogs: the pending updates and the restart question.

use std::time::SystemTime;

use egui::{Context, Id, Label, ScrollArea, Ui};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, Tone, confirm_buttons, rich, split_row,
    tag,
};
use toolbox_gui_kit::{Loadable, tokens};

use super::HomePage;
use super::view::{Overview, Pending};
use crate::command::HomeCommand;
use crate::i18n;
use crate::pages::PageContext;

const LIST_HEIGHT: f32 = 380.0;
const VERSIONS_WIDTH: f32 = 220.0;

pub fn show(page: &mut HomePage, ctx: &Context, context: &PageContext<'_>) {
    if page.updates_open {
        updates(page, ctx, context);
    }

    if page.asking_to_restart {
        restart(page, ctx, context);
    }
}

fn updates(page: &mut HomePage, ctx: &Context, context: &PageContext<'_>) {
    let tokens = tokens::current(ctx);
    let overview = context
        .home
        .status
        .and_then(Loadable::get)
        .map(|status| Overview::of(status, SystemTime::now()));

    let pending = overview
        .as_ref()
        .map_or(&[][..], |overview| &overview.pending);

    let title = i18n::text(ctx, "home-pending");
    let close = i18n::text(ctx, "dialog-close");
    let modal = ModalWrapper::new(Id::unique("toolbox_gui::home::updates"), &title)
        .title_bar(&close)
        .width(DialogWidth::Small)
        .show(ctx, |ui| {
            if pending.is_empty() {
                let none = i18n::text(ctx, "home-pending-none");
                ui.label(rich(ui, &none, Weight::Regular, BODY, tokens.secondary));
            } else {
                ScrollArea::vertical()
                    .max_height(LIST_HEIGHT)
                    .show(ui, |ui| {
                        for update in pending {
                            row(ui, update);
                        }
                    });
            }
        });

    if modal.closed {
        page.updates_open = false;
    }
}

/// Names and versions are the package manager's words, shown as they are.
fn row(ui: &mut Ui, update: &Pending<'_>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    split_row(
        ui,
        VERSIONS_WIDTH,
        |ui| {
            ui.horizontal(|ui| {
                ui.label(rich(ui, update.name, Weight::Medium, BODY, tokens.text));
                if update.aur {
                    tag(ui, &i18n::text(&ctx, "home-pending-aur"), Tone::Neutral);
                }
            });
        },
        |ui| {
            let versions = i18n::text_with(
                &ctx,
                "home-pending-versions",
                &[("from", update.from), ("to", update.to)],
            );

            let versions = rich(ui, &versions, Weight::Regular, SMALL, tokens.secondary)
                .family(egui::FontFamily::Monospace);

            ui.add(Label::new(versions).truncate());
        },
    );
}

fn restart(page: &mut HomePage, ctx: &Context, context: &PageContext<'_>) {
    // A plan started while the question was open: the restart waits.
    if context.busy {
        page.asking_to_restart = false;
        return;
    }

    let tokens = tokens::current(ctx);
    let question = i18n::text(ctx, "home-restart-question");
    let close = i18n::text(ctx, "dialog-close");
    let modal = ModalWrapper::new(Id::unique("toolbox_gui::home::restart"), &question)
        .title_bar(&close)
        .show(ctx, |ui| {
            let detail = i18n::text(ctx, "home-restart-question-detail");
            ui.add(Label::new(rich(ui, &detail, Weight::Regular, BODY, tokens.secondary)).wrap());
            confirm_buttons(
                ui,
                &i18n::text(ctx, "home-cancel"),
                &i18n::text(ctx, "home-restart-confirm"),
                ButtonKind::Danger,
            )
        });

    if modal.inner == Some(Confirmation::Confirmed) {
        context.commands.send(HomeCommand::Restart);
    }

    if modal.inner.is_some() || modal.closed {
        page.asking_to_restart = false;
    }
}
