//! The sidebar's entries and the labels of its groups.

use egui::{Color32, Response, Role, Sense, Ui, WidgetInfo, pos2, vec2};

use super::focus::item_focus_line;
use super::press::{Pointer, item_look, paint_item};
use super::text::cut;
use super::{line, rich};
use crate::fonts::{BODY, SMALL, Weight};
use crate::icons::{self, Icon};
use crate::tokens;

/// Entries and headings start their icon and text this far in, and a
/// sidebar's own lines line up with them.
pub const NAV_INSET: f32 = 12.0;

/// The sidebar's side inset: the left edge of the search field and of every
/// item highlight, and the right edge of both. System Settings' item
/// delegate insets its highlight by 6.
pub const SIDE_INSET: f32 = 6.0;

/// The highlight's top inset in its row: the 30 point pill sits 6 points
/// below the row's top, so rows 36 points apart leave 6 between pills.
pub const HIGHLIGHT_TOP: f32 = 6.0;

/// Kirigami's smallMedium icon, as KDE's sidebar draws its entries.
pub const ICON: f32 = 22.0;

const ICON_GAP: f32 = 12.0;
const LABEL_SCALE: f32 = SMALL;
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
        self.fade_within(top, bottom, true)
    }

    /// [`Self::opacity`] for a list whose bottom cut another fill covers, so
    /// the entries near that edge keep their strength and only the top fades.
    pub fn opacity_top_only(&self, top: f32, bottom: f32) -> f32 {
        self.fade_within(top, bottom, false)
    }

    fn fade_within(&self, top: f32, bottom: f32, below: bool) -> f32 {
        let centre = f32::midpoint(top, bottom);
        let near_top = if self.above {
            (centre - self.view.top()) / self.reach
        } else {
            1.0
        };

        let near_bottom = if self.below && below {
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
        height
    };

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Tab, ui.is_enabled(), label));
    mark_selected(ui, &response, selected);

    let mut was_cut = false;
    if ui.is_rect_visible(rect) {
        let pointer = Pointer::of(&response);
        let highlight = highlight_rect(rect);

        // The icon and the text centre on the highlight, which starts
        // `HIGHLIGHT_TOP` below the row's top, not on the row.
        let middle = highlight.center().y;

        paint_item(ui, highlight, item_look(&tokens, selected, pointer));
        item_focus_line(ui, &response, highlight, selected);

        let ink = if selected {
            tokens.selection_text
        } else {
            tokens.sidebar_text
        };

        let theme = icons::theme_in_use(ui.ctx());
        let image = icon.image(theme.as_deref(), ink);
        let glyph = vec2(ICON, ICON);
        let icon_left = if expanded {
            rect.left() + NAV_INSET
        } else {
            rect.center().x - glyph.x / 2.0
        };

        let glyph_rect = egui::Rect::from_min_size(pos2(icon_left, middle - glyph.y / 2.0), glyph);

        image.paint_at(ui, glyph_rect);

        if expanded {
            let text = rich(ui, label, Weight::Regular, BODY, ink);
            let text_left = glyph_rect.right() + ICON_GAP;
            was_cut = paint_label(ui, rect, (text_left, middle), text, ink);
        }
    }

    // Collapsed, the label shows nowhere else. Cut, it shows in part.
    if expanded && !was_cut {
        response
    } else {
        response.on_hover_text(label)
    }
}

/// The pill an entry paints inside its `row`: the row's sides inset by
/// `SIDE_INSET`, its top by `HIGHLIGHT_TOP`.
pub fn highlight_rect(row: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        row.min + vec2(SIDE_INSET, HIGHLIGHT_TOP),
        row.max - vec2(SIDE_INSET, 0.0),
    )
}

/// The label above a group of entries. `compact` is the packed
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
        ui.add_space(NAV_INSET);
        let shown = ui.label(rich(
            ui,
            text,
            Weight::SemiBold,
            LABEL_SCALE,
            tokens.sidebar_text,
        ));

        super::mark_heading(ui, &shown, text, 2);
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
        (rect.left() + NAV_INSET)..=(rect.right() - NAV_INSET),
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

/// Paints `text` from `left` to the padding of `rect`, its line box
/// centred on `middle`. Returns whether it was cut.
fn paint_label(
    ui: &Ui,
    rect: egui::Rect,
    (left, middle): (f32, f32),
    text: egui::RichText,
    ink: Color32,
) -> bool {
    let room = (rect.right() - NAV_INSET - left).max(0.0);
    let galley = cut(ui, text, room);
    let was_cut = galley.elided;
    let at = pos2(left, middle - galley.size().y / 2.0);
    ui.painter().galley(at, galley, ink);
    was_cut
}
