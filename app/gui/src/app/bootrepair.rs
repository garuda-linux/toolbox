//! What the app does with a click on the Boot Repair page. A repair runs
//! at once in the Console and is never queued. A refused click stays on
//! the page until dismissed or the next click is taken.

use toolbox_core::features::bootrepair::page::Offer;
use toolbox_core::features::bootrepair::{Refusal, Target, Wish, plan_regenerate, plan_reinstall};
use toolbox_system::Host;

use super::App;
use crate::command::BootRepairCommand;
use crate::page::PageId;
use crate::page_ui;

impl App {
    pub(super) fn run_boot_repair(&mut self, command: BootRepairCommand) {
        match command {
            BootRepairCommand::Refresh => self.boot_repair.refresh(),
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
            BootRepairCommand::Forget => self.boot_repair.forget(),
            BootRepairCommand::ShowConsole => self.state.console_open = true,
            BootRepairCommand::DismissRefusal => self.refusals.boot_repair = None,
        }
    }

    /// Builds the plan from what the page last read and starts it. A
    /// Console that runs something else refuses it, as the apply service
    /// would.
    fn repair(&mut self, wish: Wish, target: Option<Target>) -> Result<(), Refusal> {
        let repair = &self.boot_repair;
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
        self.boot_repair.started();
        if !self.run_at_once(plan) {
            self.boot_repair.plan_refused();
        }

        Ok(())
    }

    /// The page reads the disks once it shows. The snapshot says whether
    /// this is the live ISO, and the sidebar learns whether to list it.
    pub(super) fn take_boot_repair(&mut self, ctx: &egui::Context) {
        let live = self.seen.part::<Host>().is_some_and(|host| host.live_iso);

        self.boot_repair.see_live(live);
        page_ui::offer_boot_repair(ctx, self.boot_repair.offered());
        if self.state.page == PageId::BootRepair {
            if self.boot_repair.offered() {
                self.boot_repair.open();
            } else {
                self.state.page = PageId::Home;
            }
        }

        self.boot_repair.take();
    }
}
