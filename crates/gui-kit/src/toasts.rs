//! Short notes about background results nobody waited on.
//!
//! They follow Kirigami's passive notifications: at the bottom centre of
//! the page area, the newest on top, in the Complementary colour set and
//! without an icon.
//!
//! A note stays for seven seconds, waits while the pointer is on it or the
//! window is away, and a click closes it. Notes come in through
//! [`Toasts::add`] or, from a widget without a way to the app, through
//! [`raise`].

#[cfg(test)]
mod tests;

use std::time::Duration;

use egui::layers::ShapeIdx;
use egui::{
    Align2, Context, CornerRadius, Frame, Id, Label, LayerId, Margin, Order, Painter, Rect, Role,
    Sense, Shadow, Shape, TextStyle, TextWrapMode, Ui, WidgetText, pos2, vec2,
};

use crate::fonts::{BODY, Weight};
use crate::frost::Glass;
use crate::motion::{self, Timing};
use crate::theme::CARD_RADIUS;
use crate::tokens::{self, Tokens};
use crate::widgets::rich;

/// Darkly's popup shadow strengths, the shadow shade at these shares.
const POPUP_NEAR: f32 = 0.44;
const POPUP_FAR: f32 = 0.24;

/// Kirigami's default `closeInterval`, for an error as for any note: it
/// has no timeout of its own.
const TIME: Duration = Duration::from_secs(7);
const TIMING: Timing = Timing::new(motion::TOAST_ENTER, motion::TOAST_EXIT);
/// Kirigami's `gridUnit * 25`, the widest a note grows before its text wraps.
const MAX_WIDTH: f32 = 450.0;
/// Kirigami's `largeSpacing`, inside a note and from the bottom edge.
const PADDING: i8 = 8;
const BOTTOM: f32 = 8.0;
/// Kirigami's `maximumNotificationCount`: a fifth note takes the oldest's
/// place, which then leaves.
const MAX_SHOWN: usize = 4;
/// Kirigami's note background, at 90 % opacity.
const OPACITY: f32 = 0.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    /// Read as a status message.
    Info,
    /// Read at once as an alert, for a failure.
    Error,
}

#[derive(Clone, Debug, PartialEq)]
struct Toast {
    id: u64,
    kind: ToastKind,
    text: String,
    /// How long it shows, the pauses not counted.
    time: Duration,
    /// The `input.time` it leaves at, set on its first drawn frame so a
    /// note raised after an idle window does not lose that window.
    deadline: Option<f64>,
    /// The `input.time` of the first frame that saw the pointer on it or
    /// the window away, while that lasts.
    paused: Option<f64>,
    run_out: bool,
    closed: bool,
}

impl Toast {
    const fn leaving(&self) -> bool {
        self.closed || self.run_out
    }
}

#[derive(Debug, Default)]
pub struct Toasts {
    shown: Vec<Toast>,
    next: u64,
}

/// What the notes are called for screen readers: the Alert role's name.
#[derive(Clone, Copy, Debug)]
pub struct ToastTexts<'a> {
    pub name: &'a str,
}

impl Toasts {
    pub fn add(&mut self, kind: ToastKind, text: impl Into<String>) {
        let open = self.shown.iter().filter(|toast| !toast.leaving()).count();
        if open >= MAX_SHOWN
            && let Some(oldest) = self.shown.iter_mut().find(|toast| !toast.leaving())
        {
            oldest.closed = true;
        }

        self.shown.push(Toast {
            id: self.next,
            kind,
            text: text.into(),
            time: TIME,
            deadline: None,
            paused: None,
            run_out: false,
            closed: false,
        });
        self.next += 1;
    }

    pub const fn is_empty(&self) -> bool {
        self.shown.is_empty()
    }

    /// The texts on screen, oldest first.
    pub fn texts(&self) -> Vec<&str> {
        self.shown.iter().map(|toast| toast.text.as_str()).collect()
    }

    /// Draws the notes stacked up from the bottom centre of `page`, the
    /// oldest at the bottom, and counts down the notes not under the pointer.
    /// Asks for a frame only while one counts down.
    pub fn show(&mut self, ctx: &Context, page: Rect, texts: ToastTexts<'_>) {
        let raised: Vec<(ToastKind, String)> =
            ctx.data_mut(|data| data.remove_temp(inbox_id()).unwrap_or_default());

        for (kind, text) in raised {
            self.add(kind, text);
        }

        let now = ctx.input(|input| input.time);
        let gap = tokens::current(ctx).sizes.tight_gap;
        let widest = MAX_WIDTH.min(page.width() / 1.5);
        let centre = page.center().x;
        let mut bottom = page.bottom() - BOTTOM;

        let mut gone = Vec::new();
        let mut next_frame: Option<Duration> = None;

        for toast in &mut self.shown {
            let id = Id::unique(("toolbox_gui_kit::toast", toast.id));
            if let Some(wait) = count_down(ctx, toast, id, now) {
                next_frame = Some(next_frame.map_or(wait, |other| other.min(wait)));
            }

            let leaving = toast.leaving();
            let shown = motion::fresh_presence(ctx, id.with("shown"), !leaving, TIMING);
            if leaving && shown <= 0.0 {
                gone.push(toast.id);
                continue;
            }

            // The notes above one that left move down rather than jump.
            let restack = motion::duration(ctx, motion::RESTACK);
            let y = ctx.animate_value_with_time(id.with("y"), bottom, restack);
            let drop = if leaving {
                0.0
            } else {
                (1.0 - shown) * motion::TOAST_DROP
            };

            let area = egui::Area::new(id)
                .order(Order::Foreground)
                .pivot(Align2::CENTER_BOTTOM)
                .fixed_pos(pos2(centre, y - drop))
                .fade_in(false)
                .interactable(!leaving)
                .movable(false)
                .role(match toast.kind {
                    ToastKind::Info => Role::Status,
                    ToastKind::Error => Role::Alert,
                })
                .accessible_name(format!("{}: {}", texts.name, toast.text))
                .show(ctx, |ui| {
                    ui.multiply_opacity(shown);
                    note(ui, toast, widest)
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

            bottom -= area.response.rect.height() + gap;
        }

        self.shown.retain(|toast| !gone.contains(&toast.id));
        if let Some(wait) = next_frame {
            ctx.request_repaint_after(wait);
        }
    }
}

/// Counts the note down while the pointer is off it and the window has
/// focus. Returns how long until it leaves, while it still waits.
///
/// egui draws no frame while the window is away, so a pause runs from the
/// first frame that sees it to the frame it ends on.
fn count_down(ctx: &Context, toast: &mut Toast, id: Id, now: f64) -> Option<Duration> {
    // Where the note was last frame tells whether the pointer rests on it,
    // so it leaves in the frame its time runs out.
    let layer = LayerId::new(Order::Foreground, id);
    let hovered = ctx
        .memory(|memory| memory.area_rect(id))
        .is_some_and(|rect| ctx.rect_contains_pointer(layer, rect));

    let away = !ctx.input(|input| input.focused);
    if toast.leaving() {
        return None;
    }

    let mut deadline = *toast.deadline.get_or_insert(now + toast.time.as_secs_f64());

    if hovered || away {
        if toast.paused.is_none() {
            toast.paused = Some(now);
        }

        return None;
    }

    if let Some(since) = toast.paused.take() {
        deadline += now - since;
        toast.deadline = Some(deadline);
    }

    let wait = Duration::try_from_secs_f64(deadline - now)
        .ok()
        .filter(|left| !left.is_zero());

    toast.run_out = wait.is_none();
    wait
}

/// Darkly's popup shadow, two layers: one 5 px down and 20 px soft, one 3 px
/// up and 10 px soft. Dialogs and notes both cast it.
pub(crate) fn popup_shadows(tokens: &Tokens) -> [Shadow; 2] {
    [
        Shadow {
            offset: [0, 5],
            blur: 20,
            spread: 0,
            color: tokens.shadow_shade.gamma_multiply(POPUP_NEAR),
        },
        Shadow {
            offset: [0, -3],
            blur: 10,
            spread: 0,
            color: tokens.shadow_shade.gamma_multiply(POPUP_FAR),
        },
    ]
}

/// Paints the [`popup_shadows`] of `rect` into `slot`, a placeholder added
/// before the frame so the shadow stays under it.
pub(crate) fn paint_popup_shadow(
    painter: &Painter,
    slot: ShapeIdx,
    rect: Rect,
    corners: CornerRadius,
    tokens: &Tokens,
) {
    let shapes = popup_shadows(tokens)
        .map(|shadow| Shape::from(shadow.as_shape(rect, corners)))
        .to_vec();

    painter.set(slot, Shape::Vec(shapes));
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

/// One note of at most `widest` wide: its text in the Complementary set,
/// on a rounded, shadowed frame. Returns whether a click closed it.
fn note(ui: &mut Ui, toast: &Toast, widest: f32) -> bool {
    let tokens = tokens::current(ui.ctx());
    let pad = f32::from(PADDING);
    let wrap = (-pad).mul_add(2.0, widest).max(0.0);
    let words = rich(
        ui,
        &toast.text,
        Weight::Regular,
        BODY,
        tokens.roles.complementary_text,
    );

    let galley =
        WidgetText::from(words).into_galley(ui, Some(TextWrapMode::Wrap), wrap, TextStyle::Body);

    let text = galley.size();

    // A wrapped text keeps the widest note, so its right edge does not
    // follow the words that happened to wrap.
    let width = if galley.rows.len() > 1 { wrap } else { text.x };
    let corners = CornerRadius::same(CARD_RADIUS);
    let glass = Glass::new(
        Frame::new()
            .corner_radius(corners)
            .inner_margin(Margin::same(PADDING)),
        tokens.roles.complementary.gamma_multiply(OPACITY),
    );

    let slot = ui.painter().add(Shape::Noop);
    let response = glass
        .show(ui, |ui| {
            let (row, _) = ui.allocate_exact_size(vec2(width, text.y), Sense::hover());
            ui.put(Rect::from_min_size(row.min, text), Label::new(galley));
        })
        .response;

    paint_popup_shadow(ui.painter(), slot, response.rect, corners, &tokens);

    ui.interact(response.rect, response.id.with("tap"), Sense::click())
        .clicked()
}

fn inbox_id() -> Id {
    Id::unique("toolbox_gui_kit::toasts::inbox")
}
