mod options;

use std::f32::consts::PI;

use egui::{
    Align, Color32, Event, Layout, Popup, PopupCloseBehavior, PopupKind, Rect, Response, Role,
    ScrollArea, Sense, Ui, Vec2, WidgetInfo, pos2, vec2,
};
#[cfg(test)]
pub(super) use options::option_fill;
use options::{SWATCH_GAP, SWATCH_WIDTH, options_in, paint_swatch, swatch_room};

use super::menu::{ITEM_INSET, MENU_WIDTH, menu_glass};
use super::press::{Pointer, hover_amount};
use super::text::cut;
use super::{Choice, focus_ring, line, rich};
use crate::fonts::{BODY, Weight};
use crate::motion::{self, Timing};
use crate::theme::{CONTROL_RADIUS, PILL_HEIGHT};
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

const PADDING_X: f32 = 14.0;
const CHEVRON: f32 = 16.0;
/// An option of the open list, as tall as a pill outside a row.
pub(super) const OPTION_HEIGHT: f32 = PILL_HEIGHT;
/// The widest an open list grows for a long option, such as a shell's
/// whole store path, before it cuts the option.
pub(super) const LIST_MAX_WIDTH: f32 = MENU_WIDTH * 2.0;
/// Between the pill and its open list, and between the list and the
/// window edge.
const PILL_GAP: f32 = 6.0;
/// The outline of the list's glass, outside its padding.
const LIST_STROKE: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
#[must_use = "a dropdown shows nothing until `show` is called"]
pub struct Select<'a> {
    label: &'a str,
    options: &'a [&'a str],
    selected: Option<usize>,
    placeholder: &'a str,
    /// Shown left of the pill, for a dropdown whose purpose the page
    /// does not say otherwise, such as a sort.
    caption: Option<&'a str>,
    /// More about the chosen option than its short name, such as a
    /// shell's whole path, on hover and for screen readers.
    detail: Option<&'a str>,
    width: f32,
    /// One colour strip per option, the same order, empty for none.
    swatches: &'a [[Color32; 3]],
}

impl<'a> Select<'a> {
    pub const fn new(label: &'a str, options: &'a [&'a str]) -> Self {
        Self {
            label,
            options,
            selected: None,
            placeholder: "",
            caption: None,
            detail: None,
            width: MENU_WIDTH,
            swatches: &[],
        }
    }

    pub const fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Colours to show before each option: window, text and accent, in
    /// that order. The chosen one shows in the closed pill too.
    pub const fn swatches(mut self, swatches: &'a [[Color32; 3]]) -> Self {
        self.swatches = swatches;
        self
    }

    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub const fn detail(mut self, detail: Option<&'a str>) -> Self {
        self.detail = detail;
        self
    }

    pub const fn caption(mut self, caption: &'a str) -> Self {
        self.caption = Some(caption);
        self
    }

    /// The pill width that shows the widest of `texts` whole, padding and
    /// chevron included, up to the widest an open list grows. For a
    /// [`Self::width`] whose options must not be cut.
    pub fn fitting_width<'t>(ui: &Ui, texts: impl IntoIterator<Item = &'t str>) -> f32 {
        let tokens = tokens::current(ui.ctx());
        let widest = texts
            .into_iter()
            .map(|text| {
                line(ui, text, Weight::Medium, BODY, Color32::PLACEHOLDER)
                    .size()
                    .x
            })
            .fold(0.0, f32::max);

        let chrome = PADDING_X.mul_add(2.0, CHEVRON) + tokens.sizes.text_to_icon;
        (widest + chrome).min(LIST_MAX_WIDTH)
    }

    pub(super) fn full_width(&self, ui: &Ui) -> f32 {
        self.caption.map_or(self.width, |caption| {
            let tokens = tokens::current(ui.ctx());
            let galley = line(ui, caption, Weight::Regular, BODY, tokens.secondary);
            galley.size().x + tokens.sizes.control_gap + self.width
        })
    }

    /// Returns the option picked this frame, also when it is the chosen
    /// one.
    pub fn show(self, ui: &mut Ui) -> Choice<usize> {
        let Some(caption) = self.caption else {
            return self.pill(ui);
        };

        // Caption first whatever way the row runs, so it reads before the
        // pill it names. The pill carries the name for screen readers.
        let size = vec2(self.full_width(ui), PILL_HEIGHT);
        ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let tokens = tokens::current(ui.ctx());
            let galley = line(ui, caption, Weight::Regular, BODY, tokens.secondary);
            let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
            ui.painter().galley(rect.min, galley, tokens.secondary);
            ui.add_space(tokens.sizes.control_gap);
            self.pill(ui)
        })
        .inner
    }

    fn pill(self, ui: &mut Ui) -> Choice<usize> {
        let Self {
            label,
            options,
            selected,
            placeholder,
            caption: _,
            detail,
            width,
            swatches,
        } = self;

        let shown = selected
            .and_then(|index| options.get(index))
            .copied()
            .unwrap_or(placeholder);

        let height = PILL_HEIGHT;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::ComboBox, ui.is_enabled(), label);
            info.current_text_value = Some(shown.to_owned());
            info
        });

        if let Some(detail) = detail {
            let detail = detail.to_owned();
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_description(detail));
        }

        // The value is whole for screen readers already, a tooltip shows
        // it whole to the eye when the pill cut it, or its detail.
        let swatch = selected.and_then(|index| swatches.get(index)).copied();
        let cut = paint_pill(ui, &response, rect, shown, swatch);
        let response = match (detail, cut) {
            (Some(detail), _) => response.on_hover_ui(|tip| _ = tip.label(detail)),
            (None, true) => response.on_hover_ui(|tip| _ = tip.label(shown)),
            (None, false) => response,
        };

        let picked = list(ui, &response, options, swatches, selected);

        // Read after the list ran, which opens or closes it this frame.
        let open = Popup::is_id_open(ui.ctx(), Popup::default_response_id(&response));
        paint_chevron(ui, &response, open);
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_expanded(open));
        Choice {
            response,
            value: picked,
        }
    }
}

/// How wide the open list is: as wide as its pill or its widest option,
/// at most [`LIST_MAX_WIDTH`]. A wider option is cut.
fn list_width(ui: &Ui, pill: f32, options: &[&str], swatch: f32) -> f32 {
    let widest = options
        .iter()
        .map(|option| {
            line(ui, option, Weight::SemiBold, BODY, Color32::PLACEHOLDER)
                .size()
                .x
        })
        .fold(0.0, f32::max);

    let padding = tokens::current(ui.ctx()).sizes.list_padding.sum().x;
    let wanted = ITEM_INSET.mul_add(2.0, widest) + padding + swatch;
    wanted.clamp(pill, LIST_MAX_WIDTH.max(pill))
}

/// How tall the options may grow before they scroll: the room on the
/// roomier side of `pill`, so the popup flips above when that is larger.
fn list_height(ui: &Ui, pill: Rect) -> f32 {
    let window = ui.ctx().content_rect();
    let padding = tokens::current(ui.ctx()).sizes.list_padding.sum().y;
    let room = (window.bottom() - pill.bottom()).max(pill.top() - window.top());
    let chrome = LIST_STROKE.mul_add(2.0, PILL_GAP.mul_add(2.0, padding));
    (room - chrome).max(OPTION_HEIGHT)
}

/// The open list under `pill`.
fn list(
    ui: &Ui,
    pill: &Response,
    options: &[&str],
    swatches: &[[Color32; 3]],
    selected: Option<usize>,
) -> Option<usize> {
    let glass = menu_glass(ui);
    let scrolled_id = Popup::default_response_id(pill).with("scrolled");
    let height = list_height(ui, pill.rect);

    // A menu, so egui walks its options with the arrow keys.
    let shown = Popup::from_toggle_button_response(pill)
        .kind(PopupKind::Menu)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .gap(PILL_GAP)
        .width(list_width(
            ui,
            pill.rect.width(),
            options,
            swatch_room(swatches),
        ))
        .frame(glass.frame())
        .show(|popup| {
            // Once per opening, and not in the sizing pass, which lays the
            // list out unseen and scrolls nothing.
            let reveal = !popup.is_sizing_pass()
                && popup.data(|data| data.get_temp::<bool>(scrolled_id).is_none());

            if reveal {
                popup.data_mut(|data| data.insert_temp(scrolled_id, true));
            }

            glass.inside(popup, |inside| {
                ScrollArea::vertical()
                    .max_height(height)
                    .auto_shrink([false, true])
                    .show(inside, |list| {
                        options_in(list, options, swatches, selected, reveal)
                    })
                    .inner
            })
        });

    let Some(open) = shown else {
        ui.data_mut(|data| data.remove::<bool>(scrolled_id));
        return None;
    };

    open.inner
}

/// Draws the pill. Returns whether `shown` was cut to fit.
fn paint_pill(
    ui: &Ui,
    response: &Response,
    rect: Rect,
    shown: &str,
    swatch: Option<[Color32; 3]>,
) -> bool {
    if !ui.is_rect_visible(rect) {
        return false;
    }

    let tokens = tokens::current(ui.ctx());
    let pointer = Pointer::of_focusable(response);
    let fill = if pointer == Pointer::Pressed {
        tokens.pressed
    } else {
        tokens
            .field
            .lerp_to_gamma(tokens.hover, hover_amount(ui, response, pointer))
    };

    // A disabled ui fades it once, see `press.rs`.
    let ink = tokens.text;

    ui.painter().rect_filled(rect, CONTROL_RADIUS, fill);
    let before = swatch.map_or(0.0, |colours| {
        paint_swatch(ui, rect.left() + PADDING_X, rect.center().y, colours);
        SWATCH_WIDTH + SWATCH_GAP
    });

    let text = Rect::from_min_max(
        pos2(rect.left() + PADDING_X + before, rect.top()),
        pos2(
            rect.right() - PADDING_X - CHEVRON - tokens.sizes.text_to_icon,
            rect.bottom(),
        ),
    );

    let galley = cut(
        ui,
        rich(ui, shown, Weight::Medium, BODY, ink),
        text.width().max(0.0),
    );

    let was_cut = galley.elided;
    let text_top = rect.center().y - galley.size().y / 2.0;
    ui.painter()
        .with_clip_rect(text)
        .galley(pos2(text.left(), text_top), galley, ink);

    focus_ring(ui, response, CONTROL_RADIUS);
    was_cut
}

fn paint_chevron(ui: &Ui, pill: &Response, open: bool) {
    let turn = Timing::both(motion::CHEVRON_TURN);
    let id = pill.id.with("chevron");

    // A key that opened or closed the list jumps, a click turns. A key
    // pressed while a click's turn runs leaves that turn alone.
    let was_open = ui.data_mut(|data| {
        let key = id.with("open");
        let was = data.get_temp::<bool>(key);
        data.insert_temp(key, open);
        was
    });

    let toggled = was_open.is_some_and(|was| was != open);
    let keyed = toggled
        && ui.input(|input| {
            input.events.iter().any(|event| {
                matches!(
                    event,
                    Event::Key {
                        pressed: true,
                        key: _,
                        physical_key: _,
                        repeat: _,
                        modifiers: _,
                    }
                )
            })
        });

    if keyed {
        motion::settle(ui.ctx(), id, open);
    }

    let turned = motion::presence(ui.ctx(), id, open, turn);
    let rect = pill.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }

    let chevron = Rect::from_center_size(
        pos2(rect.right() - PADDING_X - CHEVRON / 2.0, rect.center().y),
        vec2(CHEVRON, CHEVRON),
    );

    let image = icon_image(ui, icons::GO_DOWN, tokens::current(ui.ctx()).secondary);
    let image = if turned > 0.0 {
        image.rotate(turned * PI, Vec2::splat(0.5))
    } else {
        image
    };

    image.paint_at(ui, chevron);
}
