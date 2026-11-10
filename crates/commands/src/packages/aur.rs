//! `packages search --aur`, `show --aur` and `install --aur`: the AUR's
//! copy of its daily dump, searched in the page's order, one package's
//! facts, and builds through paru as the page queues them.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use serde::Serialize;
use toolbox_aur::{AurError, AurSource, package_page};
use toolbox_cli::output::{Mark, Paint, Row};
use toolbox_core::features::packages::aur::{AurCatalog, AurPackage, row_state, toggle};
use toolbox_core::features::packages::queue::Refusal;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::super::change::Refused;
use super::super::out::Listing;
use super::{facts_text, listed};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AurRow {
    #[serde(rename = "id")]
    pub name: String,
    pub description: String,
    pub version: String,
    pub votes: u32,
    pub popularity: f64,
    /// Seconds since the epoch when it was flagged out of date.
    pub out_of_date: Option<i64>,
    pub installed: bool,
}

/// One AUR package as `packages show --aur` prints it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AurInfo {
    #[serde(rename = "id")]
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: Option<String>,
    pub votes: u32,
    pub popularity: f64,
    pub out_of_date: Option<i64>,
    pub maintainer: Option<String>,
    /// Seconds since the epoch.
    pub first_submitted: i64,
    /// Seconds since the epoch.
    pub last_modified: i64,
    pub depends: Vec<String>,
    pub make_depends: Vec<String>,
    pub opt_depends: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub licenses: Vec<String>,
    pub keywords: Vec<String>,
    pub aur_page: String,
    pub installed: bool,
}

impl AurRow {
    fn of(package: &AurPackage, installed: &BTreeSet<PackageName>) -> Self {
        Self {
            name: package.name.as_str().to_owned(),
            description: package.description.clone(),
            version: package.version.clone(),
            votes: package.votes,
            popularity: package.popularity,
            out_of_date: package.out_of_date,
            installed: installed.contains(&package.name),
        }
    }
}

impl AurInfo {
    fn of(package: AurPackage, installed: &BTreeSet<PackageName>) -> Self {
        Self {
            aur_page: package_page(&package.name),
            installed: installed.contains(&package.name),
            name: package.name.as_str().to_owned(),
            version: package.version,
            description: package.description,
            url: package.url,
            votes: package.votes,
            popularity: package.popularity,
            out_of_date: package.out_of_date,
            maintainer: package.maintainer,
            first_submitted: package.first_submitted,
            last_modified: package.last_modified,
            depends: package.depends,
            make_depends: package.make_depends,
            opt_depends: package.opt_depends,
            provides: package.provides,
            conflicts: package.conflicts,
            licenses: package.licenses,
            keywords: package.keywords,
        }
    }
}

/// The AUR packages `text` matches, the exact name first, then names that
/// start with it, then the rest, each part the most popular first.
///
/// # Errors
///
/// [`AurError`] when there is no copy of the AUR.
pub fn found(
    text: Option<&str>,
    aur: &dyn AurSource,
    installed: &BTreeSet<PackageName>,
) -> Result<Vec<AurRow>, AurError> {
    let catalog = AurCatalog::new(aur.index()?);
    let entries = catalog.entries();

    Ok(catalog
        .view(text.unwrap_or_default())
        .into_iter()
        .filter_map(|place| entries.get(place))
        .map(|package| AurRow::of(package, installed))
        .collect())
}

/// One AUR package, `None` when the AUR has none of that name.
///
/// # Errors
///
/// [`AurError`] when there is no copy of the AUR.
pub fn aur_info(
    package: &PackageName,
    aur: &dyn AurSource,
    installed: &BTreeSet<PackageName>,
) -> Result<Option<AurInfo>, AurError> {
    let index = aur.index()?;

    Ok(index
        .packages
        .into_iter()
        .find(|entry| entry.name == *package)
        .map(|entry| AurInfo::of(entry, installed)))
}

/// A paru build of every package of `packages`, as the page queues it.
///
/// # Errors
///
/// [`Refused`] without a copy of the AUR, for a name the AUR does not
/// have, a package installed already, or a system whose packages could
/// not be read.
pub fn install_onto(
    packages: &[PackageName],
    aur: &dyn AurSource,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let catalog = AurCatalog::new(
        aur.index()
            .map_err(|error| Refused::Unavailable(error.to_string()))?,
    );

    // Every name is checked before the first is queued, so a refused
    // command leaves no part of itself behind.
    for package in packages {
        if catalog.entry(package).is_none() {
            let known = catalog.entries().iter().map(|entry| entry.name.as_str());
            return Err(Refused::Unknown(format!(
                "`{package}` is not in the AUR, see `packages search --aur`"
            ))
            .hinted(package.as_str(), known));
        }

        if row_state(package, snapshot, changes).installed {
            return Err(Refused::NothingToDo(format!(
                "`{package}` is installed already"
            )));
        }
    }

    for package in packages {
        toggle(package, Toggle::On, snapshot, changes).map_err(|refusal| match refusal {
            Refusal::Unread | Refusal::NotOffered => {
                Refused::Unavailable("pacman's local database could not be read".to_owned())
            }
            kept @ (Refusal::RunningKernel
            | Refusal::RunningUnknown
            | Refusal::LastKernel
            | Refusal::Protected(_)
            | Refusal::TakesAlong(..)) => super::kept_package(package, kept),
        })?;
    }

    Ok(())
}

/// The AUR packages, on when installed, with the command that shows one.
pub fn aur_listing(rows: &[AurRow]) -> Listing<'_> {
    let installed = rows.iter().filter(|row| row.installed).count();

    Listing {
        title: "AUR packages",
        summary: Some(format!("{}, {installed} installed", rows.len())),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let mark = if row.installed { Mark::On } else { Mark::Off };
                let votes = format!("{} votes", row.votes);
                let flagged = if row.out_of_date.is_some() {
                    "out of date"
                } else {
                    ""
                };

                let cells = [
                    row.name.as_str(),
                    row.version.as_str(),
                    votes.as_str(),
                    flagged,
                    row.description.as_str(),
                ];

                Row::new(mark, cells.map(str::to_owned).to_vec())
            })
            .collect(),
        hint: (!rows.is_empty()).then(|| "garuda-toolbox packages show --aur <package>".to_owned()),
    }
}

/// The facts of one AUR package, the lists joined by commas.
pub fn aur_info_text(paint: Paint, info: &AurInfo) -> String {
    let none = || "none".to_owned();
    let facts: Vec<(&str, String)> = vec![
        ("description", info.description.clone()),
        ("version", info.version.clone()),
        (
            "installed",
            if info.installed { "yes" } else { "no" }.to_owned(),
        ),
        ("maintainer", info.maintainer.clone().unwrap_or_else(none)),
        ("votes", info.votes.to_string()),
        ("popularity", format!("{:.2}", info.popularity)),
        ("first submitted", day(info.first_submitted)),
        ("last updated", day(info.last_modified)),
        (
            "out of date",
            info.out_of_date.map_or_else(|| "no".to_owned(), day),
        ),
        ("licenses", listed(&info.licenses)),
        ("website", info.url.clone().unwrap_or_else(none)),
        ("aur page", info.aur_page.clone()),
        ("depends on", listed(&info.depends)),
        ("make dependencies", listed(&info.make_depends)),
        ("optional", listed(&info.opt_depends)),
        ("provides", listed(&info.provides)),
        ("conflicts with", listed(&info.conflicts)),
        ("keywords", listed(&info.keywords)),
    ];

    facts_text(paint, &info.name, &facts)
}

/// Why there is no copy yet, with the day of the next allowed download.
pub fn waiting_text(next: i64) -> String {
    format!(
        "the AUR's package list could not be downloaded, the next try is on {} (UTC)",
        moment(next)
    )
}

/// `2025-01-10`, in UTC.
fn day(seconds: i64) -> String {
    stamp(seconds, "%Y-%m-%d")
}

/// `2025-01-10 14:30`, in UTC.
fn moment(seconds: i64) -> String {
    stamp(seconds, "%Y-%m-%d %H:%M")
}

fn stamp(seconds: i64, format: &str) -> String {
    jiff::Timestamp::from_second(seconds).map_or_else(
        |_out_of_range| seconds.to_string(),
        |time| time.strftime(format).to_string(),
    )
}
