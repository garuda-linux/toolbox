mod body;

use std::fmt::Debug;
use std::hash::{BuildHasher as _, BuildHasherDefault, DefaultHasher, Hash};

pub use body::BodyShape;
use body::Room;
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, Rect, ScrollArea, Shape, Ui, UiBuilder, pos2, vec2};

use super::layout::moved;
use crate::theme::CONTENT_WIDTH;
use crate::{frost, motion, tokens};

/// How far the shadow over a footer reaches, in points.
const SHADOW_DEPTH: f32 = 8.0;

/// What a page draws, each from the page in turn: the fixed header, the
/// body that scrolls and the fixed footer. `header` and `footer` return
/// whether they drew anything. One that drew nothing takes no room.
pub struct PageParts<H, B, F> {
    pub header: H,
    pub body: B,
    pub footer: F,
}

/// A band's height as the last frame left it.
#[derive(Clone, Copy)]
struct Remembered {
    id: egui::Id,
    height: f32,
}

impl Remembered {
    fn of(ui: &Ui, salt: impl Hash + Debug) -> Self {
        let id = ui.scope_id().with(salt);
        let height = ui.data(|data| data.get_temp(id)).unwrap_or(0.0);
        Self { id, height }
    }

    /// Redraws this frame when the height changed, so no frame shows the old height.
    fn update(self, ui: &Ui, height: f32, reason: &'static str) {
        if moved(self.height, height) {
            ui.data_mut(|data| data.insert_temp(self.id, height));
            ui.ctx().request_discard(reason);
        }
    }
}

/// The header and the footer return whether they drew anything: a page without a header starts its body at the top. `salt` keeps each place's scroll position apart.
pub fn page_layout_shaped<P, H, B, F>(
    ui: &mut Ui,
    salt: impl Hash + Debug,
    page: &mut P,
    parts: PageParts<H, B, F>,
    shape: BodyShape,
) where
    H: FnOnce(&mut P, &mut Ui) -> bool,
    B: FnOnce(&mut P, &mut Ui),
    F: FnOnce(&mut P, &mut Ui) -> bool,
{
    let width = shape.width();
    let PageParts {
        header,
        body,
        footer,
    } = parts;

    let sizes = tokens::current(ui.ctx()).sizes;
    let outer = ui.available_rect_before_wrap();

    // The header comes first for the keyboard, above the body, its height
    // from the last frame. One header serves every place: a new area per
    // tab would stay hidden for its first frame.
    let head = Remembered::of(ui, "page header");
    let line = outer.top() + head.height;

    // Measured from the band's edge, which is where the eye sees the page
    // start, so every page's first block keeps the same room.
    let top = if head.height > 0.0 {
        sizes.header_gap
    } else {
        f32::from(sizes.page_padding.top)
    };

    let band = Rect::from_min_max(outer.min, pos2(outer.right(), line));
    let header_layer = blurred_header(ui, band, head, width, |ui| header(page, ui));

    let foot = Remembered::of(ui, "page footer");
    let footer_top = outer.bottom() - foot.height;
    let bottom = f32::from(sizes.page_padding.bottom);
    let (view_bottom, floor) = body_reach(ui, outer, line, footer_top);

    let (area, changed) = body_area(ui, &salt);
    let room = Room {
        view: Rect::from_min_max(pos2(outer.left(), line), pos2(outer.right(), view_bottom)),
        top,
        bottom,
        page_top: outer.top(),
        floor,
        entered: entry(ui, changed),
    };

    let (offset, hidden) = match shape {
        BodyShape::Panes { width: widest } => {
            body::panes(ui, room, widest, |inner| body(page, inner));
            (0.0, 0.0)
        }
        BodyShape::Column => body::scrolled(ui, area, room, |inner| body(page, inner)),
    };

    if head.height > 0.0 && offset > 0.5 {
        let hairline = egui::Stroke::new(1.0, tokens::current(ui.ctx()).outline);
        ui.ctx()
            .layer_painter(header_layer)
            .hline(band.x_range(), band.bottom() - 0.5, hairline);
    }

    let rest = Rect::from_min_max(pos2(outer.left(), footer_top), outer.max);
    let mut footer_ui =
        ui.new_child(UiBuilder::new().max_rect(column_rect(ui, rest, sizes.header_gap, width)));

    let height = if footer(page, &mut footer_ui) {
        footer_ui.min_rect().bottom() + f32::from(sizes.page_padding.bottom) - footer_top
    } else {
        0.0
    };

    if height > 0.0 && hidden > 0.5 {
        shadow(ui, footer_top);
    }

    foot.update(ui, height, "the page footer changed its height");
    ui.advance_cursor_after_rect(outer);
}

/// What floats over the page's bottom ends the area above it, so the last
/// row scrolls up above the cover and a control scrolled into sight lands
/// there. The painting runs on down to the footer, under the cover.
fn body_reach(ui: &Ui, outer: Rect, line: f32, footer_top: f32) -> (f32, f32) {
    let floor = footer_top.min(ui.clip_rect().bottom());
    let view = footer_top
        .min(outer.bottom() - take_covered(ui.ctx()))
        .max(line);

    (view, floor)
}

/// How far the band reaches below the header, so what runs under it
/// never touches the header's last line.
fn band_below(ui: &Ui) -> f32 {
    let sizes = tokens::current(ui.ctx()).sizes;
    sizes.card_gap.min(sizes.header_gap)
}

fn blurred_header(
    ui: &Ui,
    band: Rect,
    head: Remembered,
    width: f32,
    header: impl FnOnce(&mut Ui) -> bool,
) -> egui::LayerId {
    let page_top = f32::from(tokens::current(ui.ctx()).sizes.page_padding.top);
    let below = band_below(ui);
    let clip = ui.clip_rect();

    let shown = egui::Area::new(head.id.with("area"))
        .order(egui::Order::Middle)
        .fixed_pos(band.min)
        .movable(false)
        .constrain(false)
        .show(ui.ctx(), |area| {
            area.set_clip_rect(clip.intersect(Rect::from_min_size(
                band.min,
                vec2(band.width(), f32::INFINITY),
            )));
            if head.height > 0.0 {
                let tokens = tokens::current(area.ctx());

                // Without the blur, no scrim keeps the sharp page under it
                // from showing, so the band turns solid.
                let cover = if frost::live(area.ctx()) {
                    frost::backdrop_within(area.painter(), band, egui::CornerRadius::ZERO);
                    tokens.header_scrim
                } else {
                    tokens.window
                };

                area.painter()
                    .rect_filled(band, egui::CornerRadius::ZERO, cover);
            }

            let drawn = header_band(area, band, page_top, width, header);
            let height = drawn.map_or(0.0, |bottom| bottom + below - band.top());
            head.update(area, height, "the page header changed its height");
            area.allocate_rect(
                Rect::from_min_size(band.min, vec2(band.width(), head.height)),
                egui::Sense::hover(),
            );
        });

    shown.response.layer_id
}

/// Another page or tab starts at the top, also one shown before. The
/// same one keeps its place through dialogs, the Console and refreshes.
/// Also returns whether it replaced another place, which the window's
/// first page did not.
fn body_area(ui: &Ui, salt: &(impl Hash + Debug)) -> (ScrollArea, bool) {
    let place = BuildHasherDefault::<DefaultHasher>::default().hash_one(salt);
    let place_id = ui.scope_id().with("page place");
    let before = ui.data(|data| data.get_temp::<u64>(place_id));
    ui.data_mut(|data| data.insert_temp(place_id, place));

    let area = ScrollArea::vertical().auto_shrink(false).id_salt(salt);
    if before == Some(place) {
        (area, false)
    } else {
        (area.vertical_scroll_offset(0.0), before.is_some())
    }
}

/// How far the body has come in, 0 to 1. Only another page the pointer
/// opened ([`start_page_entry`]) fades in and rises: a tab of the same page
/// or a shortcut shows in place.
fn entry(ui: &Ui, changed: bool) -> f32 {
    let id = ui.scope_id().with("page entry");
    let started = ui
        .ctx()
        .data_mut(|data| data.remove_temp::<bool>(start_id()))
        .unwrap_or(false);

    if changed {
        motion::settle(ui.ctx(), id, !started);
    }

    let timing = motion::Timing::new(motion::PAGE_ENTER, 0.0);
    motion::presence(ui.ctx(), id, true, timing)
}

/// Motion follows the pointer, never the keyboard.
pub fn start_page_entry(ctx: &egui::Context) {
    ctx.data_mut(|data| data.insert_temp(start_id(), true));
}

fn start_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::page::start_entry")
}

/// The header in the page column of `outer`, `top` below its edge. Where
/// it ends, or nothing when `header` drew nothing.
fn header_band(
    ui: &mut Ui,
    outer: Rect,
    top: f32,
    width: f32,
    header: impl FnOnce(&mut Ui) -> bool,
) -> Option<f32> {
    let rect = column_rect(ui, outer, top, width);
    let mut band = ui.new_child(UiBuilder::new().max_rect(rect));
    band.set_width(rect.width());

    header(&mut band).then(|| band.min_rect().bottom())
}

pub(super) fn column<R>(ui: &mut Ui, top: f32, bottom: f32, body: impl FnOnce(&mut Ui) -> R) -> R {
    let outer = ui.available_rect_before_wrap();
    let rect = column_rect(ui, outer, top, CONTENT_WIDTH);
    let inner = ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.set_width(rect.width());
        body(ui)
    });

    let end = inner.response.rect.bottom() + bottom;
    ui.advance_cursor_after_rect(Rect::from_min_max(outer.min, pos2(outer.right(), end)));
    inner.inner
}

fn column_rect(ui: &Ui, outer: Rect, top: f32, widest: f32) -> Rect {
    let padding = tokens::current(ui.ctx()).sizes.page_padding;
    let sides = f32::from(padding.left) + f32::from(padding.right);
    let width = (outer.width() - sides).clamp(0.0, widest);
    Rect::from_min_size(
        pos2(outer.center().x - width / 2.0, outer.top() + top),
        vec2(width, f32::INFINITY),
    )
}

fn shadow(ui: &Ui, line: f32) {
    let colour = tokens::current(ui.ctx()).shadow;
    let (near, far) = (line, line - SHADOW_DEPTH);

    let (left, right) = (ui.max_rect().left(), ui.max_rect().right());
    let mut mesh = Mesh::default();
    let corner = |x, y, color| Vertex {
        pos: pos2(x, y),
        uv: WHITE_UV,
        color,
    };

    mesh.vertices.extend([
        corner(left, near, colour),
        corner(right, near, colour),
        corner(right, far, Color32::TRANSPARENT),
        corner(left, far, Color32::TRANSPARENT),
    ]);
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    ui.painter().add(Shape::mesh(mesh));
}

/// The page paints on under the cover, so a blur there shows it, and its
/// scroll area ends above it, so every row can come up above the cover.
pub fn cover_page_bottom(ctx: &egui::Context, height: f32) {
    ctx.data_mut(|data| data.insert_temp(covered_id(), height.max(0.0)));
}

fn take_covered(ctx: &egui::Context) -> f32 {
    ctx.data_mut(|data| data.remove_temp::<f32>(covered_id()))
        .unwrap_or(0.0)
}

fn covered_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::page::covered")
}
