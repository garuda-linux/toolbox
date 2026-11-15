//! Closing it keeps the changes, and the page offers it again.

mod lines;

use egui::accesskit::Role;
use egui::{
    Align2, Context, FontId, Id, Label, Painter, Rect, ScrollArea, Sense, Stroke, TextStyle, Ui,
    WidgetInfo, pos2, vec2,
};
use toolbox_core::features::configs::session::{Changes, EditedFile, Session};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{self, SMALL, Weight};
use toolbox_gui_kit::tokens::Tokens;
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, ModalWrapper, Pending, Question, Tone, banner, mono_small,
    pill_button, question, right_buttons,
};

use self::lines::{Covered, Kind, Row};
use super::texts;
use crate::command::ConfigsCommand;
use crate::i18n;
use crate::pages::PageContext;

/// Room above and below a diff line, so the tints of neighbouring rows
/// do not touch the text.
const ROW_PADDING: f32 = 2.0;

/// How the diff's rows are measured, the same for every row.
struct Look {
    font: FontId,
    /// The width of one character of the fixed font.
    column: f32,
    /// The width of one gutter's numbers.
    numbers: f32,
    gap: f32,
    height: f32,
    /// The width the widest line needs, so the sideways scroll reaches it
    /// before it is in sight.
    extent: f32,
}

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
            super::stacked(
                ui,
                |ui| {
                    words(ui, session, changes, &path);
                    diff(ui, changes);
                },
                |ui| buttons(ui, context, session),
            )
        });

    if let Some(command) = outcome.inner {
        context.commands.send(command);
    }

    if discard_question().get(ctx).is_some() {
        ask_discard(ctx, context, &path);
    }

    outcome.closed
}

fn discard_question() -> Pending<()> {
    Pending::new(Id::unique("toolbox_gui::configs::review::discard"))
}

/// Drops an open question, as when the page is left.
pub fn forget(ctx: &Context) {
    discard_question().release(ctx);
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

    let asked = question(ctx, discard_question().dialog(), texts);
    if asked.confirmed {
        context.commands.send(ConfigsCommand::Discard);
    }

    if asked.done {
        discard_question().release(ctx);
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

    let Some(problem) = session.problem() else {
        return;
    };

    let text = texts::review_problem(&ctx, problem.code, path);

    banner(ui, Tone::Error, &text, &[]);
    if let Some(detail) = texts::detail(problem) {
        ui.add(Label::new(mono_small(ui, detail, ui.tokens().text)).wrap());
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

/// A unified diff with both files' line numbers in a gutter and tinted
/// added and removed rows. A rewrite near the size cap has tens of
/// thousands of lines, so only the rows in sight are painted.
fn diff(ui: &mut Ui, changes: &Changes) {
    let ctx = ui.ctx().clone();
    let rows = lines::cached(&ctx, &changes.plain);
    let font = TextStyle::Monospace.resolve(ui.style());
    let (line_height, column) =
        ctx.fonts_mut(|atlas| (atlas.row_height(&font), atlas.glyph_width(&font, ' ')));

    let gap = ui.tokens().sizes.control_gap;
    let numbers = column * f32::from(rows.digits);
    let text_left = numbers.mul_add(2.0, gap * 3.0);
    let look = Look {
        font,
        column,
        numbers,
        gap,
        height: ROW_PADDING.mul_add(2.0, line_height),
        extent: column.mul_add(f32::from(rows.columns) + 2.0, text_left + gap),
    };

    let name = i18n::text(&ctx, "configs-review-diff").into_owned();
    let height = super::text_room(ui);
    let area = ui
        .scope(|ui| {
            // show_rows counts the spacing of this ui between rows.
            ui.spacing_mut().item_spacing.y = 0.0;
            ScrollArea::both()
                .max_height(height)
                .auto_shrink([false, true])
                .show_rows(ui, look.height, rows.lines.len(), |ui, range| {
                    ui.set_min_width(look.extent);
                    let width = look.extent.max(ui.available_width());
                    for row in rows.lines.get(range).unwrap_or_default() {
                        paint_row(ui, row, &look, width);
                    }
                })
        })
        .inner;

    ctx.accesskit_node_builder(area.id, |node| node.set_label(name));
}

fn paint_row(ui: &mut Ui, row: &Row, look: &Look, width: f32) {
    let tokens = ui.tokens();
    let (rect, response) = ui.allocate_exact_size(vec2(width, look.height), Sense::hover());
    let (fill, mark) = match row.kind {
        Kind::Added => (Some(tokens.banner_wash(tokens.success)), tokens.success),
        Kind::Removed => (Some(tokens.banner_wash(tokens.error)), tokens.error),
        Kind::Same | Kind::Note | Kind::Hunk(_) => (None, tokens.secondary),
    };

    let painter = ui.painter();
    if let Some(fill) = fill {
        painter.rect_filled(rect, 0.0, fill);
    }

    let middle = rect.center().y;
    let left = paint_numbers(painter, rect, row, look, &tokens);

    let at = pos2(left, middle);
    let heard = match row.kind {
        // Diff markup is jargon: the hunk header becomes the lines it
        // covers, in words.
        Kind::Hunk(covered) => {
            let words = lines_in_words(ui.ctx(), covered);
            let small = fonts::font(ui.style(), Weight::Regular, SMALL);
            painter.text(at, Align2::LEFT_CENTER, &words, small, tokens.secondary);
            words
        }
        Kind::Same | Kind::Added | Kind::Removed => {
            let marked = row.line.get(..1).unwrap_or_default();
            painter.text(at, Align2::LEFT_CENTER, marked, look.font.clone(), mark);
            let text_at = pos2(look.column.mul_add(2.0, left), middle);
            let text = row.text();
            painter.text(
                text_at,
                Align2::LEFT_CENTER,
                text,
                look.font.clone(),
                tokens.text,
            );
            row.line.clone()
        }
        Kind::Note => {
            painter.text(
                at,
                Align2::LEFT_CENTER,
                &row.line,
                look.font.clone(),
                tokens.secondary,
            );
            row.line.clone()
        }
    };

    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, &heard));
}

/// The old and the new line number and the gutter's line after them.
/// Returns where the line's own text starts.
fn paint_numbers(painter: &Painter, rect: Rect, row: &Row, look: &Look, tokens: &Tokens) -> f32 {
    let middle = rect.center().y;
    let mut left = rect.left() + look.gap;
    for number in [row.old, row.new] {
        left += look.numbers;
        if let Some(number) = number {
            let at = pos2(left, middle);
            painter.text(
                at,
                Align2::RIGHT_CENTER,
                number,
                look.font.clone(),
                tokens.secondary,
            );
        }

        left += look.gap;
    }

    let edge = left - look.gap / 2.0;
    painter.vline(edge, rect.y_range(), Stroke::new(1.0, tokens.outline));
    left
}

fn lines_in_words(ctx: &Context, covered: Covered) -> String {
    let count = (covered.last - covered.first + 1).to_string();
    let (first, last) = (covered.first.to_string(), covered.last.to_string());
    i18n::text_with(
        ctx,
        "configs-review-lines",
        &[("first", &first), ("last", &last), ("count", &count)],
    )
    .into_owned()
}

/// One row at the right: Discard, Edit again, Compare again after a failed
/// save, Save last.
fn buttons(ui: &mut Ui, context: &PageContext<'_>, session: &Session) -> Option<ConfigsCommand> {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let failed = session.problem().is_some();
    let (discard, edit_again, compare_again, save) = (
        text("configs-discard-ask"),
        text("configs-edit-again"),
        text("configs-compare-again"),
        text("configs-save"),
    );

    let labels: Vec<&str> = [discard.as_str(), edit_again.as_str()]
        .into_iter()
        .chain(failed.then_some(compare_again.as_str()))
        .chain([save.as_str()])
        .collect();

    right_buttons(ui, &labels, |ui| {
        let mut chosen = None;
        if pill_button(ui, &discard, ButtonKind::Soft).clicked() {
            discard_question().hold(ui.ctx(), ());
        }

        let idle = !context.busy;
        let again = ui
            .enabled_ui(idle, |ui| pill_button(ui, &edit_again, ButtonKind::Soft))
            .inner;

        if again.clicked() {
            chosen = Some(ConfigsCommand::EditAgain);
        }

        if failed && pill_button(ui, &compare_again, ButtonKind::Soft).clicked() {
            chosen = Some(ConfigsCommand::Reread);
        }

        let saved = ui
            .enabled_ui(idle, |ui| pill_button(ui, &save, ButtonKind::Primary))
            .inner;

        if saved.clicked() {
            chosen = Some(ConfigsCommand::Apply);
        }

        chosen
    })
}
