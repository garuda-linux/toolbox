//! The next steps and the pages used last. The sidebar holds every page,
//! so the steps are the few that fit the system's state.

use egui::Ui;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{chip, link_tile, rich};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_system::Host;

use super::steps::heading;
use crate::command::UiCommand;
use crate::i18n;
use crate::page::PageId;
use crate::page_ui::{self, PageLook as _};
use crate::pages::PageContext;
use crate::pages::about::tiles;

/// How many steps fit one row.
const STEPS: u8 = 3;

/// Review while changes wait, the Setup Assistant until it ran once on
/// an installed system, then Maintenance and Diagnostics.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, host: Option<&Host>) {
    let ctx = ui.ctx().clone();
    let live = host.is_some_and(|host| host.live_iso);
    let queued = context.changes.len();
    let page = |page: PageId, subtitle: &str| {
        (
            page.icon(),
            i18n::text(&ctx, page.text_id()),
            i18n::text(&ctx, subtitle),
            UiCommand::Navigate(page),
        )
    };

    let steps = [
        (queued > 0).then(|| {
            let count = queued.to_string();
            (
                icons::VIEW_PIM_TASKS,
                i18n::text(&ctx, "command-review-changes"),
                i18n::text_with(&ctx, "home-next-review", &[("count", &count)]),
                UiCommand::ReviewChanges,
            )
        }),
        (!live && !context.home.setup_finished).then(|| page(PageId::Setup, "home-next-setup")),
        Some(page(PageId::Maintenance, "home-next-maintenance")),
        Some(page(PageId::Diagnostics, "home-next-diagnostics")),
    ];

    heading(ui, "home-next");
    ui.add_space(ui.tokens().sizes.card_gap);

    tiles(ui, Some(STEPS), |ui, width| {
        for (icon, title, subtitle, command) in steps.into_iter().flatten().take(usize::from(STEPS))
        {
            if link_tile(ui, icon, &title, &subtitle, width).clicked() {
                context.commands.send(command);
            }
        }
    });

    recent(ui, context);
}

/// A chip per page used last, newest first. A page hidden from the
/// sidebar is no shortcut either.
fn recent(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let pages: Vec<PageId> = context
        .home
        .recent
        .iter()
        .copied()
        .filter(|page| page_ui::listed(&ctx, *page))
        .collect();

    if pages.is_empty() {
        return;
    }

    ui.add_space(ui.tokens().sizes.card_gap);
    ui.horizontal_wrapped(|ui| {
        let tokens = tokens::current(&ctx);
        let label = i18n::text(&ctx, "home-recent");
        ui.label(rich(ui, &label, Weight::Regular, SMALL, tokens.secondary));

        for page in pages {
            let name = i18n::text(&ctx, page.text_id());
            if chip(ui, &name, None, false).clicked() {
                context.commands.send(UiCommand::Navigate(page));
            }
        }
    });
}
