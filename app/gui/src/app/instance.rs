use egui::{UserAttentionType, ViewportCommand};
use toolbox_core::domains::AurPackages;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_instance::bus::queue::Question;

use super::App;
use crate::command::{DiagnosticsCommand, UiCommand};
use crate::i18n;
use crate::instance::Arrival;

impl App {
    /// Shows the window, opens the page and starts its action. On Wayland
    /// the compositor decides, so the window asks for focus (X11 honours
    /// it) and for attention.
    pub(super) fn arrive(&mut self, ctx: &egui::Context, arrival: &Arrival) {
        let minimized = ctx.input(|input| input.viewport().minimized.unwrap_or(false));
        let in_front = self.focused && !minimized;

        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        if !arrival.raised {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                UserAttentionType::Informational,
            ));
        }

        if let Some(page) = arrival.page {
            self.run_ui(ctx, UiCommand::Navigate(page));
        }

        match arrival.collection(in_front) {
            Some(selection) => self.run_diagnostics(DiagnosticsCommand::Collect(selection)),
            None if arrival.action.is_some() => {
                tracing::info!("a collection asked from outside waits for a click");
            }
            None => {}
        }
    }

    /// Answers the commands that asked about the queue since the last
    /// pass. A caller that gave up meanwhile misses its answer.
    pub(super) fn answer_queue(&mut self, ctx: &egui::Context) {
        let Some(arrivals) = self.arrivals.clone() else {
            return;
        };

        for ask in arrivals.questions() {
            let snapshot = self.current_snapshot();
            let edit = matches!(ask.question, Question::Edit(_));
            let answer = self
                .queue
                .answer(ask.question, self.console.running(), &snapshot);

            // Any process of the session can edit the queue, so a change
            // the user did not make here never lands unseen.
            if edit && answer.is_ok() {
                self.toasts.add(
                    ToastKind::Info,
                    i18n::text(ctx, "toast-queue-edited-outside"),
                );
            }

            if ask.answer.send(answer).is_err() {
                tracing::debug!("a command stopped waiting for the queue's answer");
            }
        }

        // An AUR build from a command waits for the AUR's copy to meet
        // the queue's guard. Reading it downloads at most once a day, and
        // never while the user keeps the AUR off.
        let aur_on = self.prefs.setup.settings.packages.show_aur;

        if aur_on && !self.queue.changes().queued::<AurPackages>().is_empty() {
            self.packages.open_aur();
        }
    }

    /// The window closes. The queue is written here, as the runtime stops
    /// its writer task soon after.
    pub(super) fn leave(&self) {
        if let Err(failure) = self.queue.flush() {
            tracing::warn!("{failure}");
        }
    }
}
