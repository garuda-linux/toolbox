use egui::{UserAttentionType, ViewportCommand};
use toolbox_core::domains::AurPackages;

use super::App;
use crate::command::{DiagnosticsCommand, UiCommand};
use crate::instance::{Arrival, PageAction};

impl App {
    /// Shows the window, opens the page and starts its action. Without a
    /// token used on Wayland the compositor decides, so the window also
    /// asks for focus (X11 honours it) and for attention.
    pub(super) fn arrive(&mut self, ctx: &egui::Context, arrival: &Arrival) {
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

        if let Some(PageAction::Collect(selection)) = arrival.action {
            self.run_diagnostics(DiagnosticsCommand::Collect(selection));
        }
    }

    /// Answers the commands that asked about the queue since the last
    /// pass. A caller that gave up meanwhile misses its answer.
    pub(super) fn answer_queue(&mut self) {
        let Some(arrivals) = self.arrivals.clone() else {
            return;
        };

        for ask in arrivals.questions() {
            let snapshot = self.current_snapshot();
            let answer = self
                .queue
                .answer(ask.question, self.console.running(), &snapshot);

            if ask.answer.send(answer).is_err() {
                tracing::debug!("a command stopped waiting for the queue's answer");
            }
        }

        // An AUR build from a command waits for the AUR's copy to meet
        // the queue's guard. Reading it downloads at most once a day.
        if !self.queue.changes().queued::<AurPackages>().is_empty() {
            self.packages.open_aur();
        }
    }

    /// The window closes: later starts may still arrive, but nothing
    /// raises it any more.
    pub(super) fn leave(&self) {
        if let Some(arrivals) = &self.arrivals {
            arrivals.close();
        }
    }
}
