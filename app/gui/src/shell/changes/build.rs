//! The lines under a queued AUR build: where its order stands, what the
//! build does, and why a setup takes paru's own way.

use egui::{Label, Ui};
use toolbox_core::features::packages::aur::build::Outline;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{counted_disclosure, rich};

use crate::i18n::{self, Arg};

/// What the popover says about one queued AUR build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildNote {
    /// Its order is being read.
    Reading,
    /// Its order has not come back yet.
    Waiting,
    /// Its order could not be read. The text id names the reason, and its
    /// `$name` is the package.
    Unread(&'static str),
    /// Its order is read: what the build does.
    Ready(Outline),
    /// The setup takes paru's own way. The text id names the reason, see
    /// `route_text_id`.
    Old(&'static str),
}

pub fn lines(ui: &mut Ui, key: &str, note: &BuildNote) {
    let ctx = ui.ctx().clone();
    match note {
        BuildNote::Reading => secondary(ui, &i18n::text(&ctx, "changes-build-reading")),
        BuildNote::Waiting => secondary(ui, &i18n::text(&ctx, "changes-build-waiting")),
        BuildNote::Unread(id) => {
            let text = i18n::text_with(&ctx, id, &[("name", Arg::from(key))]);
            secondary(ui, &text);
        }
        BuildNote::Old(reason_id) => {
            let reason = i18n::text(&ctx, reason_id);
            let text = i18n::text_with(&ctx, "aur-old-path", &[("reason", Arg::from(&*reason))]);
            secondary(ui, &text);
            secondary(ui, &i18n::text(&ctx, "changes-old-path-password"));
        }
        BuildNote::Ready(outline) => ready(ui, key, outline),
    }
}

/// What the build does. The list folds open with every package it builds and
/// every repository package it installs, none left out.
fn ready(ui: &mut Ui, key: &str, outline: &Outline) {
    let ctx = ui.ctx().clone();
    let bases = outline
        .bases
        .iter()
        .map(toolbox_plan::PackageName::as_str)
        .collect::<Vec<_>>()
        .join(", ");

    let text = i18n::text_with(
        &ctx,
        "changes-build-ready",
        &[
            ("bases", Arg::from(bases.as_str())),
            ("count", Arg::from(outline.dependencies.len())),
        ],
    );

    secondary(ui, &text);

    let total = outline.bases.len() + outline.dependencies.len();
    let open_id = ui.scope_id().with(("aur-build-packages", key));
    let open = ui
        .data(|data| data.get_temp::<bool>(open_id))
        .unwrap_or(false);

    let title = i18n::text(&ctx, "changes-build-packages");
    if counted_disclosure(ui, &title, &total.to_string(), open).clicked() {
        ui.data_mut(|data| data.insert_temp::<bool>(open_id, !open));
    }

    if open {
        for name in outline.bases.iter().chain(&outline.dependencies) {
            secondary(ui, name.as_str());
        }
    }
}

fn secondary(ui: &mut Ui, text: &str) {
    let tokens = tokens::current(ui.ctx());
    ui.add(Label::new(rich(ui, text, Weight::Regular, SMALL, tokens.secondary)).wrap());
}
