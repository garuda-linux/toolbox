#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::{KernelRelease, LoaderDefault, LoaderEntryId};

const MAX_TEXT_CHARS: usize = 512;

/// A snapshot entry goes when snapper deletes the snapshot, so it never
/// becomes the default.
const SNAPSHOT_SORT_KEY: &str = "snapper-sdboot";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntryKind {
    Uki,
    TypeOne,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderEntry {
    pub id: LoaderEntryId,
    pub title: String,
    pub version: Option<String>,
    pub uname: Option<KernelRelease>,
    pub sort_key: Option<String>,
    pub kind: EntryKind,
}

impl LoaderEntry {
    pub fn uki(file: &str, os_release: &str, uname: Option<&str>) -> Option<Self> {
        let id = entry_id(file)?;
        let field = |name: &str| os_release_value(os_release, name);
        let title = field("PRETTY_NAME")
            .or_else(|| field("NAME"))
            .or_else(|| field("ID"))
            .unwrap_or_else(|| id.as_str().to_owned());

        Some(Self {
            title: shown(&title),
            version: field("VERSION_ID")
                .or_else(|| field("BUILD_ID"))
                .map(|value| shown(&value)),
            uname: uname.and_then(|value| KernelRelease::new(value.trim()).ok()),
            sort_key: field("IMAGE_ID")
                .or_else(|| field("ID"))
                .map(|value| shown(&value)),
            kind: EntryKind::Uki,
            id,
        })
    }

    pub fn type_one(file: &str, text: &str) -> Option<Self> {
        let id = entry_id(file)?;
        let field = |name: &str| {
            text.lines()
                .filter_map(|line| {
                    line.trim()
                        .split_once(|character: char| character.is_ascii_whitespace())
                })
                .filter(|(key, _)| *key == name)
                .map(|(_, value)| shown(value.trim()))
                .next_back()
        };

        Some(Self {
            title: field("title").unwrap_or_else(|| id.as_str().to_owned()),
            version: field("version"),
            uname: None,
            sort_key: field("sort-key"),
            kind: EntryKind::TypeOne,
            id,
        })
    }

    pub fn is_snapshot(&self) -> bool {
        self.sort_key.as_deref() == Some(SNAPSHOT_SORT_KEY)
    }

    /// The kernel's version is part of a UKI's id and changes with every
    /// update, its flavour (`garuda`, `lts`) does not. A glob over the
    /// version keeps the default on the same kernel.
    pub fn default_pattern(&self) -> LoaderDefault {
        let exact = || LoaderDefault::from(&self.id);

        let Some(release) = &self.uname else {
            return exact();
        };

        let flavour = release
            .as_str()
            .rsplit_once('-')
            .map(|(_, last)| last)
            .filter(|last| last.starts_with(|first: char| first.is_ascii_alphabetic()));

        let around = self.id.as_str().split_once(release.as_str());

        match (flavour, around) {
            (Some(flavour), Some((before, after))) => {
                LoaderDefault::new(format!("{before}*-{flavour}{after}"))
                    .unwrap_or_else(|_| exact())
            }
            (None | Some(_), None) | (None, Some(_)) => exact(),
        }
    }
}

/// Without the boot counter (`+3`, `+2-1`), as `bootctl` names it: the
/// counter changes with every boot attempt.
pub fn entry_id(file: &str) -> Option<LoaderEntryId> {
    let (stem, kind) = file
        .strip_suffix(".efi")
        .map(|stem| (stem, ".efi"))
        .or_else(|| file.strip_suffix(".conf").map(|stem| (stem, ".conf")))?;

    let stem = without_counter(stem);

    if stem.is_empty() {
        return None;
    }

    LoaderEntryId::new(format!("{stem}{kind}")).ok()
}

fn without_counter(stem: &str) -> &str {
    let Some((name, tries)) = stem.rsplit_once('+') else {
        return stem;
    };

    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());

    let numbered = match tries.split_once('-') {
        Some((left, done)) => digits(left) && digits(done),
        None => digits(tries),
    };

    if numbered { name } else { stem }
}

fn os_release_value(text: &str, name: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.trim().split_once('='))
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| {
            value
                .trim()
                .trim_matches(|quote| quote == '"' || quote == '\'')
                .to_owned()
        })
        .rfind(|value| !value.is_empty())
}

fn shown(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .take(MAX_TEXT_CHARS)
        .collect()
}
