//! The one dialog every modal question shares, after Rerun's
//! `ModalWrapper`: named for screen readers, a title bar with Close,
//! Escape to leave, and never larger than the window.

mod answers;
mod focus;
mod look;
mod pending;
mod question;

pub use answers::{confirm_buttons, confirm_buttons_when, confirm_buttons_why};
use egui::{
    Align, Align2, AreaState, Color32, Context, CornerRadius, Id, LayerId, Layout, Modal, Sense,
    Ui, vec2,
};
pub use pending::Pending;
pub use question::{Asked, Question, question};

use super::top_light::paint_top_light;
use super::{icon_button, icon_button_width, rich};
use crate::fonts::{PAGE_TITLE, Weight};
use crate::toasts::paint_popup_shadow;
use crate::{frost, icons, tokens};

const WINDOW_MARGIN: f32 = 20.0;
/// Where a top-anchored dialog hangs, as a share of the window's height.
const TOP_SHARE: f32 = 0.15;

/// How wide a dialog is, by what it holds. The widths are in
/// [`tokens::Sizes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DialogWidth {
    #[default]
    Small,
    Medium,
    Wide,
    /// As wide as the window allows, for a picture shown large.
    Window,
}

impl DialogWidth {
    const fn points(self, sizes: &tokens::Sizes) -> f32 {
        match self {
            Self::Small => sizes.dialog_small,
            Self::Medium => sizes.dialog_medium,
            Self::Wide => sizes.dialog_wide,
            Self::Window => f32::INFINITY,
        }
    }
}

/// A modal dialog. egui's own area has no name, this one carries `name`.
#[derive(Debug)]
pub struct ModalWrapper<'a> {
    id: Id,
    name: &'a str,
    close: Option<&'a str>,
    fade_in: bool,
    width: DialogWidth,
    fill: bool,
    form: bool,
    place: Place,
}

/// Where a dialog sits in the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Centred,
    /// Hangs from a line near the top, so a list under its field can
    /// change length without moving the field.
    Top,
}

/// What [`ModalWrapper::show`] returns: the content's result, and whether
/// the user closed the dialog with Close, Escape or a click beside it
/// (not for a [`ModalWrapper::form`]).
#[derive(Debug)]
pub struct ModalOutcome<R> {
    pub inner: R,
    pub closed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirmation {
    Cancelled,
    Confirmed,
}

impl<'a> ModalWrapper<'a> {
    /// A dialog named `name` for screen readers, without a title bar.
    #[must_use = "a dialog does nothing until shown"]
    pub const fn new(id: Id, name: &'a str) -> Self {
        Self {
            id,
            name,
            close: None,
            fade_in: true,
            width: DialogWidth::Small,
            fill: false,
            form: false,
            place: Place::Centred,
        }
    }

    /// Hangs from a fixed line near the window's top rather than sitting
    /// in its middle, for a dialog whose list grows and shrinks as the user
    /// types (the palette): its field stays where the eye is.
    #[must_use = "a dialog does nothing until shown"]
    pub const fn top_anchored(mut self) -> Self {
        self.place = Place::Top;
        self
    }

    /// Stays open on a click beside it, as libadwaita and macOS keep a
    /// form, so a stray click drops nothing typed. Close and Escape still
    /// leave.
    #[must_use = "a dialog does nothing until shown"]
    pub const fn form(mut self) -> Self {
        self.form = true;
        self
    }

    #[must_use = "a dialog does nothing until shown"]
    pub const fn title_bar(mut self, close: &'a str) -> Self {
        self.close = Some(close);
        self
    }

    /// Appears at once, for a surface a shortcut opens (the palette): a
    /// key press should answer without delay.
    #[must_use = "a dialog does nothing until shown"]
    pub const fn instant(mut self) -> Self {
        self.fade_in = false;
        self
    }

    /// As tall as the window allows, for a text that should not open in
    /// a dialog that jumps as it loads or leaves most of the window unused.
    #[must_use = "a dialog does nothing until shown"]
    pub const fn full_height(mut self) -> Self {
        self.fill = true;
        self
    }

    #[must_use = "a dialog does nothing until shown"]
    pub const fn width(mut self, width: DialogWidth) -> Self {
        self.width = width;
        self
    }

    pub fn show<R>(self, ctx: &Context, content: impl FnOnce(&mut Ui) -> R) -> ModalOutcome<R> {
        let hang = self.hang(ctx);
        let mut area = Modal::default_area(self.id)
            .accessible_name(self.name)
            .fade_in(self.fade_in);

        if matches!(self.place, Place::Top) {
            area = area.anchor(Align2::CENTER_TOP, vec2(0.0, hang));
        }

        let current = tokens::current(ctx);
        let sizes = current.sizes;

        // Card padding, not the popup margin, keeps the text's room from the
        // edge. The frame is Kirigami's `Dialog`, the shadow Darkly's popup one.
        let style = ctx.global_style();
        let frame = look::frame(&style, &current, sizes.card_padding.left);

        let layer = area.layer();
        backdrop(ctx, layer);
        let shadow = ctx.layer_painter(layer).add(egui::Shape::Noop);
        let last_size = AreaState::load(ctx, self.id).and_then(|state| state.size);
        focus::shown(ctx, self.id);

        let mut closed = false;
        let modal = Modal::new(self.id.with("modal"))
            .area(area)
            .frame(frame)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let room = (ui.ctx().content_rect().size()
                    - vec2(WINDOW_MARGIN, WINDOW_MARGIN) * 2.0
                    - vec2(0.0, hang - WINDOW_MARGIN)
                    - frame.total_margin().sum())
                .max(vec2(0.0, 0.0));

                ui.set_width(self.width.points(&sizes).min(room.x));

                ui.set_max_height(room.y);
                if self.fill {
                    ui.set_min_height(room.y);
                }

                focus::holder(ui, self.id, self.name);

                if let Some(close) = self.close {
                    closed |= title_bar(ui, self.name, close);
                }

                content(ui)
            });

        let painter = ctx.layer_painter(layer);
        paint_popup_shadow(
            &painter,
            shadow,
            modal.response.rect,
            frame.corner_radius,
            &current,
        );
        paint_top_light(&painter, modal.response.rect, frame.corner_radius);

        // egui places an area by the size it had last frame, so a dialog whose
        // content grew or shrank would show one frame off the centre.
        if last_size.is_some_and(|size| size != modal.response.rect.size()) {
            ctx.request_discard("the dialog changed its size");
        }

        let beside = modal.backdrop_response.clicked();
        let asked = modal.should_close();
        ModalOutcome {
            closed: closed || (asked && !(self.form && beside)),
            inner: modal.inner,
        }
    }

    /// How far the dialog's top may come to the window's top: a share of
    /// the window for a [`Self::top_anchored`] one, the margin otherwise.
    fn hang(&self, ctx: &Context) -> f32 {
        if matches!(self.place, Place::Top) {
            (ctx.content_rect().height() * TOP_SHARE).max(WINDOW_MARGIN)
        } else {
            WINDOW_MARGIN
        }
    }
}

/// What lies under a dialog: the window blurred, and the scrim over it.
/// Painted on the dialog's own layer before the dialog, so it stays under
/// it.
fn backdrop(ctx: &Context, layer: LayerId) {
    let painter = ctx.layer_painter(layer);
    let screen = ctx.content_rect();
    frost::backdrop(&painter, screen, CornerRadius::ZERO);
    painter.rect_filled(screen, 0.0, tokens::current(ctx).scrim);
}

fn title_bar(ui: &mut Ui, title: &str, close: &str) -> bool {
    let tokens = tokens::current(ui.ctx());

    // As tall as the Close tool button, so Close and the title sit on the
    // dialog's margin. A horizontal row reads its height before it runs.
    let clicked = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size.y = icon_button_width();
            ui.horizontal(|ui| {
                // Kirigami's dialog title is a level 1 heading in the DemiBold
                // weight, which the desktop's own weight stands for here.
                let heading = ui.label(rich(ui, title, Weight::SemiBold, PAGE_TITLE, tokens.text));
                super::mark_heading(ui, &heading, title, 1);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    icon_button(ui, icons::WINDOW_CLOSE, close).clicked()
                })
                .inner
            })
            .inner
        })
        .inner;

    // Kirigami's dialog header sets the title off from the content with a
    // rule in the outline colour.
    ui.add_space(tokens.sizes.tight_gap);
    let (rule, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rule, 0.0, tokens.outline);
    ui.add_space(tokens.sizes.card_gap);
    clicked
}
