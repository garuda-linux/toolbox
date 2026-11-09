//! The helper's own look at `/etc/passwd` and `/etc/group` before it
//! removes or locks an account. The app guards the same things, but a
//! request on the wire is all the helper can trust.

#[cfg(test)]
mod tests;

use std::ops::RangeInclusive;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::{fs, io};

use toolbox_plan::procedure::{FIRST_USER_ID, GROUP, PASSWD};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{GroupName, HomeDirectory, LAST_USER_ID, StepId, UserName};

use crate::apply::EventSink;
use crate::report::{failure, note};

/// Root, system accounts and `nobody` (65534) lie outside, and files on
/// disk refer to them.
const ORDINARY_IDS: RangeInclusive<u32> = FIRST_USER_ID..=LAST_USER_ID;

const WORLD_WRITABLE: u32 = 0o002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listed {
    Id(u32),
    /// No entry, or one whose id is no number.
    Missing,
}

/// # Errors
///
/// When the file cannot be read.
pub fn user_id(root: &Path, user: &UserName) -> io::Result<Listed> {
    Ok(id_of(
        &fs::read_to_string(root.join(PASSWD))?,
        user.as_str(),
    ))
}

/// # Errors
///
/// When the file cannot be read.
pub fn group_id(root: &Path, group: &GroupName) -> io::Result<Listed> {
    Ok(id_of(
        &fs::read_to_string(root.join(GROUP))?,
        group.as_str(),
    ))
}

pub fn require_ordinary_user(
    root: &Path,
    step: StepId,
    user: &UserName,
    sink: &mut dyn EventSink,
) -> Outcome {
    match user_id(root, user) {
        Ok(Listed::Id(id)) if !ORDINARY_IDS.contains(&id) => failure(
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
    home: &HomeDirectory,
    sink: &mut dyn EventSink,
) -> Outcome {
    let path = root.join(home.as_str().trim_start_matches('/'));

    if let Some(refusal) = shared_ancestor(root, &path, home) {
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

    shared_ancestor(root, &path, home)
        .map_or(Outcome::Succeeded, |refusal| failure(step, sink, &refusal))
}

/// Below a folder anyone may write, another user can put a link on the
/// way to `path` between this check and the program that makes or moves
/// the home. Every folder up to `root` that exists counts, followed as the
/// program follows it.
fn shared_ancestor(root: &Path, path: &Path, home: &HomeDirectory) -> Option<String> {
    for ancestor in path
        .ancestors()
        .skip(1)
        .take_while(|folder| folder.starts_with(root))
    {
        match fs::metadata(ancestor) {
            Ok(folder) if folder.permissions().mode() & WORLD_WRITABLE != 0 => {
                return Some(format!(
                    "a folder above {home} is writable for everyone, so it is not used"
                ));
            }
            Ok(_) => {}
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
        Ok(Listed::Id(id)) if !ORDINARY_IDS.contains(&id) => failure(
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

/// Field 3 of the first entry named `name`. Both files share the layout
/// `name:password:id:…`.
fn id_of(file: &str, name: &str) -> Listed {
    file.lines()
        .find_map(|line| {
            let mut fields = line.split(':');

            (fields.next()? == name).then(|| fields.nth(1)?.trim().parse::<u32>().ok())
        })
        .flatten()
        .map_or(Listed::Missing, Listed::Id)
}
