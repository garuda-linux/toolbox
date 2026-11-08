//! Pure text functions without I/O. Enabling or disabling one entry touches only its line,
//! so comments and every other entry stay byte for byte. The probe and the helper
//! both use this module, so what one writes the other reads back the same way.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::lines::{ending, push_line};
use crate::{Charset, LocaleName};

/// Relative to `/`, like every path the helper writes below its root.
pub const LOCALE_GEN: &str = "etc/locale.gen";

/// Relative to `/` as well.
pub const LOCALE_CONF: &str = "etc/locale.conf";

const LANG: &str = "LANG";

/// The locales glibc has without `locale-gen`.
const BUILTIN: [&str; 3] = ["C", "POSIX", "C.UTF-8"];

/// One line of `locale.gen`, also one of `/usr/share/i18n/SUPPORTED`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocaleEntry {
    pub locale: LocaleName,
    pub charset: Charset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Line {
    Enabled,
    /// `#de_DE.UTF-8 UTF-8`, the shape of the shipped file. A `#` followed
    /// by a blank is prose (the header's examples) and stays.
    Commented,
    Unrelated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Parts<'name> {
    base: &'name str,
    codeset: Option<&'name str>,
    modifier: Option<&'name str>,
}

impl LocaleEntry {
    /// The entry of a line that holds exactly a locale and a character
    /// set, as `locale-gen` reads it.
    pub fn parse(line: &str) -> Option<Self> {
        let mut words = line.split_whitespace();
        let (Some(locale), Some(charset), None) = (words.next(), words.next(), words.next()) else {
            return None;
        };

        Some(Self {
            locale: LocaleName::new(locale).ok()?,
            charset: Charset::new(charset).ok()?,
        })
    }

    pub fn enabled_in(config: &str) -> BTreeSet<Self> {
        config.lines().filter_map(Self::parse).collect()
    }

    /// `config` with the entry switched on or off. On keeps an enabled
    /// line, or else revives the first commented one, or else appends the
    /// line. Off comments every enabled line of the entry out.
    pub fn set(&self, config: &str, on: bool) -> String {
        let enabled = config
            .split_inclusive('\n')
            .any(|line| self.classify(line) == Line::Enabled);

        if on == enabled {
            return config.to_owned();
        }

        let mut placed = false;
        let mut result = String::with_capacity(config.len());

        for line in config.split_inclusive('\n') {
            match self.classify(line) {
                Line::Enabled if !on => result.push_str(&commented(line)),
                Line::Commented if on && !placed => {
                    placed = true;
                    result.push_str(&line.replacen('#', "", 1));
                }
                Line::Enabled | Line::Commented | Line::Unrelated => result.push_str(line),
            }
        }

        if on && !placed {
            push_line(&mut result, &self.to_string());
        }

        result
    }

    /// Whether generating this entry makes `lang` usable as `LANG`: the
    /// same language, territory and modifier, and the same character set,
    /// which glibc compares without case and dashes (`UTF-8` is `utf8`).
    pub fn provides(&self, lang: &LocaleName) -> bool {
        let own = Parts::of(self.locale.as_str());
        let wanted = Parts::of(lang.as_str());

        if own.base != wanted.base || own.modifier != wanted.modifier {
            return false;
        }

        match (own.codeset, wanted.codeset) {
            (None, None) => true,
            (Some(own), Some(wanted)) => same_codeset(own, wanted),
            (None, Some(wanted)) => same_codeset(self.charset.as_str(), wanted),
            (Some(_), None) => false,
        }
    }

    fn classify(&self, line: &str) -> Line {
        let trimmed = line.trim();

        if Self::parse(trimmed).as_ref() == Some(self) {
            return Line::Enabled;
        }

        match trimmed.strip_prefix('#') {
            Some(rest)
                if !rest.starts_with(char::is_whitespace)
                    && Self::parse(rest).as_ref() == Some(self) =>
            {
                Line::Commented
            }
            Some(_) | None => Line::Unrelated,
        }
    }
}

impl fmt::Display for LocaleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.locale, self.charset)
    }
}

impl<'name> Parts<'name> {
    fn of(name: &'name str) -> Self {
        let (rest, modifier) = match name.split_once('@') {
            Some((rest, modifier)) => (rest, Some(modifier)),
            None => (name, None),
        };

        let (base, codeset) = match rest.split_once('.') {
            Some((base, codeset)) => (base, Some(codeset)),
            None => (rest, None),
        };

        Self {
            base,
            codeset,
            modifier,
        }
    }
}

/// Whether `lang` needs no generated locale.
pub fn is_builtin_locale(lang: &LocaleName) -> bool {
    BUILTIN.contains(&lang.as_str())
}

pub fn same_locale(left: &LocaleName, right: &LocaleName) -> bool {
    let left = Parts::of(left.as_str());
    let right = Parts::of(right.as_str());

    let codeset = match (left.codeset, right.codeset) {
        (None, None) => true,
        (Some(left), Some(right)) => same_codeset(left, right),
        (Some(_), None) | (None, Some(_)) => false,
    };

    codeset && left.base == right.base && left.modifier == right.modifier
}

/// The system language of a `locale.conf`: its last `LANG` assignment, as
/// systemd reads the file.
pub fn system_lang(config: &str) -> Option<LocaleName> {
    config
        .lines()
        .rev()
        .find_map(lang_value)
        .and_then(|value| LocaleName::new(value).ok())
}

/// `config` with `LANG` set to `locale`. Every other line stays, so the
/// regional formats (`LC_TIME` and the like) keep their own locales.
pub fn set_system_lang(config: &str, locale: &LocaleName) -> String {
    let wanted = format!("{LANG}={locale}");
    let mut placed = false;
    let mut result = String::with_capacity(config.len() + wanted.len() + 1);

    for line in config.split_inclusive('\n') {
        if lang_value(line).is_some() {
            placed = true;
            result.push_str(&wanted);
            result.push_str(ending(line));
        } else {
            result.push_str(line);
        }
    }

    if !placed {
        push_line(&mut result, &wanted);
    }

    result
}

fn lang_value(line: &str) -> Option<&str> {
    let value = line.trim().strip_prefix(LANG)?.strip_prefix('=')?;

    Some(value.trim().trim_matches(['"', '\'']))
}

fn same_codeset(left: &str, right: &str) -> bool {
    let key = |codeset: &str| -> String {
        codeset
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|character| character.to_ascii_lowercase())
            .collect()
    };

    key(left) == key(right)
}

fn commented(line: &str) -> String {
    let indent = line.len() - line.trim_start().len();
    let (head, tail) = line.split_at(indent);

    format!("{head}#{tail}")
}
