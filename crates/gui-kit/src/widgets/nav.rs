//! The sidebar's entries and the labels of its groups.

use egui::{Color32, CornerRadius, Response, Role, Sense, Ui, WidgetInfo, pos2, vec2};

use super::buttons::glyph_size;
use super::press::Pointer;
use super::{focus_ring, line, rich};
use crate::fonts::{BODY, SMALL, Weight};
use crate::icons::Icon;
use crate::theme::RADIUS;
use crate::tokens;
use crate::ui_ext::icon_image;

const PADDING_X: f32 = 12.0;
const ICON_GAP: f32 = 12.0;
/// The group labels' tracking, in points, as small capitals want.
const TRACKING: f32 = 0.7;
const LABEL_SCALE: f32 = SMALL * 0.85;
/// Above a group label in a sidebar that is not packed.
const LABEL_ROOM: f32 = 4.0;
/// How opaque an entry at a cut edge of a scrolled list stays.
const FADED: f32 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavLook {
    /// The labels show. Collapsed, the icon alone does, and the label
    /// names it and is its hover text.
    pub expanded: bool,
    /// One of the sidebar's row heights in the tokens.
    pub height: f32,
}

/// The fade of a scrolled list's entries toward an edge with more beyond.
///
/// It fades the entries themselves, not a band of the panel's colour over
/// them, so the selection tint and the hover fill fade with them.
#[derive(Clone, Copy, Debug)]
pub struct EdgeFade {
    view: egui::Rect,
    above: bool,
    below: bool,
    reach: f32,
}

impl EdgeFade {
    /// The fade inside a vertical scroll area's content `ui`, whose
    /// content is `height` tall, over `reach` from each edge.
    #[must_use = "the fade does nothing until asked"]
    pub fn of(ui: &Ui, height: f32, reach: f32) -> Self {
        let view = ui.clip_rect();
        let top = ui.max_rect().top();
        Self {
            view,
            above: top < view.top() - 0.5,
            below: top + height > view.bottom() + 0.5,
            reach,
        }
    }

    /// Whether the list goes on beyond either edge.
    pub const fn is_cut(&self) -> bool {
        self.above || self.below
    }

    /// How opaque an entry `top` to `bottom` draws: whole away from a cut
    /// edge, fading toward one.
    pub fn opacity(&self, top: f32, bottom: f32) -> f32 {
        let centre = f32::midpoint(top, bottom);
        let near_top = if self.above {
            (centre - self.view.top()) / self.reach
        } else {
            1.0
        };

        let near_bottom = if self.below {
            (self.view.bottom() - centre) / self.reach
        } else {
            1.0
        };

        egui::lerp(FADED..=1.0, near_top.min(near_bottom).clamp(0.0, 1.0))
    }

    /// Whether a cut edge runs through `top` to `bottom`.
    pub fn cuts(&self, top: f32, bottom: f32) -> bool {
        let through = |edge: f32| top < edge - 0.5 && bottom > edge + 0.5;
        (self.above && through(self.view.top())) || (self.below && through(self.view.bottom()))
    }

    pub fn show<R>(&self, ui: &mut Ui, height: f32, entry: impl FnOnce(&mut Ui) -> R) -> R {
        let top = ui.cursor().top();
        faded(ui, self.opacity(top, top + height), entry)
    }
}

fn faded<R>(ui: &mut Ui, opacity: f32, entry: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.multiply_opacity(opacity);
        entry(ui)
    })
    .inner
}

pub fn nav_row(ui: &mut Ui, icon: Icon, label: &str, selected: bool, look: NavLook) -> Response {
    let NavLook { expanded, height } = look;
    let tokens = tokens::current(ui.ctx());

    // Collapsed, every entry keeps the full square, so the icons stay
    // centred in the narrow sidebar.
    let width = if expanded {
        ui.available_width()
    } else {
        tokens.sizes.nav_row_height
    };

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Tab, ui.is_enabled(), label));
    mark_selected(ui, &response, selected);

    if ui.is_rect_visible(rect) {
        let pointer = Pointer::of(&response);
        let hovered = pointer.is_under();
        let fill = match (pointer, selected) {
            (Pointer::Pressed, _) => Some(tokens.pressed),
            (_, true) => Some(tokens.accent_tint),
            (Pointer::Hovered, false) => Some(tokens.field),
            (Pointer::Rest, false) => None,
        };

        if let Some(fill) = fill {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(RADIUS), fill);
        }

        let ink = if selected {
            tokens.on_tint
        } else if hovered {
            tokens.text
        } else {
            tokens.sidebar_text
        };

        let image = icon_image(ui, icon, ink);
        let glyph = glyph_size(&image);
        let icon_left = if expanded {
            rect.left() + PADDING_X
        } else {
            rect.center().x - glyph.x / 2.0
        };

        let glyph_rect =
            egui::Rect::from_min_size(pos2(icon_left, rect.center().y - glyph.y / 2.0), glyph);

        image.paint_at(ui, glyph_rect);

        if expanded {
            let weight = if selected {
                Weight::SemiBold
            } else {
                Weight::Medium
            };

            let galley = line(ui, label, weight, BODY, ink);
            let at = pos2(
                glyph_rect.right() + ICON_GAP,
                rect.center().y - galley.size().y / 2.0,
            );

            ui.painter().galley(at, galley, ink);
        }

        focus_ring(ui, &response, RADIUS);
    }

    if expanded {
        response
    } else {
        response.on_hover_text(label)
    }
}

/// The small capitals above a group of entries. `compact` is the packed
/// sidebar of a short window.
pub fn group_label(ui: &mut Ui, text: &str, compact: bool) {
    let tokens = tokens::current(ui.ctx());
    if !compact {
        ui.add_space(LABEL_ROOM);
    }

    // A short window packs the label to its text's height. The row takes
    // its height from the ui it starts in, so the packing goes on before.
    let saved = ui.spacing().interact_size.y;
    if compact {
        ui.spacing_mut().interact_size.y = 0.0;
    }

    ui.horizontal(|ui| {
        ui.add_space(PADDING_X);
        ui.label(
            rich(
                ui,
                &text.to_uppercase(),
                Weight::SemiBold,
                LABEL_SCALE,
                tokens.sidebar_text,
            )
            .extra_letter_spacing(TRACKING),
        );
    });

    ui.spacing_mut().interact_size.y = saved;
}

/// How tall [`group_label`] stands, its room above included, measured
/// without drawing it, so the sidebar can tell beforehand whether its
/// entries fit.
pub fn group_label_height(ui: &Ui, compact: bool) -> f32 {
    let text = line(ui, "A", Weight::SemiBold, LABEL_SCALE, Color32::PLACEHOLDER)
        .size()
        .y;

    if compact {
        text
    } else {
        LABEL_ROOM + text.max(ui.spacing().interact_size.y)
    }
}

/// The height [`nav_divider`] takes with `room` above and below its line.
pub const fn nav_divider_height(room: f32) -> f32 {
    room.mul_add(2.0, 1.0)
}

/// A thin line that parts the groups of a collapsed sidebar, where their
/// labels do not show. `room` above and below it.
pub fn nav_divider(ui: &mut Ui, room: f32) {
    let tokens = tokens::current(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), nav_divider_height(room)),
        Sense::hover(),
    );

    ui.painter().hline(
        (rect.left() + PADDING_X)..=(rect.right() - PADDING_X),
        rect.center().y,
        egui::Stroke::new(1.0, tokens.sidebar_edge),
    );
}

/// Says "selected" for the open page or the chosen option. egui says
/// "pressed" for a selected button, which reads as a toggle.
pub(super) fn mark_selected(ui: &Ui, response: &Response, selected: bool) {
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.clear_toggled();
        node.set_selected(selected);
    });
}
