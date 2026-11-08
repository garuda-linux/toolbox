//! What `packages search` and `packages show` print: a row per package,
//! and one package's facts. JSON keeps sizes in bytes and dates in
//! seconds, the text shows them as people read them.

use serde::Serialize;
use toolbox_cli::output::{Mark, Row, size};
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, OptionalDependency, Origin, PackageDetails,
};

use super::super::out::Listing;
use super::{facts_text, listed};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackageRow {
    #[serde(rename = "id")]
    pub name: String,
    pub description: String,
    /// The repository's version, the installed one for a foreign package.
    pub version: String,
    /// `None` for a package no configured repository offers.
    pub repository: Option<String>,
    /// Bytes once installed.
    pub size: u64,
    pub installed: Option<InstalledRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstalledRow {
    pub version: String,
    /// `explicit` or `dependency`.
    pub reason: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackageInfo {
    #[serde(rename = "id")]
    pub name: String,
    pub description: String,
    pub version: String,
    pub repository: Option<String>,
    pub size: u64,
    pub url: Option<String>,
    pub licenses: Vec<String>,
    pub groups: Vec<String>,
    pub depends: Vec<String>,
    pub optional: Vec<OptionalRow>,
    pub required_by: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub installed: Option<InstalledInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OptionalRow {
    #[serde(rename = "id")]
    pub name: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstalledInfo {
    pub version: String,
    pub reason: &'static str,
    /// Seconds since the epoch.
    pub date: Option<i64>,
    pub files: Vec<String>,
}

impl PackageRow {
    pub fn of(entry: &IndexEntry) -> Self {
        Self {
            name: entry.name.as_str().to_owned(),
            description: entry.description.clone(),
            version: entry.version.clone(),
            repository: repository(&entry.origin),
            size: entry.size,
            installed: entry.installed.as_ref().map(|installed| InstalledRow {
                version: installed.version.clone(),
                reason: reason(installed.reason),
            }),
        }
    }
}

impl PackageInfo {
    pub fn of(details: PackageDetails) -> Self {
        Self {
            name: details.name.as_str().to_owned(),
            description: details.description,
            version: details.version,
            repository: repository(&details.origin),
            size: details.size,
            url: details.url,
            licenses: details.licenses,
            groups: details.groups,
            depends: details.depends,
            optional: details
                .optional
                .into_iter()
                .map(|OptionalDependency { name, reason }| OptionalRow { name, reason })
                .collect(),
            required_by: details.required_by,
            provides: details.provides,
            conflicts: details.conflicts,
            replaces: details.replaces,
            installed: details.installed.map(|installed| InstalledInfo {
                version: installed.version,
                reason: reason(installed.reason),
                date: installed.date,
                files: installed.files,
            }),
        }
    }
}

/// The packages, on when installed, with the command that shows one.
pub fn listing(rows: &[PackageRow]) -> Listing<'_> {
    let installed = rows.iter().filter(|row| row.installed.is_some()).count();

    Listing {
        title: "Packages",
        summary: Some(format!("{}, {installed} installed", rows.len())),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let mark = if row.installed.is_some() {
                    Mark::On
                } else {
                    Mark::Off
                };

                let version = row
                    .installed
                    .as_ref()
                    .map_or(row.version.as_str(), |own| own.version.as_str());

                let cells = [
                    row.name.as_str(),
                    version,
                    row.repository.as_deref().unwrap_or("no repository"),
                    &row.description,
                ];

                Row::new(mark, cells.map(str::to_owned).to_vec())
            })
            .collect(),
        hint: (!rows.is_empty()).then(|| "garuda-toolbox packages show <package>".to_owned()),
    }
}

/// The facts of one package, the lists joined by commas.
pub fn info_text(info: &PackageInfo) -> String {
    let optional: Vec<String> = info
        .optional
        .iter()
        .map(|dependency| format!("{}: {}", dependency.name, dependency.reason))
        .collect();

    let installed = info.installed.as_ref();
    let facts: Vec<(&str, String)> = vec![
        ("description", info.description.clone()),
        (
            "repository",
            info.repository.clone().unwrap_or_else(|| "none".to_owned()),
        ),
        ("version", info.version.clone()),
        (
            "installed",
            installed.map_or_else(
                || "no".to_owned(),
                |installed| format!("{} ({})", installed.version, installed.reason),
            ),
        ),
        ("size", size(info.size)),
        (
            "website",
            info.url.clone().unwrap_or_else(|| "none".to_owned()),
        ),
        ("licenses", listed(&info.licenses)),
        ("groups", listed(&info.groups)),
        ("depends on", listed(&info.depends)),
        ("optional", listed(&optional)),
        ("required by", listed(&info.required_by)),
        ("provides", listed(&info.provides)),
        ("conflicts with", listed(&info.conflicts)),
        ("replaces", listed(&info.replaces)),
        (
            "files",
            installed.map_or_else(
                || "none".to_owned(),
                |installed| installed.files.len().to_string(),
            ),
        ),
    ];

    facts_text(&info.name, &facts)
}

fn repository(origin: &Origin) -> Option<String> {
    match origin {
        Origin::Repository(repo) => Some(repo.clone()),
        Origin::Foreign => None,
    }
}

const fn reason(reason: InstallReason) -> &'static str {
    match reason {
        InstallReason::Explicit => "explicit",
        InstallReason::Dependency => "dependency",
    }
}
