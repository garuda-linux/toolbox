use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use strum::IntoEnumIterator as _;
use toolbox_system::Probe;

use super::{available, set_language, text, text_or};

fn ids_of(locale: &Path) -> BTreeSet<String> {
    fs::read_dir(locale)
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .flat_map(|file| {
            file.lines()
                .filter_map(|line| line.split_once(" =").map(|(id, _)| id.to_owned()))
                .filter(|id| {
                    id.chars()
                        .next()
                        .is_some_and(|first| first.is_ascii_lowercase())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn every_locale_has_the_same_message_ids() {
    let root = Path::new(toolbox_services::i18n::DIR);
    let english = ids_of(&root.join("en-US"));

    for locale in available() {
        assert_eq!(
            ids_of(&root.join(locale.to_string())),
            english,
            "{locale} differs from en-US"
        );
    }
}

#[test]
fn every_probe_has_a_name() {
    let english = ids_of(&Path::new(toolbox_services::i18n::DIR).join("en-US"));

    for probe in Probe::iter() {
        let id = format!("probe-{}", probe.as_str());
        assert!(english.contains(&id), "{id} is missing");
    }
}

#[test]
fn texts_come_in_the_language_set_on_the_context() {
    let ctx = egui::Context::default();
    assert_eq!(text(&ctx, "language-name"), "English (United States)");

    set_language(&ctx, &"de-DE".parse().unwrap());
    assert_eq!(text(&ctx, "language-name"), "Deutsch (Deutschland)");
}

#[test]
fn an_unknown_id_shows_itself() {
    let ctx = egui::Context::default();

    assert_eq!(text(&ctx, "no-such-message"), "no-such-message");
}

#[test]
fn catalog_texts_come_translated_or_as_the_catalog_says() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());

    assert_eq!(
        text_or(&ctx, "catalog-components-virtualization", "Virtualization"),
        "Virtualisierung"
    );
    assert_eq!(text_or(&ctx, "catalog-components-new", "New"), "New");
}

#[test]
fn every_account_refusal_has_a_text() {
    let english = ids_of(&Path::new(toolbox_services::i18n::DIR).join("en-US"));

    for refusal in toolbox_core::features::users::Refusal::iter() {
        let id = format!("account-refusal-{}", refusal.as_str());
        assert!(english.contains(&id), "{id} is missing");
    }
}

#[test]
fn a_text_without_arguments_is_looked_up_once_per_language() {
    let ctx = egui::Context::default();
    let first = text(&ctx, "page-home");
    let again = text(&ctx, "page-home");

    set_language(&ctx, &"de-DE".parse().unwrap());
    let german = text(&ctx, "page-home");

    assert!(matches!(again, Cow::Borrowed(_)));
    assert!(std::ptr::eq(first.as_ptr(), again.as_ptr()));
    assert_ne!(first, german);
    assert!(std::ptr::eq(
        german.as_ptr(),
        text(&ctx, "page-home").as_ptr()
    ));
}
