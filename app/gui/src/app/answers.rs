//! What App does for the page services, which know nothing of it.

use toolbox_gui_kit::toasts::ToastKind;

use super::App;
use crate::apply::PlanId;
use crate::command::AppCommand;
use crate::page_service::{Answer, Handled, Owner, PageService};
use crate::{i18n, page_ui};

impl App {
    pub(super) fn answer(&mut self, ctx: &egui::Context, answer: Answer) {
        match answer {
            Answer::InstallMirrorlist(plan) => {
                if !self.run_owned(plan, Owner::Mirrorlist) {
                    self.services.maintenance.mirrorlist_refused();
                }
            }
            Answer::Recombine => {
                self.combine();
                self.queue.meet(&self.seen);
            }
            Answer::EditBoot(edit) => self.edit_boot(ctx, edit),
            Answer::Configs(act) => self.act_configs(Some(act)),
            Answer::CloseConsole => self.state.console_open = false,
            Answer::Copy { text, toast } => {
                if self.prefs.setup.settings.diagnostics.copy_to_clipboard {
                    ctx.copy_text(text);
                    self.toasts.add(ToastKind::Info, i18n::text(ctx, toast));
                }
            }
            Answer::OfferBootRepair(offered) => page_ui::offer_boot_repair(ctx, offered),
            Answer::ShowPage(page) => self.navigate(ctx, page),
        }
    }

    /// Offers `command` to every page service before App. Each click's
    /// edit is taken at once, so the next command of this frame sees it.
    /// Returns whether a service took the command, so App leaves it, and
    /// what the services ask for next.
    pub(super) fn hand_to_services(&mut self, command: &AppCommand) -> (bool, Vec<AppCommand>) {
        let snapshot = self.current_snapshot();
        let mut taken = false;
        let mut asked = Vec::new();
        for service in self.services.all_mut() {
            let handled = service.receive(command, self.queue.changes(), &snapshot);
            if let Some(edit) = handled.edit {
                self.queue.take_click(edit);
            }

            taken |= handled.taken;
            asked.extend(handled.answers);
        }

        (taken, asked)
    }

    /// Takes what a service made of a click App ran for it.
    pub(super) fn take_handled(&mut self, ctx: &egui::Context, handled: Handled) {
        if let Some(edit) = handled.edit {
            self.queue.take_click(edit);
        }

        for command in handled.answers {
            self.run_app(ctx, command);
        }
    }

    /// Every service's results since the last frame, and what it asks for.
    pub(super) fn take_page_services(&mut self, ctx: &egui::Context) {
        let page = self.state.page;
        let snapshot = self.current_snapshot();
        let asked: Vec<_> = self
            .services
            .all_mut()
            .into_iter()
            .flat_map(|service| service.take(page, &snapshot))
            .collect();

        for command in asked {
            self.run_app(ctx, command);
        }
    }

    /// Only the sender of `plan` hears of its start, so a sender whose
    /// own plan is refused never takes another plan's run for its own.
    pub(super) fn services_plan_started(&mut self, ctx: &egui::Context, plan: PlanId) {
        let Some(owner) = self.services.waiting.answered(plan) else {
            return;
        };

        match owner {
            Owner::Queue => self.queue.plan_started(),
            Owner::Mirrorlist => self.services.maintenance.mirrorlist_started(),
            Owner::Maintenance | Owner::Users | Owner::Configs | Owner::BootRepair => {}
        }

        let asked = self
            .services
            .owned_mut(owner)
            .map(PageService::plan_started)
            .unwrap_or_default();

        for command in asked {
            self.run_app(ctx, command);
        }
    }

    pub(super) fn services_plan_refused(&mut self, ctx: &egui::Context, plan: PlanId) {
        let Some(owner) = self.services.waiting.answered(plan) else {
            return;
        };

        match owner {
            Owner::Queue => self.queue.plan_refused(),
            Owner::Mirrorlist => self.services.maintenance.mirrorlist_refused(),
            Owner::Maintenance | Owner::Users | Owner::Configs | Owner::BootRepair => {}
        }

        let asked = self
            .services
            .owned_mut(owner)
            .map(PageService::plan_refused)
            .unwrap_or_default();

        for command in asked {
            self.run_app(ctx, command);
        }
    }

    pub(super) fn services_plan_finished(&mut self, ctx: &egui::Context, succeeded: bool) {
        let asked: Vec<_> = self
            .services
            .all_mut()
            .into_iter()
            .flat_map(|service| service.plan_finished(succeeded))
            .collect();

        for command in asked {
            self.run_app(ctx, command);
        }
    }
}
