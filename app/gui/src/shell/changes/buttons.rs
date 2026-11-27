use egui::Ui;
use toolbox_gui_kit::widgets::{
    ButtonKind, pill_button_with_icon, pill_button_with_icon_at_least, pill_width_with_icon,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::{ChangesState, Hold};
use crate::command::{CommandSender, UiCommand};
use crate::i18n;

// As wide as the longer of the two toggle texts, so the footer keeps its width.
fn toggle_width(ui: &Ui) -> f32 {
    ["changes-review", "changes-hide"]
        .map(|id| pill_width_with_icon(ui, &i18n::text(ui.ctx(), id)))
        .into_iter()
        .fold(0.0, f32::max)
}

/// The width of Review (or Show) and, with `apply`, of Apply and the gap
/// between them, so the footer's group can sit at its right edge.
pub(super) fn group_width(ui: &Ui, apply: bool) -> f32 {
    let apply_width = if apply {
        let text = i18n::text(ui.ctx(), "changes-apply");
        ui.spacing().item_spacing.x + pill_width_with_icon(ui, &text)
    } else {
        0.0
    };

    toggle_width(ui) + apply_width
}

pub(super) fn show_console_button(ui: &mut Ui, commands: &CommandSender) -> egui::Response {
    let width = toggle_width(ui);
    let show = pill_button_with_icon_at_least(
        ui,
        &i18n::text(ui.ctx(), "changes-show-console"),
        ButtonKind::Soft,
        width,
        icons::UTILITIES_TERMINAL,
    );

    if show.clicked() {
        commands.send(UiCommand::ToggleConsole);
    }

    show
}

pub(super) fn review_button(ui: &mut Ui, state: &mut ChangesState) -> egui::Response {
    let toggle = if state.open {
        "changes-hide"
    } else {
        "changes-review"
    };

    let width = toggle_width(ui);
    let review = pill_button_with_icon_at_least(
        ui,
        &i18n::text(ui.ctx(), toggle),
        ButtonKind::Soft,
        width,
        icons::DOCUMENT_PREVIEW,
    );

    if review.clicked() {
        state.open = !state.open;
        state.dismissed |= !state.open;
        state.keyed = !review.clicked_by(egui::PointerButton::Primary);
    }

    if state.open && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.open = false;
        state.dismissed = true;
        review.request_focus();
    }

    let open = state.open;
    ui.ctx()
        .accesskit_node_builder(review.id, |node| node.set_expanded(open));
    review
}

/// Applies the queue, or says to show it first when it `removes`
/// something, so a removal queued days ago is never applied unseen.
pub(super) fn apply_button(
    ui: &mut Ui,
    held: Option<&Hold>,
    commands: &CommandSender,
    removes: bool,
) -> (egui::Response, bool) {
    let text = i18n::text(ui.ctx(), "changes-apply");
    let apply = apply_pill(ui, &text, held);
    if apply.clicked() && !removes {
        commands.send(UiCommand::Apply);
    }

    let show = apply.clicked() && removes;
    (apply, show)
}

/// A disabled button tells no screen reader why, so the reason goes into
/// its description as well as its tooltip.
pub(super) fn apply_pill(ui: &mut Ui, text: &str, held: Option<&Hold>) -> egui::Response {
    let apply = ui
        .enabled_ui(held.is_none(), |ui| {
            pill_button_with_icon(ui, text, ButtonKind::Primary, icons::DIALOG_OK_APPLY)
        })
        .inner;

    let Some(pending) = held else {
        return apply;
    };

    let why = pending.text.clone();
    ui.ctx()
        .accesskit_node_builder(apply.id, |node| node.set_description(why.as_str()));
    apply.on_disabled_hover_text(why)
}
