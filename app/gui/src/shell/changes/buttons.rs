use egui::Ui;
use toolbox_gui_kit::widgets::{ButtonKind, pill_button, pill_button_at_least, pill_width};

use super::ChangesState;
use crate::command::{CommandSender, UiCommand};
use crate::i18n;

// As wide as the longer of the two toggle texts, so the pill keeps its width.
fn toggle_width(ui: &Ui) -> f32 {
    ["changes-review", "changes-hide"]
        .map(|id| pill_width(ui, &i18n::text(ui.ctx(), id)))
        .into_iter()
        .fold(0.0, f32::max)
}

pub(super) fn show_console_button(ui: &mut Ui, commands: &CommandSender) {
    let width = toggle_width(ui);
    let show = pill_button_at_least(
        ui,
        &i18n::text(ui.ctx(), "changes-show-console"),
        ButtonKind::Soft,
        width,
    );

    if show.clicked() {
        commands.send(UiCommand::ToggleConsole);
    }
}

pub(super) fn review_button(ui: &mut Ui, state: &mut ChangesState) -> egui::Response {
    let toggle = if state.open {
        "changes-hide"
    } else {
        "changes-review"
    };

    let width = toggle_width(ui);
    let review = pill_button_at_least(ui, &i18n::text(ui.ctx(), toggle), ButtonKind::Soft, width);
    if review.clicked() {
        state.open = !state.open;
        state.keyed = !review.clicked_by(egui::PointerButton::Primary);
    }

    if state.open && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.open = false;
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
    held: Option<&'static str>,
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
pub(super) fn apply_pill(ui: &mut Ui, text: &str, held: Option<&'static str>) -> egui::Response {
    let apply = ui
        .add_enabled_ui(held.is_none(), |ui| {
            pill_button(ui, text, ButtonKind::Primary)
        })
        .inner;

    let Some(id) = held else {
        return apply;
    };

    let why = i18n::text(ui.ctx(), id).into_owned();
    ui.ctx()
        .accesskit_node_builder(apply.id, |node| node.set_description(why.as_str()));
    apply.on_disabled_hover_text(why)
}
