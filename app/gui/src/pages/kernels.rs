//! The Kernels page: the installed kernels and the ones the repositories
//! offer, with headers and DKMS modules.

#[cfg(test)]
mod fixtures;
mod row;
#[cfg(test)]
mod tests;
pub mod view;

use egui::Ui;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    RowHeights, RowList, Tone, after_skeleton, banner, card, dimmed, hold_content, rich,
    section_heading, skeleton_rows,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use view::{Dkms, Listing, Row};

use super::header::refresh_header;
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::{KernelCommand, KernelEdit};
use crate::i18n;

/// Placeholder rows while the system is read, about as many kernels as a
/// system has installed and on offer.
const SKELETON_ROWS: usize = 4;

#[derive(Debug, Default)]
pub struct KernelsPage;

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

        after_skeleton(ui, first_load, |ui| listing(ui, context));
    }
}

/// The kernels once the system was read, or why there are none.
fn listing(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let Some(listing) = Listing::of(context.snapshot) else {
        banner(
            ui,
            Tone::Warning,
            &i18n::text(&ctx, "kernels-unavailable"),
            &[],
        );
        return;
    };

    if listing.kernels().is_empty() {
        banner(ui, Tone::Neutral, &i18n::text(&ctx, "kernels-none"), &[]);
        return;
    }

    notices(ui, context, &listing);

    let rows = listing.rows(context.changes, context.dkms);
    let (installed, available): (Vec<&Row<'_>>, Vec<&Row<'_>>) =
        rows.iter().partition(|row| row.kernel.is_installed());

    section(ui, context, "kernels-installed", &installed);
    section(ui, context, "kernels-available", &available);
}

fn header(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    if refresh_header(ui, "page-kernels", "kernels-subtitle", "kernels-refresh") {
        context.commands.send(KernelCommand::Refresh);
    }

    let release = view::release(context.snapshot);
    if !release.is_empty() {
        let tokens = tokens::current(&ctx);
        let text = i18n::text_with(&ctx, "kernels-running-release", &[("release", release)]);
        ui.add_space(ui.tokens().sizes.caption_gap);
        ui.label(rich(ui, &text, Weight::Medium, SMALL, tokens.secondary));
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refused) = &context.refusals.kernel else {
        return;
    };

    let id = format!("kernels-refusal-{}", refused.refusal.as_str());
    let text = i18n::text_with(ui.ctx(), &id, &[("name", refused.kernel.as_str())]);
    if refusal_banner(ui, Tone::Warning, &text, "kernels-dismiss") {
        context.commands.send(KernelCommand::DismissRefusal);
    }
}

/// What dkms said, and the banners that queue what is missing.
fn notices(ui: &mut Ui, context: &PageContext<'_>, listing: &Listing) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let missing_builds = listing.modules_missing(context.changes, context.dkms);
    let dkms = match (context.dkms.error(), context.dkms.get()) {
        (Some(_), _) => {
            let text = i18n::text(&ctx, "kernels-dkms-failed");
            let again = i18n::text(&ctx, "common-read-again");
            if banner(ui, Tone::Warning, &text, &[&again]) == Some(0) {
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

    let missing_headers = listing.headers_missing(context.changes);
    let offers = [
        (
            missing_headers,
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

        let text = i18n::text_with(&ctx, text, &[("count", &count.to_string())]);
        let action = i18n::text(&ctx, action);
        if banner(ui, Tone::Warning, &text, &[&action]) == Some(0) {
            context.commands.send(command);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

fn section(ui: &mut Ui, context: &PageContext<'_>, id: &str, rows: &[&Row<'_>]) {
    if rows.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let count = i18n::text_with(&ctx, "kernels-count", &[("count", &rows.len().to_string())]);
    let title = i18n::text(&ctx, id);
    section_heading(ui, icons::RUN_BUILD, &title, &count);
    ui.add_space(ui.tokens().sizes.heading_gap);

    let heights = RowHeights::Measured {
        estimate: row::estimate(ui),
    };

    let _rows = tracing::trace_span!("rows").entered();
    RowList::new(id, &title, heights).show(ui, rows, |ui, shown, state| {
        dimmed(ui, context.busy, |ui| row::show(ui, context, shown, state));
    });
    ui.add_space(ui.tokens().sizes.section_gap);
}
