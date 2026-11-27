//! AUR packages: paru fetches, makepkg builds as the user, the helper installs.
//! No path crosses to root, so the helper joins these folders to the user
//! who owns its stdin socket.

use serde::{Deserialize, Serialize};

/// Below the user's home: the app's own clone of each pkgbase.
///
/// paru fetches into it and makepkg builds in it, so it holds what the user
/// reviewed. Only the user may write it, which the review and the build check.
pub const AUR_CLONE: &str = ".cache/garuda-toolbox/aur-clone";

/// Below the user's home: one folder per build run, makepkg's `PKGDEST`.
/// The helper cannot read the user's `XDG_CACHE_HOME`.
pub const AUR_BUILT: &str = ".cache/garuda-toolbox/aur-built";

/// The options of an AUR build's makepkg.
///
/// They rebuild over a package file that exists and clean the source folder
/// first. Dependencies, installing and cleaning up are the helper's, so
/// `-s`, `-i`, `-c` and `-r` stay out.
pub const MAKEPKG_ARGS: [&str; 2] = ["-f", "-C"];

/// What the review step's command says after the folder it works in. The
/// Console finds the review by this end of the text, so the shown text and
/// that check both take it from here.
pub const REVIEW_TAIL: &str = ", then show the build files and ask before any build";

/// Relative to `/`: where the helper copies built packages before
/// `pacman -U`, so no other user can swap a file after the check.
pub const AUR_STORE: &str = "var/cache/garuda-toolbox/aur";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallReason {
    Explicit,
    Dependency,
    /// An upgrade of an installed package keeps the reason pacman has for it.
    Keep,
}

impl InstallReason {
    pub const fn pacman_flag(self) -> Option<&'static str> {
        match self {
            Self::Explicit => Some("--asexplicit"),
            Self::Dependency => Some("--asdeps"),
            Self::Keep => None,
        }
    }
}
