//! The queued changes: a footer toolbar under the page while there are
//! any, as KDE's footer of a settings module, with the count, Review and
//! Apply, and a popover above it that lists them, each with its own remove
//! button, and removes all after a question.

mod bar;
mod buttons;
mod clear;
mod held;
mod place;

use egui::{
    Context, CornerRadius, Frame, Id, Label, LayerId, Popup, PopupCloseBehavior, PopupKind, Rect,
    RectAlign, Stroke, Ui, vec2,
};
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::frost::{self, Glass};
use toolbox_gui_kit::motion::{self, Timing};
use toolbox_gui_kit::theme::{POPOVER_RADIUS, show_scroll};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, icon_button, pill_button, rich, split_row, tag};
use toolbox_gui_kit::{UiExt as _, icons, tokens};

use self::bar::count_text;
pub use self::bar::footer;
use self::buttons::{apply_button, apply_pill, review_button};
use self::clear::ask_to_clear;
use self::held::unread;
use self::place::{Fit, place};
use super::PANE_TITLE;
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::i18n;
use crate::i18n::Arg;
use crate::i18n::changes::sentence;

#[derive(Debug, Default)]
pub struct ChangesState {
    pub open: bool,
    /// A key opened the popover, so it shows at once rather than rising.
    pub keyed: bool,
    /// A destructive command waiting for the user's yes, which the app
    /// sets. Its question shows over the window.
    pub confirming: Option<UiCommand>,
}

#[derive(Clone, Copy, Debug)]
pub struct Queued<'view> {
    pub changes: &'view [ChangeView],
    pub held: Option<&'static str>,
    pub running: bool,
    pub console_open: bool,
}

fn count_words(ctx: &Context, count: usize, running: bool) -> String {
    let id = if running {
        "changes-running"
    } else {
        "changes-pill"
    };

    i18n::text_with(ctx, id, &[("count", Arg::from(count))]).into_owned()
}

/// In the Console's header while it is open, so nothing floats over the
/// rows above it: the count, Review and Apply. `ui` lays out right to
/// left, beside the header's Close.
pub fn inline(ui: &mut Ui, queued: Queued<'_>, state: &mut ChangesState, commands: &CommandSender) {
    if queued.changes.is_empty() {
        *state = ChangesState::default();
        return;
    }

    let ctx = ui.ctx().clone();
    let removes = queued.changes.iter().any(ChangeView::removes);
    let count = ui.label(count_text(ui, queued));

    let review = review_button(ui, state);
    let (apply, show) = apply_button(ui, queued.held, commands, removes);
    if show {
        state.open = true;
    }

    let rect = apply.rect.union(review.rect).union(count.rect);
    overlays(&ctx, (rect, ui.layer_id()), queued, state, commands);
}

fn overlays(
    ctx: &Context,
    anchor: (Rect, LayerId),
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) {
    let Queued {
        changes: list,
        held,
        running: _,
        console_open: _,
    } = queued;

    // It rises from Review while egui fades it in. Closed, it
    // goes at once, and the next opening starts low again. A key's opening
    // does not rise.
    let id = popover_id();
    if state.open && std::mem::take(&mut state.keyed) {
        motion::settle(ctx, id.with("shown"), true);
    }

    let timing = Timing::new(motion::POPOVER_ENTER, 0.0);
    let shown = motion::presence(ctx, id.with("shown"), state.open, timing);

    // egui puts every popup on the dialogs' layer, so over a dialog the
    // popover steps aside. It stays open and comes back after it.
    let dialog =
        state.confirming.is_some() || ctx.memory(|memory| memory.top_modal_layer().is_some());

    if state.open && !dialog {
        let rise = (1.0 - shown) * motion::POPOVER_RISE;
        popover_at(
            ctx,
            (anchor.0, anchor.1, rise),
            (list, held),
            &mut state.open,
            commands,
        );
    }

    if state.confirming == Some(UiCommand::ClearChanges) {
        ask_to_clear(ctx, list.len(), &mut state.confirming, commands);
    }
}

/// The popover above or below `review`, `rise` points from its resting
/// place toward Review. egui closes it on a click beside it and on
/// Escape.
fn popover_at(
    ctx: &Context,
    (review, parent, rise): (Rect, LayerId, f32),
    (list, held): (&[ChangeView], Option<&'static str>),
    open: &mut bool,
    commands: &CommandSender,
) {
    let sizes = tokens::current(ctx).sizes;
    let fit_id = popover_id().with("fit");
    let fit = ctx
        .data(|data| data.get_temp::<Fit>(fit_id))
        .unwrap_or_default();

    // Its own choice of side, as egui's would not shorten the list where
    // neither side has room.
    let placed = place(review, ctx.content_rect(), fit, sizes.popover_gap);
    let (align, toward) = if placed.above {
        (RectAlign::TOP_END, rise)
    } else {
        (RectAlign::BOTTOM_END, -rise)
    };

    let glass = glass(ctx);
    let anchor = review.translate(vec2(0.0, toward));
    let popup = Popup::new(popover_id(), ctx.clone(), anchor, parent)
        .align(align)
        .align_alternatives(&[])
        .gap(sizes.popover_gap)
        .frame(glass.frame())
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .open_bool(open)
        .show(|ui| glass.inside(ui, |ui| popover(ui, list, (held, placed.list), commands)));

    if let Some(shown) = popup {
        let (scrolled, whole) = shown.inner;
        let measured = Fit {
            chrome: shown.response.rect.height() - scrolled,
            list: whole,
            most: sizes.changes_list_height,
        };

        ctx.data_mut(|data| data.insert_temp(fit_id, measured));
    }
}

/// [`Glass::raised`] for a popup, which takes its frame before it has a
/// `Ui`.
fn glass(ctx: &Context) -> Glass {
    let tokens = tokens::current(ctx);
    let frame = Frame::new()
        .stroke(Stroke::new(1.0, tokens.outline))
        .corner_radius(CornerRadius::same(POPOVER_RADIUS))
        .inner_margin(tokens.sizes.popover_padding)
        .shadow(ctx.global_style().visuals.popup_shadow);

    Glass::new(frame, frost::raised(ctx)).lit()
}

/// `list` is the list's height before it scrolls. Returns the height the
/// list showed and its whole height.
fn popover(
    ui: &mut Ui,
    changes: &[ChangeView],
    (held, list): (Option<&'static str>, f32),
    commands: &CommandSender,
) -> (f32, f32) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);

    ui.set_width(tokens.sizes.popover_width);
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

    let scrolled = show_scroll(ui, egui::ScrollArea::vertical().max_height(list), |ui| {
        for change in changes {
            row(ui, change, commands);
        }
    });

    match held {
        Some(reason) => unread(ui, reason, commands),
        None => held::forget(ui.ctx()),
    }

    ui.add_space(tokens.sizes.control_gap);
    let top = ui.cursor().top();
    ui.painter()
        .hline(ui.max_rect().x_range(), top, Stroke::new(1.0, tokens.rule));
    ui.add_space(tokens.sizes.card_gap);

    list_footer(ui, changes.len(), held, commands);
    (scrolled.inner_rect.height(), scrolled.content_size.y)
}

/// Remove all at the left, Apply at the right.
fn list_footer(ui: &mut Ui, count: usize, held: Option<&'static str>, commands: &CommandSender) {
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        let remove_all = i18n::text(&ctx, "changes-remove-all-ask");
        if pill_button(ui, &remove_all, ButtonKind::Soft).clicked() {
            commands.send(UiCommand::ClearChanges);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let text = i18n::text_with(&ctx, "changes-apply-all", &[("count", Arg::from(count))]);
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
    let remove = i18n::text_with(&ctx, "changes-remove", &[("change", Arg::from(&words))]);

    split_row(
        ui,
        tokens.sizes.remove_width,
        |ui| {
            ui.horizontal_top(|ui| {
                let cell = tokens.sizes.dot_cell;
                let (dot, _) = ui.allocate_exact_size(cell, egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), cell.x / 2.0, tokens.accent);
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

/// Whether the popover showed last frame, so a key handled before the
/// footer draws can leave it to the popover.
pub fn popover_shows(ctx: &Context) -> bool {
    let layer = LayerId::new(PopupKind::Popup.order(), popover_id());
    ctx.memory(|memory| memory.areas().is_visible(&layer))
}

fn popover_id() -> Id {
    Id::unique("toolbox_gui::changes::popover")
}
