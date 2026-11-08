//! The metadata dump: a gzipped JSON array of every package. Read as a
//! stream, and each entry is cleaned the moment it is read, so only the
//! trimmed packages stay in memory, never the whole dump.

#[cfg(test)]
pub mod tests;

use std::fmt;
use std::io::{BufReader, Read};

use flate2::read::MultiGzDecoder;
use serde::Deserialize;
use serde::de::{Deserializer, SeqAccess, Visitor};
use toolbox_plan::PackageName;
use toolbox_state::aur::AurPackage;

use crate::clean::{
    DESCRIPTION_CHARS, MAINTAINER_CHARS, URL_CHARS, VERSION_CHARS, list, optional, plain,
};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DumpError {
    #[error("the package list is larger than {0} bytes unpacked")]
    TooLarge(u64),
    #[error("the package list is not understood: {0}")]
    Malformed(String),
}

/// The fields of one entry the app keeps. Every other field is skipped
/// while it is read. Missing and `null` read as empty.
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Raw {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    #[serde(rename = "URL")]
    url: Option<String>,
    num_votes: Option<u64>,
    popularity: Option<f64>,
    out_of_date: Option<i64>,
    maintainer: Option<String>,
    first_submitted: Option<i64>,
    last_modified: Option<i64>,
    depends: Option<Vec<String>>,
    make_depends: Option<Vec<String>>,
    opt_depends: Option<Vec<String>>,
    provides: Option<Vec<String>>,
    conflicts: Option<Vec<String>>,
    license: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
}

struct Packages(Vec<AurPackage>);

struct PackagesVisitor;

impl<'de> Deserialize<'de> for Packages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PackagesVisitor)
    }
}

impl<'de> Visitor<'de> for PackagesVisitor {
    type Value = Packages;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of AUR packages")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Packages, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut packages = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        while let Some(raw) = seq.next_element::<Raw>()? {
            packages.extend(cleaned(raw));
        }

        Ok(Packages(packages))
    }
}

/// # Errors
///
/// [`DumpError`] when the stream is no gzipped JSON array of packages or
/// unpacks to more than `max_bytes`.
pub fn parse(gzipped: impl Read, max_bytes: u64) -> Result<Vec<AurPackage>, DumpError> {
    // One byte over the limit tells a dump of exactly the limit from a
    // larger one.
    let mut unpacked = MultiGzDecoder::new(gzipped).take(max_bytes.saturating_add(1));
    let read = serde_json::from_reader::<_, Packages>(BufReader::new(&mut unpacked));

    if unpacked.limit() == 0 {
        return Err(DumpError::TooLarge(max_bytes));
    }

    let Packages(mut packages) = read.map_err(|error| DumpError::Malformed(error.to_string()))?;
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    packages.dedup_by(|later, earlier| later.name == earlier.name);
    Ok(packages)
}

/// name, which no command line and no list ever sees.
fn cleaned(raw: Raw) -> Option<AurPackage> {
    let name = PackageName::new(raw.name?).ok()?;
    let texts = |values: Option<Vec<String>>| list(&values.unwrap_or_default());

    Some(AurPackage {
        name,
        version: plain(raw.version.as_deref().unwrap_or_default(), VERSION_CHARS),
        description: plain(
            raw.description.as_deref().unwrap_or_default(),
            DESCRIPTION_CHARS,
        ),
        url: optional(raw.url.as_deref(), URL_CHARS),
        votes: raw
            .num_votes
            .map_or(0, |votes| u32::try_from(votes).unwrap_or(u32::MAX)),
        popularity: raw
            .popularity
            .filter(|popularity| popularity.is_finite() && *popularity >= 0.0_f64)
            .unwrap_or_default(),
        out_of_date: raw.out_of_date,
        maintainer: optional(raw.maintainer.as_deref(), MAINTAINER_CHARS),
        first_submitted: raw.first_submitted.unwrap_or_default(),
        last_modified: raw.last_modified.unwrap_or_default(),
        depends: texts(raw.depends),
        make_depends: texts(raw.make_depends),
        opt_depends: texts(raw.opt_depends),
        provides: texts(raw.provides),
        conflicts: texts(raw.conflicts),
        licenses: texts(raw.license),
        keywords: texts(raw.keywords),
    })
}
