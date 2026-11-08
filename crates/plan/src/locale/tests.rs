use std::collections::BTreeSet;

use super::{LocaleEntry, is_builtin_locale, same_locale, set_system_lang, system_lang};
use crate::LocaleName;

const SHIPPED: &str = "# Configuration file for locale-gen\n#\n#  Examples:\n#  en_US ISO-8859-1\n#  en_US.UTF-8 UTF-8\n#\n#de_DE.UTF-8 UTF-8  \n#de_DE ISO-8859-1  \n#de_DE@euro ISO-8859-15  \nen_US.UTF-8 UTF-8  \n#en_US ISO-8859-1  \n#sr_RS@latin UTF-8  \n";

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

fn locale(name: &str) -> LocaleName {
    LocaleName::new(name).unwrap()
}

#[test]
fn a_line_is_a_locale_and_a_character_set() {
    let german = entry("de_DE.UTF-8 UTF-8");

    assert_eq!(german.locale.as_str(), "de_DE.UTF-8");
    assert_eq!(german.charset.as_str(), "UTF-8");
    assert_eq!(german.to_string(), "de_DE.UTF-8 UTF-8");
    assert_eq!(
        entry("  en_US\tISO-8859-1  ").to_string(),
        "en_US ISO-8859-1"
    );
    assert_eq!(entry("sr_RS@latin UTF-8").locale.as_str(), "sr_RS@latin");
}

#[test]
fn anything_else_is_no_entry() {
    for line in [
        "",
        "de_DE.UTF-8",
        "#de_DE.UTF-8 UTF-8",
        "de_DE.UTF-8 UTF-8 extra",
        "-x UTF-8",
        "de_DE.UTF-8 UTF/8",
    ] {
        assert_eq!(LocaleEntry::parse(line), None, "{line}");
    }
}

#[test]
fn only_uncommented_lines_are_enabled() {
    assert_eq!(
        LocaleEntry::enabled_in(SHIPPED),
        BTreeSet::from([entry("en_US.UTF-8 UTF-8")])
    );
    assert_eq!(LocaleEntry::enabled_in(""), BTreeSet::new());
}

#[test]
fn enabling_removes_only_the_comment_sign_of_that_line() {
    let changed = entry("de_DE.UTF-8 UTF-8").set(SHIPPED, true);

    assert_eq!(
        changed,
        SHIPPED.replace("#de_DE.UTF-8 UTF-8  \n", "de_DE.UTF-8 UTF-8  \n")
    );
    assert_eq!(
        LocaleEntry::enabled_in(&changed),
        BTreeSet::from([entry("de_DE.UTF-8 UTF-8"), entry("en_US.UTF-8 UTF-8")])
    );
}

#[test]
fn the_character_set_tells_two_lines_of_a_locale_apart() {
    let changed = entry("de_DE ISO-8859-1").set(SHIPPED, true);

    assert_eq!(
        changed,
        SHIPPED.replace("#de_DE ISO-8859-1  \n", "de_DE ISO-8859-1  \n")
    );
    assert!(changed.contains("#de_DE.UTF-8 UTF-8"));
    assert!(changed.contains("#de_DE@euro ISO-8859-15"));
}

#[test]
fn the_header_examples_are_never_revived() {
    let changed = entry("en_US ISO-8859-1").set(SHIPPED, true);

    assert!(changed.contains("#  en_US ISO-8859-1\n"));
    assert!(changed.contains("\nen_US ISO-8859-1  \n"));
}

#[test]
fn a_locale_missing_from_the_file_is_appended() {
    let german = entry("de_AT.UTF-8 UTF-8");

    assert_eq!(
        german.set(SHIPPED, true),
        format!("{SHIPPED}de_AT.UTF-8 UTF-8\n")
    );
    assert_eq!(german.set("", true), "de_AT.UTF-8 UTF-8\n");
    assert_eq!(
        german.set("en_US.UTF-8 UTF-8", true),
        "en_US.UTF-8 UTF-8\nde_AT.UTF-8 UTF-8\n"
    );
}

#[test]
fn disabling_comments_every_line_of_the_entry_out() {
    let config = "en_US.UTF-8 UTF-8\r\n  en_US.UTF-8   UTF-8\nen_US ISO-8859-1\n";

    assert_eq!(
        entry("en_US.UTF-8 UTF-8").set(config, false),
        "#en_US.UTF-8 UTF-8\r\n  #en_US.UTF-8   UTF-8\nen_US ISO-8859-1\n"
    );
}

#[test]
fn a_file_that_is_already_as_wanted_stays_byte_for_byte() {
    assert_eq!(entry("en_US.UTF-8 UTF-8").set(SHIPPED, true), SHIPPED);
    assert_eq!(entry("de_DE.UTF-8 UTF-8").set(SHIPPED, false), SHIPPED);
    assert_eq!(entry("xx_XX.UTF-8 UTF-8").set(SHIPPED, false), SHIPPED);
}

#[test]
fn off_then_on_gives_the_first_file_back() {
    let english = entry("en_US.UTF-8 UTF-8");
    let off = english.set(SHIPPED, false);

    assert!(off.contains("\n#en_US.UTF-8 UTF-8  \n"));
    assert_eq!(english.set(&off, true), SHIPPED);
}

#[test]
fn an_entry_provides_the_lang_values_glibc_accepts_for_it() {
    let utf8 = entry("de_DE.UTF-8 UTF-8");
    let latin = entry("de_DE ISO-8859-1");
    let euro = entry("de_DE@euro ISO-8859-15");

    assert!(utf8.provides(&locale("de_DE.UTF-8")));
    assert!(utf8.provides(&locale("de_DE.utf8")));
    assert!(!utf8.provides(&locale("de_DE")));
    assert!(!utf8.provides(&locale("de_AT.UTF-8")));
    assert!(latin.provides(&locale("de_DE")));
    assert!(latin.provides(&locale("de_DE.ISO-8859-1")));
    assert!(latin.provides(&locale("de_DE.iso88591")));
    assert!(!latin.provides(&locale("de_DE.UTF-8")));
    assert!(euro.provides(&locale("de_DE@euro")));
    assert!(!euro.provides(&locale("de_DE")));
}

#[test]
fn lang_values_compare_without_codeset_spelling() {
    assert!(same_locale(&locale("de_DE.UTF-8"), &locale("de_DE.utf8")));
    assert!(same_locale(&locale("sr_RS@latin"), &locale("sr_RS@latin")));
    assert!(!same_locale(&locale("de_DE.UTF-8"), &locale("de_DE")));
    assert!(!same_locale(&locale("de_DE.UTF-8"), &locale("de_AT.UTF-8")));
    assert!(!same_locale(&locale("sr_RS"), &locale("sr_RS@latin")));
}

#[test]
fn c_and_posix_need_no_generation() {
    assert!(is_builtin_locale(&locale("C")));
    assert!(is_builtin_locale(&locale("C.UTF-8")));
    assert!(is_builtin_locale(&locale("POSIX")));
    assert!(!is_builtin_locale(&locale("en_US.UTF-8")));
}

#[test]
fn the_system_language_is_the_last_lang_assignment() {
    assert_eq!(
        system_lang("LANG=en_US.UTF-8\nLC_TIME=de_DE.UTF-8\n"),
        Some(locale("en_US.UTF-8"))
    );
    assert_eq!(
        system_lang("LANG=en_US.UTF-8\n  LANG=\"de_DE.UTF-8\"  \n"),
        Some(locale("de_DE.UTF-8"))
    );
    assert_eq!(system_lang("#LANG=en_US.UTF-8\nLANGUAGE=de\n"), None);
    assert_eq!(system_lang(""), None);
    assert_eq!(system_lang("LANG=\n"), None);
}

#[test]
fn setting_the_language_keeps_every_other_line() {
    let config = "# made by the installer\nLANG=en_US.UTF-8\r\nLC_TIME=de_DE.UTF-8\nLANGUAGE=en\n";

    assert_eq!(
        set_system_lang(config, &locale("de_DE.UTF-8")),
        "# made by the installer\nLANG=de_DE.UTF-8\r\nLC_TIME=de_DE.UTF-8\nLANGUAGE=en\n"
    );
}

#[test]
fn a_file_without_lang_gets_the_line_appended() {
    assert_eq!(
        set_system_lang("", &locale("de_DE.UTF-8")),
        "LANG=de_DE.UTF-8\n"
    );
    assert_eq!(
        set_system_lang("LC_TIME=de_DE.UTF-8", &locale("en_US.UTF-8")),
        "LC_TIME=de_DE.UTF-8\nLANG=en_US.UTF-8\n"
    );
}

#[test]
fn what_is_written_reads_back() {
    let german = locale("de_DE.UTF-8");

    assert_eq!(
        system_lang(&set_system_lang("LANG=C\n", &german)),
        Some(german)
    );
}
