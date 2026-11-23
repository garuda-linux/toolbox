//! The answers of a question: the confirm button and Cancel at a dialog's
//! foot, in KDE's order.

use egui::{Align, Layout, Ui, vec2};

use super::super::buttons::{pill_button_with_icon, pill_width_with_icon};
use super::super::{ButtonKind, pill_button, pill_width, push_height, rich};
use super::Confirmation;
use crate::fonts::{BODY, Weight};
use crate::icons::{self, Icon};
use crate::{UiExt as _, tokens};

#[derive(Clone, Copy)]
struct Answers<'a> {
    cancel: &'a str,
    confirm: &'a str,
    kind: ButtonKind,
    icon: Option<Icon>,
}

/// [`confirm_buttons`] whose affirmative answer wears `icon` instead of the
/// standard icon of its kind, for an action that has an icon of its own.
pub(super) fn confirm_buttons_iconed(
    ui: &mut Ui,
    cancel: &str,
    confirm: &str,
    kind: ButtonKind,
    icon: Option<Icon>,
) -> Option<Confirmation> {
    let answers = Answers {
        cancel,
        confirm,
        kind,
        icon,
    };

    buttons(ui, answers, true, None)
}

/// The `confirm` button of `kind` and `cancel` at the dialog's right.
///
/// KDE's order puts the affirmative answer left of Cancel, which stays at
/// the far right, and Tab reaches the answer first. A destructive question
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
        icon: None,
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
        icon: None,
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
        icon,
    } = answers;

    let tokens = tokens::current(ui.ctx());
    let sizes = tokens.sizes;
    let confirm_icon = icon.or_else(|| confirm_icon(kind));
    ui.add_space(sizes.dialog_gap);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = sizes.control_gap;
        let confirm_width = match confirm_icon {
            Some(_) => pill_width_with_icon(ui, confirm),
            None => pill_width(ui, confirm),
        };

        let buttons = confirm_width + sizes.control_gap + pill_width_with_icon(ui, cancel);
        let room = (ui.available_width() - buttons - sizes.control_gap).max(0.0);
        if let Some(why) = why {
            // As tall as the buttons, so a one-line hint centres on them. The
            // row's own spacing follows it, no added gap.
            let text = rich(ui, why, Weight::Regular, BODY, tokens.secondary);
            let line = Layout::left_to_right(Align::Center);
            ui.allocate_ui_with_layout(vec2(room, push_height(ui)), line, |ui| {
                ui.set_width(room);
                ui.add(egui::Label::new(text).wrap());
            });
        } else {
            ui.add_space(room + sizes.control_gap);
        }

        let confirmed = ui
            .enabled_ui(ready, |ui| match confirm_icon {
                Some(shown) => pill_button_with_icon(ui, confirm, kind, shown),
                None => pill_button(ui, confirm, kind),
            })
            .inner
            .clicked();

        if confirmed {
            return Some(Confirmation::Confirmed);
        }

        pill_button_with_icon(ui, cancel, ButtonKind::Soft, icons::DIALOG_CANCEL)
            .clicked()
            .then_some(Confirmation::Cancelled)
    })
    .inner
}

/// The standard icon of the affirmative answer, as KDE's `KStandardGuiItem`
/// gives it. A Danger answer is KDE's delete item, a Soft one is no standard
/// answer.
const fn confirm_icon(kind: ButtonKind) -> Option<Icon> {
    match kind {
        ButtonKind::Primary => Some(icons::DIALOG_OK),
        ButtonKind::Danger => Some(icons::EDIT_DELETE),
        ButtonKind::Soft | ButtonKind::Ghost => None,
    }
}
