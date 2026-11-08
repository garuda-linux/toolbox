use std::fs;

use serde_json::json;

use super::task::{SOURCE_CODE, json_path};
use super::{Files, folder_for, import_locale, locale_json, read_files, source_json, tx_code};
use crate::{LOCALES_DIR, root};

fn files(pairs: &[(&str, &str)]) -> Files {
    pairs
        .iter()
        .map(|(stem, text)| ((*stem).to_owned(), (*text).to_owned()))
        .collect()
}

fn real(locale: &str) -> Files {
    read_files(&root().join(LOCALES_DIR).join(locale)).expect("the locale reads")
}

#[test]
fn exporting_and_importing_the_real_files_changes_nothing() {
    let english = real("en-US");
    let german = real("de-DE");
    let exported = locale_json(&english, &german).unwrap();

    let (changed, problems) = import_locale(&english, &german, &exported);

    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(
        changed.keys().collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "the round trip touched files"
    );
}

#[test]
fn every_real_message_can_go_to_transifex() {
    let (_, untranslatable) = source_json(&real("en-US")).unwrap();

    assert_eq!(untranslatable, Vec::<String>::new());
}

#[test]
fn the_source_json_carries_the_note_and_the_icu_text() {
    let english = files(&[(
        "console",
        "# Steps with nothing to do.\npartly = { $count ->\n        [one] 1 step\n       *[other] { $count } steps\n    }\n",
    )]);

    let (json, _) = source_json(&english).unwrap();

    assert_eq!(
        json,
        json!({"console": {"partly": {
            "string": "{count, plural, one {1 step} other {{count} steps}}",
            "developer_comment": "Steps with nothing to do."
        }}})
    );
}

#[test]
fn a_choice_goes_out_as_one_entry_per_variant() {
    let english = files(&[(
        "share",
        "ask = { $subject ->\n    [report] Upload the report?\n   *[output] Upload the output?\n}\n",
    )]);

    let (json, _) = source_json(&english).unwrap();

    assert_eq!(
        json["share"]["ask"]["report"]["string"],
        "Upload the report?"
    );
    assert_eq!(
        json["share"]["ask"]["output"]["string"],
        "Upload the output?"
    );
}

#[test]
fn an_untranslatable_message_is_named_and_left_out() {
    let english = files(&[("f", "ok = Fine\nbad = { NUMBER($n) }\n")]);

    let (json, untranslatable) = source_json(&english).unwrap();

    assert!(json["f"].get("bad").is_none());
    assert_eq!(untranslatable, ["f.ftl: bad calls a function"]);
}

#[test]
fn a_new_translation_replaces_only_its_message_and_its_mark() {
    let english = files(&[("c", "a = Apple\nb = Banana\n")]);
    let german = files(&[("c", "a = Apfel\n# untranslated\nb = Banana\n")]);
    let incoming = json!({"c": {"a": {"string": "Apfel"}, "b": {"string": "Banane"}}});

    let (changed, problems) = import_locale(&english, &german, &incoming);

    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(changed["c"], "a = Apfel\nb = Banane\n");
}

#[test]
fn the_english_text_coming_back_never_replaces_a_translation() {
    let english = files(&[("c", "a = Apple\n")]);
    let german = files(&[("c", "a = Apfel\n")]);
    let incoming = json!({"c": {"a": {"string": "Apple"}}});

    let (changed, _) = import_locale(&english, &german, &incoming);

    assert!(changed.is_empty());
}

#[test]
fn a_missing_message_is_added_after_the_one_before_it() {
    let english = files(&[("c", "a = Apple\nb = Banana\nc = Cherry\n")]);
    let german = files(&[("c", "a = Apfel\nc = Kirsche\n")]);
    let incoming = json!({"c": {"b": {"string": "Banane"}}});

    let (changed, _) = import_locale(&english, &german, &incoming);

    assert_eq!(changed["c"], "a = Apfel\nb = Banane\nc = Kirsche\n");
}

#[test]
fn a_translation_with_a_foreign_variable_or_bad_syntax_is_refused() {
    let english = files(&[("c", "a = Hello { $user }\nb = Bye\n")]);
    let german = files(&[("c", "a = Hallo { $user }\nb = Tschüss\n")]);
    let incoming = json!({"c": {
        "a": {"string": "Hallo {name}"},
        "b": {"string": "Tschüss {s, select, x {y} other {z}}"}
    }});

    let (changed, problems) = import_locale(&english, &german, &incoming);

    assert!(changed.is_empty());
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("name"), "{problems:?}");
    assert!(problems[1].contains("select"), "{problems:?}");
}

#[test]
fn an_english_file_that_is_no_fluent_fails_the_export() {
    let english = files(&[("c", "a = Apple\nbroken =\n")]);

    let error = source_json(&english).unwrap_err();

    assert!(error.starts_with("c.ftl: not valid Fluent"), "{error}");
}

#[test]
fn a_locale_file_that_is_no_fluent_is_reported_and_left_alone() {
    let english = files(&[("c", "a = Apple\n")]);
    let german = files(&[("c", "a = Apfel\nbroken =\n")]);
    let incoming = json!({"c": {"a": {"string": "Apfelchen"}}});

    let (changed, problems) = import_locale(&english, &german, &incoming);

    assert!(changed.is_empty(), "{changed:?}");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("c.ftl: not valid Fluent"),
        "{problems:?}"
    );
    locale_json(&english, &german).unwrap_err();
}

#[test]
fn locale_folders_and_transifex_codes_map_both_ways() {
    let folders = ["en-US", "de-DE", "pt-BR", "pt-PT"].map(String::from);

    assert_eq!(tx_code("de-DE", &folders), "de");
    assert_eq!(tx_code("pt-BR", &folders), "pt_BR");
    assert_eq!(folder_for("de", &folders), "de-DE");
    assert_eq!(folder_for("pt_BR", &folders), "pt-BR");
    assert_eq!(folder_for("fr", &folders), "fr");
}

#[test]
fn the_checked_in_source_json_is_current() {
    let (json, _) = source_json(&real("en-US")).unwrap();
    let written = fs::read_to_string(json_path(SOURCE_CODE)).unwrap_or_default();

    assert_eq!(written, super::pretty(&json));
}

#[test]
fn only_language_tags_are_locale_folders() {
    for (name, locale) in [
        ("de-DE", true),
        ("pt_BR", false),
        ("fr", true),
        (".claude", false),
        ("tests", false),
        ("zh-Hant-TW", true),
    ] {
        assert_eq!(super::is_locale(name), locale, "{name}");
    }
}
