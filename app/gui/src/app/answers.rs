//! What App does for the page services, which know nothing of it.

use toolbox_gui_kit::toasts::ToastKind;

use super::App;
use crate::page::PageId;
use crate::page_service::{Answer, PageService};
use crate::{i18n, page_ui};

impl App {
    pub(super) fn answer(&mut self, ctx: &egui::Context, answer: Answer) {
        match answer {
            Answer::RunPlan(plan) => {
                self.run_at_once(plan);
            }
            Answer::Recombine => {
                self.combine();
                self.queue.meet(&self.seen);
            }
            Answer::EditBoot(edit) => self.edit_boot(edit),
            Answer::Configs(act) => self.act_configs(Some(act)),
            Answer::CloseConsole => self.state.console_open = false,
            Answer::Copy { text, toast } => {
                if self.prefs.setup.settings.diagnostics.copy_to_clipboard {
                    ctx.copy_text(text);
                    self.toasts.add(ToastKind::Info, i18n::text(ctx, toast));
                }
            }
            Answer::OfferBootRepair(offered) => page_ui::offer_boot_repair(ctx, offered),
            Answer::LeaveBootRepair => self.state.page = PageId::Home,
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

    pub(super) fn services_plan_started(&mut self, ctx: &egui::Context) {
        let asked: Vec<_> = self
            .services
            .all_mut()
            .into_iter()
            .flat_map(PageService::plan_started)
            .collect();

        for command in asked {
            self.run_app(ctx, command);
        }
    }

    pub(super) fn services_plan_refused(&mut self, ctx: &egui::Context) {
        let asked: Vec<_> = self
            .services
            .all_mut()
            .into_iter()
            .flat_map(PageService::plan_refused)
            .collect();

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
