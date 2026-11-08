//! The procedures of the root actions that create, change and remove
//! users and groups. Every name follows a `--`, so none can be read as an
//! option, and the password never is an argument.

#[cfg(test)]
mod tests;

use super::run;
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{FullName, GroupName, HomeChange, HomeFolder, NewHome, NewUser, Password, UserName};

/// The home folder is always asked for or refused with a flag (`-m`,
/// `-M`), so `CREATE_HOME` of `/etc/login.defs` decides nothing. `-m`
/// makes it from `/etc/skel`. The shell is checked like in `SetShell`:
/// useradd as root takes any program. The user id comes without `-o`, so
/// useradd refuses one that is in use.
pub fn create_user(user: &NewUser) -> Vec<Invocation> {
    let user = user.parts();
    let mut steps = Vec::new();
    let mut command = Command::new(Program::Useradd);

    if user.system {
        command = command.arg("-r");
    }

    command = match &user.home {
        NewHome::Standard => command.arg("-m"),
        NewHome::At(home) => command.args(["-m", "-d", home.as_str()]),
        NewHome::Absent => command.arg("-M"),
    };

    if let Some(uid) = user.uid {
        command = command.args(["-u", &uid.to_string()]);
    }

    if !user.full_name.is_empty() {
        command = command.args(["-c", user.full_name.as_str()]);
    }

    if let Some(shell) = &user.shell {
        steps.push(Invocation::Builtin(Builtin::RequireListedShell(
            shell.clone(),
        )));
        command = command.args(["-s", shell.as_str()]);
    }

    if !user.groups.is_empty() {
        let groups: Vec<&str> = user.groups.iter().map(GroupName::as_str).collect();

        command = command.args(["-G", &groups.join(",")]);
    }

    steps.push(run(command.args(["--", user.name.as_str()])));
    steps
}

/// chpasswd reads `user:password` lines from its standard input and
/// prints nothing on success.
pub fn set_password(user: &UserName, password: Option<&Password>) -> Vec<Invocation> {
    vec![Invocation::Feed {
        command: Command::new(Program::Chpasswd),
        input: password.map(|password| password.chpasswd_input(user)),
    }]
}

pub fn delete_user(user: &UserName, home: HomeFolder) -> Vec<Invocation> {
    let command = Command::new(Program::Userdel);
    let command = match home {
        HomeFolder::Keep => command,
        HomeFolder::Delete => command.arg("-r"),
    };

    vec![
        Invocation::Builtin(Builtin::RequireOrdinaryUser(user.clone())),
        run(command.args(["--", user.as_str()])),
    ]
}

/// Unlocking root is harmless, locking it can shut the last way in.
pub fn set_locked(user: &UserName, locked: bool) -> Vec<Invocation> {
    let flag = if locked { "-L" } else { "-U" };
    let change = run(Command::new(Program::Usermod).args([flag, "--", user.as_str()]));

    if locked {
        vec![
            Invocation::Builtin(Builtin::RequireOrdinaryUser(user.clone())),
            change,
        ]
    } else {
        vec![change]
    }
}

/// `-m` moves the content of the old folder to the new one, which usermod
/// creates. Without it only the entry in `/etc/passwd` changes. usermod
/// refuses an account that still runs a program.
pub fn set_home_folder(user: &UserName, change: &HomeChange) -> Vec<Invocation> {
    let command = Command::new(Program::Usermod).args(["-d", change.home.as_str()]);
    let command = if change.move_files {
        command.arg("-m")
    } else {
        command
    };

    let mut procedure = vec![Invocation::Builtin(Builtin::RequireOrdinaryUser(
        user.clone(),
    ))];

    if change.move_files {
        procedure.push(Invocation::Builtin(Builtin::RequireFreeHome(
            change.home.clone(),
        )));
    }

    procedure.push(run(command.args(["--", user.as_str()])));
    procedure
}

/// chfn changes only the name part of the GECOS field. `usermod -c` would
/// replace the whole field and drop room and phone numbers.
pub fn set_full_name(user: &UserName, full_name: &FullName) -> Vec<Invocation> {
    vec![run(Command::new(Program::Chfn).args([
        "-f",
        full_name.as_str(),
        "--",
        user.as_str(),
    ]))]
}

pub fn create_group(group: &GroupName) -> Vec<Invocation> {
    vec![run(
        Command::new(Program::Groupadd).args(["--", group.as_str()])
    )]
}

pub fn delete_group(group: &GroupName) -> Vec<Invocation> {
    vec![
        Invocation::Builtin(Builtin::RequireUserGroup(group.clone())),
        run(Command::new(Program::Groupdel).args(["--", group.as_str()])),
    ]
}
