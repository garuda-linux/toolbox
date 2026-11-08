#[cfg(test)]
mod tests;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use toolbox_aur::{AurSource, Cache, Https, SystemClock};
use toolbox_cli::complete::Names;
use toolbox_config::Paths;

use super::cache;

/// The first download waits this long at most, since a shell waits on
/// it. A slower one counts as a failed attempt, so the next waits a day
/// as after any failure.
const DOWNLOAD_WAIT: Duration = Duration::from_mins(2);

/// The names of the copy, kept beside the other lists until the copy
/// changes. Reading the copy takes most of a second.
pub fn of() -> Vec<String> {
    let aur = copy();

    remembered(cache::dir().as_deref(), || aur.modified(), || names(&aur))
}

/// The copy in the cache folder, downloaded with the shorter wait when
/// there is none.
pub fn copy() -> Cache {
    Cache::new(
        Paths::from_process().aur_dir(),
        Arc::new(Https::with_timeout(DOWNLOAD_WAIT)),
        Arc::new(SystemClock),
    )
}

/// The list `dir` keeps for the copy's `stamp`, else `read`'s names. They
/// are kept only when the stamp is the same before and after, so a copy
/// that was replaced meanwhile never stands for the names of the old one.
pub fn remembered(
    dir: Option<&Path>,
    stamp: impl Fn() -> Option<u64>,
    read: impl FnOnce() -> Vec<String>,
) -> Vec<String> {
    let before = stamp();
    if let (Some(dir), Some(before)) = (dir, before)
        && let Some(found) = cache::read_stamped(dir, Names::AurPackages, before)
    {
        return found;
    }

    let found = read();
    if let (Some(dir), Some(after)) = (dir, stamp())
        && before == Some(after)
    {
        cache::write(dir, Names::AurPackages, after, &found);
    }

    found
}

/// Every package of the copy as it is. A copy a day old is not refreshed
/// for a Tab, `packages search --aur` and the window do that.
pub fn names(aur: &dyn AurSource) -> Vec<String> {
    aur.stored()
        .map(|index| {
            index
                .packages
                .iter()
                .map(|package| package.name.as_str().to_owned())
                .collect()
        })
        .unwrap_or_default()
}
