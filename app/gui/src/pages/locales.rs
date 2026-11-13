//! Every click waits in the queue. Apply edits `locale.gen`, runs
//! `locale-gen` once and sets the system language last.

mod list;
mod packs;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::locales::view::{
    locales_known, pack_rows, packs_known, packs_missing, system_choice_names, system_choices,
    system_queued, system_shown,
};
use toolbox_core::features::locales::{Filter, PackOrder};
use toolbox_gui_kit::widgets::{
    ButtonKind, Select, Tone, banner, card, empty_state, page_toolbar, pill_button, toggle_chips,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::LocaleName;

use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::{refusal_banner, sort_select};
use super::setting_row::{Words, row};
use super::{PageContext, PageView, plan_running};
use crate::command::LocaleCommand;
use crate::i18n;

const WIDE_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_wide;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Tab {
    #[default]
    Locales,
    Packs,
}

impl TabText for Tab {
    fn text_id(self) -> &'static str {
        match self {
            Self::Locales => "locales-tab-locales",
            Self::Packs => "locales-tab-packs",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Chip {
    Utf8,
    Enabled,
}

#[derive(Debug, Default)]
pub struct LocalesPage {
    tab: Tab,
    filter: Filter,
    order: PackOrder,
    locales: list::Rows,
    packs: packs::Rows,
}

impl PageView for LocalesPage {
    fn place(&self) -> u64 {
        super::place_of(&self.tab)
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let ids = TabbedIds {
            title: "page-locales",
            subtitle: "page-locales-subtitle",
            tabs: "locales-tabs",
            refresh: "locales-refresh",
        };

        let done = tabbed_header(ui, ids, &[Tab::Locales, Tab::Packs], self.tab);
        if let Some(picked) = done.picked {
            self.tab = picked;
            if context.refusals.locale.is_some() {
                context.commands.send(LocaleCommand::DismissRefusal);
            }
        }

        if done.refresh {
            context.commands.send(LocaleCommand::Refresh);
        }

        page_toolbar(
            ui,
            self,
            |page, ui| {
                if page.tab == Tab::Locales {
                    let field = &mut page.filter.search;
                    _ = search(ui, field, "locales-search", "locales-search-hint");
                }
            },
            |page, ui| {
                if page.tab == Tab::Packs {
                    page.sort(ui);
                }
            },
        );

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("locales ui").entered();
        refusal(ui, context);
        plan_running(ui, context);
        match self.tab {
            Tab::Locales => self.locales(ui, context),
            Tab::Packs => self.packs(ui, context),
        }
    }
}

impl LocalesPage {
    fn locales(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        if !context.probed {
            empty_state(
                ui,
                icons::LANGUAGES,
                &i18n::text(&ctx, "locales-reading"),
                "",
            );
            return;
        }

        if !locales_known(context.snapshot) {
            empty_state(
                ui,
                icons::HELP_ABOUT,
                &i18n::text(&ctx, "locales-unavailable"),
                &i18n::text(&ctx, "locales-unavailable-detail"),
            );
            return;
        }

        system_language(ui, context);
        ui.add_space(ui.tokens().sizes.card_gap);
        self.chips(ui);
        ui.add_space(ui.tokens().sizes.card_gap);

        let items = list::rows(ui, context, &self.filter, &mut self.locales);
        if items.is_empty() {
            empty_state(
                ui,
                icons::SYSTEM_SEARCH,
                &i18n::text(&ctx, "locales-none"),
                &i18n::text(&ctx, "locales-none-detail"),
            );

            let clear = i18n::text(&ctx, "locales-clear-search");
            if !self.filter.search.is_empty() && pill_button(ui, &clear, ButtonKind::Soft).clicked()
            {
                self.filter.search.clear();
            }

            return;
        }

        ui.add_enabled_ui(!context.busy, |ui| list::show(ui, context, &items));
    }

    fn chips(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let (utf8, enabled) = (
            i18n::text(&ctx, "locales-filter-utf8"),
            i18n::text(&ctx, "locales-filter-enabled"),
        );

        let options = [(Chip::Utf8, &*utf8), (Chip::Enabled, &*enabled)];
        let filter = &self.filter;
        let flipped = toggle_chips(ui, &options, |chip| match chip {
            Chip::Utf8 => filter.utf8_only,
            Chip::Enabled => filter.enabled_only,
        });

        match flipped {
            Some(Chip::Utf8) => self.filter.utf8_only ^= true,
            Some(Chip::Enabled) => self.filter.enabled_only ^= true,
            None => {}
        }
    }

    fn packs(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        if !context.probed {
            empty_state(
                ui,
                icons::PACKAGE,
                &i18n::text(&ctx, "locales-packs-reading"),
                "",
            );
            return;
        }

        if !packs_known(context.snapshot, context.changes) {
            empty_state(
                ui,
                icons::HELP_ABOUT,
                &i18n::text(&ctx, "locales-packs-unavailable"),
                &i18n::text(&ctx, "locales-packs-unavailable-detail"),
            );
            return;
        }

        let missing = packs_missing(&pack_rows(context.snapshot, context.changes));
        let intro = if missing > 0 {
            i18n::text_with(
                &ctx,
                "locales-packs-intro",
                &[("count", &missing.to_string())],
            )
        } else {
            i18n::text(&ctx, "locales-packs-intro-plain")
        };

        // Missing packs warn as the Kernels page's missing headers do, and
        // its banner's buttons, on a wash of their own, need no rim.
        let tone = if missing > 0 {
            Tone::Warning
        } else {
            Tone::Neutral
        };

        let add = i18n::text(&ctx, "locales-packs-add-missing");
        let action = (missing > 0 && !context.busy).then_some(&*add);
        if banner(ui, tone, &intro, action.as_slice()) == Some(0) {
            context.commands.send(LocaleCommand::AddMissingPacks);
        }

        ui.add_space(ui.tokens().sizes.card_gap);

        let items = packs::rows(ui, context, self.order, &mut self.packs);
        if items.is_empty() {
            empty_state(
                ui,
                icons::PACKAGE,
                &i18n::text(&ctx, "locales-packs-none"),
                &i18n::text(&ctx, "locales-packs-none-detail"),
            );
            return;
        }

        ui.add_enabled_ui(!context.busy, |ui| packs::show(ui, context, &items));
    }

    fn sort(&mut self, ui: &mut Ui) {
        let orders = [
            (PackOrder::Package, "locales-packs-sort-package"),
            (PackOrder::Program, "locales-packs-sort-program"),
            (PackOrder::Language, "locales-packs-sort-language"),
            (PackOrder::Selected, "locales-packs-sort-selected"),
        ];

        sort_select(ui, "locales-packs-sort", &orders, &mut self.order);
    }
}

fn system_language(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let (snapshot, changes) = (context.snapshot, context.changes);
    let codes = system_choices(snapshot, changes);
    let names: Vec<String> = codes
        .iter()
        .zip(system_choice_names(snapshot, changes))
        .map(|(code, (language, territory))| choice(&ctx, code, &language, &territory))
        .collect();

    let shown = system_shown(snapshot, changes);
    let title = i18n::text(&ctx, "locales-system");
    let description = i18n::text(&ctx, "locales-system-detail");
    let queued = (!system_queued(changes).is_empty()).then(|| i18n::text(&ctx, "locales-queued"));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: queued.as_deref(),
    };

    let unset = i18n::text(&ctx, "locales-system-unset");
    let width = system_width(
        ui,
        names.iter().map(String::as_str).chain([&*unset, &*shown]),
    );

    card(ui, |ui| {
        row(ui, true, &words, width, |ui| {
            let options: Vec<&str> = names.iter().map(String::as_str).collect();
            let current = codes.iter().position(|code| *code == shown);
            let placeholder = if shown.is_empty() { &*unset } else { &shown };
            let picked = ui
                .add_enabled_ui(!codes.is_empty() && !context.busy, |ui| {
                    Select::new(&title, &options)
                        .selected(current)
                        .placeholder(placeholder)
                        .width(width.min(ui.available_width()))
                        .show(ui)
                })
                .inner;

            let locale = picked
                .and_then(|index| codes.get(index))
                .and_then(|code| LocaleName::new(code.clone()).ok());

            if let Some(locale) = locale {
                context.commands.send(LocaleCommand::System(locale));
            }
        });
    });
}

/// How wide the system language's dropdown is: as wide as its longest
/// name, so the locale code at its end shows whole, from the settings'
/// wide control up to the widest an open list grows.
fn system_width<'a>(ui: &Ui, shown: impl Iterator<Item = &'a str>) -> f32 {
    Select::fitting_width(ui, shown).max(WIDE_CONTROL)
}

fn choice(ctx: &egui::Context, code: &str, language: &str, territory: &str) -> String {
    if language.is_empty() {
        return code.to_owned();
    }

    let (id, args) = if territory.is_empty() {
        (
            "locales-system-choice",
            vec![("language", language), ("code", code)],
        )
    } else {
        (
            "locales-system-choice-territory",
            vec![
                ("language", language),
                ("territory", territory),
                ("code", code),
            ],
        )
    };

    i18n::text_with(ctx, id, &args).into_owned()
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = &context.refusals.locale else {
        return;
    };

    let id = format!("locales-refused-{}", refusal.code);
    let text = i18n::text_with(ui.ctx(), &id, &[("locale", &refusal.locale)]);
    if refusal_banner(ui, Tone::Error, &text, "locales-dismiss") {
        context.commands.send(LocaleCommand::DismissRefusal);
    }
}
