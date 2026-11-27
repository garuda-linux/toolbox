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
    pub version: OutsideText,
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
    pub depends: Vec<OutsideText>,
    pub make_depends: Vec<OutsideText>,
    /// As makepkg writes them: `name: what it adds`.
    pub opt_depends: Vec<OutsideText>,
    pub provides: Vec<OutsideText>,
    pub conflicts: Vec<OutsideText>,
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

/// Whether the helper accepts this user's way to the build folder. The
/// split path needs `Accepted`. A refused setup builds through paru.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AurRoute {
    Accepted,
    Refused {
        kind: RouteKind,
        /// The problem as the helper's check words it, for `-v` and logs.
        detail: String,
    },
}

impl crate::Part for AurRoute {}

/// The refusals a text can tell apart. Every other problem is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum RouteKind {
    /// No account in `/etc/passwd` for the user, as with systemd-homed.
    NoAccount,
    /// The home or `~/.cache` is a link, or a folder on the way is missing.
    Linked,
    /// A folder on the way may be written by group or others.
    Shared,
    /// The command line starts the helper through sudo, which does not say
    /// who asked, so the split path is not used.
    ThroughSudo,
    Other,
}
