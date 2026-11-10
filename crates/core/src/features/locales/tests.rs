mod names;
mod packs;
mod restored;

use std::collections::BTreeSet;

use toolbox_plan::LocaleEntry;

use super::{
    Filter, LocaleRow, Refusal, SupportedLocales, all_rows, choose_system, parse_supported, pick,
    system_choices, toggle, wanted_enabled, wanted_system,
};
use crate::domains::{CurrentSystemLocale, EnabledLocales, Locales, SystemLocales};
use crate::test_support::{locale, locale_entry};
use crate::{ChangeSet, Single, Snapshot, Toggle};

/// `/usr/share/i18n/SUPPORTED` of the Lima VM, 502 lines.
const SUPPORTED: &str = toolbox_plan::testing::locales::SUPPORTED;

/// The languages the tests keep of [`SUPPORTED`].
const KEPT: [&str; 4] = ["de_DE", "en_GB", "en_US", "sr_RS"];

/// [`SUPPORTED`] cut to the lines of [`KEPT`].
pub(super) fn supported() -> String {
    SUPPORTED
        .lines()
        .filter(|line| KEPT.contains(&line.split(['.', '@', ' ']).next().unwrap_or_default()))
        .flat_map(|line| [line, "\n"])
        .collect()
}

fn rows(snapshot: &Snapshot, changes: &ChangeSet, filter: &Filter) -> Vec<LocaleRow> {
    let all = all_rows(snapshot, changes);

    pick(&all, filter)
        .into_iter()
        .map(|at| all[at].clone())
        .collect()
}

pub(super) fn entries(lines: &[&str]) -> BTreeSet<LocaleEntry> {
    lines.iter().map(|line| locale_entry(line)).collect()
}

fn snapshot(enabled: &[&str], system: Option<&str>) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported(&supported())));
    snapshot.insert(EnabledLocales(entries(enabled)));
    snapshot.insert(CurrentSystemLocale(system.map(locale)));
    snapshot
}

fn all() -> Filter {
    Filter {
        search: String::new(),
        utf8_only: false,
        enabled_only: false,
    }
}

fn names(snapshot: &Snapshot, changes: &ChangeSet, filter: &Filter) -> Vec<String> {
    rows(snapshot, changes, filter)
        .into_iter()
        .map(|row| row.entry.to_string())
        .collect()
}

#[test]
fn supported_is_read_in_arch_and_glibc_format() {
    let arch = parse_supported(SUPPORTED);

    assert_eq!(arch.len(), 502);
    assert_eq!(arch[0], locale_entry("aa_DJ.UTF-8 UTF-8"));
    assert!(arch.contains(&locale_entry("sr_RS@latin UTF-8")));
    assert_eq!(arch[501], locale_entry("zu_ZA ISO-8859-1"));

    // No capture has glibc's own format, which Arch rewrites when it
    // packages the file.
    let glibc = parse_supported(
        "# This file names the currently supported and somewhat tested locales.\nSUPPORTED-LOCALES=\\\nC.UTF-8/UTF-8 \\\naa_DJ.UTF-8/UTF-8 \\\naa_DJ/ISO-8859-1 \\\naa_DJ/ISO-8859-1 \\\n",
    );

    assert_eq!(
        glibc,
        [
            locale_entry("C.UTF-8 UTF-8"),
            locale_entry("aa_DJ.UTF-8 UTF-8"),
            locale_entry("aa_DJ ISO-8859-1"),
        ]
    );
}

#[test]
fn enabled_locales_come_first_and_other_charsets_only_when_asked() {
    let snapshot = snapshot(
        &["en_US.UTF-8 UTF-8", "de_DE ISO-8859-1"],
        Some("en_US.UTF-8"),
    );

    let changes = ChangeSet::default();

    assert_eq!(
        names(&snapshot, &changes, &all()),
        [
            "de_DE ISO-8859-1",
            "en_US.UTF-8 UTF-8",
            "de_DE.UTF-8 UTF-8",
            "de_DE@euro ISO-8859-15",
            "en_GB.UTF-8 UTF-8",
            "en_GB ISO-8859-1",
            "en_US ISO-8859-1",
            "sr_RS UTF-8",
            "sr_RS@latin UTF-8",
        ]
    );

    // An enabled locale shows whatever its character set.
    assert_eq!(
        names(&snapshot, &changes, &Filter::default()),
        [
            "de_DE ISO-8859-1",
            "en_US.UTF-8 UTF-8",
            "de_DE.UTF-8 UTF-8",
            "en_GB.UTF-8 UTF-8",
            "sr_RS UTF-8",
            "sr_RS@latin UTF-8",
        ]
    );
}

#[test]
fn the_search_matches_locale_and_charset_without_case() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], Some("en_US.UTF-8"));
    let changes = ChangeSet::default();
    let search = |text: &str| Filter {
        search: text.to_owned(),
        ..all()
    };

    assert_eq!(
        names(&snapshot, &changes, &search("de_de")),
        [
            "de_DE.UTF-8 UTF-8",
            "de_DE ISO-8859-1",
            "de_DE@euro ISO-8859-15"
        ]
    );
    assert_eq!(
        names(&snapshot, &changes, &search(" iso-8859-15 ")),
        ["de_DE@euro ISO-8859-15"]
    );
    assert_eq!(
        names(&snapshot, &changes, &search("klingon")),
        Vec::<String>::new()
    );
}

#[test]
fn enabled_only_keeps_what_is_on_or_queued() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], Some("en_US.UTF-8"));
    let mut changes = ChangeSet::default();
    toggle(
        &locale_entry("de_DE.UTF-8 UTF-8"),
        Toggle::On,
        &mut changes,
        &snapshot,
    )
    .unwrap();

    let filter = Filter {
        enabled_only: true,
        ..all()
    };

    let shown = rows(&snapshot, &changes, &filter);

    assert_eq!(shown.len(), 2);
    assert_eq!(shown[0].entry, locale_entry("en_US.UTF-8 UTF-8"));
    assert!(shown[0].enabled && shown[0].system && shown[0].queued.is_none());
    assert_eq!(shown[1].entry, locale_entry("de_DE.UTF-8 UTF-8"));
    assert!(!shown[1].enabled && !shown[1].system);
    assert_eq!(shown[1].queued, Some(Toggle::On));
}

#[test]
fn an_enabled_locale_missing_from_supported_is_still_listed() {
    let snapshot = snapshot(&["tlh_QO.UTF-8 UTF-8"], None);

    assert_eq!(
        names(&snapshot, &ChangeSet::default(), &all())[0],
        "tlh_QO.UTF-8 UTF-8"
    );
}

#[test]
fn a_click_queues_one_change_and_a_second_takes_it_back() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], Some("en_US.UTF-8"));
    let mut changes = ChangeSet::default();
    let german = locale_entry("de_DE.UTF-8 UTF-8");

    toggle(&german, Toggle::On, &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<Locales>(&german), Some(&Toggle::On));
    assert_eq!(
        wanted_enabled(&snapshot, &changes),
        entries(&["de_DE.UTF-8 UTF-8", "en_US.UTF-8 UTF-8"])
    );

    toggle(&german, Toggle::Off, &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn the_locale_of_the_system_language_cannot_be_disabled() {
    let snapshot = snapshot(
        &["en_US.UTF-8 UTF-8", "de_DE.UTF-8 UTF-8"],
        Some("en_US.utf8"),
    );

    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            &locale_entry("en_US.UTF-8 UTF-8"),
            Toggle::Off,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::SystemLocale)
    );
    assert!(changes.is_empty());

    // Once another language is queued, the current one is free to go.
    choose_system(locale("de_DE.UTF-8"), &mut changes, &snapshot).unwrap();
    toggle(
        &locale_entry("en_US.UTF-8 UTF-8"),
        Toggle::Off,
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(
        toggle(
            &locale_entry("de_DE.UTF-8 UTF-8"),
            Toggle::Off,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::SystemLocale)
    );
}

#[test]
fn at_least_one_locale_stays_enabled() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8", "de_DE.UTF-8 UTF-8"], None);
    let mut changes = ChangeSet::default();

    toggle(
        &locale_entry("en_US.UTF-8 UTF-8"),
        Toggle::Off,
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(
        toggle(
            &locale_entry("de_DE.UTF-8 UTF-8"),
            Toggle::Off,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::LastLocale)
    );
    assert_eq!(changes.len(), 1);
}

#[test]
fn the_system_language_is_chosen_among_enabled_or_queued_locales() {
    let snapshot = snapshot(&["en_US.UTF-8 UTF-8"], Some("en_US.UTF-8"));
    let mut changes = ChangeSet::default();

    assert_eq!(
        choose_system(locale("de_DE.UTF-8"), &mut changes, &snapshot),
        Err(Refusal::NotEnabled)
    );

    toggle(
        &locale_entry("de_DE.UTF-8 UTF-8"),
        Toggle::On,
        &mut changes,
        &snapshot,
    )
    .unwrap();
    assert_eq!(
        system_choices(&snapshot, &changes),
        [locale("de_DE.UTF-8"), locale("en_US.UTF-8")]
    );

    choose_system(locale("de_DE.UTF-8"), &mut changes, &snapshot).unwrap();
    assert_eq!(
        wanted_system(&snapshot, &changes),
        Some(locale("de_DE.UTF-8"))
    );
    assert_eq!(
        changes.wish::<SystemLocales>(&Single),
        Some(&locale("de_DE.UTF-8"))
    );

    // Taking the locale back would leave the queued language without it.
    assert_eq!(
        toggle(
            &locale_entry("de_DE.UTF-8 UTF-8"),
            Toggle::Off,
            &mut changes,
            &snapshot
        ),
        Err(Refusal::SystemLocale)
    );

    choose_system(locale("en_US.UTF-8"), &mut changes, &snapshot).unwrap();
    assert_eq!(changes.wish::<SystemLocales>(&Single), None);
    assert_eq!(Refusal::NotEnabled.as_str(), "not-enabled");
}

/// Where nothing is known about the locales (no `locale.gen`), a queued
/// one can still be taken back.
#[test]
fn a_queued_locale_can_be_taken_back_without_any_enabled_one() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    let german = locale_entry("de_DE.UTF-8 UTF-8");

    toggle(&german, Toggle::On, &mut changes, &snapshot).unwrap();
    assert_eq!(changes.len(), 1);

    toggle(&german, Toggle::Off, &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}
