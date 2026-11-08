//! Language packs: packages named after a program plus a language, such
//! as `firefox-i18n-de`, `libreoffice-fresh-pt-br` or `hunspell-en_us`.
//! A pack shows when a locale of its language is enabled and its program
//! is installed. The language ending is cut from the end of the name, as
//! `firefox-developer-edition-i18n-de` holds `-de` in its middle too, and
//! a program named without `-i18n`, `-l10n`, `-help`, `-spell` or `-lang`
//! still counts, such as `firefox`.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::{LocaleEntry, LocaleName, PackageName};
use toolbox_state::locales::LanguageName;

use crate::Toggle;
use crate::domains::{InstalledPackages, SyncPackages};

const INFIXES: [&str; 5] = ["-i18n", "-l10n", "-help", "-spell", "-lang"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePack {
    pub package: PackageName,
    pub base: PackageName,
    /// The first enabled locale the pack fits.
    pub locale: LocaleName,
}

/// What the list of packs is ordered by. Ties fall back to the package.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackOrder {
    #[default]
    Package,
    Program,
    /// The language name, the locale code where glibc has no name.
    Language,
    Selected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackRow {
    pub pack: LanguagePack,
    pub name: Option<LanguageName>,
    pub installed: bool,
    pub queued: Option<Toggle>,
}

impl PackRow {
    pub fn is_on(&self) -> bool {
        self.queued.map_or(self.installed, Toggle::is_on)
    }

    /// Neither installed nor queued: what Queue All Missing queues.
    pub const fn is_missing(&self) -> bool {
        !self.installed && self.queued.is_none()
    }

    fn language(&self) -> String {
        self.name.as_ref().map_or_else(
            || self.pack.locale.as_str().to_lowercase(),
            |name| name.language.to_lowercase(),
        )
    }
}

/// The packs for `locales`, sorted by package. A locale without a
/// territory (`C`, `eo`) has none.
pub fn language_packs(
    locales: &BTreeSet<LocaleEntry>,
    offered: &SyncPackages,
    installed: &InstalledPackages,
) -> Vec<LanguagePack> {
    let mut packs: BTreeMap<PackageName, LanguagePack> = BTreeMap::new();
    let lowered: Vec<(&PackageName, String)> = offered
        .0
        .iter()
        .map(|package| (package, package.as_str().to_ascii_lowercase()))
        .collect();

    for entry in locales {
        let Some(endings) = endings(&entry.locale) else {
            continue;
        };

        for &(package, ref name) in &lowered {
            if packs.contains_key(package) {
                continue;
            }

            let base = endings
                .iter()
                .filter_map(|ending| name.strip_suffix(ending.as_str()))
                .find_map(|stem| base(stem, installed));

            if let Some(base) = base {
                packs.insert(
                    package.clone(),
                    LanguagePack {
                        package: package.clone(),
                        base,
                        locale: entry.locale.clone(),
                    },
                );
            }
        }
    }

    packs.into_values().collect()
}

/// The name endings of a locale's packs, the territory forms first:
/// `-de_de`, `-de-de`, `-de`. Package names are compared in lower case.
fn endings(locale: &LocaleName) -> Option<[String; 3]> {
    let name = locale.as_str().to_ascii_lowercase();
    let plain = name.split(['.', '@']).next().unwrap_or_default();
    let (language, territory) = plain.split_once('_')?;

    Some([
        format!("-{language}_{territory}"),
        format!("-{language}-{territory}"),
        format!("-{language}"),
    ])
}

fn base(stem: &str, installed: &InstalledPackages) -> Option<PackageName> {
    let shorter = INFIXES
        .iter()
        .find_map(|infix| stem.strip_suffix(infix))
        .into_iter();

    std::iter::once(stem)
        .chain(shorter)
        .filter_map(|candidate| PackageName::new(candidate).ok())
        .find(|candidate| installed.0.contains(candidate))
}

/// The places in `rows` in `order`, for a page that keeps the rows and
/// only shows them in another order.
pub fn pack_order(rows: &[PackRow], order: PackOrder) -> Vec<usize> {
    let mut places: Vec<(usize, &PackRow)> = rows.iter().enumerate().collect();

    places.sort_by(|(_, left), (_, right)| compare(left, right, order));
    places.into_iter().map(|(at, _)| at).collect()
}

fn compare(left: &PackRow, right: &PackRow, order: PackOrder) -> Ordering {
    let first = match order {
        PackOrder::Package => Ordering::Equal,
        PackOrder::Program => left.pack.base.cmp(&right.pack.base),
        PackOrder::Language => left.language().cmp(&right.language()),
        PackOrder::Selected => right.is_on().cmp(&left.is_on()),
    };

    first.then_with(|| left.pack.package.cmp(&right.pack.package))
}
