//! The Network page's clicks that need no run: the queued blacklist and
//! hotspot tool, the radios and the address. A refused one stays on the
//! page until dismissed or the next blacklist click is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::network::{
    HOTSPOT_PACKAGE, ModuleFacts, Refusal, set_blacklisted, toggle_install,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{KernelModule, Toggle, UnitScope};

use super::{Network, NetworkRefusal};
use crate::command::{AppCommand, NetworkCommand, ServiceCommand};
use crate::page_service::Handled;
use crate::refused::{Refused, queue, queue_or_refuse};

impl Network {
    pub const fn refusal(&self) -> Option<&NetworkRefusal> {
        self.refusal.as_ref()
    }

    pub fn set_refusal(&mut self, refusal: Option<NetworkRefusal>) {
        self.refusal = refusal;
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Network(command) = command else {
            return Handled::default();
        };

        match command {
            NetworkCommand::Blacklist { module, on } => {
                self.blacklist(module, *on, changes, snapshot)
            }
            NetworkCommand::ToggleHotspot => queue(changes, |changes| {
                toggle_install(&HOTSPOT_PACKAGE, changes, snapshot);
            }),
            // The page also lists the system's units, which App reads in.
            NetworkCommand::Refresh => {
                self.refresh();
                Handled::answers(vec![
                    AppCommand::ReadSystem,
                    AppCommand::Service(ServiceCommand::Refresh(UnitScope::System)),
                ])
            }
            NetworkCommand::SetRadio { target, blocked } => {
                self.set_radio(*target, *blocked);
                Handled::taken()
            }
            NetworkCommand::LookupAddress => {
                self.lookup_address();
                Handled::taken()
            }
            NetworkCommand::ForgetAddress => {
                self.forget_address();
                Handled::taken()
            }
            NetworkCommand::StartService(_)
            | NetworkCommand::LoadDriver { module: _, load: _ }
            | NetworkCommand::Ping {
                host: _,
                count: _,
                timeout: _,
            }
            | NetworkCommand::Traceroute { host: _, hops: _ }
            | NetworkCommand::OpenHotspot => Handled::default(),
        }
    }

    fn blacklist(
        &mut self,
        module: &KernelModule,
        on: bool,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let subject = module.as_str().to_owned();
        let Some(facts) = snapshot.part::<ModuleFacts>() else {
            self.refusal = Some(Refused::new(Refusal::Unavailable, subject));
            return Handled::taken();
        };

        queue_or_refuse(&mut self.refusal, subject, changes, |changes| {
            set_blacklisted(module, Toggle::from_on(on), facts, changes, snapshot)
        })
    }
}
