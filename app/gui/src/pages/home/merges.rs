//! Home's dialog of pacman's copies. Every choice joins the queue.

mod detail;
mod edit;

use std::collections::BTreeMap;

use egui::{Context, Id, Label, Layout, ScrollArea, Ui, vec2};
use toolbox_core::features::merges::{CachedComparison, Pick};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::theme::show_scroll;
use toolbox_gui_kit::widgets::{
    DialogWidth, ModalWrapper, NavLook, Pending, Tone, banner, nav_row, rich,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::{CopyKind, MergeCopy};
use toolbox_system::MergePair;

use crate::command::HomeCommand;
use crate::home::Home;
use crate::i18n;
use crate::pages::PageContext;

/// Wide enough for `/etc/mkinitcpio.conf`, a longer path is cut.
const LIST_WIDTH: f32 = 240.0;

/// Room for a change or two with their context.
const BODY_HEIGHT: f32 = 480.0;

#[derive(Debug, Default)]
pub(super) struct MergeDialog {
    shown: usize,
    picks: BTreeMap<MergeCopy, Vec<Pick>>,
    editing: Option<(MergeCopy, String)>,
    /// Built again only when a text changed, never once per frame.
    compared: BTreeMap<MergeCopy, CachedComparison>,
}

impl MergeDialog {
    fn compare(&mut self, copy: &MergeCopy, current: &str, copied: &str) {
        let held = self
            .compared
            .get(copy)
            .is_some_and(|cached| cached.comparison().holds(current, copied));

        if !held {
            self.compared.insert(
                copy.clone(),
                CachedComparison::new(current.to_owned(), copied.to_owned()),
            );
        }
    }

    fn pick(&mut self, copy: &MergeCopy, index: usize, pick: Pick) {
        let picks = self.picks.entry(copy.clone()).or_default();
        if picks.len() <= index {
            picks.resize(index + 1, Pick::Keep);
        }

        if let Some(slot) = picks.get_mut(index) {
            *slot = pick;
        }
    }
}

pub fn open(dialog: &mut Option<MergeDialog>, context: &PageContext<'_>) {
    *dialog = Some(MergeDialog::default());
    context.commands.send(HomeCommand::ReadMerges);
}

/// Whether the user closed the dialog.
pub fn show(dialog: &mut MergeDialog, ctx: &Context, context: &PageContext<'_>) -> bool {
    let title = i18n::text(ctx, "home-merges-title");
    let close = i18n::text(ctx, "dialog-close");
    let pairs = context.service::<Home>().merges().pairs();

    let closed = ModalWrapper::new(Id::unique("toolbox_gui::home::merges"), &title)
        .title_bar(&close)
        .width(DialogWidth::Wide)
        .show(ctx, |ui| match pairs.get() {
            Some(pairs) if !pairs.is_empty() => body(ui, dialog, context, pairs),
            Some(_) => {
                let none = i18n::text(ctx, "home-merges-none");
                ui.label(rich(
                    ui,
                    &none,
                    Weight::Regular,
                    BODY,
                    ui.tokens().secondary,
                ));
            }
            None if pairs.error().is_some() => {
                let failed = i18n::text(ctx, "home-merges-failed");
                let again = i18n::text(ctx, "common-read-again");
                if banner(ui, Tone::Error, &failed, &[&again]).value == Some(0) {
                    context.commands.send(HomeCommand::ReadMerges);
                }
            }
            None => {
                ui.loading(&i18n::text(ctx, "home-merges-reading"));
            }
        })
        .closed;

    let cancelled = cancel().get(ctx).is_some();
    if cancelled {
        cancel().release(ctx);
    }

    closed || cancelled
}

fn body(ui: &mut Ui, dialog: &mut MergeDialog, context: &PageContext<'_>, pairs: &[MergePair]) {
    dialog.shown = dialog.shown.min(pairs.len().saturating_sub(1));
    let height = ui.available_height().min(BODY_HEIGHT);
    let row = vec2(ui.available_width(), height);

    ui.allocate_ui_with_layout(row, Layout::left_to_right(egui::Align::Min), |ui| {
        ui.allocate_ui_with_layout(
            vec2(LIST_WIDTH, height),
            Layout::top_down(egui::Align::Min),
            |ui| list(ui, dialog, context, pairs),
        );
        ui.separator();
        ui.vertical(|ui| {
            if let Some(pair) = pairs.get(dialog.shown) {
                if dialog
                    .editing
                    .as_ref()
                    .is_some_and(|(copy, _)| *copy == pair.copy)
                {
                    edit::show(ui, dialog, context, pair);
                } else {
                    detail::show(ui, dialog, context, pair);
                }
            }
        });
    });
}

fn list(ui: &mut Ui, dialog: &mut MergeDialog, context: &PageContext<'_>, pairs: &[MergePair]) {
    let ctx = ui.ctx().clone();
    let heading = i18n::text(&ctx, "home-merges-files");
    ui.add(Label::new(rich(
        ui,
        &heading,
        Weight::SemiBold,
        BODY,
        ui.tokens().text,
    )));
    ui.add_space(ui.tokens().sizes.control_gap);

    let look = NavLook {
        expanded: true,
        height: ui.tokens().sizes.nav_row_height,
    };

    show_scroll(
        ui,
        ScrollArea::vertical().id_salt("toolbox_gui::home::merges::list"),
        |ui| {
            for (index, pair) in pairs.iter().enumerate() {
                let queued = toolbox_core::features::merges::queued(context.changes, &pair.copy);
                let label = entry_label(&ctx, &pair.copy, queued.is_some());
                if nav_row(ui, icons::VCS_DIFF, &label, index == dialog.shown, look).clicked() {
                    dialog.shown = index;
                    dialog.editing = None;
                }
            }
        },
    );
}

fn entry_label(ctx: &Context, copy: &MergeCopy, queued: bool) -> String {
    let name = copy.original().as_str();

    if queued {
        let word = i18n::text(ctx, "home-merges-queued");
        format!("{name}, {word}")
    } else {
        name.to_owned()
    }
}

/// A saved copy is the user's own version coming back.
const fn take_id(kind: CopyKind) -> &'static str {
    match kind {
        CopyKind::Saved | CopyKind::OlderSave => "home-merges-take-saved",
        CopyKind::New | CopyKind::Orig => "home-merges-take-new",
    }
}

const fn take_file_id(kind: CopyKind) -> &'static str {
    match kind {
        CopyKind::Saved | CopyKind::OlderSave => "home-merges-restore-file",
        CopyKind::New | CopyKind::Orig => "home-merges-use-file",
    }
}

/// Cancel closes the dialog and keeps the queued changes, as the title's
/// close does.
pub(super) fn cancel() -> Pending<()> {
    Pending::new(Id::unique("toolbox_gui::home::merges::cancel"))
}
