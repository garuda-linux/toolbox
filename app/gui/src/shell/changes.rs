//! The queued changes: a pill floating at the bottom right of the page
//! while there are any, with Apply, and a popover above it that lists
//! them, each with its own remove button, and removes all after a
//! question.

mod badge;
mod buttons;
mod clear;
mod place;

use egui::{
    Align2, Area, Context, CornerRadius, Id, Label, LayerId, Margin, Order, Rect, Stroke, Ui, Vec2,
    vec2,
};
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::frost::Glass;
use toolbox_gui_kit::motion::{self, Timing};
use toolbox_gui_kit::theme::POPOVER_RADIUS;
use toolbox_gui_kit::widgets::{ButtonKind, Tone, icon_button, pill_button, rich, split_row, tag};
use toolbox_gui_kit::{UiExt as _, icons, tokens};

use self::badge::badge;
use self::buttons::{apply_button, apply_pill, review_button, show_console_button};
use self::clear::ask_to_clear;
use self::place::{Fit, place};
use super::PANE_TITLE;
use crate::command::{AppCommand, CommandSender, UiCommand};
use crate::i18n;
use crate::i18n::changes::sentence;

/// Under the dialogs' layer, so a dialog's scrim covers the pill as it
/// covers the page. egui raises an area in the first frame it shows, which
/// on the dialogs' layer put a returning pill over a dialog's buttons.
const LAYER: Order = Order::Middle;
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
    pub running: bool,
    pub console_open: bool,
}

fn pill_words(ctx: &Context, count: &str, running: bool) -> String {
    let id = if running {
        "changes-running"
    } else {
        "changes-pill"
    };

    i18n::text_with(ctx, id, &[("count", count)]).into_owned()
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
    let id = Id::unique("toolbox_gui::changes::pill");
    let pill_rect = Area::new(id)
        .order(LAYER)
        .pivot(Align2::RIGHT_CENTER)
        .fixed_pos(corner)
        .show(ctx, |ui| pill(ui, queued, state, commands))
        .response
        .rect;

    ctx.move_to_top(LayerId::new(LAYER, id));
    overlays(ctx, (pill_rect, None), queued, state, commands);
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
    let words = pill_words(&ctx, &count, queued.running);
    let label = format!("{count} {words}");
    badge.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));

    let review = review_button(ui, state);
    let (apply, show) = apply_button(ui, queued.held, commands, removes);
    if show {
        state.open = true;
    }

    let rect = apply.rect.union(review.rect).union(badge.rect);
    overlays(&ctx, (rect, Some(ui.layer_id())), queued, state, commands);
}

/// `sheet` is the Console's layer when the pill sits in its header.
fn overlays(
    ctx: &Context,
    (pill, sheet): (Rect, Option<LayerId>),
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
        let fit_id = id.with("fit");
        let fit = ctx
            .data(|data| data.get_temp::<Fit>(fit_id))
            .unwrap_or_default();

        let placed = place(pill, ctx.content_rect(), fit, gap);
        let (corner, pivot) = placed.corner(pill, gap, (1.0 - shown) * motion::POPOVER_RISE);
        Area::new(id)
            .order(LAYER)
            .pivot(pivot)
            .fixed_pos(corner)
            .fade_in(false)
            .show(ctx, |ui| {
                ui.multiply_opacity(shown);
                let measured = popover(ui, list, (held, placed.list), commands);
                ui.data_mut(|data| data.insert_temp(fit_id, measured));
            });

        // Above the page's header on the same layer. egui keeps the last
        // order of areas that all rise each frame, which left a list
        // opened again under the sheet, so there it hangs off the sheet.
        let layer = LayerId::new(LAYER, id);
        ctx.move_to_top(layer);
        if let Some(sheet) = sheet {
            ctx.set_sublayer(sheet, layer);
        }
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
            let text = pill_words(&ctx, &count_text, queued.running);
            ui.label(rich(ui, &text, Weight::SemiBold, BODY, tokens.text));
            ui.add_space(tokens.sizes.tight_gap);
            if queued.running && !queued.console_open {
                show_console_button(ui, commands);
            } else {
                review_button(ui, state);
            }

            // Open, the list carries the one Apply.
            if !state.open && apply_button(ui, held, commands, removes).1 {
                state.open = true;
            }
        });
    })
}

/// `list` is the list's height before it scrolls. Returns the heights
/// the popover measured.
fn popover(
    ui: &mut Ui,
    changes: &[ChangeView],
    (held, list): (Option<&'static str>, f32),
    commands: &CommandSender,
) -> Fit {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let count = changes.len().to_string();
    let mut shown = 0.0;
    let mut whole = 0.0;

    let rect = raised(
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

            let scrolled = egui::ScrollArea::vertical()
                .max_height(list)
                .show(ui, |ui| {
                    for change in changes {
                        row(ui, change, commands);
                    }
                });

            shown = scrolled.inner_rect.height();
            whole = scrolled.content_size.y;

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

    Fit {
        chrome: rect.height() - shown,
        list: whole,
        most: LIST_HEIGHT,
    }
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
