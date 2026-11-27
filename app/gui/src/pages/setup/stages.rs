use egui::{Id, Label, Ui};
use toolbox_catalog::ToggleEntry;
use toolbox_core::features::console::ending::Ending;
use toolbox_core::features::toggles::summary::summary as queued;
use toolbox_gui_kit::fonts::{SECTION, Weight};
use toolbox_gui_kit::widgets::{
    self as widgets, ActionRow, ButtonKind, DialogWidth, Question, Tone, card, card_row,
    pill_button, rich, status_tile, text_link,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::{SetupPage, Stage};
use crate::catalogs::CatalogId;
use crate::command::{HomeCommand, UiCommand};
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::pages::toggles::catalog_text;
use crate::pages::{PageContext, about};

const LINKS: [(&str, &str); 3] = [
    ("setup-forum", about::FORUM),
    ("setup-garuda-wiki", about::GARUDA_WIKI),
    ("setup-arch-wiki", about::ARCH_WIKI),
];

/// The names of what the assistant's picks queue, as the summary and the
/// success page list them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picks {
    install: Vec<String>,
    remove: Vec<String>,
    units: Vec<String>,
}

impl Picks {
    /// `None` while the catalog could not be read.
    pub fn of(ctx: &egui::Context, context: &PageContext<'_>) -> Option<Self> {
        let Ok(catalog) = &context.catalogs.setup.catalog else {
            return None;
        };

        let picks = queued(catalog, context.snapshot, context.changes);
        let names = |entries: &[&ToggleEntry]| -> Vec<String> {
            entries
                .iter()
                .map(|entry| catalog_text(ctx, CatalogId::Setup, entry.id(), entry.title()))
                .collect()
        };

        Some(Self {
            install: names(&picks.install),
            remove: names(&picks.remove),
            units: picks
                .units
                .iter()
                .chain(&picks.user_units)
                .map(ToString::to_string)
                .collect(),
        })
    }

    const fn is_empty(&self) -> bool {
        self.install.is_empty() && self.remove.is_empty() && self.units.is_empty()
    }

    /// A row per kind of change that has one, headed by the text `ids`
    /// name for installs, removals and services.
    fn card(&self, ui: &mut Ui, [install, remove, units]: [&str; 3]) {
        let ctx = ui.ctx().clone();
        let groups = [
            (install, &self.install),
            (remove, &self.remove),
            (units, &self.units),
        ];

        card(ui, |ui| {
            let shown = groups.iter().filter(|(_, listed)| !listed.is_empty());
            for (index, (id, listed)) in shown.enumerate() {
                let count = listed.len();
                let heading = i18n::text_with(&ctx, id, &[("count", Arg::from(count))]);
                let listing = listed.join(", ");
                card_row(ui, index == 0, |ui| {
                    ActionRow::new(icons::CHECKMARK, &heading)
                        .detail(&listing)
                        .show_without_actions(ui);
                });
            }
        });
    }
}

fn title(ui: &mut Ui, text: &str) {
    label(ui, text);
    ui.add_space(ui.tokens().sizes.heading_gap);
}

fn label(ui: &mut Ui, text: &str) {
    let color = ui.tokens().text;
    ui.label(rich(ui, text, Weight::Bold, SECTION, color));
}

fn paragraph(ui: &mut Ui, text: &str) {
    ui.add(Label::new(text).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);
}

pub fn welcome(ui: &mut Ui) {
    paragraph(ui, &i18n::text(ui.ctx(), "setup-welcome-text"));
}

pub(super) fn step_heading(ui: &mut Ui, (number, count): (usize, usize), name: &str) {
    let ctx = ui.ctx().clone();
    let words = [
        ("number", Arg::from(number)),
        ("count", Arg::from(count)),
        ("title", Arg::from(name)),
    ];

    title(ui, &i18n::text_with(&ctx, "setup-step", &words));
}

pub fn summary(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    title(ui, &i18n::text(&ctx, "setup-summary"));
    let Some(picks) = Picks::of(&ctx, context) else {
        return;
    };

    if picks.is_empty() {
        paragraph(ui, &i18n::text(&ctx, "setup-nothing"));
        return;
    }

    picks.card(ui, ["setup-install", "setup-remove", "setup-units"]);
    ui.add_space(ui.tokens().sizes.card_gap);
    paragraph(ui, &i18n::text(&ctx, "setup-review-note"));
}

pub fn success(ui: &mut Ui, context: &PageContext<'_>, page: &mut SetupPage) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    ui.horizontal(|ui| {
        _ = status_tile(ui, Tone::Success);
        label(ui, &text("setup-success"));
    });
    ui.add_space(ui.tokens().sizes.heading_gap);

    if !page.told_home {
        page.told_home = true;
        context.commands.send(HomeCommand::SetupFinished);
    }

    // Only a run that changed the system has something a restart would start.
    let (ready, changed) = match page.ending {
        Some(Ending::Tested) => ("setup-ready-tested", false),
        Some(Ending::Nothing) => ("setup-ready-nothing", false),
        Some(Ending::Applied | Ending::Partly { skipped: _ }) => ("setup-ready", true),
        None => ("setup-ready", false),
    };

    paragraph(ui, &text(ready));
    if changed
        && let Some((_, picks)) = &page.applied
        && !picks.is_empty()
    {
        picks.card(ui, ["setup-installed", "setup-removed", "setup-enabled"]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    if changed {
        paragraph(ui, &text("setup-restart-note"));
    }

    let finish = if changed {
        ButtonKind::Soft
    } else {
        ButtonKind::Primary
    };

    ui.horizontal(|ui| {
        if changed && pill_button(ui, &text("setup-restart"), ButtonKind::Primary).clicked() {
            page.asking_to_restart = true;
        }

        if pill_button(ui, &text("setup-finish-setup"), finish).clicked() {
            page.stage = Stage::Welcome;
            context.commands.send(UiCommand::Navigate(PageId::Home));
        }
    });

    ui.add_space(ui.tokens().sizes.block_gap);
    links(ui, &text);

    if page.asking_to_restart {
        restart(&ctx, context, page);
    }
}

fn links(ui: &mut Ui, text: &impl Fn(&str) -> String) {
    let ctx = ui.ctx().clone();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = ui.tokens().sizes.block_gap;
        for (id, url) in LINKS {
            let name = text(id);
            if text_link(ui, &name, &name).on_hover_text(url).clicked() {
                crate::links::open(&ctx, url);
            }
        }
    });
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
        icon: None,
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
