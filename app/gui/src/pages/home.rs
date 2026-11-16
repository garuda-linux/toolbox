mod dialogs;
mod hero;
pub mod logo;
mod news;
mod next;
mod stats;
mod status;
mod steps;

#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_gui_kit::widgets::{Tone, banner};
use toolbox_gui_kit::{Loadable, UiExt as _};
use toolbox_news::News;
use toolbox_system::{CallError, Facts, Host, Status};

use super::{PageContext, PageView};
use crate::command::HomeCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::programs::Programs;

/// What Home reads besides the snapshot.
#[derive(Clone, Copy, Debug)]
pub struct HomeFacts<'app> {
    /// `None` without a status reader, as in tests that need none.
    pub status: Option<&'app Loadable<Status>>,
    /// Changes with every status reading, see [`crate::reader::Watched::revision`].
    pub status_revision: u64,
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
    /// This many saved changes did not come back at start.
    QueueLost(usize),
    /// A run of the page did not start, as another plan runs.
    Busy,
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
        let overview = context
            .home
            .status
            .and_then(Loadable::get)
            .map(|status| status::overview(status, context));

        hero::show(ui, context, host, overview.as_ref());
        ui.add_space(ui.tokens().sizes.block_gap);
        notice(ui, context);

        // The status and the news bring their own gap above them, so a
        // part that draws nothing leaves no second gap.
        if live {
            steps::live(ui, context);
        } else {
            stats::show(ui, context, overview.as_ref());
            status::show(self, ui, context, host, overview.as_ref());
            news::show(ui, context);
        }

        ui.add_space(ui.tokens().sizes.block_gap);
        next::show(ui, context, host);
        dialogs::show(self, ui.ctx(), context, overview.as_ref());
    }
}

/// Saved changes that did not come back at start, or a run that did not
/// start as another one ran, until the user read it.
fn notice(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let text = match context.home.problem {
        Some(HomeProblem::QueueLost(count)) => {
            i18n::text_with(&ctx, "home-queue-lost", &[("count", Arg::from(count))])
        }
        Some(HomeProblem::Busy) => i18n::text(&ctx, "home-busy"),
        Some(HomeProblem::Restart(_) | HomeProblem::Installer(_)) | None => return,
    };

    let dismiss = i18n::text(&ctx, "home-queue-lost-dismiss");

    if banner(ui, Tone::Warning, &text, &[&dismiss]).value == Some(0) {
        context.commands.send(HomeCommand::DismissProblem);
    }

    ui.add_space(ui.tokens().sizes.block_gap);
}
