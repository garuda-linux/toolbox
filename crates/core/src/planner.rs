//! Turns queued changes into an ordered plan. Each domain plans its own
//! changes. The order between domains comes from their stage, so packages
//! install before the units they ship and the user's session comes last.

#[cfg(test)]
mod tests;

use toolbox_plan::Plan;

use crate::ChangeSet;

#[must_use = "a plan does nothing until it is run"]
pub fn plan(changes: &ChangeSet) -> Plan {
    Plan::new(changes.actions())
}
