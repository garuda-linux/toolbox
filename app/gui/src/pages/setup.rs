//! Every pick goes into the queue the other pages use, so Review shows it
//! and one apply covers it. The step is the page's own state.

mod nav;
mod stages;
#[cfg(test)]
mod tests;

use egui::{Key, Modifiers, Ui};
use toolbox_core::features::console::ending::Ending;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{empty_state, page_header};

use super::toggles::{Steps, TogglePage};
use super::{PageContext, PageView, QueueShown};
use crate::catalogs::CatalogId;
use crate::i18n;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Stage {
    #[default]
    Welcome,
    Step(String),
    Summary,
    Success,
}

#[derive(Debug)]
pub struct SetupPage {
    toggles: TogglePage,
    stage: Stage,
    /// How many plans had ended when Review and apply was clicked, so the
    /// next one that ends well with an empty queue is the assistant's.
    review_at: Option<u64>,
    /// How the assistant's own apply read. `None` when Finish skipped it.
    ending: Option<Ending>,
    /// The picks the queue last held while a review was open, with the
    /// queue revision they were read at. The apply empties the queue, so
    /// the success page names them from here.
    applied: Option<(u64, stages::Picks)>,
    asking_to_restart: bool,
    told_home: bool,
}

impl Default for SetupPage {
    fn default() -> Self {
        Self {
            toggles: TogglePage::new(CatalogId::Setup, "page-setup", "page-setup-subtitle"),
            stage: Stage::default(),
            review_at: None,
            ending: None,
            applied: None,
            asking_to_restart: false,
            told_home: false,
        }
    }
}

impl PageView for SetupPage {
    fn place(&self) -> u64 {
        super::place_of(&self.stage)
    }

    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        let ctx = ui.ctx().clone();
        page_header(
            ui,
            &i18n::text(&ctx, "page-setup"),
            &i18n::text(&ctx, "page-setup-subtitle"),
            0.0,
            |_ui| {},
        );

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("setup ui").entered();
        let ctx = ui.ctx().clone();
        let text = |id: &str| i18n::text(&ctx, id).into_owned();
        if context.catalogs.setup.catalog.is_err() {
            empty_state(ui, icons::HELP_ABOUT, &text("setup-unavailable"), "");
            return;
        }

        let steps = self.toggles.steps(ui, context);
        if steps.is_empty() && context.probed {
            empty_state(ui, icons::HELP_ABOUT, &text("setup-none"), "");
            return;
        }

        self.forget_gone_step(&steps);
        self.remember_picks(&ctx, context);
        self.finish_when_applied(context);
        match self.stage.clone() {
            Stage::Welcome => stages::welcome(ui),
            Stage::Step(id) => {
                let step = steps
                    .pairs()
                    .enumerate()
                    .find(|(_, (each, _))| *each == id.as_str());

                if let Some((index, (_, title))) = step {
                    stages::step_heading(ui, (index + 1, steps.len()), title);
                }

                self.toggles.show_section(ui, context, &id);
            }
            Stage::Summary => stages::summary(ui, context),
            Stage::Success => stages::success(ui, context, self),
        }
    }

    /// The Summary applies through its own Review and apply, so the queue
    /// shows there only for that review. The success page follows an
    /// apply, a second Apply there would only confuse.
    fn queue_shown(&self) -> QueueShown {
        match self.stage {
            Stage::Summary => QueueShown::WhileReviewing,
            Stage::Success | Stage::Welcome | Stage::Step(_) => QueueShown::Never,
        }
    }

    fn footer(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        if self.stage == Stage::Success || context.catalogs.setup.catalog.is_err() {
            return false;
        }

        let steps = self.toggles.steps(ui, context);
        if steps.is_empty() {
            return false;
        }

        let stages = nav::stages(ui.ctx(), &steps);
        let moved = nav::bar(ui, context, self, &stages);
        let keys = ui.input_mut(|input| {
            (
                input.consume_key(Modifiers::ALT, Key::ArrowRight),
                input.consume_key(Modifiers::ALT, Key::ArrowLeft),
            )
        });

        if !moved {
            nav::keys(context, self, &stages, keys);
        }

        true
    }
}

impl SetupPage {
    /// A step whose section no longer lists, as a new reading of the
    /// system hid all its entries, has no place in the list, so the
    /// assistant starts over where the list does.
    fn forget_gone_step(&mut self, steps: &Steps) {
        if let Stage::Step(id) = &self.stage
            && !steps.is_empty()
            && !steps.pairs().any(|(each, _)| each == id.as_str())
        {
            self.stage = Stage::Welcome;
        }
    }

    fn remember_picks(&mut self, ctx: &egui::Context, context: &PageContext<'_>) {
        let read = self.applied.as_ref().map(|(revision, _)| *revision);
        if self.review_at.is_none() || context.changes.is_empty() || read == Some(context.revision)
        {
            return;
        }

        self.applied = stages::Picks::of(ctx, context).map(|picks| (context.revision, picks));
    }

    fn finish_when_applied(&mut self, context: &PageContext<'_>) {
        let Some(at) = self.review_at else {
            return;
        };

        let plans = context.plans;
        if plans.finished > at && plans.last_succeeded && context.changes.is_empty() {
            self.stage = Stage::Success;
            self.review_at = None;
            self.ending = Some(plans.last_ending);
        }
    }
}
