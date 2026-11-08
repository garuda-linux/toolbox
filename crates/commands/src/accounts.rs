//! The plans come from the same core rules the dialogs use.
//! Passwords go from the terminal or standard input into the plan and nowhere else.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io::Write;

use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{
    AddUser, EditUser, GroupArgs, GroupsCommand, PasswordArgs, PasswordSource, RemoveUser,
    UserArgs, UsersCommand,
};
use toolbox_cli::confirm::Prompt;
use toolbox_core::features::users::home::set_home;
use toolbox_core::features::users::now::{new_user, plan_add_user, plan_set_password};
use toolbox_core::features::users::{
    Refusal, accounts, add_group, remove_group, remove_user, set_full_name, set_groups, set_locked,
    set_shell,
};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::{FullName, GroupName, HomeChange, HomeFolder, Password, Plan, UserName};

use super::act::run;
use super::change::{Refused, fresh, page_edit};
use super::terminal::{Terminal, wipe, without_line_end};
use super::{groups, system, users};

const ADMIN_GROUP: &str = "wheel";

pub fn users_command(command: UsersCommand, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let snapshot = system();
    match command {
        UsersCommand::List(list) => users(&list.read, list.all, &snapshot, out),
        UsersCommand::Add(args) => {
            let planned = add(&args, &snapshot, &mut Terminal);
            run(planned, args.confirm, None, runtime, out)
        }
        UsersCommand::Edit(args) => run(edit(&args, &snapshot), args.confirm, None, runtime, out),
        UsersCommand::Password(args) => {
            let planned = password(&args, &snapshot, &mut Terminal);
            run(planned, args.confirm, None, runtime, out)
        }
        UsersCommand::Lock(args) => run(
            lock(&args, Toggle::On, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        UsersCommand::Unlock(args) => run(
            lock(&args, Toggle::Off, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        UsersCommand::Remove(args) => {
            let typed = args.delete_home.then(|| args.user.to_string());
            let planned = remove(&args, &snapshot);
            run(planned, args.confirm, typed.as_deref(), runtime, out)
        }
    }
}

pub fn groups_command(command: GroupsCommand, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let snapshot = system();
    match command {
        GroupsCommand::List(list) => groups(&list, &snapshot, out),
        GroupsCommand::Add(args) => run(
            group(&args, Toggle::On, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        GroupsCommand::Remove(args) => run(
            group(&args, Toggle::Off, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
    }
}

/// The account and its password, at once: the add-user dialog's rules.
///
/// # Errors
///
/// [`Refused`] for a bad or taken name, an unlisted shell, an unknown
/// group or no password.
pub fn add(args: &AddUser, snapshot: &Snapshot, prompt: &mut dyn Prompt) -> Result<Plan, Refused> {
    let name = args.user.clone();
    let full_name = match &args.full_name {
        Some(full_name) => full_name.clone(),
        None => full_name("")?,
    };

    let shell = args.shell.clone();
    let mut groups = args.groups.iter().cloned().collect::<BTreeSet<_>>();

    if args.admin {
        groups.insert(group_name(ADMIN_GROUP)?);
    }

    let user = new_user(name, full_name, shell, groups, snapshot).map_err(rule)?;
    let password = read_password(args.password, args.user.as_str(), prompt)?;

    Ok(plan_add_user(user, password))
}

/// Full name, login shell and groups, through the queue.
///
/// # Errors
///
/// [`Refused`] for an unknown user, a bad value or what the page refuses.
pub fn edit(args: &EditUser, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| edit_onto(args, snapshot, changes))
}

/// # Errors
///
/// As [`edit`].
pub fn edit_onto(
    args: &EditUser,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let user = &args.user;
    let full_name = args.full_name.clone();
    let shell = args.shell.clone();
    let wanted = wanted_groups(user, &args.add_groups, &args.remove_groups, snapshot)?;
    let home = args.home.clone().map(|home| HomeChange {
        home,
        move_files: args.move_files,
    });

    page_edit(changes, snapshot, Refusal::as_str, |changes, snapshot| {
        if let Some(full_name) = full_name {
            set_full_name(user, full_name, changes, snapshot)?;
        }

        if let Some(home) = home {
            set_home(user, home, changes, snapshot)?;
        }

        if let Some(shell) = shell {
            set_shell(user, shell, changes, snapshot)?;
        }

        wanted.map_or(Ok(()), |wanted| {
            set_groups(user, &wanted, changes, snapshot)
        })
    })
    .map_err(|refused| known_user(refused, user, snapshot))
}

/// # Errors
///
/// [`Refused`] for root, an unknown user or no password.
pub fn password(
    args: &PasswordArgs,
    snapshot: &Snapshot,
    prompt: &mut dyn Prompt,
) -> Result<Plan, Refused> {
    // Checked before the question, so nobody types a password in vain.
    accounts(snapshot).map_err(rule)?;
    let password = read_password(args.password, args.user.as_str(), prompt)?;

    plan_set_password(&args.user, password, snapshot)
        .map_err(|refusal| known_user(rule(refusal), &args.user, snapshot))
}

/// # Errors
///
/// [`Refused`] for an unknown user or what the page refuses.
pub fn lock(args: &UserArgs, locked: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| lock_onto(args, locked, snapshot, changes))
}

/// # Errors
///
/// As [`lock`].
pub fn lock_onto(
    args: &UserArgs,
    locked: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    page_edit(changes, snapshot, Refusal::as_str, |changes, snapshot| {
        set_locked(&args.user, locked, None, changes, snapshot)
    })
    .map_err(|refused| known_user(refused, &args.user, snapshot))
}

/// # Errors
///
/// [`Refused`] for an unknown user, root or the user running this.
pub fn remove(args: &RemoveUser, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| remove_onto(args, snapshot, changes))
}

/// # Errors
///
/// As [`remove`].
pub fn remove_onto(
    args: &RemoveUser,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let home = if args.delete_home {
        HomeFolder::Delete
    } else {
        HomeFolder::Keep
    };

    page_edit(changes, snapshot, Refusal::as_str, |changes, snapshot| {
        remove_user(&args.user, home, changes, snapshot)
    })
    .map_err(|refused| known_user(refused, &args.user, snapshot))
}

/// # Errors
///
/// [`Refused`] for a bad name or what the page refuses.
pub fn group(args: &GroupArgs, wanted: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| group_onto(args, wanted, snapshot, changes))
}

/// # Errors
///
/// As [`group`].
pub fn group_onto(
    args: &GroupArgs,
    wanted: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    page_edit(changes, snapshot, Refusal::as_str, |changes, snapshot| {
        if wanted == Toggle::On {
            add_group(&args.group, changes, snapshot)
        } else {
            remove_group(&args.group, changes, snapshot)
        }
    })
    .map_err(|refused| known_group(refused, &args.group, snapshot))
}

/// The user's groups after `add` and `remove`, `None` when neither asks
/// for a change.
fn wanted_groups(
    user: &UserName,
    add: &[GroupName],
    remove: &[GroupName],
    snapshot: &Snapshot,
) -> Result<Option<BTreeSet<GroupName>>, Refused> {
    if add.is_empty() && remove.is_empty() {
        return Ok(None);
    }

    let mut wanted = accounts(snapshot).map_err(rule)?.groups_of(user);
    for group in add {
        wanted.insert(group.clone());
    }

    for group in remove {
        wanted.remove(group);
    }

    Ok(Some(wanted))
}

/// The password from standard input's first line, or asked twice on the
/// terminal without echo.
fn read_password(
    source: PasswordSource,
    user: &str,
    prompt: &mut dyn Prompt,
) -> Result<Password, Refused> {
    let text = if source.password_stdin {
        prompt
            .line()
            .ok_or_else(|| Refused::Rule("no password was given".to_owned()))?
    } else {
        let no_terminal = || {
            Refused::Rule("no terminal to ask the password on, pass --password-stdin".to_owned())
        };

        let first = prompt
            .secret(&format!("New password for {user}: "))
            .ok_or_else(no_terminal)?;

        let again = prompt
            .secret("Repeat the password: ")
            .ok_or_else(no_terminal);

        let matches = again.as_ref().is_ok_and(|again| *again == first);
        again.map(wipe).ok();

        if !matches {
            wipe(first);
            return Err(Refused::Rule("the passwords do not match".to_owned()));
        }

        first
    };

    Password::new(without_line_end(text))
        .map_err(|invalid| Refused::Rule(format!("the password is not usable: {invalid}")))
}

fn group_name(text: &str) -> Result<GroupName, Refused> {
    GroupName::new(text).map_err(|_invalid| {
        Refused::Unknown(format!("`{text}` is no group name, see `groups list`"))
    })
}

fn full_name(text: &str) -> Result<FullName, Refused> {
    FullName::new(text)
        .map_err(|invalid| Refused::Rule(format!("the full name is not usable: {invalid}")))
}

fn rule(refusal: Refusal) -> Refused {
    Refused::page_rule(refusal.as_str())
}

fn known_user(refused: Refused, user: &UserName, snapshot: &Snapshot) -> Refused {
    let known = super::names::user_names(snapshot);
    closest(refused, Refusal::UnknownUser, user.as_str(), &known)
}

fn known_group(refused: Refused, group: &GroupName, snapshot: &Snapshot) -> Refused {
    let known = super::names::group_names(snapshot);
    closest(refused, Refusal::UnknownGroup, group.as_str(), &known)
}

fn closest(refused: Refused, unknown: Refusal, wanted: &str, known: &[String]) -> Refused {
    if refused.message().contains(unknown.as_str()) {
        refused.hinted(wanted, known.iter().map(String::as_str))
    } else {
        refused
    }
}
