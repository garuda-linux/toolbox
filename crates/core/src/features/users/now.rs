//! They never enter the queue (which is shown, saved to disk and restored):
//! each is a plan that runs at once, with the password inside the action where
//! no text form of the plan shows it.

use std::collections::BTreeSet;

use toolbox_plan::{
    FullName, GroupName, NewHome, NewUser, NewUserParts, Password, Plan, PlannedAction, RootAction,
    ShellPath, UserName,
};

use super::{Refusal, accounts, editable, shell_is_listed};
use crate::Snapshot;

/// What the add-user dialog collected, checked against the system: an
/// account of a person with the home folder and user id useradd picks.
/// [`super::home::place`] changes those.
///
/// # Errors
///
/// [`Refusal`] when a user or a group of that name exists (useradd makes
/// a group named like the user), the shell is not listed, or a group is
/// unknown. Queued new groups do not count: this runs before the queue.
pub fn new_user(
    name: UserName,
    full_name: FullName,
    shell: Option<ShellPath>,
    groups: BTreeSet<GroupName>,
    snapshot: &Snapshot,
) -> Result<NewUser, Refusal> {
    check_new_name(&name, snapshot)?;

    if shell
        .as_ref()
        .is_some_and(|shell| !shell_is_listed(shell, snapshot))
    {
        return Err(Refusal::ShellNotListed);
    }

    let accounts = accounts(snapshot)?;

    if groups.iter().any(|group| accounts.group(group).is_none()) {
        return Err(Refusal::UnknownGroup);
    }

    NewUser::new(NewUserParts {
        name,
        full_name,
        shell,
        groups,
        home: NewHome::Standard,
        uid: None,
        system: false,
    })
    .map_err(|_out_of_range| Refusal::UidOutOfRange)
}

/// # Errors
///
/// [`Refusal::UserExists`], [`Refusal::GroupExists`] or
/// [`Refusal::Unavailable`].
pub fn check_new_name(name: &UserName, snapshot: &Snapshot) -> Result<(), Refusal> {
    let accounts = accounts(snapshot)?;

    if accounts.user(name).is_some() {
        return Err(Refusal::UserExists);
    }

    let own_group_taken =
        GroupName::new(name.as_str()).is_ok_and(|group| accounts.group(&group).is_some());

    if own_group_taken {
        return Err(Refusal::GroupExists);
    }

    Ok(())
}

/// `useradd`, then `chpasswd`: one plan, so a failed first step keeps the
/// second from running.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_add_user(user: NewUser, password: Password) -> Plan {
    let name = user.parts().name.clone();

    Plan::new([
        PlannedAction::Root(RootAction::CreateUser { user }),
        PlannedAction::Root(RootAction::SetPassword {
            user: name,
            password: Some(password),
        }),
    ])
}

/// # Errors
///
/// [`Refusal`] for root or an unknown user.
pub fn plan_set_password(
    user: &UserName,
    password: Password,
    snapshot: &Snapshot,
) -> Result<Plan, Refusal> {
    editable(user, accounts(snapshot)?)?;

    Ok(Plan::root(RootAction::SetPassword {
        user: user.clone(),
        password: Some(password),
    }))
}
