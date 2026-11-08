//! Closing it keeps the changes, and the page offers it again.

use egui::{Context, Id, Label, RichText, ScrollArea, Ui};
use toolbox_core::features::configs::hunk_lines;
use toolbox_core::features::configs::session::{Changes, EditedFile, Session};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, ModalWrapper, Question, Tone, banner, pill_button, pill_width,
    question,
};

use super::texts;
use crate::command::ConfigsCommand;
use crate::i18n;
use crate::pages::PageContext;

const ROOM_BELOW: f32 = 220.0;

/// Whether the user closed the review without an answer, which keeps
/// the changes.
pub fn show(ctx: &Context, context: &PageContext<'_>, session: &Session) -> bool {
    let Some(changes) = session.changes() else {
        return false;
    };

    let path = texts::path(session.path());
    let title = i18n::text_with(ctx, "configs-review-title", &[("path", &path)]);
    let close = i18n::text(ctx, "configs-review-close");
    let outcome = ModalWrapper::new(Id::unique("toolbox_gui::configs::review"), &title)
        .title_bar(&close)
        .width(DialogWidth::Wide)
        .show(ctx, |ui| {
            words(ui, session, changes, &path);
            diff(ui, changes);
            buttons(ui, context, session)
        });

    if let Some(command) = outcome.inner {
        context.commands.send(command);
    }

    if asking_discard(ctx) {
        ask_discard(ctx, context, &path);
    }

    outcome.closed
}

fn discard_id() -> Id {
    Id::unique("toolbox_gui::configs::review::discard")
}

fn asking_discard(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(discard_id()).unwrap_or(false))
}

fn set_asking_discard(ctx: &Context, asking: bool) {
    ctx.data_mut(|data| data.insert_temp(discard_id(), asking));
}

/// Discarding throws the user's edits away, so it asks first.
fn ask_discard(ctx: &Context, context: &PageContext<'_>, path: &str) {
    let title = i18n::text(ctx, "configs-discard-title");
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text_with(ctx, "configs-discard-question", &[("path", path)]);
    let cancel = i18n::text(ctx, "configs-discard-keep");
    let confirm = i18n::text(ctx, "configs-discard");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = question(ctx, discard_id().with("dialog"), texts);
    if asked.confirmed {
        context.commands.send(ConfigsCommand::Discard);
    }

    if asked.done {
        set_asking_discard(ctx, false);
    }
}

fn words(ui: &mut Ui, session: &Session, changes: &Changes, path: &str) {
    let ctx = ui.ctx().clone();
    let target = texts::path(session.target());
    if target != path {
        let text = i18n::text_with(
            &ctx,
            "configs-review-link",
            &[("path", path), ("target", &target)],
        );

        ui.add(Label::new(text).wrap());
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let backup = session.backup().unwrap_or_default();
    let (added, removed) = (changes.added.to_string(), changes.removed.to_string());
    let words = [
        ("added", added.as_str()),
        ("removed", removed.as_str()),
        ("backup", backup.as_str()),
    ];

    // A file of the user is written as the user, without a password.
    let id = match session.target() {
        Some(EditedFile::Home(_)) => "configs-review-summary-home",
        Some(EditedFile::Root(_)) | None => "configs-review-summary",
    };

    let summary = i18n::text_with(&ctx, id, &words);
    ui.add(Label::new(summary).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);

    let problem = session.problem();
    let text = match problem.code {
        "" => return,
        "write-failed" => {
            i18n::text_with(&ctx, "configs-review-write-failed", &[("path", path)]).into_owned()
        }
        code => texts::problem(&ctx, code, path),
    };

    banner(ui, Tone::Error, &text, &[]);
    if let Some(detail) = texts::detail(problem) {
        ui.add(Label::new(RichText::new(detail).small().monospace()).wrap());
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn diff(ui: &mut Ui, changes: &Changes) {
    let tokens = ui.tokens();
    let name = i18n::text(ui.ctx(), "configs-review-diff").into_owned();
    let height = (ui.available_height() - ROOM_BELOW).max(super::MIN_TEXT_HEIGHT);
    let area = ScrollArea::both()
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for line in changes.plain.lines() {
                // Diff markup is jargon: the hunk header becomes the lines it
                // covers, in words.
                if let Some((first, last)) = hunk_lines(line) {
                    let count = (last - first + 1).to_string();
                    let (first, last) = (first.to_string(), last.to_string());
                    let words = i18n::text_with(
                        ui.ctx(),
                        "configs-review-lines",
                        &[("first", &first), ("last", &last), ("count", &count)],
                    );

                    ui.add_space(ui.tokens().sizes.tight_gap);
                    ui.label(
                        RichText::new(words.as_ref())
                            .small()
                            .color(tokens.secondary),
                    );
                    continue;
                }

                let color = match line.chars().next() {
                    Some('+') => tokens.success,
                    Some('-') => tokens.error,
                    _ => tokens.secondary,
                };

                ui.add(Label::new(RichText::new(line).monospace().color(color)).extend());
            }
        });

    ui.ctx()
        .accesskit_node_builder(area.id, |node| node.set_label(name));
}

/// Discard and Edit again at the left, Compare again after a failed save,
/// Save last.
fn buttons(ui: &mut Ui, context: &PageContext<'_>, session: &Session) -> Option<ConfigsCommand> {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let failed = !session.problem().code.is_empty();
    ui.add_space(ui.tokens().sizes.dialog_gap);
    let mut labels = vec![
        text("configs-discard-ask"),
        text("configs-edit-again"),
        text("configs-save"),
    ];

    if failed {
        labels.push(text("configs-compare-again"));
    }

    ui.horizontal(|ui| {
        let gap = ui.tokens().sizes.control_gap;
        ui.spacing_mut().item_spacing.x = gap;
        let count = f32::from(u8::try_from(labels.len()).unwrap_or(u8::MAX));
        let buttons = labels
            .iter()
            .map(|label| pill_width(ui, label))
            .sum::<f32>();

        let width = gap.mul_add(count - 1.0, buttons);
        ui.add_space((ui.available_width() - width).max(0.0));
        let mut chosen = None;
        if pill_button(ui, &text("configs-discard-ask"), ButtonKind::Soft).clicked() {
            set_asking_discard(ui.ctx(), true);
        }

        let idle = !context.busy;
        let again = ui
            .add_enabled_ui(idle, |ui| {
                pill_button(ui, &text("configs-edit-again"), ButtonKind::Soft)
            })
            .inner;

        if again.clicked() {
            chosen = Some(ConfigsCommand::EditAgain);
        }

        if failed && pill_button(ui, &text("configs-compare-again"), ButtonKind::Soft).clicked() {
            chosen = Some(ConfigsCommand::Reread);
        }

        let save = ui
            .add_enabled_ui(idle, |ui| {
                pill_button(ui, &text("configs-save"), ButtonKind::Primary)
            })
            .inner;

        if save.clicked() {
            chosen = Some(ConfigsCommand::Apply);
        }

        chosen
    })
    .inner
}
