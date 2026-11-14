//! The Network page's clicks that wait in the queue: the blacklist and
//! the hotspot tool. A refused one stays on the page until dismissed or
//! the next blacklist click is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::network::{
    HOTSPOT_PACKAGE, ModuleFacts, Refusal, set_blacklisted, toggle_install,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{KernelModule, Toggle};
use toolbox_services::queue::edit::QueueEdit;

use super::{Network, NetworkRefusal};
use crate::command::{AppCommand, NetworkCommand};
use crate::page_service::Handled;
use crate::refused::{Refused, queue_or_refuse};

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
            NetworkCommand::ToggleHotspot => Handled::edit(
                QueueEdit::of_click(changes, |changes| {
                    toggle_install(&HOTSPOT_PACKAGE, changes, snapshot);
                })
                .0,
            ),
            NetworkCommand::DismissRefusal => {
                self.refusal = None;
                Handled::taken()
            }
            NetworkCommand::Refresh
            | NetworkCommand::SetRadio {
                target: _,
                blocked: _,
            }
            | NetworkCommand::LookupAddress
            | NetworkCommand::ForgetAddress
            | NetworkCommand::StartService(_)
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
