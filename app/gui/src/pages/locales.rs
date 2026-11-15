//! Every click waits in the queue. Apply edits `locale.gen`, runs
//! `locale-gen` once and sets the system language last.

mod list;
mod packs;
mod system;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::locales::view::{locales_known, packs_known};
use toolbox_core::features::locales::{Filter, PackOrder};
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::widgets::{
    Tone, banner, card, empty_state, empty_state_action, page_toolbar, row_height, skeleton_rows,
    text_block_height, toggle_chips,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::{refused_banner, sort_select};
use super::{PageContext, PageView, plan_running};
use crate::command::{AppCommand, LocaleCommand};
use crate::i18n;
use crate::locales::Locales;
use crate::page::PageId;

/// Placeholder rows while the system is read, about as many locales as
/// fill a window.
const SKELETON_ROWS: usize = 8;

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
    /// The Language packs tab's own search, kept across a tab switch.
    pack_search: String,
    order: PackOrder,
    locales: list::Rows,
    packs: packs::Rows,
    system: system::Choices,
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
            if context.service::<Locales>().refusal().is_some() {
                context
                    .commands
                    .send(AppCommand::DismissRefusal(PageId::Locales));
            }
        }

        if done.refresh {
            context.commands.send(LocaleCommand::Refresh);
        }

        page_toolbar(
            ui,
            self,
            |page, ui| match page.tab {
                Tab::Locales => {
                    let field = &mut page.filter.search;
                    _ = search(ui, field, "locales-search", "locales-search-hint");
                }
                Tab::Packs => {
                    let field = &mut page.pack_search;
                    _ = search(
                        ui,
                        field,
                        "locales-packs-search",
                        "locales-packs-search-hint",
                    );
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
            reading(ui, "locales-reading");
            return;
        }

        if !locales_known(context.snapshot) {
            // No locale.gen is how the system is made, not a failed read.
            empty_state(
                ui,
                icons::HELP_ABOUT,
                &i18n::text(&ctx, "locales-unavailable"),
                &i18n::text(&ctx, "locales-unavailable-detail"),
            );
            return;
        }

        system::show(ui, context, &mut self.system);
        ui.add_space(ui.tokens().sizes.card_gap);
        self.chips(ui);
        ui.add_space(ui.tokens().sizes.card_gap);

        let items = list::rows(ui, context, &self.filter, &mut self.locales);
        if items.is_empty() {
            let shown = (icons::SYSTEM_SEARCH, "locales-none");
            nothing(ui, shown, &mut self.filter.search);
            return;
        }

        ui.enabled_ui(!context.busy, |ui| list::show(ui, context, &items));
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
        })
        .value;

        match flipped {
            Some(Chip::Utf8) => self.filter.utf8_only ^= true,
            Some(Chip::Enabled) => self.filter.enabled_only ^= true,
            None => {}
        }
    }

    fn packs(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        if !context.probed {
            reading(ui, "locales-packs-reading");
            return;
        }

        if !packs_known(context.snapshot, context.changes) {
            unread(ui, context, "locales-packs-unavailable");
            return;
        }

        let listed = packs::rows(
            ui,
            context,
            (self.order, &self.pack_search),
            &mut self.packs,
        );

        let missing = listed.missing;
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
        if banner(ui, tone, &intro, action.as_slice()).value == Some(0) {
            context.commands.send(LocaleCommand::AddMissingPacks);
        }

        ui.add_space(ui.tokens().sizes.card_gap);

        if listed.all == 0 {
            empty_state(
                ui,
                icons::PACKAGE,
                &i18n::text(&ctx, "locales-packs-none"),
                &i18n::text(&ctx, "locales-packs-none-detail"),
            );
            return;
        }

        if listed.items.is_empty() {
            let shown = (icons::SYSTEM_SEARCH, "locales-packs-no-match");
            nothing(ui, shown, &mut self.pack_search);
            return;
        }

        ui.enabled_ui(!context.busy, |ui| packs::show(ui, context, &listed.items));
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

/// Placeholder rows of a tab's list, named by `id`.
fn reading(ui: &mut Ui, id: &str) {
    let name = i18n::text(ui.ctx(), id);
    let height = row_height(ui, text_block_height(ui, 1));
    card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name));
}

/// A tab whose list could not be read: `id` above its `-detail`.
fn unread(ui: &mut Ui, context: &PageContext<'_>, id: &str) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, id);
    let detail = i18n::text(&ctx, &format!("{id}-detail"));
    let again = i18n::text(&ctx, "common-read-again");
    if empty_state_action(ui, icons::HELP_ABOUT, &title, &detail, &again).clicked() {
        context.commands.send(LocaleCommand::Refresh);
    }
}

/// A list the search or the chips leave empty: `id` above its `-detail`,
/// with Clear search under it while there is a search to clear.
fn nothing(ui: &mut Ui, (icon, id): (Icon, &str), search: &mut String) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, id);
    let detail = i18n::text(&ctx, &format!("{id}-detail"));
    if search.is_empty() {
        _ = empty_state(ui, icon, &title, &detail);
        return;
    }

    let clear = i18n::text(&ctx, "search-clear");
    if empty_state_action(ui, icon, &title, &detail, &clear).clicked() {
        search.clear();
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let refused = context.service::<Locales>().refusal();
    if refused_banner(ui, refused, Tone::Error, "locales-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Locales));
    }
}
