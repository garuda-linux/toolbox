//! The language and territory a locale stands for.
//!
//! As glibc's own locale sources name them (`language "German"`,
//! `territory "Germany"` in the `LC_IDENTIFICATION` part of
//! `/usr/share/i18n/locales/de_DE`). No flags: a flag stands for a
//! country, not for a language.

use std::collections::BTreeMap;

use toolbox_plan::LocaleName;

/// Longer than any name glibc ships. A file that holds more is cut.
const MAX_NAME_CHARS: usize = 64;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LanguageName {
    pub language: String,
    /// Empty for a language without a country, such as Esperanto.
    pub territory: String,
}

/// The names of the locales, by the file name of the locale's source
/// (`de_DE`, `ca_ES@valencia`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocaleNames(pub BTreeMap<String, LanguageName>);

impl LocaleNames {
    pub fn of(&self, locale: &LocaleName) -> Option<&LanguageName> {
        self.0.get(&source_name(locale.as_str()))
    }
}

/// The file of glibc's locale sources that defines `locale`: the name
/// without its character set, the modifier kept (`ca_ES.UTF-8@valencia`
/// is defined by `ca_ES@valencia`).
pub fn source_name(locale: &str) -> String {
    let (plain, modifier) = match locale.split_once('@') {
        Some((plain, modifier)) => (plain, Some(modifier)),
        None => (locale, None),
    };

    let base = plain.split('.').next().unwrap_or_default();

    modifier.map_or_else(|| base.to_owned(), |modifier| format!("{base}@{modifier}"))
}

/// The `language` and `territory` of a locale source's
/// `LC_IDENTIFICATION` part. `None` for a file without a language, which
/// is how the sources of collation rules and of `C` look.
pub fn parse_identification(text: &str) -> Option<LanguageName> {
    let mut name = LanguageName::default();

    for line in text.lines() {
        let line = line.trim();

        if line.starts_with("END LC_IDENTIFICATION") {
            break;
        }

        if let Some(value) = field(line, "language") {
            name.language = value;
        }

        if let Some(value) = field(line, "territory") {
            name.territory = value;
        }
    }

    (!name.language.is_empty()).then_some(name)
}

fn field(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;

    if !rest.starts_with(char::is_whitespace) {
        return None;
    }

    let quoted = rest.trim().strip_prefix('"')?;
    let value = quoted.split('"').next().unwrap_or_default();

    Some(
        decode(value)
            .chars()
            .filter(|character| !character.is_control())
            .take(MAX_NAME_CHARS)
            .collect::<String>()
            .trim()
            .to_owned(),
    )
}

/// Older sources spell every character outside ASCII as `<U00E5>`.
fn decode(value: &str) -> String {
    let mut decoded = String::new();
    let mut rest = value;

    while let Some((before, after)) = rest.split_once("<U") {
        decoded.push_str(before);

        let character = after.split_once('>').and_then(|(digits, tail)| {
            let character = u32::from_str_radix(digits, 16)
                .ok()
                .and_then(char::from_u32)?;

            Some((character, tail))
        });

        if let Some((character, tail)) = character {
            decoded.push(character);
            rest = tail;
        } else {
            decoded.push_str("<U");
            rest = after;
        }
    }

    decoded.push_str(rest);
    decoded
}
