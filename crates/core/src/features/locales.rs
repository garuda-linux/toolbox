mod packs;
#[cfg(test)]
mod tests;
pub mod view;

use std::collections::BTreeSet;
use std::iter;

pub use packs::{LanguagePack, PackOrder, PackRow, language_packs, pack_order};
use toolbox_plan::{LocaleEntry, LocaleName, is_builtin_locale};
pub use toolbox_state::locales::{
    LanguageName, LocaleNames, SupportedLocales, parse_identification, parse_supported, source_name,
};

use crate::domains::{CurrentSystemLocale, EnabledLocales, Locales, SystemLocales};
use crate::features::drop_refused;
use crate::features::search::{Needle, Searchable};
use crate::{ChangeSet, Single, Snapshot, Toggle};

const UTF8: &str = "UTF-8";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub search: String,
    pub utf8_only: bool,
    pub enabled_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleRow {
    pub entry: LocaleEntry,
    /// The entry as `/etc/locale.gen` writes it, made once with the row
    /// for the search.
    pub text: String,
    pub enabled: bool,
    pub queued: Option<Toggle>,
    pub name: Option<LanguageName>,
    /// It provides the system language, so it cannot be switched off.
    pub system: bool,
}

impl LocaleRow {
    pub fn is_on(&self) -> bool {
        self.queued.map_or(self.enabled, Toggle::is_on)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// The locale of the system language stays enabled.
    SystemLocale,
    /// At least one locale stays enabled.
    LastLocale,
    /// Only an enabled locale can become the system language.
    NotEnabled,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            search: String::new(),
            utf8_only: true,
            enabled_only: false,
        }
    }
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Searchable for LocaleRow {
    fn texts(&self) -> impl Iterator<Item = &str> {
        let names = self
            .name
            .as_ref()
            .map(|name| [name.language.as_str(), name.territory.as_str()]);

        iter::once(self.text.as_str()).chain(names.into_iter().flatten())
    }
}

pub fn wanted_enabled(snapshot: &Snapshot, changes: &ChangeSet) -> BTreeSet<LocaleEntry> {
    known(snapshot)
        .into_iter()
        .filter(|entry| wanted_on(entry, snapshot, changes))
        .collect()
}

pub fn wanted_system(snapshot: &Snapshot, changes: &ChangeSet) -> Option<LocaleName> {
    changes
        .wish::<SystemLocales>(&Single)
        .cloned()
        .or_else(|| snapshot.part::<CurrentSystemLocale>()?.0.clone())
}

/// Every row, enabled locales first, then the rest, each group in the
/// order of `SUPPORTED`. A page keeps these and lets [`pick`] choose.
pub fn all_rows(snapshot: &Snapshot, changes: &ChangeSet) -> Vec<LocaleRow> {
    let system = wanted_system(snapshot, changes);
    let names = snapshot.part::<LocaleNames>();

    let mut rows: Vec<LocaleRow> = known(snapshot)
        .into_iter()
        .map(|entry| LocaleRow {
            enabled: is_enabled(&entry, snapshot),
            queued: changes.wish::<Locales>(&entry).copied(),
            system: system.as_ref().is_some_and(|lang| entry.provides(lang)),
            name: names.and_then(|names| names.of(&entry.locale)).cloned(),
            text: entry.to_string(),
            entry,
        })
        .collect();

    rows.sort_by_key(|row| !row.enabled);
    rows
}

pub fn pick(rows: &[LocaleRow], filter: &Filter) -> Vec<usize> {
    let needle = Needle::new(&filter.search);

    rows.iter()
        .enumerate()
        .filter(|(_, row)| lets_through(row, filter, &needle))
        .map(|(at, _)| at)
        .collect()
}

fn lets_through(row: &LocaleRow, filter: &Filter, needle: &Needle) -> bool {
    let on = row.is_on();

    (!filter.utf8_only || row.entry.charset.as_str() == UTF8 || row.enabled || on)
        && (!filter.enabled_only || row.enabled || on)
        && needle.finds(row)
}

/// # Errors
///
/// [`Refusal`] when switching it off would leave the system language
/// without its locale, or no locale at all.
pub fn toggle(
    entry: &LocaleEntry,
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let on = wish.is_on();
    if !on {
        if let Some(refusal) = refuses_off(entry, snapshot, changes) {
            return Err(refusal);
        }

        // Taking back a locale that is only queued removes nothing.
        if is_enabled(entry, snapshot) {
            changes.want::<Locales>(entry.clone(), Toggle::Off, snapshot);
        } else {
            changes.remove::<Locales>(entry);
        }

        // Its language packs would install for a locale that goes.
        view::drop_stranded_packs(changes, snapshot);
        return Ok(());
    }

    changes.want::<Locales>(entry.clone(), Toggle::On, snapshot);
    Ok(())
}

/// `None` without `/etc/locale.gen` or the system language, as nothing can be checked then.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    snapshot.part::<EnabledLocales>()?;
    snapshot.part::<CurrentSystemLocale>()?;

    Some(drop_refused::<Locales>(
        changes,
        "switch-off of",
        |entry, wish, changes| {
            (!wish.is_on() && refuses_off(entry, snapshot, changes).is_some())
                .then_some("the page refuses it")
        },
    ))
}

fn refuses_off(entry: &LocaleEntry, snapshot: &Snapshot, changes: &ChangeSet) -> Option<Refusal> {
    let mut left = wanted_enabled(snapshot, changes);

    left.remove(entry);

    if wanted_system(snapshot, changes)
        .is_some_and(|lang| entry.provides(&lang) && !provided(&lang, &left))
    {
        return Some(Refusal::SystemLocale);
    }

    (is_enabled(entry, snapshot) && left.is_empty()).then_some(Refusal::LastLocale)
}

/// # Errors
///
/// [`Refusal::NotEnabled`] unless an enabled locale (or one queued to be
/// enabled) provides it.
pub fn choose_system(
    locale: LocaleName,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    if !provided(&locale, &wanted_enabled(snapshot, changes)) {
        return Err(Refusal::NotEnabled);
    }

    changes.want::<SystemLocales>(Single, locale, snapshot);
    Ok(())
}

pub fn system_choices(snapshot: &Snapshot, changes: &ChangeSet) -> Vec<LocaleName> {
    let names: BTreeSet<LocaleName> = wanted_enabled(snapshot, changes)
        .into_iter()
        .map(|entry| entry.locale)
        .collect();

    names.into_iter().collect()
}

fn provided(lang: &LocaleName, enabled: &BTreeSet<LocaleEntry>) -> bool {
    is_builtin_locale(lang) || enabled.iter().any(|entry| entry.provides(lang))
}

fn wanted_on(entry: &LocaleEntry, snapshot: &Snapshot, changes: &ChangeSet) -> bool {
    changes
        .wish::<Locales>(entry)
        .map_or_else(|| is_enabled(entry, snapshot), |wish| wish.is_on())
}

fn is_enabled(entry: &LocaleEntry, snapshot: &Snapshot) -> bool {
    snapshot
        .part::<EnabledLocales>()
        .is_some_and(|enabled| enabled.0.contains(entry))
}

fn known(snapshot: &Snapshot) -> Vec<LocaleEntry> {
    let mut entries = snapshot
        .part::<SupportedLocales>()
        .map(|supported| supported.0.clone())
        .unwrap_or_default();

    if let Some(enabled) = snapshot.part::<EnabledLocales>() {
        let listed: BTreeSet<&LocaleEntry> = entries.iter().collect();
        let extra: Vec<LocaleEntry> = enabled
            .0
            .iter()
            .filter(|entry| !listed.contains(entry))
            .cloned()
            .collect();

        entries.extend(extra);
    }

    entries
}
