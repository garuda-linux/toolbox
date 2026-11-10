use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime};

use tempfile::TempDir;
use toolbox_plan::{LocaleEntry, LocaleName};
use toolbox_state::locales::{CurrentSystemLocale, EnabledLocales};

use super::{EtcLocales, LocaleSource as _};
use crate::ProbeError;

const SUPPORTED: &str = toolbox_plan::testing::locales::SUPPORTED;
/// Written by cloud-init in the Lima VM, with `en_US.UTF-8` only.
const LOCALE_GEN: &str = toolbox_plan::testing::locales::LOCALE_GEN;
const LOCALE_CONF: &str = toolbox_plan::testing::locales::LOCALE_CONF;

const SOURCES: [(&str, &[u8]); 5] = [
    (
        "ca_ES@valencia",
        include_bytes!("fixtures/locales/ca_ES@valencia"),
    ),
    ("de_DE", crate::testing::locales::LOCALES_DE_DE.as_bytes()),
    ("eo", crate::testing::locales::LOCALES_EO.as_bytes()),
    ("ja_JP", include_bytes!("fixtures/locales/ja_JP")),
    (
        "sr_RS@latin",
        include_bytes!("fixtures/locales/sr_RS@latin"),
    ),
];

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

fn source(root: &TempDir, name: &str, text: &[u8]) {
    let sources = root.path().join("usr/share/i18n/locales");

    fs::create_dir_all(&sources).unwrap();
    fs::write(sources.join(name), text).unwrap();
}

fn captured(name: &str) -> &'static [u8] {
    SOURCES
        .iter()
        .find(|(source, _)| *source == name)
        .map(|(_, text)| *text)
        .unwrap()
}

/// The VM's list without the lines of one locale.
fn supported_without(prefix: &str) -> String {
    SUPPORTED
        .lines()
        .filter(|line| !line.starts_with(prefix))
        .flat_map(|line| [line, "\n"])
        .collect()
}

#[test]
fn supported_locales_keep_the_order_of_the_file() {
    let root = system(Some(SUPPORTED), None, None);
    let supported = EtcLocales::new(root.path()).supported().unwrap().0;

    assert_eq!(supported.len(), SUPPORTED.lines().count());
    assert_eq!(
        supported[..3],
        [
            entry("aa_DJ.UTF-8 UTF-8"),
            entry("aa_DJ ISO-8859-1"),
            entry("aa_ER UTF-8"),
        ]
    );
    assert_eq!(supported[110], entry("de_DE.UTF-8 UTF-8"));
    assert_eq!(supported[111], entry("de_DE ISO-8859-1"));
}

#[test]
fn a_system_without_the_list_has_no_locales_page() {
    let root = system(None, None, Some(LOCALE_CONF));
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

/// The VM's `locale.gen` with a commented and an enabled line appended.
#[test]
fn enabled_locales_are_the_uncommented_lines() {
    let locale_gen = format!("{LOCALE_GEN}#de_DE.UTF-8 UTF-8\nsr_RS@latin UTF-8\n");
    let root = system(Some(SUPPORTED), Some(&locale_gen), None);
    let unchanged = system(Some(SUPPORTED), Some(LOCALE_GEN), None);

    assert_eq!(
        EtcLocales::new(root.path()).enabled(),
        Ok(EnabledLocales(BTreeSet::from([
            entry("en_US.UTF-8 UTF-8"),
            entry("sr_RS@latin UTF-8"),
        ])))
    );
    assert_eq!(
        EtcLocales::new(unchanged.path()).enabled(),
        Ok(EnabledLocales(BTreeSet::from([entry("en_US.UTF-8 UTF-8")])))
    );
}

#[test]
fn a_missing_locale_gen_beside_the_list_enables_nothing() {
    let root = system(Some(SUPPORTED), None, None);
    let locales = EtcLocales::new(root.path());

    assert_eq!(locales.enabled(), Ok(EnabledLocales::default()));
    assert_eq!(locales.system_locale(), Ok(CurrentSystemLocale(None)));
}

fn shown(locales: &EtcLocales) -> Vec<(String, String, String)> {
    locales
        .names()
        .0
        .iter()
        .map(|(source, name)| {
            (
                source.clone(),
                name.language.clone(),
                name.territory.clone(),
            )
        })
        .collect()
}

fn named(source: &str, language: &str, territory: &str) -> (String, String, String) {
    (source.to_owned(), language.to_owned(), territory.to_owned())
}

/// Most locales of the list have no captured source and so no name.
#[test]
fn locales_are_named_by_the_sources_glibc_ships() {
    let root = system(Some(SUPPORTED), None, None);

    for (name, text) in SOURCES {
        source(&root, name, text);
    }

    assert_eq!(
        shown(&EtcLocales::new(root.path())),
        [
            named("ca_ES@valencia", "Catalan", "Spain"),
            named("de_DE", "German", "Germany"),
            named("eo", "Esperanto", ""),
            named("ja_JP", "Japanese", "Japan"),
            named("sr_RS@latin", "Serbian", "Serbia"),
        ]
    );
}

/// The list without its `ja_JP` lines, and the `eo` source without its
/// `language` line.
#[test]
fn only_supported_locales_with_a_language_are_named() {
    let root = system(Some(&supported_without("ja_JP")), None, None);
    let esperanto =
        String::from_utf8_lossy(captured("eo")).replacen("language   \"Esperanto\"", "", 1);

    for (name, text) in SOURCES {
        source(&root, name, text);
    }

    source(&root, "eo", esperanto.as_bytes());

    assert_eq!(
        shown(&EtcLocales::new(root.path())),
        [
            named("ca_ES@valencia", "Catalan", "Spain"),
            named("de_DE", "German", "Germany"),
            named("sr_RS@latin", "Serbian", "Serbia"),
        ]
    );
}

/// The `ja_JP` source with a megabyte of bytes that are no UTF-8
/// appended, and the `sr_RS@latin` source behind 16 KiB of comment.
#[test]
fn only_the_head_of_a_source_is_read_and_odd_bytes_do_not_matter() {
    let root = system(Some(SUPPORTED), None, None);
    let mut japanese = captured("ja_JP").to_vec();

    japanese.extend(std::iter::repeat_n(0xFF_u8, 1 << 20));
    source(&root, "ja_JP", &japanese);

    let mut late = vec![b'%'; 16 * 1024];

    late.extend_from_slice(captured("sr_RS@latin"));
    source(&root, "sr_RS@latin", &late);

    let names = EtcLocales::new(root.path()).names();

    assert_eq!(names.0.len(), 1);
    assert_eq!(
        names.0.get("ja_JP").map(|name| name.language.as_str()),
        Some("Japanese")
    );
}

#[test]
fn a_system_without_the_sources_or_the_list_has_no_names() {
    let listed = system(Some(SUPPORTED), None, None);
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
/// glibc does: it replaces the files. The `de_DE` source is rewritten
/// with `Deutsch` for its language, and the list first lacks `ja_JP`.
#[test]
fn the_names_are_read_again_only_after_the_sources_or_the_list_changed() {
    let root = system(Some(&supported_without("ja_JP")), None, None);
    let folder = root.path().join("usr/share/i18n/locales");
    let list = root.path().join("usr/share/i18n/SUPPORTED");
    let deutsch = String::from_utf8_lossy(captured("de_DE")).replacen(
        "language   \"German\"",
        "language   \"Deutsch\"",
        1,
    );

    source(&root, "de_DE", captured("de_DE"));

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

    fs::write(folder.join("de_DE"), deutsch.as_bytes()).unwrap();
    stamp(&folder, 1000);
    assert_eq!(
        language(&locales, "de_DE").as_deref(),
        Some("German"),
        "the sources were read again"
    );

    stamp(&folder, 2000);
    assert_eq!(language(&locales, "de_DE").as_deref(), Some("Deutsch"));

    source(&root, "ja_JP", captured("ja_JP"));
    stamp(&folder, 2000);
    fs::write(&list, SUPPORTED).unwrap();
    stamp(&list, 3000);
    assert_eq!(language(&locales, "ja_JP").as_deref(), Some("Japanese"));
}
