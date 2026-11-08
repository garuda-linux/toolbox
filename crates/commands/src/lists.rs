//! The `list` commands' rows: what the pages show, as plain values for a
//! table or `--json`. Read from the same snapshot, catalogs and core
//! functions as the pages, so both always agree.

#[cfg(test)]
mod tests;

pub mod listing;

use std::ops::RangeInclusive;

use serde::Serialize;
use toolbox_catalog::ToggleCatalog;
use toolbox_cli::args::TogglePage;
use toolbox_core::features::discover::Apps;
use toolbox_core::features::kernels::kernels_of;
use toolbox_core::features::locales::all_rows;
use toolbox_core::features::services::{FileState, UnitInfo};
use toolbox_core::features::toggles::shown::sections;
use toolbox_core::features::toggles::{EntryState, is_offered, state};
use toolbox_core::features::users::accounts;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_services::page::PageId;

/// `UID_MIN` to `UID_MAX` of `/etc/login.defs`, the bounds the window's
/// people filter uses too.
const PEOPLE: RangeInclusive<u32> = 1000..=60_000;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceRow {
    #[serde(rename = "id")]
    pub name: String,
    /// systemd's `ActiveState`, such as `active` or `failed`.
    pub active: &'static str,
    /// systemd's `SubState`, such as `running` or `exited`.
    #[serde(rename = "sub_state")]
    pub state: String,
    pub at_boot: bool,
    pub description: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToggleRow {
    pub page: &'static str,
    pub section: String,
    pub id: String,
    pub name: String,
    /// `on`, `off`, or `unknown` when the system part was not read. JSON
    /// has it as `on`: true, false, or null for unknown.
    #[serde(rename = "on", serialize_with = "on_or_null")]
    pub state: &'static str,
    /// Whether it can be switched now: its requirement holds.
    pub available: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct KernelRow {
    #[serde(rename = "id")]
    pub name: String,
    pub version: String,
    pub installed: bool,
    pub running: bool,
    pub headers: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LocaleRow {
    #[serde(rename = "id")]
    pub locale: String,
    pub charset: String,
    pub name: Option<String>,
    pub enabled: bool,
    pub system: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserRow {
    #[serde(rename = "id")]
    pub name: String,
    pub uid: u32,
    #[serde(rename = "name")]
    pub full_name: String,
    pub home: String,
    pub shell: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GroupRow {
    #[serde(rename = "id")]
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
    pub system: bool,
}

pub fn services(units: &[UnitInfo]) -> Vec<ServiceRow> {
    let mut rows: Vec<ServiceRow> = units
        .iter()
        .map(|unit| ServiceRow {
            name: unit.name.as_str().to_owned(),
            active: unit.active.as_str(),
            state: unit.sub_state.clone(),
            at_boot: matches!(unit.file_state, FileState::Enabled),
            description: unit.description.clone(),
        })
        .collect();

    rows.sort_by(|left, right| left.name.cmp(&right.name));
    rows
}

/// The entries each page offers on this system and desktop, in page order.
pub fn toggles(
    pages: &[(TogglePage, &ToggleCatalog)],
    snapshot: &Snapshot,
    desktops: &[String],
) -> Vec<ToggleRow> {
    let changes = ChangeSet::default();

    pages
        .iter()
        .flat_map(|(page, catalog)| {
            sections(catalog, snapshot, &changes, desktops)
                .into_iter()
                .flat_map(move |section| {
                    section.entries.into_iter().map(move |entry| ToggleRow {
                        page: page_id(*page),
                        section: section.section.id().to_owned(),
                        id: entry.entry.id().to_owned(),
                        name: entry.entry.title().to_owned(),
                        state: shown(entry.state),
                        available: entry.state.available,
                    })
                })
        })
        .collect()
}

/// The apps the system offers, of one category or all.
pub fn apps(apps: &Apps, category: Option<&str>, snapshot: &Snapshot) -> Vec<ToggleRow> {
    let changes = ChangeSet::default();

    apps.all()
        .filter(|hit| category.is_none_or(|id| hit.category.id() == id))
        .filter(|hit| is_offered(hit.entry, snapshot, &changes))
        .map(|hit| {
            let entry = state(hit.catalog, hit.entry, snapshot, &changes);
            ToggleRow {
                page: "apps",
                section: hit.category.id().to_owned(),
                id: hit.entry.id().to_owned(),
                name: hit.entry.title().to_owned(),
                state: shown(entry),
                available: entry.available,
            }
        })
        .collect()
}

/// `None` without the kernel packages: pacman's databases were not read.
pub fn kernels(snapshot: &Snapshot) -> Option<Vec<KernelRow>> {
    let (listed, booted) = kernels_of(snapshot)?;

    Some(
        listed
            .iter()
            .map(|kernel| KernelRow {
                name: kernel.name.as_str().to_owned(),
                version: kernel
                    .installed
                    .clone()
                    .unwrap_or_else(|| kernel.version.clone()),
                installed: kernel.installed.is_some(),
                running: booted.as_ref() == Some(&kernel.name),
                headers: kernel.headers_installed,
            })
            .collect(),
    )
}

pub fn locales(snapshot: &Snapshot) -> Vec<LocaleRow> {
    all_rows(snapshot, &ChangeSet::default())
        .into_iter()
        .map(|row| LocaleRow {
            locale: row.entry.locale.to_string(),
            charset: row.entry.charset.to_string(),
            name: row.name.map(|name| {
                if name.territory.is_empty() {
                    name.language
                } else {
                    format!("{} ({})", name.language, name.territory)
                }
            }),
            enabled: row.enabled,
            system: row.system,
        })
        .collect()
}

/// `None` without the accounts.
pub fn users(snapshot: &Snapshot) -> Option<Vec<UserRow>> {
    let found = accounts(snapshot).ok()?;

    Some(
        found
            .users
            .iter()
            .map(|user| UserRow {
                name: user.name.as_str().to_owned(),
                uid: user.uid,
                full_name: user.full_name().to_owned(),
                home: user.home.clone(),
                shell: user.shell.clone(),
            })
            .collect(),
    )
}

/// The people's accounts and root, unless `all`, and how many system
/// accounts stayed out.
pub fn people(rows: Vec<UserRow>, all: bool) -> (Vec<UserRow>, usize) {
    if all {
        return (rows, 0);
    }

    let total = rows.len();
    let shown: Vec<UserRow> = rows
        .into_iter()
        .filter(|row| row.uid == 0 || PEOPLE.contains(&row.uid))
        .collect();

    let hidden = total - shown.len();
    (shown, hidden)
}

/// `None` without the accounts.
pub fn groups(snapshot: &Snapshot) -> Option<Vec<GroupRow>> {
    let found = accounts(snapshot).ok()?;

    Some(
        found
            .groups
            .iter()
            .map(|group| GroupRow {
                name: group.name.as_str().to_owned(),
                gid: group.gid,
                members: group
                    .members
                    .iter()
                    .map(|member| member.as_str().to_owned())
                    .collect(),
                system: group.is_system(),
            })
            .collect(),
    )
}

/// The page's id, as the window's `--page` takes it.
fn page_id(page: TogglePage) -> &'static str {
    match page {
        TogglePage::Components => PageId::Components,
        TogglePage::Tweaks => PageId::SystemSettings,
        TogglePage::Gaming => PageId::Gaming,
        TogglePage::Setup => PageId::Setup,
    }
    .slug()
}

/// `on` and `off` as a bool, `unknown` as null.
fn on_or_null<S>(state: &&'static str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match *state {
        "on" => serializer.serialize_bool(true),
        "off" => serializer.serialize_bool(false),
        _ => serializer.serialize_none(),
    }
}

const fn shown(entry: EntryState) -> &'static str {
    match entry.system {
        None => "unknown",
        Some(Toggle::On) => "on",
        Some(Toggle::Off) => "off",
    }
}
