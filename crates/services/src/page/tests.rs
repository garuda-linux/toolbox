use std::collections::BTreeSet;

use super::{Group, PageId};
use crate::i18n;

#[test]
fn every_page_has_a_name_in_every_language() {
    for language in i18n::available() {
        for page in PageId::ALL {
            assert_ne!(
                i18n::text_in(language.clone(), page.text_id()),
                page.text_id(),
                "{page:?} has no name in {language}"
            );
        }
    }
}

#[test]
fn pages_have_distinct_names() {
    let names: BTreeSet<_> = PageId::ALL.iter().map(|page| page.text_id()).collect();

    assert_eq!(names.len(), PageId::ALL.len());
}

#[test]
fn groups_keep_the_drawer_order() {
    let groups: Vec<Group> = PageId::ALL.iter().map(|page| page.group()).collect();
    let mut sorted = groups.clone();
    sorted.sort();

    assert_eq!(
        groups, sorted,
        "pages of one group sit together, in group order"
    );
}

#[test]
fn home_is_where_the_app_starts() {
    assert_eq!(PageId::default(), PageId::Home);
}

#[test]
fn a_page_is_found_by_the_word_other_starts_name_it_with() {
    assert_eq!(PageId::from_slug("diagnostics"), Some(PageId::Diagnostics));
    assert_eq!(
        PageId::from_slug("system-settings"),
        Some(PageId::SystemSettings)
    );
    assert_eq!(PageId::from_slug("boot-repair"), Some(PageId::BootRepair));
    assert_eq!(PageId::from_slug("review"), None);

    for page in PageId::ALL {
        assert_eq!(PageId::from_slug(page.slug()), Some(page));
    }
}

/// Desktop actions, the command line and the Fluent files use these
/// words, so none may change.
#[test]
fn slugs_and_text_ids_stay_the_same() {
    let words = [
        "home",
        "setup",
        "components",
        "gaming",
        "discover",
        "packages",
        "services",
        "system-settings",
        "kernels",
        "locales",
        "users",
        "configs",
        "network",
        "boot",
        "maintenance",
        "boot-repair",
        "diagnostics",
        "settings",
        "guide",
        "about",
    ];

    let slugs: Vec<_> = PageId::ALL.into_iter().map(PageId::slug).collect();
    let text_ids: Vec<_> = PageId::ALL.into_iter().map(PageId::text_id).collect();
    let expected: Vec<_> = words.iter().map(|word| format!("page-{word}")).collect();

    assert_eq!(slugs, words);
    assert_eq!(text_ids, expected);
}
