//! What needs attention: a row per thing to act on, or the checks that
//! passed.

mod checks;
mod rows;

use std::time::SystemTime;

use egui::{Label, Ui};
use toolbox_core::features::maintenance::RunNow;
use toolbox_gui_kit::fonts::{SECTION, SMALL, Weight};
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::widgets::{
    ActionRow, ButtonKind, Tone, banner, card, card_row, icon_button_width, loading_indicator,
    pill_button, refresh_button, rich, split_row,
};
use toolbox_gui_kit::{Loadable, UiExt as _, tokens};
use toolbox_system::{CallError, HealthLevel, Host, Status};

use super::view::Overview;
use super::{HomePage, HomeProblem};
use crate::command::HomeCommand;
use crate::i18n;
use crate::pages::PageContext;

/// One thing to act on, in the order the rows show.
#[derive(Clone, Copy, Debug)]
enum Row {
    Updates,
    Aur,
    Stale,
    Merges,
    Disk,
    Services,
    Cleanup,
    Health(HealthLevel),
    Restart,
}

pub fn show(page: &mut HomePage, ui: &mut Ui, context: &PageContext<'_>, host: Option<&Host>) {
    let Some(reading) = context.home.status else {
        return;
    };

    heading(ui, context, reading);
    ui.add_space(ui.tokens().sizes.card_gap);
    restart_problem(ui, context);

    let Some(status) = reading.get() else {
        if reading.error().is_some() && !reading.is_loading() {
            let text = i18n::text(ui.ctx(), "home-status-failed");
            let again = i18n::text(ui.ctx(), "common-read-again");
            if banner(ui, Tone::Warning, &text, &[&again]) == Some(0) {
                context.commands.send(HomeCommand::Refresh);
            }
        } else {
            ui.loading(&i18n::text(ui.ctx(), "home-checking"));
        }

        return;
    };

    let overview = overview(status, context);
    failures(page, ui, context, &overview);

    let reboot = host.is_some_and(|host| host.reboot_pending);
    if !overview.attention(reboot) {
        if overview.failed.is_empty() {
            checks::all_clear(ui, &overview);
        }

        return;
    }

    card(ui, |ui| {
        let _rows = tracing::trace_span!("rows").entered();
        for (index, row) in rows(&overview, reboot).into_iter().flatten().enumerate() {
            card_row(ui, index == 0, |ui| match row {
                Row::Updates => rows::updates(page, ui, context, &overview),
                Row::Aur => rows::aur(page, ui, context, &overview),
                Row::Stale => rows::stale(ui, context),
                Row::Merges => rows::merges(ui, &overview),
                Row::Disk => rows::disk(ui, context),
                Row::Services => rows::services(ui, context, &overview),
                Row::Cleanup => rows::cleanup(ui, context, &overview),
                Row::Health(level) => rows::health(ui, context, &overview, level),
                Row::Restart => rows::restart(page, ui, context),
            });
        }
    });
}

/// The status with the facts' checks, once they are read.
pub(super) fn overview<'read>(
    status: &'read Status,
    context: &PageContext<'read>,
) -> Overview<'read> {
    let overview = Overview::of(status, SystemTime::now());

    match context.home.facts.and_then(Loadable::get) {
        Some(facts) => overview.with_facts(facts),
        None => overview,
    }
}

/// How many rows the status shows, for the greeting's summary.
pub fn attention_rows(overview: &Overview<'_>, reboot: bool) -> usize {
    rows(overview, reboot).into_iter().flatten().count()
}

/// The rows the status shows, in their order.
fn rows(overview: &Overview<'_>, reboot: bool) -> [Option<Row>; 9] {
    [
        (overview.updates > 0).then_some(Row::Updates),
        (overview.aur_updates > 0).then_some(Row::Aur),
        (overview.stale && overview.updates == 0).then_some(Row::Stale),
        (!overview.merges.is_empty()).then_some(Row::Merges),
        overview.machine.disk_low.then_some(Row::Disk),
        (!overview.machine.failed_units.is_empty()).then_some(Row::Services),
        overview.machine.cleanup.then_some(Row::Cleanup),
        overview.health.map(Row::Health),
        reboot.then_some(Row::Restart),
    ]
}

fn heading(ui: &mut Ui, context: &PageContext<'_>, reading: &Loadable<Status>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    split_row(
        ui,
        icon_button_width(ui),
        |ui| {
            ui.horizontal(|ui| {
                let title = i18n::text(&ctx, "home-status");
                ui.label(rich(ui, &title, Weight::Bold, SECTION, tokens.text));
                if reading.is_loading() && reading.get().is_some() {
                    loading_indicator(ui, &i18n::text(&ctx, "home-checking"));
                }
            });
        },
        |ui| {
            let name = i18n::text(&ctx, "home-refresh");
            if refresh_button(ui, &name) {
                context.commands.send(HomeCommand::Refresh);
            }
        },
    );
}

fn restart_problem(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(HomeProblem::Restart(error)) = context.home.problem else {
        return;
    };

    let id = match error {
        CallError::Denied => "home-restart-denied",
        CallError::Unavailable | CallError::Failed(_) => "home-restart-failed",
    };

    banner(ui, Tone::Error, &i18n::text(ui.ctx(), id), &[]);
    ui.add_space(ui.tokens().sizes.card_gap);
}

/// The reasons are a program's words, so they wait behind a button.
fn failures(page: &mut HomePage, ui: &mut Ui, context: &PageContext<'_>, overview: &Overview<'_>) {
    if overview.failed.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let count = overview.failed.len().to_string();
    let text = i18n::text_with(&ctx, "home-failed", &[("count", &count)]);
    let action = i18n::text(
        &ctx,
        if page.details {
            "home-hide-details"
        } else {
            "home-show-details"
        },
    );

    let again = i18n::text(&ctx, "common-read-again");
    match banner(ui, Tone::Warning, &text, &[&action, &again]) {
        Some(0) => page.details ^= true,
        Some(_) => context.commands.send(HomeCommand::Refresh),
        None => {}
    }

    if page.details {
        let tokens = tokens::current(&ctx);
        for (check, reason) in &overview.failed {
            let id = format!("home-failed-{}", check.as_str());
            let line = i18n::text_with(&ctx, &id, &[("reason", reason)]);
            ui.add(Label::new(rich(ui, &line, Weight::Regular, SMALL, tokens.secondary)).wrap());
        }
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

/// A button that runs a plan at once. It waits while one runs.
pub fn run_button(ui: &mut Ui, context: &PageContext<'_>, id: &str, run: RunNow, kind: ButtonKind) {
    let text = i18n::text(ui.ctx(), id);
    let pressed = ui
        .add_enabled_ui(!context.busy, |ui| pill_button(ui, &text, kind))
        .inner;

    if pressed.clicked() {
        context.commands.send(HomeCommand::RunNow(run));
    }
}

/// A row whose one Soft button runs a plan at once. `ids` are the title,
/// the detail and the button.
pub fn run_row(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (icon, tone): (Icon, Tone),
    [title, detail, button]: [&str; 3],
    run: RunNow,
) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, title);
    let detail = i18n::text(&ctx, detail);
    ActionRow::new(icon, tone, &title)
        .detail(&detail)
        .show(ui, |ui| {
            run_button(ui, context, button, run, ButtonKind::Soft);
        });
}
