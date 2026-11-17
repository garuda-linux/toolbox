//! The Kernels page: the installed kernels and the ones the repositories
//! offer, with headers and DKMS modules.

#[cfg(test)]
pub(crate) mod fixtures;
mod row;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::Ui;
use row::Item;
use toolbox_core::Snapshot;
use toolbox_core::features::kernels::view::{self, BootMenus, Listing, Row};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    RowHeights, RowList, Tone, after_skeleton, banner, card, dimmed, empty_state_action,
    hold_content, rich, section_heading, skeleton_rows,
};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_services::boot_menu::Menu;
use toolbox_services::dkms::Dkms;

use super::header::refresh_header_reading;
use super::rows::refused_banner;
use super::{PageContext, PageView, plan_running};
use crate::boot::Boot;
use crate::command::{AppCommand, KernelCommand, KernelEdit};
use crate::i18n;
use crate::i18n::Arg;
use crate::kernels::Kernels;
use crate::page::PageId;

/// Placeholder rows while the system is read, about as many kernels as a
/// system has installed and on offer.
const SKELETON_ROWS: usize = 4;

#[derive(Debug, Default)]
pub struct KernelsPage {
    shown: Shown,
}

/// The rows and counts of one system, queue and dkms answer. `None`
/// without the package databases.
type Shown = RowCache<(Same<Snapshot>, u64, u64, Same<Menu>), Option<Built>>;

#[derive(Debug)]
struct Built {
    rows: Vec<Row>,
    headers_missing: usize,
    modules_missing: usize,
}

impl PageView for KernelsPage {
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        header(ui, context);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("kernels ui").entered();
        plan_running(ui, context);

        let ctx = ui.ctx().clone();
        refusal(ui, context);

        let first_load = ui.make_persistent_id("kernels-content");
        if !context.probed {
            let height = row::estimate(ui);
            let name = i18n::text(&ctx, "kernels-reading");
            hold_content(&ctx, first_load);
            card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name));
            return;
        }

        let built = self.built(context);
        after_skeleton(ui, first_load, |ui| {
            listing(ui, context, (*built).as_ref());
        });
    }
}

impl KernelsPage {
    fn built(&mut self, context: &PageContext<'_>) -> Arc<Option<Built>> {
        let kernels = context.service::<Kernels>();
        let boot = context.service::<Boot>();
        let rest = (
            Same::of(context.snapshot),
            context.revision,
            kernels.dkms_revision(),
            boot.menu_identity(),
        );

        self.shown.get(&[], rest, || {
            let _build = tracing::trace_span!("build rows").entered();
            let menus = BootMenus {
                grub: boot.menu().known(),
                sdboot: boot.menu().sdboot(),
            };

            let found = Listing::of(context.snapshot, menus)?;
            let dkms = kernels.dkms_entries();

            Some(Built {
                rows: found.rows(context.changes, dkms),
                headers_missing: found.headers_missing(context.changes),
                modules_missing: found.modules_missing(context.changes, dkms),
            })
        })
    }
}

/// The kernels once the system was read, or why there are none.
fn listing(ui: &mut Ui, context: &PageContext<'_>, built: Option<&Built>) {
    let ctx = ui.ctx().clone();
    let Some(built) = built else {
        let said = i18n::text(&ctx, "kernels-unavailable");
        let again = i18n::text(&ctx, "common-read-again");
        if empty_state_action(ui, icons::DIALOG_ERROR, &said, "", &again).clicked() {
            context.commands.send(KernelCommand::Refresh);
        }

        return;
    };

    if built.rows.is_empty() {
        banner(ui, Tone::Neutral, &i18n::text(&ctx, "kernels-none"), &[]);
        return;
    }

    notices(ui, context, built);

    let (installed, available): (Vec<Item<'_>>, Vec<Item<'_>>) = built
        .rows
        .iter()
        .map(Item)
        .partition(|item| item.0.kernel.is_installed());

    section(
        ui,
        context,
        ("kernels-installed", icons::CHECKBOX),
        &installed,
    );
    section(
        ui,
        context,
        ("kernels-available", icons::FOLDER_DOWNLOAD),
        &available,
    );
}

fn header(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let reading = context.service::<Kernels>().reading();
    let titles = ("page-kernels", "kernels-subtitle");
    if refresh_header_reading(ui, titles, "kernels-refresh", reading) {
        context.commands.send(KernelCommand::Refresh);
    }

    let release = view::release(context.snapshot);
    if !release.is_empty() {
        let tokens = tokens::current(&ctx);
        let text = i18n::text_with(
            &ctx,
            "kernels-running-release",
            &[("release", Arg::from(release))],
        );

        ui.add_space(ui.tokens().sizes.caption_gap);

        ui.label(rich(ui, &text, Weight::Medium, SMALL, tokens.secondary));
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let refused = context.service::<Kernels>().refusal();
    if refused_banner(ui, refused, Tone::Error, "kernels-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Kernels));
    }
}

/// What dkms said, and the banners that queue what is missing.
fn notices(ui: &mut Ui, context: &PageContext<'_>, built: &Built) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let missing_builds = built.modules_missing;
    let dkms = context.service::<Kernels>().dkms();

    let dkms = match (dkms.error(), dkms.get()) {
        (Some(_), _) => {
            let text = i18n::text(&ctx, "kernels-dkms-failed");
            let again = i18n::text(&ctx, "common-read-again");
            if banner(ui, Tone::Warning, &text, &[&again]).value == Some(0) {
                context.commands.send(KernelCommand::Refresh);
            }

            None
        }
        (None, None) => {
            ui.loading(&i18n::text(&ctx, "kernels-dkms-loading"));
            None
        }
        (None, Some(Dkms::NotInstalled)) => Some("kernels-dkms-unavailable"),
        (None, Some(Dkms::Modules(_))) => (missing_builds == 0).then_some("kernels-dkms-ok"),
    };

    if let Some(id) = dkms {
        ui.label(rich(
            ui,
            &i18n::text(&ctx, id),
            Weight::Regular,
            SMALL,
            tokens.secondary,
        ));
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let offers = [
        (
            built.headers_missing,
            "kernels-headers-banner",
            "kernels-queue-all-headers",
            KernelEdit::AllHeaders,
        ),
        (
            missing_builds,
            "kernels-builds-banner",
            "kernels-queue-all-builds",
            KernelEdit::AllBuilds,
        ),
    ];

    for (count, text, action, command) in offers {
        if count == 0 {
            continue;
        }

        let text = i18n::text_with(&ctx, text, &[("count", Arg::from(count))]);
        let action = i18n::text(&ctx, action);
        if banner(ui, Tone::Warning, &text, &[&action]).value == Some(0) {
            context.commands.send(command);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// A section of kernels, its heading and its rows on the glyph of the
/// section.
fn section(ui: &mut Ui, context: &PageContext<'_>, (id, glyph): (&str, Icon), rows: &[Item<'_>]) {
    if rows.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let count = i18n::text_with(&ctx, "kernels-count", &[("count", Arg::from(rows.len()))]);
    let title = i18n::text(&ctx, id);
    section_heading(ui, glyph, &title, &count);
    ui.add_space(ui.tokens().sizes.heading_gap);

    let heights = RowHeights::Measured {
        estimate: row::estimate(ui),
    };

    let _rows = tracing::trace_span!("rows").entered();
    RowList::new(id, &title, heights).show(ui, rows, |ui, shown, state| {
        dimmed(ui, context.busy, |ui| {
            row::show(ui, context, shown.0, glyph, state);
        });
    });
    ui.add_space(ui.tokens().sizes.section_gap);
}
