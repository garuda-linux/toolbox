//! The one dialog every modal question shares, after Rerun's
//! `ModalWrapper`: named for screen readers, a title bar with Close,
//! Escape to leave, and never larger than the window.

use egui::{
    Align, Color32, Context, CornerRadius, Frame, Id, LayerId, Layout, Margin, Modal, Ui, vec2,
};

use super::{ButtonKind, icon_button, pill_button, pill_width, rich};
use crate::fonts::{BODY, SECTION, Weight};
use crate::{frost, icons, tokens};

mod focus;
mod question;

pub use question::{Asked, Question, question};

const WINDOW_MARGIN: f32 = 20.0;
/// The scrim's darkness over the window, as egui's own modal has it.
const SCRIM: u8 = 100;

/// How wide a dialog is, by what it holds. The widths are in
/// [`tokens::Sizes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DialogWidth {
    #[default]
    Small,
    Medium,
    Wide,
}

impl DialogWidth {
    const fn points(self, sizes: &tokens::Sizes) -> f32 {
        match self {
            Self::Small => sizes.dialog_small,
            Self::Medium => sizes.dialog_medium,
            Self::Wide => sizes.dialog_wide,
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
}

/// What [`ModalWrapper::show`] returns: the content's result, and whether
/// the user closed the dialog with Close, Escape or a click beside it.
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

#[derive(Clone, Copy)]
struct Answers<'a> {
    cancel: &'a str,
    confirm: &'a str,
    kind: ButtonKind,
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
        }
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
        let area = Modal::default_area(self.id)
            .accessible_name(self.name)
            .fade_in(self.fade_in);

        let sizes = tokens::current(ctx).sizes;

        // Card padding on all four sides, not egui's menu-sized popup margin,
        // so Close and the footer buttons keep the text's room from the edge.
        let frame =
            Frame::popup(&ctx.global_style()).inner_margin(Margin::same(sizes.card_padding.left));

        backdrop(ctx, area.layer());
        focus::shown(ctx, self.id);

        let mut closed = false;
        let modal = Modal::new(self.id.with("modal"))
            .area(area)
            .frame(frame)
            .backdrop_color(Color32::TRANSPARENT)
            .show(ctx, |ui| {
                let room = (ui.ctx().content_rect().size()
                    - vec2(WINDOW_MARGIN, WINDOW_MARGIN) * 2.0
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

        ModalOutcome {
            closed: closed || modal.should_close(),
            inner: modal.inner,
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
    painter.rect_filled(screen, 0.0, Color32::from_black_alpha(SCRIM));
}

/// `cancel` and the `confirm` button of `kind` at the dialog's right, the
/// confirming one last.
///
/// Cancel comes first for the keyboard too, so Tab reaches the safe
/// answer before the other. A destructive question
/// passes [`ButtonKind::Danger`].
pub fn confirm_buttons(
    ui: &mut Ui,
    cancel: &str,
    confirm: &str,
    kind: ButtonKind,
) -> Option<Confirmation> {
    confirm_buttons_when(ui, cancel, confirm, kind, true)
}

/// [`confirm_buttons`] whose `confirm` is greyed out unless `ready`, for a
/// form whose fields are not complete yet. Cancel always works.
pub fn confirm_buttons_when(
    ui: &mut Ui,
    cancel: &str,
    confirm: &str,
    kind: ButtonKind,
    ready: bool,
) -> Option<Confirmation> {
    let answers = Answers {
        cancel,
        confirm,
        kind,
    };

    buttons(ui, answers, ready, None)
}

/// [`confirm_buttons_when`] that says `why` beside the buttons while
/// `confirm` waits, so a greyed out button never leaves the user guessing.
pub fn confirm_buttons_why(
    ui: &mut Ui,
    cancel: &str,
    confirm: &str,
    kind: ButtonKind,
    why: Option<&str>,
) -> Option<Confirmation> {
    let answers = Answers {
        cancel,
        confirm,
        kind,
    };

    buttons(ui, answers, why.is_none(), why)
}

fn buttons(
    ui: &mut Ui,
    answers: Answers<'_>,
    ready: bool,
    why: Option<&str>,
) -> Option<Confirmation> {
    let Answers {
        cancel,
        confirm,
        kind,
    } = answers;

    let tokens = tokens::current(ui.ctx());
    let sizes = tokens.sizes;
    ui.add_space(sizes.dialog_gap);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = sizes.control_gap;
        let buttons = pill_width(ui, cancel) + sizes.control_gap + pill_width(ui, confirm);
        let room = (ui.available_width() - buttons - sizes.control_gap).max(0.0);
        if let Some(why) = why {
            // As tall as the buttons, so a one-line hint centres on them. The
            // row's own spacing follows it, no added gap.
            let text = rich(ui, why, Weight::Regular, BODY, tokens.secondary);
            let line = Layout::left_to_right(Align::Center);
            ui.allocate_ui_with_layout(vec2(room, crate::theme::PILL_HEIGHT), line, |ui| {
                ui.set_width(room);
                ui.add(egui::Label::new(text).wrap());
            });
        } else {
            ui.add_space(room + sizes.control_gap);
        }

        if pill_button(ui, cancel, ButtonKind::Soft).clicked() {
            return Some(Confirmation::Cancelled);
        }

        ui.add_enabled_ui(ready, |ui| pill_button(ui, confirm, kind))
            .inner
            .clicked()
            .then_some(Confirmation::Confirmed)
    })
    .inner
}

fn title_bar(ui: &mut Ui, title: &str, close: &str) -> bool {
    let tokens = tokens::current(ui.ctx());
    let clicked = ui
        .horizontal(|ui| {
            let heading = ui.label(rich(ui, title, Weight::Bold, SECTION, tokens.text));
            super::mark_heading(ui, &heading, title, 2);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                icon_button(ui, icons::WINDOW_CLOSE, close).clicked()
            })
            .inner
        })
        .inner;

    ui.add_space(tokens.sizes.card_gap);
    clicked
}
