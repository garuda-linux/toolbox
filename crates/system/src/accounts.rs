//! Users, groups and login shells from `/etc/passwd`, `/etc/group`
//! and `/etc/shells`.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::PathBuf;

use toolbox_plan::{GroupName, ShellPath, UserName};
use toolbox_state::accounts::{Accounts, Group, User};
use toolbox_state::parts::{AvailableShells, CurrentShells};

use crate::error::ProbeError;
use crate::{files, names};

pub trait AccountSource: Send + Sync {
    /// Every user and group of the account files.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the account files,
    /// [`ProbeError::Failed`] when they cannot be read.
    fn accounts(&self) -> Result<Accounts, ProbeError>;

    /// The login shell of every account in `passwd`.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `passwd`,
    /// [`ProbeError::Failed`] when it cannot be read.
    fn login_shells(&self) -> Result<CurrentShells, ProbeError>;

    /// The shells `/etc/shells` allows, in its order.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the list,
    /// [`ProbeError::Failed`] when it cannot be read.
    fn available_shells(&self) -> Result<AvailableShells, ProbeError>;
}

/// [`AccountSource`] from the files below an `etc` folder. Accounts that
/// only exist in other NSS sources (LDAP, systemd-homed) are not seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcAccounts {
    etc: PathBuf,
}

impl EtcAccounts {
    /// The running system's `/etc`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/etc")
    }

    /// The account files below `etc`.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(etc: impl Into<PathBuf>) -> Self {
        Self { etc: etc.into() }
    }
}

impl AccountSource for EtcAccounts {
    fn accounts(&self) -> Result<Accounts, ProbeError> {
        let passwd = files::read(&self.etc.join("passwd"))?;
        let group = files::read(&self.etc.join("group"))?;

        Ok(accounts(&passwd, &group))
    }

    fn login_shells(&self) -> Result<CurrentShells, ProbeError> {
        let passwd = files::read(&self.etc.join("passwd"))?;

        Ok(CurrentShells(login_shells(&passwd)))
    }

    fn available_shells(&self) -> Result<AvailableShells, ProbeError> {
        let shells = files::read(&self.etc.join("shells"))?;

        Ok(AvailableShells(available_shells(&shells)))
    }
}

fn entries(file: &str) -> impl Iterator<Item = &str> {
    file.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

/// Entries with a name our rules reject or an id that is no number are
/// logged and left out: the app could not change them anyway.
fn accounts(passwd: &str, group: &str) -> Accounts {
    let mut users: Vec<User> = entries(passwd).filter_map(user).collect();
    let mut groups: Vec<Group> = entries(group).filter_map(group_entry).collect();

    users.sort_by(|left, right| left.name.cmp(&right.name));
    users.dedup_by(|later, first| later.name == first.name);
    groups.sort_by(|left, right| left.name.cmp(&right.name));
    groups.dedup_by(|later, first| later.name == first.name);

    Accounts { users, groups }
}

pub fn name_of(passwd: &str, uid: u32) -> Option<UserName> {
    entries(passwd)
        .filter_map(user)
        .find(|user| user.uid == uid)
        .map(|user| user.name)
}

/// `name:password:uid:gid:comment:home:shell`.
fn user(entry: &str) -> Option<User> {
    let mut fields = entry.split(':');
    let name = names::one(fields.next()?)?;
    let uid = fields.nth(1)?.trim().parse().ok()?;
    let gid = fields.next()?.trim().parse().ok()?;

    Some(User {
        name,
        uid,
        gid,
        gecos: fields.next().unwrap_or_default().to_owned(),
        home: fields.next().unwrap_or_default().to_owned(),
        shell: fields.next().unwrap_or_default().trim().to_owned(),
    })
}

/// `name:password:gid:member,member`.
fn group_entry(entry: &str) -> Option<Group> {
    let mut fields = entry.split(':');
    let name: GroupName = names::one(fields.next()?)?;
    let gid = fields.nth(1)?.trim().parse().ok()?;
    let members = names::valid(
        fields
            .next()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|member| !member.is_empty()),
    );

    Some(Group { name, gid, members })
}

/// Field 7 of each `passwd` entry. Accounts without a shell (login uses
/// `/bin/sh` then) or with one our rules reject are left out.
fn login_shells(passwd: &str) -> BTreeMap<UserName, ShellPath> {
    entries(passwd)
        .filter_map(|entry| {
            let mut fields = entry.split(':');
            let name = fields.next()?;
            let shell = fields.nth(5)?.trim();
            let user: UserName = names::one(name)?;

            ShellPath::new(shell)
                .inspect_err(|error| tracing::debug!("skipped the shell of {name}: {error}"))
                .ok()
                .map(|shell| (user, shell))
        })
        .collect()
}

fn available_shells(list: &str) -> Vec<ShellPath> {
    let mut shells: Vec<ShellPath> = Vec::new();

    for shell in entries(list).filter_map(|line| ShellPath::new(line).ok()) {
        if !shells.contains(&shell) {
            shells.push(shell);
        }
    }

    shells
}
