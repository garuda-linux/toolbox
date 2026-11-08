use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::{CatalogError, Problem};

/// A catalog file of titled sections, each with entries of the schema `E`.
///
/// The bounds are spelled out because serde would otherwise demand
/// `E: Default` for the defaulted lists, which no entry schema is.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "E: Deserialize<'de>"))]
pub struct RawCatalog<E> {
    #[serde(default)]
    pub section: Vec<RawSection<E>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "E: Deserialize<'de>"))]
pub struct RawSection<E> {
    pub id: String,
    pub title: String,
    pub icon: String,
    #[serde(default)]
    pub entry: Vec<E>,
}

pub fn read(dir: &Path, file: &str) -> Result<(String, String), CatalogError> {
    let path = dir.join(file);

    match fs::read_to_string(&path) {
        Ok(text) => Ok((text, path.display().to_string())),
        Err(error) => Err(CatalogError::Read { path, error }),
    }
}

pub fn parse<T>(text: &str, source_name: &str) -> Result<T, CatalogError>
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str(text).map_err(|error| CatalogError::Syntax {
        source_name: source_name.to_owned(),
        message: error.to_string(),
    })
}

/// Whether `id` is lowercase words of ASCII letters and digits joined by
/// single dashes.
pub fn is_valid_id(id: &str) -> bool {
    id.split('-').all(|word| {
        !word.is_empty()
            && word
                .chars()
                .all(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit())
    })
}

/// Claims `id` for one section or entry of a file. Sections and entries
/// share one namespace per file, so an id names one thing in self-tests
/// and translations.
pub fn claim_id(id: &str, seen: &mut BTreeSet<String>) -> Result<(), Problem> {
    if !is_valid_id(id) {
        return Err(Problem::BadId);
    }

    if !seen.insert(id.to_owned()) {
        return Err(Problem::DuplicateId);
    }

    Ok(())
}

pub fn check_section(title: &str, icon: &str, entries: usize) -> Result<(), Problem> {
    if is_blank(title) {
        return Err(Problem::EmptyTitle);
    }

    if is_blank(icon) {
        return Err(Problem::EmptyIcon);
    }

    if entries == 0 {
        return Err(Problem::NoEntries);
    }

    Ok(())
}

pub fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}
