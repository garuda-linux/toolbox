mod dialogs;
mod hero;
mod logo;
mod news;
mod next;
mod status;
mod steps;
mod view;

#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_gui_kit::{Loadable, UiExt as _};
use toolbox_news::News;
use toolbox_system::{CallError, Facts, Host, Status};

use super::{PageContext, PageView};
use crate::page::PageId;
use crate::programs::Programs;

/// What Home reads besides the snapshot.
#[derive(Clone, Copy, Debug)]
pub struct HomeFacts<'app> {
    /// `None` without a status reader, as in tests that need none.
    pub status: Option<&'app Loadable<Status>>,
    pub facts: Option<&'app Loadable<Facts>>,
    pub news: Option<&'app Loadable<News>>,
    /// `None` until the programs were looked for.
    pub programs: Option<&'app Programs>,
    pub programs_failed: bool,
    pub problem: Option<&'app HomeProblem>,
    pub recent: &'app [PageId],
    pub setup_finished: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeProblem {
    Restart(CallError),
    /// The system's reason, in English.
    Installer(String),
}

#[derive(Debug, Default)]
pub struct HomePage {
    details: bool,
    updates_open: bool,
    asking_to_restart: bool,
}

impl PageView for HomePage {
    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("home ui").entered();
        let host = context.snapshot.part::<Host>();
        let live = host.is_some_and(|host| host.live_iso);
        hero::show(ui, context, host);
        ui.add_space(ui.tokens().sizes.block_gap);

        if live {
            steps::live(ui, context);
        } else {
            status::show(self, ui, context, host);
            news::show(ui, context);
        }

        ui.add_space(ui.tokens().sizes.block_gap);
        next::show(ui, context, host);
        dialogs::show(self, ui.ctx(), context);
    }
}
