//! The AUR as its daily metadata dump describes it, cleaned of anything
//! that must not reach the display. `toolbox-aur` builds it, the Packages
//! page and the command line read it.

pub mod order;

use serde::{Deserialize, Serialize};
use toolbox_plan::PackageName;
use toolbox_text::OutsideText;

/// One package of the AUR. Every text came from the AUR and was stripped
/// of control characters and cut to a bounded length before it got here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AurPackage {
    pub name: PackageName,
    pub version: String,
    pub description: OutsideText,
    /// The upstream website as the PKGBUILD names it, not checked.
    pub url: Option<OutsideText>,
    pub votes: u32,
    /// The AUR's own weighting of recent votes.
    pub popularity: f64,
    /// Seconds since the epoch when someone flagged it out of date.
    pub out_of_date: Option<i64>,
    /// `None` for an orphaned package.
    pub maintainer: Option<OutsideText>,
    /// Seconds since the epoch.
    pub first_submitted: i64,
    /// Seconds since the epoch.
    pub last_modified: i64,
    pub depends: Vec<String>,
    pub make_depends: Vec<String>,
    /// As makepkg writes them: `name: what it adds`.
    pub opt_depends: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub licenses: Vec<OutsideText>,
    pub keywords: Vec<OutsideText>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AurIndex {
    pub packages: Vec<AurPackage>,
    /// Seconds since the epoch when this copy was downloaded.
    pub downloaded: i64,
    /// The last attempt to download a newer copy failed. Kept beside the
    /// copy, never in it.
    #[serde(skip)]
    pub stale: bool,
}
