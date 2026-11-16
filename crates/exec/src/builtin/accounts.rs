//! The helper's own look at `/etc/passwd` and `/etc/group` before it
//! removes or locks an account. The app guards the same things, but a
//! request on the wire is all the helper can trust.

#[cfg(test)]
mod tests;

use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::Path;
use std::{fs, io};

use toolbox_plan::lines::{entries, lossy};
use toolbox_plan::procedure::{FIRST_USER_ID, GROUP, PASSWD};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    GroupName, HomeDirectory, LAST_USER_ID, PEOPLE, StepId, UserName, passwd_entries,
};

use crate::apply::EventSink;
use crate::report::{failure, note};

const WORLD_WRITABLE: u32 = 0o002;
const GROUP_WRITABLE: u32 = 0o020;
const ROOT_UID: u32 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listed {
    Id(u32),
    /// No entry, or one whose ids are no numbers.
    Missing,
}

/// # Errors
///
/// When the file cannot be read.
pub fn user_id(root: &Path, user: &UserName) -> io::Result<Listed> {
    let passwd = lossy(fs::read(root.join(PASSWD))?);

    Ok(passwd_entries(&passwd)
        .find(|entry| entry.name == user.as_str())
        .map_or(Listed::Missing, |entry| Listed::Id(entry.uid)))
}

/// # Errors
///
/// When the file cannot be read.
pub fn group_id(root: &Path, group: &GroupName) -> io::Result<Listed> {
    Ok(gid_of(&lossy(fs::read(root.join(GROUP))?), group.as_str()))
}

pub fn require_ordinary_user(
    root: &Path,
    step: StepId,
    user: &UserName,
    sink: &mut dyn EventSink,
) -> Outcome {
    match user_id(root, user) {
        Ok(Listed::Id(id)) if !PEOPLE.contains(&id) => failure(
            step,
            sink,
            &format!(
                "{user} has the user id {id}, outside {FIRST_USER_ID} to {LAST_USER_ID}, so it stays as it is"
            ),
        ),
        Ok(Listed::Id(id)) => note(
            step,
            sink,
            &format!("{user} has the user id {id}"),
            Outcome::Succeeded,
        ),
        Ok(Listed::Missing) => failure(step, sink, &format!("{user} is not listed in /{PASSWD}")),
        Err(error) => failure(step, sink, &format!("cannot read /{PASSWD}: {error}")),
    }
}

/// Fails when `home` exists, also as a dangling link: `usermod -d <home>
/// -m` would write it into `/etc/passwd` and then refuse to move. Fails
/// too below a folder everyone may write.
pub fn require_free_home(
    root: &Path,
    step: StepId,
    user: &UserName,
    home: &HomeDirectory,
    sink: &mut dyn EventSink,
) -> Outcome {
    let path = root.join(home.as_str().trim_start_matches('/'));

    // A user that is not listed (or cannot be read) leaves only root's
    // folders on the way.
    let account = match user_id(root, user) {
        Ok(Listed::Id(id)) => Some(id),
        Ok(Listed::Missing) | Err(_) => None,
    };

    if let Some(refusal) = shared_ancestor(root, &path, home, account) {
        return failure(step, sink, &refusal);
    }

    match fs::symlink_metadata(&path) {
        Ok(_) => failure(
            step,
            sink,
            &format!("{home} exists already, so the files are not moved onto it"),
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            note(step, sink, &format!("{home} is free"), Outcome::Succeeded)
        }
        Err(error) => failure(step, sink, &format!("cannot check {home}: {error}")),
    }
}

/// Fails below a folder everyone may write, where `useradd -m` would make
/// the home. A home that exists already is fine: useradd only takes it.
pub fn require_safe_home(
    root: &Path,
    step: StepId,
    home: &HomeDirectory,
    sink: &mut dyn EventSink,
) -> Outcome {
    let path = root.join(home.as_str().trim_start_matches('/'));

    shared_ancestor(root, &path, home, None)
        .map_or(Outcome::Succeeded, |refusal| failure(step, sink, &refusal))
}

/// Below a folder another user may write, a link can land on the way to
/// `path` between this check and the program that makes or moves the home.
/// Every folder up to `root` that exists counts, followed as the program
/// follows it. Besides root only `account`, the uid of the home's account
/// (`None` before it exists), and the owner of `root` may own one: whoever
/// owns `root` can replace anything below it. In the helper `root` is `/`.
fn shared_ancestor(
    root: &Path,
    path: &Path,
    home: &HomeDirectory,
    account: Option<u32>,
) -> Option<String> {
    let owner_of_root = match fs::metadata(root) {
        Ok(folder) => folder.uid(),
        Err(error) => return Some(format!("cannot check the folders above {home}: {error}")),
    };

    let trusted =
        |owner: u32| owner == ROOT_UID || owner == owner_of_root || Some(owner) == account;

    for ancestor in path
        .ancestors()
        .skip(1)
        .take_while(|folder| folder.starts_with(root))
    {
        match fs::metadata(ancestor) {
            Ok(folder) => {
                let mode = folder.permissions().mode();
                let owner = folder.uid();

                if mode & WORLD_WRITABLE != 0 {
                    return Some(format!(
                        "a folder above {home} is writable for everyone, so it is not used"
                    ));
                }

                if mode & GROUP_WRITABLE != 0 {
                    return Some(format!(
                        "a folder above {home} is writable for its group, so it is not used"
                    ));
                }

                if !trusted(owner) {
                    return Some(format!(
                        "a folder above {home} belongs to user id {owner}, so it is not used"
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Some(format!("cannot check the folders above {home}: {error}")),
        }
    }

    None
}

pub fn require_user_group(
    root: &Path,
    step: StepId,
    group: &GroupName,
    sink: &mut dyn EventSink,
) -> Outcome {
    match group_id(root, group) {
        Ok(Listed::Id(id)) if !PEOPLE.contains(&id) => failure(
            step,
            sink,
            &format!(
                "{group} has the group id {id}, outside {FIRST_USER_ID} to {LAST_USER_ID}, so it stays"
            ),
        ),
        Ok(Listed::Id(id)) => note(
            step,
            sink,
            &format!("{group} has the group id {id}"),
            Outcome::Succeeded,
        ),
        Ok(Listed::Missing) => failure(step, sink, &format!("{group} is not listed in /{GROUP}")),
        Err(error) => failure(step, sink, &format!("cannot read /{GROUP}: {error}")),
    }
}

/// Field 3 of the first entry named `name`, `name:password:gid:members`.
fn gid_of(file: &str, name: &str) -> Listed {
    entries(file)
        .find_map(|line| {
            let mut fields = line.split(':');

            (fields.next()? == name).then(|| fields.nth(1)?.trim().parse::<u32>().ok())
        })
        .flatten()
        .map_or(Listed::Missing, Listed::Id)
}
