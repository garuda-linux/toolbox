//! The queued changes: a pill floating at the bottom right of the page
//! while there are any, with Apply, and a popover above it that lists
//! them, each with its own remove button, and removes all after a
//! question.

mod badge;

use egui::{
    Align2, Area, Context, CornerRadius, Id, Label, Margin, Order, Pos2, Rect, Stroke, Ui, Vec2,
    vec2,
};
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::frost::Glass;
use toolbox_gui_kit::motion::{self, Timing};
use toolbox_gui_kit::theme::POPOVER_RADIUS;
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, Question, Tone, icon_button, pill_button, pill_button_at_least,
    pill_width, question, rich, split_row, tag,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};

use self::badge::badge;
use super::PANE_TITLE;
use crate::command::{AppCommand, Command, CommandSender, UiCommand};
use crate::i18n;
use crate::i18n::changes::sentence;

const POPOVER_WIDTH: f32 = 380.0;
/// The list's height before it scrolls, in points.
const LIST_HEIGHT: f32 = 320.0;
/// The room a row keeps at its right for its remove button.
const REMOVE_WIDTH: f32 = 32.0;
/// The cell of a row's dot: its width and the first line's height.
const DOT_CELL: Vec2 = vec2(8.0, 18.0);

/// The pill's own state, which only the window cares about.
#[derive(Debug, Default)]
pub struct ChangesState {
    pub open: bool,
    /// A key opened the popover, so it shows at once rather than rising.
    pub keyed: bool,
    /// A destructive command waiting for the user's yes, which the app
    /// sets. Its question shows over the window.
    pub confirming: Option<UiCommand>,
}

/// What is queued, and the id of the text that says why Apply waits.
#[derive(Clone, Copy, Debug)]
pub struct Queued<'view> {
    pub changes: &'view [ChangeView],
    pub held: Option<&'static str>,
}

/// In `band`, the room below the page, while the Console is closed.
pub fn floating(
    ctx: &Context,
    band: Rect,
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) {
    if queued.changes.is_empty() {
        *state = ChangesState::default();
        return;
    }

    let inset = tokens::current(ctx).sizes.corner_inset.x;
    let corner = band.right_center() - vec2(inset, 0.0);
    let pill_rect = Area::new(Id::unique("toolbox_gui::changes::pill"))
        .order(Order::Foreground)
        .pivot(Align2::RIGHT_CENTER)
        .fixed_pos(corner)
        .show(ctx, |ui| pill(ui, queued, state, commands))
        .response
        .rect;

    overlays(ctx, pill_rect, queued, state, commands);
}

/// In the Console's header while it is open, so nothing floats over the
/// rows above it: the count, Review and Apply without the pill's words.
/// `ui` lays out right to left, beside the header's Close.
pub fn inline(ui: &mut Ui, queued: Queued<'_>, state: &mut ChangesState, commands: &CommandSender) {
    if queued.changes.is_empty() {
        *state = ChangesState::default();
        return;
    }

    let ctx = ui.ctx().clone();
    let count = queued.changes.len().to_string();
    let removes = queued.changes.iter().any(ChangeView::removes);
    let badge = badge(ui, &count);
    let words = i18n::text_with(&ctx, "changes-pill", &[("count", &count)]);
    let label = format!("{count} {words}");
    badge.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));

    let review = review_button(ui, state);
    let (apply, show) = apply_button(ui, queued.held, commands, removes);
    if show {
        state.open = true;
    }

    let rect = apply.rect.union(review.rect).union(badge.rect);
    overlays(&ctx, rect, queued, state, commands);
}

fn overlays(
    ctx: &Context,
    pill: Rect,
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) {
    let Queued {
        changes: list,
        held,
    } = queued;

    // It rises from the pill's side while it fades in. Closed, it goes at
    // once, and the next opening starts low again. A key's opening jumps.
    let id = Id::unique("toolbox_gui::changes::popover");
    if state.open && std::mem::take(&mut state.keyed) {
        motion::settle(ctx, id.with("shown"), true);
    }

    let timing = Timing::new(motion::POPOVER_ENTER, 0.0);
    let shown = motion::presence(ctx, id.with("shown"), state.open, timing);
    if state.open {
        let gap = tokens::current(ctx).sizes.popover_gap;
        let rise = (1.0 - shown) * motion::POPOVER_RISE;
        let above = Pos2::new(pill.right(), pill.top() - gap + rise);
        Area::new(id)
            .order(Order::Foreground)
            .pivot(Align2::RIGHT_BOTTOM)
            .fixed_pos(above)
            .fade_in(false)
            .show(ctx, |ui| {
                ui.multiply_opacity(shown);
                popover(ui, list, held, commands);
            });
    }

    if state.confirming == Some(UiCommand::ClearChanges) {
        let count = list.len().to_string();
        ask_to_clear(ctx, &count, &mut state.confirming, commands);
    }
}

fn pill(
    ui: &mut Ui,
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) -> Rect {
    let (count, held) = (queued.changes.len(), queued.held);
    let removes = queued.changes.iter().any(ChangeView::removes);
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let count_text = count.to_string();

    let padding = tokens.sizes.pill_padding;
    raised(ui, CornerRadius::same(u8::MAX), padding, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(tokens.sizes.inset);
            badge(ui, &count_text);
            let text = i18n::text_with(&ctx, "changes-pill", &[("count", &count_text)]);
            ui.label(rich(ui, &text, Weight::SemiBold, BODY, tokens.text));
            ui.add_space(tokens.sizes.tight_gap);
            review_button(ui, state);

            // Open, the list carries the one Apply.
            if !state.open && apply_button(ui, held, commands, removes).1 {
                state.open = true;
            }
        });
    })
}

fn review_button(ui: &mut Ui, state: &mut ChangesState) -> egui::Response {
    let toggle = if state.open {
        "changes-hide"
    } else {
        "changes-review"
    };

    // As wide as the longer of its two texts, so the pill keeps its width.
    let width = ["changes-review", "changes-hide"]
        .map(|id| pill_width(ui, &i18n::text(ui.ctx(), id)))
        .into_iter()
        .fold(0.0, f32::max);

    let review = pill_button_at_least(ui, &i18n::text(ui.ctx(), toggle), ButtonKind::Soft, width);
    if review.clicked() {
        state.open = !state.open;
        state.keyed = !review.clicked_by(egui::PointerButton::Primary);
    }

    if state.open && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.open = false;
        review.request_focus();
    }

    let open = state.open;
    ui.ctx()
        .accesskit_node_builder(review.id, |node| node.set_expanded(open));
    review
}

/// Applies the queue, or says to show it first when it `removes`
/// something, so a removal queued days ago is never applied unseen.
fn apply_button(
    ui: &mut Ui,
    held: Option<&'static str>,
    commands: &CommandSender,
    removes: bool,
) -> (egui::Response, bool) {
    let text = i18n::text(ui.ctx(), "changes-apply");
    let apply = apply_pill(ui, &text, held);
    if apply.clicked() && !removes {
        commands.send(UiCommand::Apply);
    }

    let show = apply.clicked() && removes;
    (apply, show)
}

/// A disabled button tells no screen reader why, so the reason goes into
/// its description as well as its tooltip.
fn apply_pill(ui: &mut Ui, text: &str, held: Option<&'static str>) -> egui::Response {
    let apply = ui
        .add_enabled_ui(held.is_none(), |ui| {
            pill_button(ui, text, ButtonKind::Primary)
        })
        .inner;

    let Some(id) = held else {
        return apply;
    };

    let why = i18n::text(ui.ctx(), id).into_owned();
    ui.ctx()
        .accesskit_node_builder(apply.id, |node| node.set_description(why.as_str()));
    apply.on_disabled_hover_text(why)
}

fn popover(
    ui: &mut Ui,
    changes: &[ChangeView],
    held: Option<&'static str>,
    commands: &CommandSender,
) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let count = changes.len().to_string();

    raised(
        ui,
        CornerRadius::same(POPOVER_RADIUS),
        tokens.sizes.popover_padding,
        |ui| {
            ui.set_width(POPOVER_WIDTH);
            ui.name_pane(&i18n::text(&ctx, "pane-changes"));
            ui.horizontal(|ui| {
                ui.label(rich(
                    ui,
                    &i18n::text(&ctx, "changes-title"),
                    Weight::Bold,
                    PANE_TITLE,
                    tokens.text,
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(rich(
                        ui,
                        &i18n::text(&ctx, "changes-together"),
                        Weight::Regular,
                        SMALL,
                        tokens.secondary,
                    ));
                });
            });
            ui.add_space(tokens.sizes.control_gap);

            egui::ScrollArea::vertical()
                .max_height(LIST_HEIGHT)
                .show(ui, |ui| {
                    for change in changes {
                        row(ui, change, commands);
                    }
                });

            ui.add_space(tokens.sizes.control_gap);
            let top = ui.cursor().top();
            ui.painter().hline(
                ui.max_rect().x_range(),
                top,
                Stroke::new(1.0, tokens.outline),
            );
            ui.add_space(tokens.sizes.card_gap);

            footer(ui, &count, held, commands);
        },
    );
}

/// Remove all at the left, Apply at the right.
fn footer(ui: &mut Ui, count: &str, held: Option<&'static str>, commands: &CommandSender) {
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        let remove_all = i18n::text(&ctx, "changes-remove-all-ask");
        if pill_button(ui, &remove_all, ButtonKind::Soft).clicked() {
            commands.send(UiCommand::ClearChanges);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let text = i18n::text_with(&ctx, "changes-apply-all", &[("count", count)]);
            if apply_pill(ui, &text, held).clicked() {
                commands.send(UiCommand::Apply);
            }
        });
    });
}

fn row(ui: &mut Ui, change: &ChangeView, commands: &CommandSender) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let words = sentence(&ctx, &change.summary);
    let remove = i18n::text_with(&ctx, "changes-remove", &[("change", &words)]);

    split_row(
        ui,
        REMOVE_WIDTH,
        |ui| {
            ui.horizontal_top(|ui| {
                let (dot, _) = ui.allocate_exact_size(DOT_CELL, egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), DOT_CELL.x / 2.0, tokens.accent);
                ui.vertical(|ui| {
                    ui.add(Label::new(rich(ui, &words, Weight::Regular, BODY, tokens.text)).wrap());
                    if change.outcome.is_failed() {
                        tag(ui, &i18n::text(&ctx, "change-unfinished"), Tone::Warning);
                    }
                });
            });
        },
        |ui| {
            if icon_button(ui, icons::WINDOW_CLOSE, &remove).clicked() {
                commands.send(AppCommand::RemoveChange {
                    domain: change.domain,
                    key: change.key.clone(),
                });
            }
        },
    );
    ui.add_space(tokens.sizes.tight_gap);
}

fn raised(ui: &mut Ui, radius: CornerRadius, margin: Margin, body: impl FnOnce(&mut Ui)) -> Rect {
    Glass::raised(ui, radius, margin)
        .show(ui, body)
        .response
        .rect
}

/// The question for [`UiCommand::ClearChanges`], which the app asks before
/// it removes anything.
fn ask_to_clear(
    ctx: &Context,
    count: &str,
    confirming: &mut Option<UiCommand>,
    commands: &CommandSender,
) {
    let title = i18n::text(ctx, "command-clear-changes");
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text_with(ctx, "changes-remove-all-question", &[("count", count)]);
    let cancel = i18n::text(ctx, "changes-keep");
    let confirm = i18n::text(ctx, "changes-remove-all");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = question(ctx, Id::unique("toolbox_gui::changes::clear"), texts);
    if asked.confirmed {
        commands.send(Command::Confirmed(UiCommand::ClearChanges));
    }

    if asked.done {
        *confirming = None;
    }
}
