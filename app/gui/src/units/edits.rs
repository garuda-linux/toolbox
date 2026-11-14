//! The Services page's clicks that need no Console: a unit's boot and
//! mask state, which wait in the queue, and a new list. A control and the
//! journal run at once, which App does.

#[cfg(test)]
mod tests;

use toolbox_core::features::services::{self, UnitWish};
use toolbox_core::{ChangeSet, Snapshot, Toggle};

use super::Units;
use crate::command::{AppCommand, ServiceCommand};
use crate::page_service::Handled;
use crate::refused::queue;

impl Units {
    pub fn handle(
        &self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Service(command) = command else {
            return Handled::default();
        };

        let (scope, unit, wish) = match command {
            ServiceCommand::Enable { scope, unit, on } => {
                (scope, unit, UnitWish::AtBoot(Toggle::from_on(*on)))
            }
            ServiceCommand::Mask { scope, unit, on } => {
                (scope, unit, UnitWish::Masked(Toggle::from_on(*on)))
            }
            ServiceCommand::Refresh(scope) => {
                self.refresh(*scope);
                return Handled::taken();
            }
            ServiceCommand::Control {
                scope: _,
                unit: _,
                operation: _,
            }
            | ServiceCommand::Logs { scope: _, unit: _ } => return Handled::default(),
        };

        queue(changes, |changes| {
            services::queue(changes, *scope, unit.clone(), wish, snapshot);
        })
    }
}
