//! The sheet over the page: it slides up from the page's bottom edge to
//! three quarters of its height, or to the height the grip dragged it
//! to. The page keeps its size below it, and what of it shows above the
//! sheet stays usable.

use egui::accesskit::Role;
use egui::{
    Align, Area, Color32, Context, CursorIcon, EventFilter, Frame, Id, Key, Layout, Modifiers,
    Order, Rect, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};
use toolbox_gui_kit::widgets::edge_ring;
use toolbox_gui_kit::{frost, motion, tokens};

use super::Sheet;

const SHARE: f32 = 0.75;
const KEY_STEP: f32 = 32.0;
/// The room between the grip's bar and its focus ring, so the ring reads
/// as the bar's and not as a line of its own.
const RING_ROOM: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub room: Rect,
    pub out: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Shown {
    /// The part of the window the sheet covers, none once it is gone.
    pub rect: Option<Rect>,
    pub dragged_shut: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct Drag {
    delta: f32,
    released: bool,
    /// The height a key on the focused grip asked for.
    keyed: Option<f32>,
}

/// The heights the grip moves between, and the page's, which a screen
/// reader hears them as a share of.
#[derive(Clone, Copy, Debug)]
struct Span {
    full: f32,
    least: f32,
    most: f32,
    room: f32,
}

impl Span {
    fn share(self, height: f32) -> f64 {
        f64::from((height / self.room.max(1.0) * 100.0).round())
    }
}

/// How far the sheet is out, advanced once a frame: 0 gone, 1 fully out.
/// Called before the page, so the shell knows whether the Changes band
/// shows.
pub fn out(ctx: &Context, sheet: Sheet) -> f32 {
    let slide = slide_id();
    if sheet.instant {
        motion::settle(ctx, slide, sheet.open);
    }

    motion::slide(ctx, slide, sheet.open, motion::SHEET)
}

pub fn show(
    ctx: &Context,
    place: Place,
    grip: &str,
    frame: Frame,
    contents: impl FnOnce(&mut Ui),
) -> Shown {
    let Place { room, out } = place;

    // The area shows every frame, so it never needs a sizing pass that
    // would hide the sheet's first frame.
    let area = Area::new(area_id())
        .order(Order::Middle)
        .constrain(false)
        .fixed_pos(room.left_bottom())
        .interactable(out > 0.0);

    if out <= 0.0 {
        area.show(ctx, |_ui| {});
        ctx.data_mut(|data| data.remove::<Rect>(cover_id()));
        return Shown::default();
    }

    let full = height(ctx, room);
    let span = Span {
        full,
        least: least(ctx),
        most: most(ctx, room),
        room: room.height(),
    };

    let top = full.mul_add(-out, room.bottom());
    let rect = Rect::from_min_max(pos2(room.left(), top), room.right_bottom());
    let whole = Rect::from_min_size(rect.min, vec2(room.width(), full));

    ctx.data_mut(|data| data.insert_temp(cover_id(), rect));
    let frame = frosted(ctx, frame);

    // The page header lies in the same order of layers. A click on it
    // raises it, the sheet stays above it.
    ctx.move_to_top(layer());

    let mut dragged_shut = false;
    area.fixed_pos(rect.min).show(ctx, |ui| {
        super::rim::shadow(ui, room, rect, frame.corner_radius);
        ui.set_clip_rect(rect);
        frost::backdrop(ui.painter(), whole, frame.corner_radius);
        let mut sheet_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(whole)
                .layout(Layout::top_down(Align::Min)),
        );

        frame.show(&mut sheet_ui, |inner| {
            inner.set_min_size(inner.available_size());
            let drag = drag_bar(inner, whole.top(), grip, span);
            dragged_shut = resize(ctx, room, full, drag);
            inner.scope_builder(
                UiBuilder::new().id_salt("toolbox_gui::console::contents"),
                contents,
            );
        });

        super::rim::top_edge(ui, rect, frame.corner_radius);
    });

    Shown {
        rect: Some(rect),
        dragged_shut,
    }
}

/// `frame` with the frost's tint for its fill while the blur lies under it.
/// With the glass on, the sheet lays no tint over the translucent window.
fn frosted(ctx: &Context, frame: Frame) -> Frame {
    let tokens = tokens::current(ctx);
    if toolbox_gui_kit::theme::translucent(ctx) {
        frame.fill(tokens.band)
    } else if frost::live(ctx) {
        frame.fill(tokens.frost)
    } else {
        frame
    }
}

/// What the terminal's box is filled with: glass over the live blur, so
/// the sheet reads as one blurred surface, solid without it.
pub fn terminal_fill(ctx: &Context) -> Color32 {
    let tokens = tokens::current(ctx);
    if frost::live(ctx) {
        tokens.terminal_glass
    } else {
        tokens.terminal
    }
}

/// The terminal box's edge: a hairline round the glass, as a thin light
/// glass on the light sheet would leave the box without an edge.
pub fn terminal_edge(ctx: &Context) -> Stroke {
    if frost::live(ctx) {
        Stroke::new(1.0, tokens::current(ctx).outline)
    } else {
        Stroke::NONE
    }
}

fn height(ctx: &Context, room: Rect) -> f32 {
    let kept: Option<f32> = ctx.data_mut(|data| data.get_persisted(height_id()));
    kept.unwrap_or_else(|| room.height() * SHARE)
        .clamp(least(ctx), most(ctx, room))
}

fn least(ctx: &Context) -> f32 {
    tokens::current(ctx).sizes.sheet_least
}

fn most(ctx: &Context, room: Rect) -> f32 {
    (room.height() - tokens::current(ctx).sizes.sheet_room_above).max(least(ctx))
}

/// The bar that shows the sheet can be dragged taller. Its hit area
/// reaches from the sheet's top edge down past the bar. It tells screen
/// readers the sheet's height as a share of the page.
fn drag_bar(ui: &mut Ui, top: f32, label: &str, span: Span) -> Drag {
    let tokens = tokens::current(ui.ctx());
    let size = vec2(ui.available_width(), tokens.sizes.grip_height);
    let (row, _) = ui.allocate_exact_size(size, Sense::hover());
    let bar = Rect::from_center_size(row.center(), tokens.sizes.grip_bar);
    let hit = Rect::from_min_max(pos2(row.left(), top), pos2(row.right(), row.bottom()));

    let response = ui
        .interact(hit, ui.scope_id().with("grip"), Sense::drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);

    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Splitter, true, label);
        info.value = Some(span.share(span.full));
        info
    });

    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_min_numeric_value(span.share(span.least));
        node.set_max_numeric_value(span.share(span.most));
    });

    let keyed = keyed(ui, &response, span);
    let colour = if response.hovered() || response.dragged() || response.has_focus() {
        tokens.secondary
    } else {
        tokens.dim
    };

    ui.painter().rect_filled(bar, bar.height() / 2.0, colour);
    edge_ring(ui, &response, bar.expand(RING_ROOM), u8::MAX);

    Drag {
        delta: response.drag_delta().y,
        released: response.drag_stopped(),
        keyed,
    }
}

/// Up and Down move the focused grip a step, Home and End to the most and
/// the least height. The arrows stay with the grip, as egui would move the
/// focus with them.
fn keyed(ui: &Ui, response: &Response, span: Span) -> Option<f32> {
    if !response.has_focus() {
        return None;
    }

    let filter = EventFilter {
        tab: false,
        horizontal_arrows: false,
        vertical_arrows: true,
        escape: false,
    };

    ui.memory_mut(|memory| memory.set_focus_lock_filter(response.id, filter));

    let (up, down, home, end) = ui.input_mut(|input| {
        (
            input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp),
            input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown),
            input.consume_key(Modifiers::NONE, Key::Home),
            input.consume_key(Modifiers::NONE, Key::End),
        )
    });

    if home {
        return Some(span.most);
    }

    if end {
        return Some(span.least);
    }

    if up == down {
        return None;
    }

    let presses = |count: usize| f32::from(u16::try_from(count).unwrap_or(u16::MAX));
    let height = KEY_STEP.mul_add(presses(up) - presses(down), span.full);
    Some(height.clamp(span.least, span.most))
}

/// Applies the grip's drag or key to the kept height. Returns whether a
/// drag let go below the least height, which closes the sheet at the
/// height it had.
fn resize(ctx: &Context, room: Rect, full: f32, drag: Drag) -> bool {
    let start_id = height_id().with("start");
    let held_id = height_id().with("held");

    if let Some(keyed) = drag.keyed {
        ctx.data_mut(|data| data.insert_persisted(height_id(), keyed));
        ctx.request_repaint();
    }

    if drag.delta != 0.0 {
        let held: f32 = ctx.data_mut(|data| {
            data.get_temp_mut_or_insert_with(start_id, || full);
            let held = data.get_temp_mut_or_insert_with(held_id, || full);
            *held -= drag.delta;
            *held
        });

        if held >= least(ctx) {
            let kept = held.min(most(ctx, room));
            ctx.data_mut(|data| data.insert_persisted(height_id(), kept));
        }
    }

    if !drag.released {
        return false;
    }

    let (start, held): (Option<f32>, Option<f32>) = ctx.data_mut(|data| {
        let both = (data.get_temp(start_id), data.get_temp(held_id));
        data.remove::<f32>(start_id);
        data.remove::<f32>(held_id);
        both
    });

    let shut = held.is_some_and(|held| held < least(ctx));
    if shut {
        // It opens again at the height it had before the drag.
        if let Some(start) = start {
            ctx.data_mut(|data| data.insert_persisted(height_id(), start));
        }

        motion::settle(ctx, slide_id(), false);
    }

    shut
}

fn slide_id() -> Id {
    Id::unique("toolbox_gui::console::slide")
}

fn height_id() -> Id {
    Id::unique("toolbox_gui::console::height")
}

fn area_id() -> Id {
    Id::unique("toolbox_gui::console::sheet")
}

pub fn layer() -> egui::LayerId {
    egui::LayerId::new(Order::Middle, area_id())
}

/// Where the sheet covered the page last frame, none while it was gone
/// or drew clear for its shot.
pub fn cover(ctx: &Context) -> Option<Rect> {
    ctx.data(|data| data.get_temp(cover_id()))
}

fn cover_id() -> Id {
    Id::unique("toolbox_gui::console::cover")
}

/// Escape closes the sheet, unless it belongs to what sits above it (a
/// dialog, an open list, the Changes popover) or to the program in the
/// terminal.
pub fn escape_closes(ctx: &Context) -> bool {
    let busy = ctx.memory(|memory| memory.top_modal_layer().is_some())
        || egui::Popup::is_any_open(ctx)
        || super::super::changes::popover_shows(ctx)
        || ctx.data(|data| data.get_temp::<bool>(terminal_focus_id()).unwrap_or(false));

    !busy && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
}

pub fn terminal_focused(ctx: &Context, focused: bool) {
    ctx.data_mut(|data| data.insert_temp(terminal_focus_id(), focused));
}

fn terminal_focus_id() -> Id {
    Id::unique("toolbox_gui::console::terminal_focus")
}
