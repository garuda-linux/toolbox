use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime};

use tempfile::TempDir;
use toolbox_plan::{LocaleEntry, LocaleName};
use toolbox_state::locales::{CurrentSystemLocale, EnabledLocales};

use super::{EtcLocales, LocaleSource as _};
use crate::ProbeError;

fn system(supported: Option<&str>, locale_gen: Option<&str>, locale_conf: Option<&str>) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");
    let i18n = root.path().join("usr/share/i18n");

    fs::create_dir_all(&etc).unwrap();
    fs::create_dir_all(&i18n).unwrap();

    if let Some(text) = supported {
        fs::write(i18n.join("SUPPORTED"), text).unwrap();
    }

    if let Some(text) = locale_gen {
        fs::write(etc.join("locale.gen"), text).unwrap();
    }

    if let Some(text) = locale_conf {
        fs::write(etc.join("locale.conf"), text).unwrap();
    }

    root
}

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

#[test]
fn supported_locales_keep_the_order_of_the_file() {
    let root = system(
        Some("de_DE.UTF-8 UTF-8  \nde_DE ISO-8859-1  \naa_DJ.UTF-8 UTF-8  \n"),
        None,
        None,
    );

    assert_eq!(
        EtcLocales::new(root.path()).supported().unwrap().0,
        [
            entry("de_DE.UTF-8 UTF-8"),
            entry("de_DE ISO-8859-1"),
            entry("aa_DJ.UTF-8 UTF-8"),
        ]
    );
}

#[test]
fn a_system_without_the_list_has_no_locales_page() {
    let root = system(None, None, Some("LANG=en_US.UTF-8\n"));
    let locales = EtcLocales::new(root.path());

    assert_eq!(locales.supported(), Err(ProbeError::Unavailable));
    assert_eq!(locales.enabled(), Err(ProbeError::Unavailable));
    assert_eq!(
        locales.system_locale(),
        Ok(CurrentSystemLocale(Some(
            LocaleName::new("en_US.UTF-8").unwrap()
        )))
    );
}

#[test]
fn enabled_locales_are_the_uncommented_lines() {
    let root = system(
        Some("de_DE.UTF-8 UTF-8\n"),
        Some("#de_DE.UTF-8 UTF-8\nen_US.UTF-8 UTF-8\nsr_RS@latin UTF-8\n"),
        None,
    );

    assert_eq!(
        EtcLocales::new(root.path()).enabled(),
        Ok(EnabledLocales(BTreeSet::from([
            entry("en_US.UTF-8 UTF-8"),
            entry("sr_RS@latin UTF-8"),
        ])))
    );
}

#[test]
fn a_missing_locale_gen_beside_the_list_enables_nothing() {
    let root = system(Some("de_DE.UTF-8 UTF-8\n"), None, None);
    let locales = EtcLocales::new(root.path());

    assert_eq!(locales.enabled(), Ok(EnabledLocales::default()));
    assert_eq!(locales.system_locale(), Ok(CurrentSystemLocale(None)));
}

fn source(root: &TempDir, name: &str, text: &[u8]) {
    let sources = root.path().join("usr/share/i18n/locales");

    fs::create_dir_all(&sources).unwrap();
    fs::write(sources.join(name), text).unwrap();
}

fn identification(language: &str, territory: &str) -> String {
    format!(
        "comment_char %\nLC_IDENTIFICATION\ntitle \"x\"\nlanguage   \"{language}\"\nterritory  \"{territory}\"\nEND LC_IDENTIFICATION\n"
    )
}

#[test]
fn locales_are_named_by_the_sources_glibc_ships() {
    let root = system(
        Some(
            "de_DE.UTF-8 UTF-8\nde_DE ISO-8859-1\nca_ES.UTF-8@valencia UTF-8\neo UTF-8\nxx_XX UTF-8\nyy_YY UTF-8\n",
        ),
        None,
        None,
    );

    source(
        &root,
        "de_DE",
        identification("German", "Germany").as_bytes(),
    );
    source(
        &root,
        "ca_ES@valencia",
        identification("Catalan", "Spain").as_bytes(),
    );
    source(&root, "eo", identification("Esperanto", "").as_bytes());
    source(&root, "yy_YY", identification("", "").as_bytes());
    source(
        &root,
        "fr_FR",
        identification("French", "France").as_bytes(),
    );

    let names = EtcLocales::new(root.path()).names();
    let shown: Vec<(&str, &str, &str)> = names
        .0
        .iter()
        .map(|(source, name)| {
            (
                source.as_str(),
                name.language.as_str(),
                name.territory.as_str(),
            )
        })
        .collect();

    assert_eq!(
        shown,
        [
            ("ca_ES@valencia", "Catalan", "Spain"),
            ("de_DE", "German", "Germany"),
            ("eo", "Esperanto", ""),
        ],
        "only supported locales, only sources that name a language"
    );
}

#[test]
fn only_the_head_of_a_source_is_read_and_odd_bytes_do_not_matter() {
    let root = system(Some("ja_JP.UTF-8 UTF-8\nzz_ZZ UTF-8\n"), None, None);
    let mut japanese = identification("Japanese", "Japan").into_bytes();

    japanese.extend(std::iter::repeat_n(0xFF_u8, 1 << 20));
    source(&root, "ja_JP", &japanese);

    let mut late = vec![b'%'; 16 * 1024];

    late.extend(identification("Late", "Nowhere").as_bytes());
    source(&root, "zz_ZZ", &late);

    let names = EtcLocales::new(root.path()).names();

    assert_eq!(names.0.len(), 1);
    assert_eq!(
        names.0.get("ja_JP").map(|name| name.language.as_str()),
        Some("Japanese")
    );
}

#[test]
fn a_system_without_the_sources_or_the_list_has_no_names() {
    let listed = system(Some("de_DE.UTF-8 UTF-8\n"), None, None);
    let bare = system(None, None, None);

    assert!(EtcLocales::new(listed.path()).names().0.is_empty());
    assert!(EtcLocales::new(bare.path()).names().0.is_empty());
}

fn language(locales: &EtcLocales, source: &str) -> Option<String> {
    locales
        .names()
        .0
        .get(source)
        .map(|name| name.language.clone())
}

/// A write into a file leaves the folder's time alone, as no update of
/// glibc does: it replaces the files.
#[test]
fn the_names_are_read_again_only_after_the_sources_or_the_list_changed() {
    let root = system(Some("de_DE.UTF-8 UTF-8\n"), None, None);
    let folder = root.path().join("usr/share/i18n/locales");
    let list = root.path().join("usr/share/i18n/SUPPORTED");

    source(
        &root,
        "de_DE",
        identification("German", "Germany").as_bytes(),
    );

    let locales = EtcLocales::new(root.path());
    let stamp = |path: &Path, seconds: u64| {
        File::open(path)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
            .unwrap();
    };

    stamp(&folder, 1000);
    stamp(&list, 1000);
    assert_eq!(language(&locales, "de_DE").as_deref(), Some("German"));

    fs::write(
        folder.join("de_DE"),
        identification("Deutsch", "Germany").as_bytes(),
    )
    .unwrap();
    stamp(&folder, 1000);
    assert_eq!(
        language(&locales, "de_DE").as_deref(),
        Some("German"),
        "the sources were read again"
    );

    stamp(&folder, 2000);
    assert_eq!(language(&locales, "de_DE").as_deref(), Some("Deutsch"));

    source(
        &root,
        "fr_FR",
        identification("French", "France").as_bytes(),
    );
    stamp(&folder, 2000);
    fs::write(&list, "de_DE.UTF-8 UTF-8\nfr_FR.UTF-8 UTF-8\n").unwrap();
    stamp(&list, 3000);
    assert_eq!(language(&locales, "fr_FR").as_deref(), Some("French"));
}
