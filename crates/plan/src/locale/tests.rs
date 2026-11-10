use std::collections::BTreeSet;

use super::{LocaleEntry, is_builtin_locale, same_locale, set_system_lang, system_lang};
use crate::LocaleName;

/// `/etc/locale.gen` of the Lima image, written by cloud-init.
const LOCALE_GEN: &str = crate::testing::locales::LOCALE_GEN;
/// `/usr/share/i18n/SUPPORTED` of the Lima image.
const SUPPORTED: &str = crate::testing::locales::SUPPORTED;
/// `/etc/locale.conf` of the Lima image.
const LOCALE_CONF: &str = crate::testing::locales::LOCALE_CONF;

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

fn locale(name: &str) -> LocaleName {
    LocaleName::new(name).unwrap()
}

/// Edit: the real file with every line of `SUPPORTED` appended behind a
/// `#`, the way glibc's stock `locale.gen` lists them.
fn shipped() -> String {
    let commented: String = SUPPORTED
        .lines()
        .flat_map(|line| ["#", line, "\n"])
        .collect();

    format!("{LOCALE_GEN}{commented}")
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
fn every_supported_line_is_an_entry() {
    for line in SUPPORTED.lines() {
        assert_eq!(entry(line).to_string(), line);
    }
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
    let english = BTreeSet::from([entry("en_US.UTF-8 UTF-8")]);

    assert_eq!(LocaleEntry::enabled_in(LOCALE_GEN), english);
    assert_eq!(LocaleEntry::enabled_in(&shipped()), english);
    assert_eq!(LocaleEntry::enabled_in(""), BTreeSet::new());
}

#[test]
fn enabling_removes_only_the_comment_sign_of_that_line() {
    let shipped = shipped();
    let changed = entry("de_DE.UTF-8 UTF-8").set(&shipped, true);

    assert_eq!(
        changed,
        shipped.replace("#de_DE.UTF-8 UTF-8\n", "de_DE.UTF-8 UTF-8\n")
    );
    assert_eq!(
        LocaleEntry::enabled_in(&changed),
        BTreeSet::from([entry("de_DE.UTF-8 UTF-8"), entry("en_US.UTF-8 UTF-8")])
    );
}

#[test]
fn the_character_set_tells_two_lines_of_a_locale_apart() {
    let shipped = shipped();
    let changed = entry("de_DE ISO-8859-1").set(&shipped, true);

    assert_eq!(
        changed,
        shipped.replace("#de_DE ISO-8859-1\n", "de_DE ISO-8859-1\n")
    );
    assert!(changed.contains("#de_DE.UTF-8 UTF-8"));
    assert!(changed.contains("#de_DE@euro ISO-8859-15"));
}

/// No captured file has the example lines of the Arch header, so this
/// edit puts one under the real header.
#[test]
fn the_header_examples_are_never_revived() {
    let edited = LOCALE_GEN.replacen('\n', "\n#  en_US ISO-8859-1\n", 1);
    let changed = entry("en_US ISO-8859-1").set(&edited, true);

    assert!(changed.contains("#  en_US ISO-8859-1\n"));
    assert!(changed.ends_with("\nen_US ISO-8859-1\n"));
}

#[test]
fn a_locale_missing_from_the_file_is_appended() {
    let austrian = entry("de_AT.UTF-8 UTF-8");

    assert_eq!(
        austrian.set(LOCALE_GEN, true),
        format!("{LOCALE_GEN}de_AT.UTF-8 UTF-8\n")
    );
    assert_eq!(austrian.set("", true), "de_AT.UTF-8 UTF-8\n");

    // Edit: the real file without its last line break.
    assert_eq!(
        austrian.set(LOCALE_GEN.trim_end(), true),
        format!("{LOCALE_GEN}de_AT.UTF-8 UTF-8\n")
    );
}

#[test]
fn disabling_comments_every_line_of_the_entry_out() {
    // Edit: the real file with CRLF endings and its entry repeated
    // indented and spaced out.
    let config = format!(
        "{}  en_US.UTF-8   UTF-8\n",
        LOCALE_GEN.replace('\n', "\r\n")
    );

    assert_eq!(
        entry("en_US.UTF-8 UTF-8").set(&config, false),
        config
            .replace("\nen_US.UTF-8 UTF-8\r\n", "\n#en_US.UTF-8 UTF-8\r\n")
            .replace("  en_US.UTF-8   UTF-8\n", "  #en_US.UTF-8   UTF-8\n")
    );
}

#[test]
fn a_file_that_is_already_as_wanted_stays_byte_for_byte() {
    let shipped = shipped();

    assert_eq!(entry("en_US.UTF-8 UTF-8").set(&shipped, true), shipped);
    assert_eq!(entry("de_DE.UTF-8 UTF-8").set(&shipped, false), shipped);
    assert_eq!(
        entry("de_DE.UTF-8 UTF-8").set(LOCALE_GEN, false),
        LOCALE_GEN
    );
}

#[test]
fn off_then_on_gives_the_first_file_back() {
    let english = entry("en_US.UTF-8 UTF-8");
    let off = english.set(LOCALE_GEN, false);

    assert!(off.ends_with("\n#en_US.UTF-8 UTF-8\n"));
    assert_eq!(english.set(&off, true), LOCALE_GEN);
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
    assert_eq!(system_lang(LOCALE_CONF), Some(locale("en_US.UTF-8")));

    // Edit: a time locale and then a quoted, indented `LANG` appended.
    assert_eq!(
        system_lang(&format!(
            "{LOCALE_CONF}LC_TIME=de_DE.UTF-8\n  LANG=\"de_DE.UTF-8\"  \n"
        )),
        Some(locale("de_DE.UTF-8"))
    );

    // Edit: the real line commented out, a `LANGUAGE` after it.
    assert_eq!(system_lang(&format!("#{LOCALE_CONF}LANGUAGE=de\n")), None);
    assert_eq!(system_lang(""), None);
    assert_eq!(system_lang("LANG=\n"), None);
}

#[test]
fn setting_the_language_keeps_every_other_line() {
    // Edit: the real file with a comment before it, CRLF and two more
    // variables after it.
    let config = format!(
        "# made by the installer\n{}LC_TIME=de_DE.UTF-8\nLANGUAGE=en\n",
        LOCALE_CONF.replace('\n', "\r\n")
    );

    assert_eq!(
        set_system_lang(&config, &locale("de_DE.UTF-8")),
        "# made by the installer\nLANG=de_DE.UTF-8\r\nLC_TIME=de_DE.UTF-8\nLANGUAGE=en\n"
    );
    assert_eq!(
        set_system_lang(LOCALE_CONF, &locale("de_DE.UTF-8")),
        "LANG=de_DE.UTF-8\n"
    );
}

#[test]
fn a_file_without_lang_gets_the_line_appended() {
    assert_eq!(
        set_system_lang("", &locale("de_DE.UTF-8")),
        "LANG=de_DE.UTF-8\n"
    );

    // Edit: the real file with its variable renamed and no last line break.
    let time_only = LOCALE_CONF.replace("LANG=", "LC_TIME=");

    assert_eq!(
        set_system_lang(time_only.trim_end(), &locale("de_DE.UTF-8")),
        "LC_TIME=en_US.UTF-8\nLANG=de_DE.UTF-8\n"
    );
}

#[test]
fn what_is_written_reads_back() {
    let german = locale("de_DE.UTF-8");

    assert_eq!(
        system_lang(&set_system_lang(LOCALE_CONF, &german)),
        Some(german)
    );
}

/// `locale-gen` splits a line on ASCII blanks only. Changed: the first
/// enabled line of the Lima file with a no-break space for its blank.
#[test]
fn a_no_break_space_does_not_part_a_line() {
    let line = LOCALE_GEN
        .lines()
        .find(|line| LocaleEntry::parse(line).is_some())
        .unwrap();

    let joined = line.replacen(' ', "\u{a0}", 1);

    assert_eq!(LocaleEntry::parse(&joined), None);
    assert_eq!(
        LocaleEntry::parse(&line.replacen(' ', "\t", 1)),
        LocaleEntry::parse(line)
    );
}

/// Changed: the enabled line of the Lima file with a comment after it,
/// and with blanks before it. `locale-gen` (glibc 2.44) generates nothing
/// for the first and drops the blanks of the second.
#[test]
fn a_trailing_comment_spoils_the_line_and_leading_blanks_do_not() {
    let line = LOCALE_GEN
        .lines()
        .find(|line| LocaleEntry::parse(line).is_some())
        .unwrap();

    assert_eq!(LocaleEntry::parse(&format!("{line} # mine")), None);
    assert_eq!(
        LocaleEntry::parse(&format!(" \t{line}")),
        LocaleEntry::parse(line)
    );
}
