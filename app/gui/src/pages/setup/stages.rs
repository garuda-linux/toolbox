use egui::{Id, Label, Ui};
use toolbox_catalog::ToggleEntry;
use toolbox_core::features::console::ending::Ending;
use toolbox_core::features::toggles::summary::summary as queued;
use toolbox_gui_kit::fonts::{SECTION, Weight};
use toolbox_gui_kit::widgets::{
    self as widgets, ActionRow, ButtonKind, DialogWidth, Question, Tone, card, card_row,
    pill_button, rich,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::{SetupPage, Stage};
use crate::catalogs::CatalogId;
use crate::command::{HomeCommand, UiCommand};
use crate::i18n;
use crate::page::PageId;
use crate::pages::toggles::catalog_text;
use crate::pages::{PageContext, about};

const LINKS: [(&str, &str); 3] = [
    ("setup-forum", about::FORUM),
    ("setup-garuda-wiki", about::GARUDA_WIKI),
    ("setup-arch-wiki", about::ARCH_WIKI),
];

fn title(ui: &mut Ui, text: &str) {
    let color = ui.tokens().text;
    ui.label(rich(ui, text, Weight::Bold, SECTION, color));
    ui.add_space(ui.tokens().sizes.heading_gap);
}

fn paragraph(ui: &mut Ui, text: &str) {
    ui.add(Label::new(text).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);
}

pub fn welcome(ui: &mut Ui, steps: usize) {
    let ctx = ui.ctx().clone();
    title(ui, &i18n::text(&ctx, "setup-welcome-title"));
    paragraph(ui, &i18n::text(&ctx, "setup-welcome-text"));
    let count = steps.to_string();
    paragraph(
        ui,
        &i18n::text_with(&ctx, "setup-welcome-steps", &[("count", &count)]),
    );
    paragraph(ui, &i18n::text(&ctx, "setup-welcome-installed"));
}

pub fn step_heading(ui: &mut Ui, (number, count): (usize, usize), name: &str) {
    let ctx = ui.ctx().clone();
    let (number, count) = (number.to_string(), count.to_string());
    let words = [
        ("number", number.as_str()),
        ("count", count.as_str()),
        ("title", name),
    ];

    title(ui, &i18n::text_with(&ctx, "setup-step", &words));
}

pub fn summary(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    title(ui, &i18n::text(&ctx, "setup-summary"));
    let Ok(catalog) = &context.catalogs.setup.catalog else {
        return;
    };

    let picks = queued(catalog, context.snapshot, context.changes);
    if picks.is_empty() {
        paragraph(ui, &i18n::text(&ctx, "setup-nothing"));
        return;
    }

    let names = |entries: &[&ToggleEntry]| -> Vec<String> {
        entries
            .iter()
            .map(|entry| catalog_text(&ctx, CatalogId::Setup, entry.id(), entry.title()))
            .collect()
    };

    let units: Vec<String> = picks
        .units
        .iter()
        .chain(&picks.user_units)
        .map(ToString::to_string)
        .collect();

    card(ui, |ui| {
        let groups = [
            ("setup-install", names(&picks.install)),
            ("setup-remove", names(&picks.remove)),
            ("setup-units", units),
        ];

        let shown = groups.iter().filter(|(_, listed)| !listed.is_empty());
        for (index, (id, listed)) in shown.enumerate() {
            let count = listed.len().to_string();
            let heading = i18n::text_with(&ctx, id, &[("count", &count)]);
            let listing = listed.join(", ");
            card_row(ui, index == 0, |ui| {
                ActionRow::new(icons::CHECKMARK, Tone::Neutral, &heading)
                    .detail(&listing)
                    .show_without_actions(ui);
            });
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    paragraph(ui, &i18n::text(&ctx, "setup-review-note"));
}

pub fn success(ui: &mut Ui, context: &PageContext<'_>, page: &mut SetupPage) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    title(ui, &text("setup-success"));
    if !page.told_home {
        page.told_home = true;
        context.commands.send(HomeCommand::SetupFinished);
    }

    let ready = match page.ending {
        Some(Ending::Tested) => "setup-ready-tested",
        Some(Ending::Nothing) => "setup-ready-nothing",
        Some(Ending::Applied | Ending::Partly { skipped: _ }) | None => "setup-ready",
    };

    paragraph(ui, &text(ready));
    paragraph(ui, &text("setup-restart-note"));
    ui.horizontal(|ui| {
        if pill_button(ui, &text("setup-restart"), ButtonKind::Primary).clicked() {
            page.asking_to_restart = true;
        }

        if pill_button(ui, &text("setup-finish-setup"), ButtonKind::Soft).clicked() {
            page.stage = Stage::Welcome;
            context.commands.send(UiCommand::Navigate(PageId::Home));
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    ui.horizontal(|ui| {
        for (id, url) in LINKS {
            if pill_button(ui, &text(id), ButtonKind::Soft).clicked() {
                crate::links::open(&ctx, url);
            }
        }
    });

    if page.asking_to_restart {
        restart(&ctx, context, page);
    }
}

fn restart(ctx: &egui::Context, context: &PageContext<'_>, page: &mut SetupPage) {
    let question = i18n::text(ctx, "home-restart-question");
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text(ctx, "home-restart-question-detail");
    let cancel = i18n::text(ctx, "setup-cancel");
    let confirm = i18n::text(ctx, "home-restart-confirm");
    let texts = Question {
        title: &question,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = widgets::question(ctx, Id::unique("toolbox_gui::setup::restart"), texts);
    if asked.confirmed {
        context.commands.send(HomeCommand::Restart);
    }

    if asked.done {
        page.asking_to_restart = false;
    }
}
