//! The unit lists of systemd's manager (`org.freedesktop.systemd1`), for the
//! Services page. The app makes the calls with async zbus and hands the
//! rows to [`unit_infos`], so this crate stays free of a runtime.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use toolbox_plan::UnitName;
use toolbox_state::services::{ActiveState, FileState, UnitInfo};
use zbus::OwnedObjectPath;

use crate::bus::Method;

pub const SYSTEMD_DESTINATION: &str = "org.freedesktop.systemd1";
const MANAGER_PATH: &str = "/org/freedesktop/systemd1";
const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";

/// Loaded units, answered as `a(ssssssouso)`.
pub const LIST_UNITS: Method = manager("ListUnits");

/// Every installed unit file with its state, answered as `a(ss)`.
pub const LIST_UNIT_FILES: Method = manager("ListUnitFiles");

/// The manager sends its signals only to subscribed clients.
pub const SUBSCRIBE: Method = manager("Subscribe");

/// Services and sockets, the units a user starts and stops.
const LISTED_TYPES: [&str; 2] = [".service", ".socket"];

/// systemd's "loaded" state for a unit only referenced, without a file.
const NOT_FOUND: &str = "not-found";

/// A file that only names another unit, which is listed under its own name.
const ALIAS: &str = "alias";

/// The sub state systemd reports for a unit that is not loaded.
const DEAD: &str = "dead";

/// `(name, description, load_state, active_state, sub_state, following,
/// unit_path, job_id, job_type, job_path)`.
pub type ListUnitsRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    OwnedObjectPath,
    u32,
    String,
    OwnedObjectPath,
);

/// `(path, state)`.
pub type UnitFileRow = (String, String);

/// The page's rows: services and sockets that can be acted on, loaded ones
/// with their states, others from their files only. Sorted by name.
pub fn unit_infos(units: Vec<ListUnitsRow>, files: Vec<UnitFileRow>) -> Vec<UnitInfo> {
    let file_states = file_states(files);
    let mut infos: BTreeMap<UnitName, UnitInfo> = BTreeMap::new();

    for (name, description, load, active, sub, _following, _path, _job, _job_type, _job_path) in
        units
    {
        let Some(name) = listed(&name).filter(|_| load != NOT_FOUND) else {
            continue;
        };

        if let Entry::Vacant(entry) = infos.entry(name.clone()) {
            entry.insert(UnitInfo {
                file_state: file_states
                    .get(name.as_str())
                    .map_or(FileState::Unknown, |state| FileState::parse(state)),
                name,
                description,
                active: ActiveState::parse(&active),
                sub_state: sub,
            });
        }
    }

    for (file, state) in &file_states {
        let Some(name) = listed(file).filter(|_| state != ALIAS) else {
            continue;
        };

        infos.entry(name.clone()).or_insert_with(|| UnitInfo {
            name,
            description: String::new(),
            active: ActiveState::Inactive,
            sub_state: DEAD.to_owned(),
            file_state: FileState::parse(state),
        });
    }

    infos.into_values().collect()
}

const fn manager(name: &'static str) -> Method {
    Method {
        destination: SYSTEMD_DESTINATION,
        path: MANAGER_PATH,
        interface: MANAGER_INTERFACE,
        name,
    }
}

/// File state by file name, from the path of highest precedence. Equal
/// ranks keep the first answer.
fn file_states(files: Vec<UnitFileRow>) -> BTreeMap<String, String> {
    let mut states: BTreeMap<String, (u8, String)> = BTreeMap::new();

    for (path, state) in files {
        let name = path.rsplit('/').next().unwrap_or_default().to_owned();
        let rank = precedence(&path);

        match states.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert((rank, state));
            }
            Entry::Occupied(mut entry) => {
                if rank < entry.get().0 {
                    entry.insert((rank, state));
                }
            }
        }
    }

    states
        .into_iter()
        .map(|(name, (_, state))| (name, state))
        .collect()
}

/// systemd reads unit files from /etc before /run before /usr. Lower
/// wins.
fn precedence(path: &str) -> u8 {
    if path.starts_with("/etc/") {
        0
    } else if path.starts_with("/run/") {
        1
    } else if path.starts_with("/usr/") {
        2
    } else {
        3
    }
}

/// The unit, when the page lists it: a valid name of a listed type and
/// not a template, which cannot start without an instance.
fn listed(name: &str) -> Option<UnitName> {
    let stem = LISTED_TYPES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))?;

    if stem.ends_with('@') {
        return None;
    }

    UnitName::new(name)
        .map_err(|error| tracing::debug!("skipped: {error}"))
        .ok()
}
