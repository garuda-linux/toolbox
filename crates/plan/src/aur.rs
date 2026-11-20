//! AUR packages: paru resolves and fetches, makepkg builds as the user, the
//! helper installs the built files. No path crosses to root: the helper
//! joins these folders to the home of the user who owns its stdin socket.

use serde::{Deserialize, Serialize};

/// Below the user's home: paru's clone of each pkgbase.
pub const PARU_CLONE: &str = ".cache/paru/clone";

/// Below the user's home: one folder per build run, makepkg's `PKGDEST`.
/// The helper cannot read the user's `XDG_CACHE_HOME`.
pub const AUR_BUILT: &str = ".cache/garuda-toolbox/aur-built";

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
