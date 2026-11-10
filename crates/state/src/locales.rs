pub mod names;

use std::collections::BTreeSet;

pub use names::{LanguageName, LocaleNames, parse_identification, source_name};
use toolbox_plan::{LocaleEntry, LocaleName};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportedLocales(pub Vec<LocaleEntry>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnabledLocales(pub BTreeSet<LocaleEntry>);

/// `LANG` of `/etc/locale.conf`, `None` when the file sets none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CurrentSystemLocale(pub Option<LocaleName>);

impl crate::Part for CurrentSystemLocale {}
impl crate::Part for EnabledLocales {}
impl crate::Part for SupportedLocales {}

/// The entries of a `SUPPORTED` file. Arch ships one `locale charset` per
/// line, glibc's own file has `locale/charset \` lines below a variable
/// name. Both are read.
pub fn parse_supported(text: &str) -> Vec<LocaleEntry> {
    let mut seen = BTreeSet::new();

    text.lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches('\\').replace('/', " ");

            LocaleEntry::parse(&line)
        })
        .filter(|entry| seen.insert(entry.clone()))
        .collect()
}
