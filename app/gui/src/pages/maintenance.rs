//! The Maintenance page: package care and tools, the settings reset and
//! the repairs, one tab each.
//!
//! What runs goes to the Console, so every button that starts a plan
//! waits while one runs. The tools open in their own windows.

mod common;
pub mod palette;
mod repairs;
mod reset;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::maintenance::page::LaunchCause;
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::widgets::{ActionRow, ButtonKind, Tone, pill_button, section_intro};
use toolbox_system::mirrors::RankProblem;

use super::header::{TabText, TabbedIds, tabbed_header};
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::maintenance::{Maintenance, MaintenanceProblem};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tab {
    #[default]
    Common,
    Reset,
    Repairs,
}

#[derive(Debug, Default)]
pub struct MaintenancePage {
    tab: Tab,
    confirming: bool,
    asking: Option<String>,
}

impl Tab {
    pub const fn text_id(self) -> &'static str {
        match self {
            Self::Common => "maintenance-tab-common",
            Self::Reset => "maintenance-tab-reset",
            Self::Repairs => "maintenance-tab-repairs",
        }
    }
}

impl TabText for Tab {
    fn text_id(self) -> &'static str {
        Self::text_id(self)
    }
}

impl MaintenancePage {
    /// Opens `tab`, for the palette and the self-test.
    pub const fn show_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }
}

impl PageView for MaintenancePage {
    fn place(&self) -> u64 {
        super::place_of(&self.tab)
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let ids = TabbedIds {
            title: "page-maintenance",
            subtitle: "page-maintenance-subtitle",
            tabs: "maintenance-tabs",
            refresh: "maintenance-refresh",
        };

        let tabs = [Tab::Common, Tab::Reset, Tab::Repairs];
        let done = tabbed_header(ui, ids, &tabs, self.tab);
        if let Some(picked) = done.picked {
            self.tab = picked;
        }

        if done.refresh {
            context.commands.send(MaintenanceCommand::Refresh);
        }

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("maintenance ui").entered();
        if self.tab != Tab::Reset {
            plan_running(ui, context);
        }

        problem(ui, context);
        match self.tab {
            Tab::Common => common::show(ui, context, &mut self.asking),
            Tab::Reset => reset::show(ui, context, &mut self.confirming),
            Tab::Repairs => repairs::show(ui, context, &mut self.asking),
        }
    }
}

/// Why the last tool did not start or the mirrors were not ranked, until
/// dismissed.
fn problem(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(problem) = context.service::<Maintenance>().problem() else {
        return;
    };

    let ctx = ui.ctx().clone();
    let text = match problem {
        MaintenanceProblem::Launch {
            tool,
            cause,
            reason,
        } => {
            let id = match cause {
                LaunchCause::Missing => "maintenance-problem-launch-missing",
                LaunchCause::Denied => "maintenance-problem-launch-denied",
                LaunchCause::Other => "maintenance-problem-launch",
            };

            let name = i18n::text(&ctx, &format!("maintenance-tool-{tool}"));
            i18n::text_with(&ctx, id, &[("tool", &name), ("reason", reason)])
        }
        MaintenanceProblem::Mirrors(problem) => {
            let id = match problem {
                RankProblem::TimedOut => "maintenance-problem-mirrors-timeout",
                RankProblem::NotStarted(_) => "maintenance-problem-mirrors-start",
                RankProblem::Failed(_) => "maintenance-problem-mirrors-exit",
                RankProblem::Unreadable(_) | RankProblem::Invalid(_) => {
                    "maintenance-problem-mirrors-invalid"
                }
            };

            i18n::text_with(&ctx, id, &[("reason", problem.reason())])
        }
    };

    if refusal_banner(ui, Tone::Error, &text, "maintenance-problem-dismiss") {
        context.commands.send(MaintenanceCommand::DismissProblem);
    }
}

fn heading(ui: &mut Ui, icon: Icon, id: &str) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, id);
    let detail = i18n::text(&ctx, &format!("{id}-detail"));
    section_intro(ui, icon, &title, &detail);
}

fn row(ui: &mut Ui, icon: Icon, id: &str, detail: &str, actions: impl FnOnce(&mut Ui)) {
    let title = i18n::text(ui.ctx(), id);
    ActionRow::new(icon, Tone::Neutral, &title)
        .detail(detail)
        .show(ui, actions);
}

/// Waits while a plan runs, so it is disabled then.
fn plan_button(ui: &mut Ui, context: &PageContext<'_>, id: &str, kind: ButtonKind) -> bool {
    let text = i18n::text(ui.ctx(), id);
    ui.add_enabled_ui(!context.busy, |ui| pill_button(ui, &text, kind))
        .inner
        .clicked()
}
