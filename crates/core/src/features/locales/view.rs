//! The locale list, the system language with its choices, and the
//! language packs.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::{LocaleEntry, LocaleName, PackageName};

use crate::domains::{
    CurrentSystemLocale, EnabledLocales, InstalledPackages, Packages, SyncPackages, SystemLocales,
};
use crate::features::Refused;
use crate::features::locales::{
    LanguageName, LanguagePack, LocaleNames, LocaleRow, PackRow, Refusal, SupportedLocales,
    all_rows, language_packs, system_choices as offered, wanted_enabled,
};
use crate::features::packages::queue;
use crate::{ChangeSet, Single, Snapshot, Toggle};

#[derive(Debug, PartialEq, Eq)]
pub struct LanguagePackRow {
    pub pack: String,
    pub base: String,
    pub locale_code: String,
    /// `None` where the system names no language for the locale.
    pub language: Option<String>,
    /// `None` where the language has no territory.
    pub territory: Option<String>,
    pub installed: bool,
    pub on: bool,
    pub queued: Option<Toggle>,
}

pub fn locales_known(snapshot: &Snapshot) -> bool {
    snapshot.part::<SupportedLocales>().is_some() && snapshot.part::<EnabledLocales>().is_some()
}

/// Every locale row, as `features::locales::all_rows` orders them. Empty
/// until the locales are known.
pub fn locales(snapshot: &Snapshot, changes: &ChangeSet) -> Vec<LocaleRow> {
    if !locales_known(snapshot) {
        return Vec::new();
    }

    all_rows(snapshot, changes)
}

pub fn system_current(snapshot: &Snapshot) -> Option<LocaleName> {
    snapshot.part::<CurrentSystemLocale>()?.0.clone()
}

pub fn system_queued(changes: &ChangeSet) -> Option<LocaleName> {
    changes.wish::<SystemLocales>(&Single).cloned()
}

/// The language the page shows: the queued one, else the current.
pub fn system_shown(snapshot: &Snapshot, changes: &ChangeSet) -> Option<LocaleName> {
    system_queued(changes).or_else(|| system_current(snapshot))
}

pub fn system_choices(snapshot: &Snapshot, changes: &ChangeSet) -> Vec<String> {
    offered(snapshot, changes)
        .iter()
        .map(LocaleName::to_string)
        .collect()
}

/// The language and territory of each system choice, in the same order,
/// `None` where the system names none.
pub fn system_choice_names(
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Vec<(Option<String>, Option<String>)> {
    let names = snapshot.part::<LocaleNames>();

    offered(snapshot, changes)
        .iter()
        .map(|locale| {
            let name = names.and_then(|names| names.of(locale));

            (
                language(name).map(str::to_owned),
                territory(name).map(str::to_owned),
            )
        })
        .collect()
}

/// Queues installing or removing the language pack `package`, or takes
/// that back. Returns whether it is one of the listed packs: nothing else
/// is queued from here.
///
/// # Errors
///
/// [`Refusal::Package`] for a removal of an installed pack the package
/// rules refuse, naming the package they name.
pub fn toggle_pack(
    package: &PackageName,
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<bool, Refused<Refusal>> {
    let listed = packs_of(snapshot, changes)
        .is_some_and(|packs| packs.iter().any(|pack| pack.package == *package));

    if !listed {
        return Ok(false);
    }

    let installed = snapshot
        .part::<InstalledPackages>()
        .is_some_and(|installed| installed.0.contains(package));

    if !wish.is_on() && installed {
        queue::may_remove(package, changes, snapshot).map_err(|refused| {
            Refused::new(
                Refusal::Package(refused.refusal),
                refused.subject.to_string(),
            )
        })?;
    }

    changes.want::<Packages>(package.clone(), wish, snapshot);
    Ok(true)
}

/// Queues every listed pack that is neither installed nor queued, the
/// ones [`packs_missing`] counts. Returns how many.
pub fn add_missing_packs(changes: &mut ChangeSet, snapshot: &Snapshot) -> usize {
    let missing: Vec<PackageName> = pack_rows(snapshot, changes)
        .into_iter()
        .filter(PackRow::is_missing)
        .map(|row| row.pack.package)
        .collect();

    for package in &missing {
        changes.want::<Packages>(package.clone(), Toggle::On, snapshot);
    }

    missing.len()
}

/// Drops the queued installs of language packs whose locale is no longer
/// wanted. Returns how many went, `None` without the package lists.
pub fn drop_stranded_packs(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let offered = snapshot.part::<SyncPackages>()?;
    let installed = snapshot.part::<InstalledPackages>()?;
    let every: BTreeSet<LocaleEntry> = super::known(snapshot).into_iter().collect();

    let kept: BTreeSet<PackageName> = packs_of(snapshot, changes)?
        .into_iter()
        .map(|pack| pack.package)
        .collect();

    let stranded: Vec<PackageName> = language_packs(&every, offered, installed)
        .into_iter()
        .map(|pack| pack.package)
        .filter(|package| !kept.contains(package))
        .filter(|package| changes.wish::<Packages>(package) == Some(&Toggle::On))
        .collect();

    for package in &stranded {
        changes.remove::<Packages>(package);
        tracing::info!("dropped the queued install of {package}: its locale is no longer wanted");
    }

    Some(stranded.len())
}

fn packs_of(snapshot: &Snapshot, changes: &ChangeSet) -> Option<Vec<LanguagePack>> {
    Some(language_packs(
        &wanted_enabled(snapshot, changes),
        snapshot.part::<SyncPackages>()?,
        snapshot.part::<InstalledPackages>()?,
    ))
}

pub fn packs_known(snapshot: &Snapshot, changes: &ChangeSet) -> bool {
    packs_of(snapshot, changes).is_some()
}

pub fn packs_missing(rows: &[PackRow]) -> usize {
    rows.iter().filter(|row| row.is_missing()).count()
}

pub fn packs(rows: &[PackRow]) -> Vec<LanguagePackRow> {
    rows.iter().map(pack_row).collect()
}

/// Every language pack with its state, sorted by package.
pub fn pack_rows(snapshot: &Snapshot, changes: &ChangeSet) -> Vec<PackRow> {
    let Some(packs) = packs_of(snapshot, changes) else {
        return Vec::new();
    };

    let installed = snapshot.part::<InstalledPackages>();
    let names = snapshot.part::<LocaleNames>();

    packs
        .into_iter()
        .map(|pack| PackRow {
            name: names.and_then(|names| names.of(&pack.locale)).cloned(),
            installed: installed.is_some_and(|installed| installed.0.contains(&pack.package)),
            queued: changes.wish::<Packages>(&pack.package).copied(),
            pack,
        })
        .collect()
}

fn pack_row(row: &PackRow) -> LanguagePackRow {
    LanguagePackRow {
        pack: row.pack.package.as_str().to_owned(),
        base: row.pack.base.as_str().to_owned(),
        locale_code: row.pack.locale.as_str().to_owned(),
        language: language(row.name.as_ref()).map(str::to_owned),
        territory: territory(row.name.as_ref()).map(str::to_owned),
        installed: row.installed,
        on: row.is_on(),
        queued: row.queued,
    }
}

fn language(name: Option<&LanguageName>) -> Option<&str> {
    name.map(|name| name.language.as_str())
        .filter(|text| !text.is_empty())
}

fn territory(name: Option<&LanguageName>) -> Option<&str> {
    name.map(|name| name.territory.as_str())
        .filter(|text| !text.is_empty())
}
