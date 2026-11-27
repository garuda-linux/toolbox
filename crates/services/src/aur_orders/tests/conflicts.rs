use fluent_templates::LanguageIdentifier;

use crate::i18n::{FALLBACK, text_with_in};

/// The refusal of a conflicting AUR build names the installed package, which
/// the banner shows, in both languages.
#[test]
fn the_conflict_refusal_names_the_installed_package_in_both_languages() {
    let german: LanguageIdentifier = "de-DE".parse().unwrap();
    for language in [FALLBACK, german] {
        let text = text_with_in(
            &language,
            "packages-refused-conflicts",
            &[("name", "yay".into())],
        );

        assert!(text.contains("yay"), "{text} in {language}");
        assert!(!text.contains('{'), "{text} in {language}");
    }
}
