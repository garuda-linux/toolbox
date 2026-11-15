//! Who sent which plan that has not started yet. The apply service
//! answers each plan under its id, so only the plan's sender hears that
//! it started or was refused, also when two plans were sent at once.

#[cfg(test)]
mod tests;

use crate::apply::PlanId;

/// A part of the app that waits for the start of a plan it sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Queue,
    Maintenance,
    /// Maintenance's install of a ranked mirrorlist, apart from its tool
    /// installs, as both may be on their way at once.
    Mirrorlist,
    Users,
    Configs,
    BootRepair,
}

#[derive(Debug, Default)]
pub struct Waiting {
    sent: Vec<(PlanId, Owner)>,
}

impl Waiting {
    pub fn sent(&mut self, plan: PlanId, owner: Owner) {
        self.sent.push((plan, owner));
    }

    /// The sender of `plan`, which waits no more, as the plan started or
    /// was refused. `None` for a plan nobody waits for.
    pub fn answered(&mut self, plan: PlanId) -> Option<Owner> {
        let at = self.sent.iter().position(|(sent, _)| *sent == plan)?;
        Some(self.sent.remove(at).1)
    }
}
