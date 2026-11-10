//! The app's texts: Fluent files under `crates/services/i18n/<language>/`,
//! compiled in by `fluent-templates`. Each lookup names its language, so
//! the window and the command line read the same texts.

pub mod changes;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::LazyLock;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader as _, langid};
use parking_lot::Mutex;

/// The language every text exists in and the others fall back to.
pub const FALLBACK: LanguageIdentifier = langid!("en-US");
/// The Fluent files' folder, for the tests that read them as files.
pub const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/i18n");
/// The environment variables gettext reads, in its order.
const LOCALE_VARIABLES: [&str; 4] = ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"];

/// The texts [`text_in`] looked up, by language and id.
static CACHE: LazyLock<Mutex<Texts>> = LazyLock::new(Mutex::default);

type Texts = HashMap<LanguageIdentifier, HashMap<String, &'static str>>;

fluent_templates::static_loader! {
    static TEXTS = {
        locales: "i18n",
        fallback_language: "en-US",
        // egui lays out no bidirectional text, so the isolation marks
        // Fluent puts around arguments would only show as boxes.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// The languages there are texts for, sorted by code.
pub fn available() -> Vec<LanguageIdentifier> {
    let mut languages: Vec<LanguageIdentifier> = TEXTS.locales().cloned().collect();
    languages.sort_by_key(ToString::to_string);
    languages
}

/// The language the system asks for, as gettext reads it, matched to the
/// ones on offer: the exact one, else another region of the same
/// language, else English.
pub fn language_from_env(lookup: impl Fn(&str) -> Option<OsString>) -> LanguageIdentifier {
    let offered = available();
    let wanted = LOCALE_VARIABLES
        .iter()
        .filter_map(|variable| lookup(variable))
        .flat_map(|value| {
            value
                .to_string_lossy()
                .split(':')
                .filter_map(posix_language)
                .collect::<Vec<_>>()
        });

    for language in wanted {
        let exact = offered.iter().find(|offer| **offer == language);
        let same = offered
            .iter()
            .find(|offer| offer.language == language.language);

        if let Some(found) = exact.or(same) {
            return found.clone();
        }
    }

    FALLBACK
}

/// The text `id` in `language`. An unknown id is logged and shown as it
/// is, so it stands out instead of leaving a gap.
///
/// Pages ask for the same texts every frame, so each is looked up once
/// per language and lent out from then on.
pub fn text_in(language: LanguageIdentifier, id: &str) -> Cow<'static, str> {
    // Held for the lookup too: two threads that missed at once would each
    // leak a copy, and the later one would replace the text the first
    // already lent out.
    let mut cache = CACHE.lock();
    let cached = cache
        .get(&language)
        .and_then(|texts| texts.get(id).copied());

    let found: &'static str = cached.unwrap_or_else(|| {
        // Kept for the whole run: the texts are a fixed set per language,
        // so this never grows past the files compiled in.
        let looked_up: &'static str = TEXTS
            .try_lookup(&language, id)
            .unwrap_or_else(|| missing(id))
            .leak();

        cache
            .entry(language)
            .or_default()
            .insert(id.to_owned(), looked_up);
        looked_up
    });

    drop(cache);
    Cow::Borrowed(found)
}

/// [`text_in`], or `fallback` for an id no file has. Catalog texts use it,
/// since their English is in the catalog itself.
pub fn text_or_in(language: &LanguageIdentifier, id: &str, fallback: &str) -> Cow<'static, str> {
    Cow::Owned(
        TEXTS
            .try_lookup(language, id)
            .unwrap_or_else(|| fallback.to_owned()),
    )
}

/// [`text_in`] with named arguments. A whole number goes in as a number,
/// so the message can pick its plural form by it.
pub fn text_with_in(
    language: &LanguageIdentifier,
    id: &str,
    args: &[(&'static str, &str)],
) -> Cow<'static, str> {
    let args: HashMap<Cow<'static, str>, FluentValue<'static>> = args
        .iter()
        .map(|(name, value)| {
            let value = value.parse::<i64>().map_or_else(
                |_| FluentValue::from((*value).to_owned()),
                FluentValue::from,
            );

            (Cow::Borrowed(*name), value)
        })
        .collect();

    Cow::Owned(
        TEXTS
            .try_lookup_with_args(language, id, &args)
            .unwrap_or_else(|| missing(id)),
    )
}

fn missing(id: &str) -> String {
    tracing::warn!("unknown text id `{id}`");
    id.to_owned()
}

/// `de_DE.UTF-8@euro` as `de-DE`. `None` for the C locale.
fn posix_language(locale: &str) -> Option<LanguageIdentifier> {
    let name = locale.split(['.', '@']).next()?;
    if name.is_empty() || name == "C" || name == "POSIX" {
        return None;
    }

    name.replace('_', "-").parse().ok()
}

/// `language` named in its own words, such as "Deutsch (Deutschland)".
pub fn name_of(language: &LanguageIdentifier) -> String {
    TEXTS.lookup(language, "language-name")
}
