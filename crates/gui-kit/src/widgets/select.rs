use egui::{
    Align, Color32, CornerRadius, Layout, Popup, PopupCloseBehavior, Rect, Response, Role, Sense,
    Ui, WidgetInfo, pos2, vec2,
};

use super::menu::{ITEM_INSET, LIST_GAP, MENU_WIDTH, menu_glass};
use super::press::Pointer;
use super::text::cut;
use super::{focus_ring, line, rich};
use crate::fonts::{BODY, Weight};
use crate::theme::{PILL_HEIGHT, PILL_RADIUS, RADIUS};
use crate::tokens::Tokens;
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

const PADDING_X: f32 = 14.0;
const CHEVRON: f32 = 16.0;
/// An option of the open list, as tall as a pill outside a row.
const OPTION_HEIGHT: f32 = PILL_HEIGHT;
/// The widest an open list grows for a long option, such as a shell's
/// whole store path, before it cuts the option.
pub(super) const LIST_MAX_WIDTH: f32 = MENU_WIDTH * 2.0;

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
    pub fn show(self, ui: &mut Ui) -> Option<usize> {
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

    fn pill(self, ui: &mut Ui) -> Option<usize> {
        let Self {
            label,
            options,
            selected,
            placeholder,
            caption: _,
            detail,
            width,
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
        let cut = paint_pill(ui, &response, rect, shown);
        let response = match (detail, cut) {
            (Some(detail), _) => response.on_hover_ui(|tip| _ = tip.label(detail)),
            (None, true) => response.on_hover_ui(|tip| _ = tip.label(shown)),
            (None, false) => response,
        };

        list(ui, &response, options, selected)
    }
}

/// How wide the open list is: as wide as its pill or its widest option,
/// at most [`LIST_MAX_WIDTH`]. A wider option is cut.
fn list_width(ui: &Ui, pill: f32, options: &[&str]) -> f32 {
    let widest = options
        .iter()
        .map(|option| {
            line(ui, option, Weight::SemiBold, BODY, Color32::PLACEHOLDER)
                .size()
                .x
        })
        .fold(0.0, f32::max);

    let padding = tokens::current(ui.ctx()).sizes.list_padding.sum().x;
    let wanted = ITEM_INSET.mul_add(2.0, widest) + padding;
    wanted.clamp(pill, LIST_MAX_WIDTH.max(pill))
}

/// The open list under `pill`.
fn list(ui: &Ui, pill: &Response, options: &[&str], selected: Option<usize>) -> Option<usize> {
    let glass = menu_glass(ui);
    let picked = Popup::from_toggle_button_response(pill)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .gap(6.0)
        .width(list_width(ui, pill.rect.width(), options))
        .frame(glass.frame())
        .show(|popup| {
            glass.inside(popup, |list| {
                list.spacing_mut().item_spacing.y = LIST_GAP;
                let mut picked = None;
                for (index, option) in options.iter().enumerate() {
                    if option_row(list, option, Some(index) == selected).clicked() {
                        picked = Some(index);
                    }
                }

                // The close behaviour only hears the pointer. A pick from the
                // keyboard or a screen reader closes the list here.
                if picked.is_some() {
                    list.close();
                }

                picked
            })
        })?;

    picked.inner
}

/// Draws the pill. Returns whether `shown` was cut to fit.
fn paint_pill(ui: &Ui, response: &Response, rect: Rect, shown: &str) -> bool {
    if !ui.is_rect_visible(rect) {
        return false;
    }

    let tokens = tokens::current(ui.ctx());
    let fill = Pointer::of_focusable(response).fill(&tokens, tokens.field);
    let ink = if ui.is_enabled() {
        tokens.text
    } else {
        tokens.dim
    };

    ui.painter().rect_filled(rect, PILL_RADIUS, fill);
    let text = Rect::from_min_max(
        pos2(rect.left() + PADDING_X, rect.top()),
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

    let chevron = Rect::from_center_size(
        pos2(rect.right() - PADDING_X - CHEVRON / 2.0, rect.center().y),
        vec2(CHEVRON, CHEVRON),
    );

    icon_image(ui, icons::GO_DOWN, tokens.secondary).paint_at(ui, chevron);
    focus_ring(ui, response, PILL_RADIUS);
    was_cut
}

/// An option's fill: the chosen one is tinted at rest, any one shows the
/// hover and press under the pointer.
pub(super) const fn option_fill(
    tokens: &Tokens,
    chosen: bool,
    pointer: Pointer,
) -> Option<Color32> {
    match (pointer, chosen) {
        (Pointer::Hovered, _) => Some(tokens.hover),
        (Pointer::Pressed, _) => Some(tokens.pressed),
        (Pointer::Rest, true) => Some(tokens.accent_soft),
        (Pointer::Rest, false) => None,
    }
}

/// Too wide text is cut, shown whole on hover and to screen readers.
fn option_row(ui: &mut Ui, text: &str, chosen: bool) -> Response {
    let tokens = tokens::current(ui.ctx());
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, OPTION_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::ListBoxOption, true, text));
    super::mark_selected(ui, &response, chosen);

    if let Some(fill) = option_fill(&tokens, chosen, Pointer::of(&response)) {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS), fill);
    }

    let weight = if chosen {
        Weight::SemiBold
    } else {
        Weight::Regular
    };

    let room = ITEM_INSET.mul_add(-2.0, rect.width()).max(0.0);
    let galley = cut(ui, rich(ui, text, weight, BODY, tokens.text), room);
    let was_cut = galley.elided;
    let position = pos2(
        rect.left() + ITEM_INSET,
        rect.center().y - galley.size().y / 2.0,
    );

    ui.painter().galley(position, galley, tokens.text);
    focus_ring(ui, &response, RADIUS);

    if was_cut {
        response.on_hover_text(text)
    } else {
        response
    }
}
