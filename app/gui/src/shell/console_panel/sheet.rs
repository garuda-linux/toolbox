//! The sheet over the page: it slides up from the page's bottom edge to
//! three quarters of its height, or to the height the grip dragged it
//! to. The page keeps its size below it, and what of it shows above the
//! sheet stays usable.

use egui::accesskit::Role;
use egui::{
    Align, Area, Color32, Context, CursorIcon, Frame, Id, Key, Layout, Modifiers, Order, Rect,
    Sense, Stroke, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};
use toolbox_gui_kit::{frost, motion, tokens};

use super::Sheet;

/// The share of the page the sheet takes when first opened.
const SHARE: f32 = 0.75;
/// Below this the grip lets go of the sheet and it closes, in points.
const SHUT: f32 = 120.0;
/// The page room the sheet always leaves above itself, in points.
const ABOVE: f32 = 48.0;
const BAR: egui::Vec2 = vec2(40.0, 4.0);
const GRIP_HEIGHT: f32 = 10.0;

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

/// `grip` names the bar for screen readers.
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
    let top = full.mul_add(-out, room.bottom());
    let rect = Rect::from_min_max(pos2(room.left(), top), room.right_bottom());
    let whole = Rect::from_min_size(rect.min, vec2(room.width(), full));

    ctx.data_mut(|data| data.insert_temp(cover_id(), rect));
    let frame = frosted(ctx, frame);

    // The page header lies in the same order of layers. A click on it
    // raises it, the sheet stays above it.
    ctx.move_to_top(egui::LayerId::new(Order::Middle, area_id()));

    let mut dragged_shut = false;
    area.fixed_pos(rect.min).show(ctx, |ui| {
        ui.set_clip_rect(rect);
        frost::backdrop(ui.painter(), whole, frame.corner_radius);
        let mut sheet_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(whole)
                .layout(Layout::top_down(Align::Min)),
        );

        frame.show(&mut sheet_ui, |inner| {
            inner.set_min_size(inner.available_size());
            let drag = drag_bar(inner, whole.top(), grip);
            dragged_shut = resize(ctx, room, full, drag);
            inner.scope_builder(
                UiBuilder::new().id_salt("toolbox_gui::console::contents"),
                contents,
            );
        });
    });

    Shown {
        rect: Some(rect),
        dragged_shut,
    }
}

/// `frame` with the frost's tint for its fill while the blur lies under it.
fn frosted(ctx: &Context, frame: Frame) -> Frame {
    if frost::live(ctx) {
        frame.fill(tokens::current(ctx).frost)
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
        .clamp(SHUT, most(room))
}

fn most(room: Rect) -> f32 {
    (room.height() - ABOVE).max(SHUT)
}

/// The bar that shows the sheet can be dragged taller. Its hit area
/// reaches from the sheet's top edge down past the bar.
fn drag_bar(ui: &mut Ui, top: f32, label: &str) -> Drag {
    let tokens = tokens::current(ui.ctx());
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), GRIP_HEIGHT), Sense::hover());
    let bar = Rect::from_center_size(row.center(), BAR);
    let hit = Rect::from_min_max(pos2(row.left(), top), pos2(row.right(), row.bottom()));

    let response = ui
        .interact(hit, ui.scope_id().with("grip"), Sense::drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);

    response.widget_info(|| WidgetInfo::labeled(Role::Splitter, true, label));

    let colour = if response.hovered() || response.dragged() {
        tokens.secondary
    } else {
        tokens.dim
    };

    ui.painter().rect_filled(bar, BAR.y / 2.0, colour);

    Drag {
        delta: response.drag_delta().y,
        released: response.drag_stopped(),
    }
}

/// Applies the grip's drag to the kept height. Returns whether it let go
/// below the least height, which closes the sheet at the height it had.
fn resize(ctx: &Context, room: Rect, full: f32, drag: Drag) -> bool {
    let start_id = height_id().with("start");
    let held_id = height_id().with("held");

    if drag.delta != 0.0 {
        let held: f32 = ctx.data_mut(|data| {
            data.get_temp_mut_or_insert_with(start_id, || full);
            let held = data.get_temp_mut_or_insert_with(held_id, || full);
            *held -= drag.delta;
            *held
        });

        if held >= SHUT {
            ctx.data_mut(|data| data.insert_persisted(height_id(), held.min(most(room))));
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

    let shut = held.is_some_and(|held| held < SHUT);
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

/// Where the sheet covered the page last frame, none while it was gone
/// or drew clear for its shot.
pub fn cover(ctx: &Context) -> Option<Rect> {
    ctx.data(|data| data.get_temp(cover_id()))
}

fn cover_id() -> Id {
    Id::unique("toolbox_gui::console::cover")
}

/// Escape closes the sheet, unless it belongs to what sits above it (a
/// dialog, an open list) or to the program in the terminal.
pub fn escape_closes(ctx: &Context) -> bool {
    let busy = ctx.memory(|memory| memory.top_modal_layer().is_some())
        || egui::Popup::is_any_open(ctx)
        || ctx.data(|data| data.get_temp::<bool>(terminal_focus_id()).unwrap_or(false));

    !busy && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
}

pub fn terminal_focused(ctx: &Context, focused: bool) {
    ctx.data_mut(|data| data.insert_temp(terminal_focus_id(), focused));
}

fn terminal_focus_id() -> Id {
    Id::unique("toolbox_gui::console::terminal_focus")
}
