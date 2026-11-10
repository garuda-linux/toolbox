//! What the app does with a click on the Boot Repair page. A repair runs
//! at once in the Console and is never queued. A refused click stays on
//! the page until dismissed or the next click is taken.

use toolbox_core::features::bootrepair::page::Offer;
use toolbox_core::features::bootrepair::{Refusal, Target, Wish, plan_regenerate, plan_reinstall};

use super::App;
use crate::command::BootRepairCommand;

impl App {
    pub(super) fn run_boot_repair(&mut self, command: BootRepairCommand) {
        match command {
            BootRepairCommand::Refresh => self.services.boot_repair.refresh(),
            BootRepairCommand::Repair {
                system,
                layout,
                passphrase,
                target,
            } => {
                let wish = Wish {
                    system,
                    layout,
                    passphrase,
                };

                let result = self.repair(wish, target);
                self.refusals.boot_repair = result.err().map(Refusal::as_str);
            }
            BootRepairCommand::Forget => self.services.boot_repair.forget(),
            BootRepairCommand::ShowConsole => self.state.console_open = true,
            BootRepairCommand::DismissRefusal => self.refusals.boot_repair = None,
        }
    }

    /// Builds the plan from what the page last read and starts it. A
    /// Console that runs something else refuses it, as the apply service
    /// would.
    fn repair(&mut self, wish: Wish, target: Option<Target>) -> Result<(), Refusal> {
        let repair = &self.services.boot_repair;
        if repair.offer() == Offer::Hidden {
            return Err(Refusal::NotOffered);
        }

        let facts = repair
            .facts()
            .known(repair.offer())
            .ok_or(Refusal::Unavailable)?;

        let (plan, what) = match target {
            Some(target) => (plan_reinstall(facts, wish, &target)?, "installing GRUB"),
            None => (plan_regenerate(facts, wish)?, "generating a boot menu"),
        };

        tracing::info!("boot repair: {what}");
        self.services.boot_repair.started();
        if !self.run_at_once(plan) {
            self.services.boot_repair.plan_refused();
        }

        Ok(())
    }
}
