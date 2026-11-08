//! Short notes about background results nobody waited on, at the top
//! right of the page area below the page's header, after Rerun's
//! notification toasts.
//!
//! A note fades after a while, an error after a longer one or when
//! closed, and hovering it pauses that. Notes come in through
//! [`Toasts::add`] or, from a widget without a way to the app, through
//! [`raise`].

#[cfg(test)]
mod tests;

use std::time::Duration;

use egui::{
    Align2, Context, CornerRadius, Id, Label, LayerId, Margin, Order, Rect, Role, Sense, TextStyle,
    TextWrapMode, Ui, WidgetText, pos2, vec2,
};

use crate::fonts::{BODY, Weight};
use crate::frost::Glass;
use crate::motion::{self, Timing};
use crate::theme::CARD_RADIUS;
use crate::widgets::{TileSize, Tone, icon_button, icon_button_width, paint_status_tile, rich};
use crate::{icons, tokens};

const INFO_TIME: Duration = Duration::from_secs(4);
/// How long an error shows, long enough to read it twice.
const ERROR_TIME: Duration = Duration::from_secs(10);
/// A frame's length when egui knows none yet.
const FRAME: Duration = Duration::from_millis(16);
/// A note appears with a drop into place and leaves by fading alone.
const TIMING: Timing = Timing::new(motion::TOAST_ENTER, motion::TOAST_EXIT);
/// The widest a note grows before its text wraps, short of a narrow page:
/// room for a sentence of about sixty characters on one line.
const MAX_WIDTH: f32 = 460.0;
/// Around a note's content, the same on every side.
const PADDING: i8 = 12;
/// From the page area's top right corner, or below its header.
const OFFSET: egui::Vec2 = vec2(-16.0, 16.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    /// It fades by itself.
    Info,
    /// It fades later, or when closed.
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Toast {
    id: u64,
    kind: ToastKind,
    text: String,
    left: Duration,
    closed: bool,
}

impl Toast {
    const fn leaving(&self) -> bool {
        self.closed || self.left.is_zero()
    }
}

#[derive(Debug, Default)]
pub struct Toasts {
    shown: Vec<Toast>,
    next: u64,
}

/// What the notes are called: the Alert role's name, and the error's
/// Close button.
#[derive(Clone, Copy, Debug)]
pub struct ToastTexts<'a> {
    pub name: &'a str,
    pub close: &'a str,
}

impl Toasts {
    pub fn add(&mut self, kind: ToastKind, text: impl Into<String>) {
        let left = match kind {
            ToastKind::Info => INFO_TIME,
            ToastKind::Error => ERROR_TIME,
        };

        self.shown.push(Toast {
            id: self.next,
            kind,
            text: text.into(),
            left,
            closed: false,
        });
        self.next += 1;
    }

    pub const fn is_empty(&self) -> bool {
        self.shown.is_empty()
    }

    /// The texts on screen, newest last.
    pub fn texts(&self) -> Vec<&str> {
        self.shown.iter().map(|toast| toast.text.as_str()).collect()
    }

    /// Draws the notes stacked down from the top right of `page`, beside
    /// the page title, and counts down the notes not under the pointer.
    /// Asks for a frame only while one counts down.
    pub fn show(&mut self, ctx: &Context, page: Rect, texts: ToastTexts<'_>) {
        let raised: Vec<(ToastKind, String)> =
            ctx.data_mut(|data| data.remove_temp(inbox_id()).unwrap_or_default());

        for (kind, text) in raised {
            self.add(kind, text);
        }

        let elapsed =
            Duration::try_from_secs_f32(ctx.input(|input| input.unstable_dt)).unwrap_or(FRAME);

        let gap = tokens::current(ctx).sizes.card_gap;
        let mut top = page.right_top() + OFFSET;
        let widest = MAX_WIDTH.min(2.0_f32.mul_add(OFFSET.x, page.width()));

        let mut gone = Vec::new();
        let mut next_frame: Option<Duration> = None;

        for toast in &mut self.shown {
            let id = Id::unique(("toolbox_gui_kit::toast", toast.id));
            if let Some(wait) = count_down(ctx, toast, id, elapsed) {
                next_frame = Some(next_frame.map_or(wait, |other| other.min(wait)));
            }

            let leaving = toast.leaving();
            let shown = motion::fresh_presence(ctx, id.with("shown"), !leaving, TIMING);
            if leaving && shown <= 0.0 {
                gone.push(toast.id);
                continue;
            }

            // The notes below one that left move up rather than jump.
            let restack = motion::duration(ctx, motion::RESTACK);
            let y = ctx.animate_value_with_time(id.with("y"), top.y, restack);
            let drop = if leaving {
                0.0
            } else {
                (1.0 - shown) * motion::TOAST_DROP
            };

            let area = egui::Area::new(id)
                .order(Order::Foreground)
                .pivot(Align2::RIGHT_TOP)
                .fixed_pos(pos2(top.x, y - drop))
                .fade_in(false)
                .interactable(!leaving)
                .movable(false)
                .role(Role::Alert)
                .accessible_name(format!("{}: {}", texts.name, toast.text))
                .show(ctx, |ui| {
                    ui.multiply_opacity(shown);
                    note(ui, toast, texts.close, widest)
                });

            if area.inner {
                toast.closed = true;
                if motion::duration(ctx, TIMING.exit) > 0.0 {
                    // The next frame starts it leaving.
                    ctx.request_repaint();
                } else {
                    gone.push(toast.id);
                }
            }

            top.y += area.response.rect.height() + gap;
        }

        self.shown.retain(|toast| !gone.contains(&toast.id));
        if let Some(wait) = next_frame {
            ctx.request_repaint_after(wait);
        }
    }
}

/// Counts the note down while the pointer is off it and the window has
/// focus, so a note is not missed by someone looking away. Returns how
/// long until it leaves, while it still waits.
fn count_down(ctx: &Context, toast: &mut Toast, id: Id, elapsed: Duration) -> Option<Duration> {
    // Where the note was last frame tells whether the pointer rests on it,
    // so it leaves in the frame its time runs out.
    let layer = LayerId::new(Order::Foreground, id);
    let hovered = ctx
        .memory(|memory| memory.area_rect(id))
        .is_some_and(|rect| ctx.rect_contains_pointer(layer, rect));

    let away = !ctx.input(|input| input.focused);
    if toast.leaving() || hovered || away {
        return None;
    }

    toast.left = toast.left.saturating_sub(elapsed);
    (!toast.left.is_zero()).then_some(toast.left)
}

/// Raises a note from a widget, which [`Toasts::show`] takes in on its
/// next call.
pub fn raise(ctx: &Context, kind: ToastKind, text: impl Into<String>) {
    let text = text.into();
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<(ToastKind, String)>>(inbox_id())
            .push((kind, text));
    });
}

/// One note at most `widest` wide, its status mark, text and Close button
/// centred on the whole text. Returns whether Close was clicked.
fn note(ui: &mut Ui, toast: &Toast, close: &str, widest: f32) -> bool {
    let tokens = tokens::current(ui.ctx());
    let (tone, closable) = match toast.kind {
        ToastKind::Info => (Tone::Accent, false),
        ToastKind::Error => (Tone::Error, true),
    };

    Glass::raised(ui, CornerRadius::same(CARD_RADIUS), Margin::same(PADDING))
        .show(ui, |ui| {
            let edge = TileSize::Line.edge(ui);
            let gap = tokens.sizes.picture_gap;
            let button = if closable { icon_button_width(ui) } else { 0.0 };
            let beside = edge + gap + if closable { gap + button } else { 0.0 };

            // The glass's 1 px rim lies outside the padding.
            let frame = 2.0 * (f32::from(PADDING) + 1.0);
            let wrap = (widest - frame - beside).max(0.0);
            let words = rich(ui, &toast.text, Weight::Medium, BODY, tokens.text);
            let galley = WidgetText::from(words).into_galley(
                ui,
                Some(TextWrapMode::Wrap),
                wrap,
                TextStyle::Body,
            );

            let text = galley.size();

            // A wrapped text keeps the widest note, so its right edge does
            // not follow the words that happened to wrap.
            let width = if galley.rows.len() > 1 { wrap } else { text.x };
            let size = vec2(beside + width, text.y.max(edge).max(button));
            let (row, _) = ui.allocate_exact_size(size, Sense::hover());
            let centre = row.center().y;
            paint_status_tile(
                ui,
                pos2(row.left() + edge / 2.0, centre),
                TileSize::Line,
                tone,
            );

            let at = pos2(row.left() + edge + gap, centre - text.y / 2.0);
            ui.put(Rect::from_min_size(at, text), Label::new(galley));

            closable && {
                let place = Rect::from_center_size(
                    pos2(row.right() - button / 2.0, centre),
                    vec2(button, button),
                );

                ui.put(place, |ui: &mut Ui| {
                    icon_button(ui, icons::WINDOW_CLOSE, close)
                })
                .clicked()
            }
        })
        .inner
}

fn inbox_id() -> Id {
    Id::unique("toolbox_gui_kit::toasts::inbox")
}
