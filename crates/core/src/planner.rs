//! Turns queued changes into an ordered plan. Each domain plans its own
//! changes. The order between domains comes from their stage, so packages
//! install before the units they ship and the user's session comes last.

#[cfg(test)]
mod tests;

use toolbox_plan::Plan;

use crate::{ChangeSet, Owners};

#[must_use = "a plan does nothing until it is run"]
pub fn plan(changes: &ChangeSet) -> Plan {
    planned(changes).0
}

/// The plan and the changes each of its steps carries out, so the end of
/// the plan tells each change how it went.
#[must_use = "a plan does nothing until it is run"]
pub fn planned(changes: &ChangeSet) -> (Plan, Owners) {
    let (domains, actions): (Vec<&'static str>, Vec<_>) = changes.actions().into_iter().unzip();
    let plan = Plan::new(actions);
    let owners = Owners::new(
        plan.steps().iter().map(|step| step.id).zip(domains),
        |domain| changes.keys(domain),
    );

    (plan, owners)
}
