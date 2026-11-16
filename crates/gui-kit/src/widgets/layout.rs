//! Layouts the pages share: the row with wrapping text on the left and
//! controls on the right, and controls at the end of a line.

mod title;

use egui::{Align, Id, Layout, Ui, UiBuilder, Vec2, vec2};
pub(super) use title::{capitals_in_line, last_descent, raised_column, slack_below};
use title::{tail_id, title_tail};
pub use title::{title_capitals_top, title_line, title_line_height};

use super::card::in_card_row;
use super::list::memory::{self, Size};
use crate::theme::PILL_HEIGHT;
use crate::tokens;

/// Narrower than this, a row's text would wrap after every word or two.
pub const MIN_TEXT_WIDTH: f32 = 240.0;
/// A measured size that moves less than this, in points, has settled.
pub(super) const SETTLED: f32 = 0.5;

/// What a remembered size that moved does to the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Redraw {
    /// Every change discards the pass, for a band whose old size would
    /// show for one frame.
    Discards,
}

/// A size kept from one frame for the next.
pub(super) trait Measure: Copy + Send + Sync + 'static {
    fn moved(self, now: Self) -> bool;
}

impl Measure for f32 {
    fn moved(self, now: Self) -> bool {
        moved(self, now)
    }
}

impl Measure for Vec2 {
    fn moved(self, now: Self) -> bool {
        (now - self).length() > SETTLED
    }
}

/// The text's width beside controls `controls` wide in `full`, `gap` from
/// them, `None` when that leaves it less than [`MIN_TEXT_WIDTH`] and the
/// controls go below.
pub(super) fn text_width(full: f32, controls: f32, gap: f32) -> Option<f32> {
    if controls <= 0.0 {
        return Some((full - controls - gap).max(0.0));
    }

    beside(full, controls, gap)
}

/// [`text_width`] that holds to [`MIN_TEXT_WIDTH`] even for no controls,
/// as an action row whose actions were not measured yet.
pub(super) fn beside(full: f32, controls: f32, gap: f32) -> Option<f32> {
    let text = (full - controls - gap).max(0.0);
    (text >= MIN_TEXT_WIDTH).then_some(text)
}

pub(super) fn moved(was: f32, now: f32) -> bool {
    (now - was).abs() > SETTLED
}

/// Keeps the size `now` in `field` of `row` unless it settled at `kept`.
/// A first measurement discards its pass, as [`remember`] does.
pub(super) fn remember_width(
    ui: &Ui,
    (row, field): (Id, Size),
    kept: Option<f32>,
    now: f32,
    reason: &'static str,
) {
    if kept.is_none_or(|was| moved(was, now)) {
        memory::write(ui.ctx(), row, |kept_row| *field(kept_row) = Some(now));
        remeasured(ui, kept.is_none(), reason);
    }
}

/// Keeps the size `now` for `id` unless it settled at `kept`, the one
/// store of every size measured to lay out the next frame.
pub(super) fn remember<T>(
    ui: &Ui,
    id: Id,
    kept: Option<T>,
    now: T,
    (redraw, reason): (Redraw, &'static str),
) where
    T: Measure,
{
    if kept.is_none_or(|was| was.moved(now)) {
        ui.data_mut(|data| data.insert_temp(id, now));
        remeasured(ui, kept.is_none() || redraw == Redraw::Discards, reason);
    }
}

/// Redraws after a size changed. A discard repeats the clicks of its
/// pass, so egui's Grid discards only a first measurement's, and so do
/// the sizes that do not ask for [`Redraw::Discards`].
pub(super) fn remeasured(ui: &Ui, discard: bool, reason: &'static str) {
    if discard {
        ui.ctx().request_discard(reason);
    } else {
        ui.ctx().request_repaint();
    }
}

/// `left` beside `right_width` wide controls, which sit centred on the
/// whole row, however many lines `left` takes.
///
/// Narrower than [`MIN_TEXT_WIDTH`] for the text, the controls go below
/// it at its left. The width comes from the caller, so the layout never
/// waits for a second frame.
pub fn split_row<L, R>(
    ui: &mut Ui,
    right_width: f32,
    left: impl FnOnce(&mut Ui) -> L,
    right: impl FnOnce(&mut Ui) -> R,
) -> (L, R) {
    split_row_at(ui, None, right_width, left, right)
}

/// [`split_row`] for a list row, `row` being the row's id, so the sizes
/// it keeps go with the row's memory and the list drops them with the row.
pub fn split_row_in<L, R>(
    ui: &mut Ui,
    row: Id,
    right_width: f32,
    left: impl FnOnce(&mut Ui) -> L,
    right: impl FnOnce(&mut Ui) -> R,
) -> (L, R) {
    split_row_at(ui, Some(row), right_width, left, right)
}

fn split_row_at<L, R>(
    ui: &mut Ui,
    row: Option<Id>,
    right_width: f32,
    left: impl FnOnce(&mut Ui) -> L,
    right: impl FnOnce(&mut Ui) -> R,
) -> (L, R) {
    let full = ui.available_width();
    let gap = tokens::current(ui.ctx()).sizes.text_to_control;
    let Some(left_width) = text_width(full, right_width, gap) else {
        return stacked(ui, full, right_width, left, right);
    };

    // Last pass's heights centre the text and the controls on each other.
    // Text taller than the controls keeps its place at the row's top, so a
    // row has its text where a row without a control has it.
    let row = row.unwrap_or_else(|| ui.next_auto_id().with("split_row"));
    let known = memory::get(ui, row).split;
    let [text_was, controls_was, measured_was, given_was] =
        known.unwrap_or([0.0, 0.0, right_width, right_width]);

    // A caller that hands in a new width, as Add turning into Remove, knows
    // it this frame. Last frame's measure would push the controls past the
    // row's edge for one frame.
    let width_was = if moved(given_was, right_width) {
        right_width
    } else {
        measured_was
    };

    // In a card's row a short text keeps a control's height, centred in
    // it, so a row is as tall with or without a control.
    let least = line_height(ui, 0.0);
    let block = controls_was.max(least);
    let text_room = ((block - text_was) / 2.0).max(0.0);

    let spacing = ui.spacing().item_spacing;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let left =
            ui.allocate_ui_with_layout(vec2(left_width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_width);

                // Only this room is the space above and below the text.
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(text_room);
                ui.data_mut(|data| data.remove_temp::<[f32; 2]>(tail_id()));
                let drawn = ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = spacing;
                    left(ui)
                });

                let taken = drawn.response.rect;
                ui.add_space((block - text_room - taken.height()).max(0.0));
                (drawn.inner, taken.height() - title_tail(ui, taken.bottom()))
            });

        ui.add_space(gap);
        let height = left.response.rect.height();
        let (left, text_height) = left.inner;

        let drop = ((text_was.max(least) - controls_was) / 2.0).max(0.0);
        let lead = (right_width - width_was).max(0.0);
        let (right, controls_height, controls_width) = controls_box(
            ui,
            (right_width, height, controls_was),
            (drop, lead, spacing.x),
            right,
        );

        let now = [text_height, controls_height, controls_width, right_width];
        if known.is_none_or(|was| was.iter().zip(now).any(|(was, now)| moved(*was, now))) {
            memory::write(ui.ctx(), row, |kept_row| kept_row.split = Some(now));

            // Controls of another width were placed from the old one, past
            // the row's edge or short of it. That frame is never shown.
            let resized = moved(measured_was, controls_width);
            remeasured(ui, known.is_none() || resized, "split_row sizes");
        }

        (left, right)
    })
    .inner
}

/// A box as high as what it holds, so its height is the controls'. Left
/// to right behind a lead, so the controls take Tab in the order they are
/// seen and still end at the row's right edge.
fn controls_box<R>(
    ui: &mut Ui,
    (width, height, line): (f32, f32, f32),
    (drop, lead, gap): (f32, f32, f32),
    right: impl FnOnce(&mut Ui) -> R,
) -> (R, f32, f32) {
    ui.allocate_ui_with_layout(vec2(width, height), Layout::top_down(Align::Min), |ui| {
        ui.set_min_width(width);
        ui.add_space(drop);

        // Laid from the top, so the box measures the controls' own height.
        // Centred in a taller first guess, it would feed that room into
        // every later frame. The row inside still centres a caption.
        let (inner, taken_width, taken_height) = unsqueezed_at(
            ui,
            (lead, width, line),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().interact_size.y = line_height(ui, ui.spacing().interact_size.y);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    right(ui)
                })
                .inner
            },
        );

        (inner, taken_height, taken_width)
    })
    .inner
}

/// Lays `body` out from `lead` with the whole `width` to grow into, past the
/// end when it grew since last frame, so a label is never cut to the room a
/// stale lead left and is measured at its natural width. The next frame's
/// lead then puts it back inside. Returns the width and height it took.
fn unsqueezed_at<R>(
    ui: &mut Ui,
    (lead, width, line): (f32, f32, f32),
    layout: Layout,
    body: impl FnOnce(&mut Ui) -> R,
) -> (R, f32, f32) {
    let top_left = ui.cursor().min + vec2(lead, 0.0);

    // Last frame's line height centres a caption on a taller switch. The
    // first frame has none and lays out against the whole height.
    let height = if line > 0.0 {
        line
    } else {
        ui.available_height()
    };

    let room = egui::Rect::from_min_size(top_left, vec2(width, height));
    let drawn = ui.scope_builder(UiBuilder::new().max_rect(room).layout(layout), body);
    let taken = drawn.response.rect;
    (drawn.inner, taken.width(), taken.height())
}

/// Controls at the right end of the space left, added in the order seen.
///
/// A right-to-left layout would hand out Tab focus from the right edge
/// backwards. The lead comes from last frame's width.
pub fn trailing<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    let row = ui.next_auto_id().with("trailing");
    let width_was = memory::get(ui, row).trailing;
    let room = ui.available_width();
    let lead = (room - width_was.unwrap_or(room)).max(0.0);

    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        let layout = Layout::left_to_right(Align::Center);
        let (inner, width, _) = unsqueezed_at(ui, (lead, room, 0.0), layout, body);

        remember_width(
            ui,
            (row, |kept_row: &mut memory::RowMemory| {
                &mut kept_row.trailing
            }),
            width_was,
            width,
            "trailing width",
        );
        inner
    })
    .inner
}

/// The text across the whole width, the controls below it at its left.
fn stacked<L, R>(
    ui: &mut Ui,
    full: f32,
    right_width: f32,
    left: impl FnOnce(&mut Ui) -> L,
    right: impl FnOnce(&mut Ui) -> R,
) -> (L, R) {
    let gap = tokens::current(ui.ctx()).sizes.caption_gap;
    ui.vertical(|ui| {
        let left = ui
            .allocate_ui_with_layout(vec2(full, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(full);
                left(ui)
            })
            .inner;

        ui.add_space(gap);

        // Controls wider than the row wrap onto more lines inside it,
        // rather than reach past its left edge under the sidebar.
        let layout = Layout::left_to_right(Align::Center).with_main_wrap(right_width > full);
        let right = ui
            .allocate_ui_with_layout(vec2(full, 0.0), layout, |ui| {
                ui.set_min_width(full);
                right(ui)
            })
            .inner;

        (left, right)
    })
    .inner
}

/// How tall a line of a row's title or controls is: a control's height
/// in a card's row, so every row of a card keeps one height and one
/// middle line with or without a control, else `elsewhere`.
pub(super) fn line_height(ui: &Ui, elsewhere: f32) -> f32 {
    if in_card_row(ui) {
        PILL_HEIGHT
    } else {
        elsewhere
    }
}
