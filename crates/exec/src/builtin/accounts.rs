//! The helper's own look at `/etc/passwd` and `/etc/group` before it
//! removes or locks an account. The app guards the same things, but a
//! request on the wire is all the helper can trust.

#[cfg(test)]
mod tests;

use std::path::Path;
use std::{fs, io};

use toolbox_plan::procedure::{FIRST_USER_ID, GROUP, PASSWD};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{GroupName, HomeDirectory, StepId, UserName};

use crate::apply::EventSink;
use crate::report::{failure, note};

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
        Ok(Listed::Id(0)) => failure(
            step,
            sink,
            &format!("{user} is the root account, which stays as it is"),
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
/// -m` would write it into `/etc/passwd` and then refuse to move.
pub fn require_free_home(
    root: &Path,
    step: StepId,
    home: &HomeDirectory,
    sink: &mut dyn EventSink,
) -> Outcome {
    let path = root.join(home.as_str().trim_start_matches('/'));

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

pub fn require_user_group(
    root: &Path,
    step: StepId,
    group: &GroupName,
    sink: &mut dyn EventSink,
) -> Outcome {
    match group_id(root, group) {
        Ok(Listed::Id(id)) if id < FIRST_USER_ID => failure(
            step,
            sink,
            &format!("{group} is a system group (group id {id}), which stays"),
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
