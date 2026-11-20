//! The order `paru -Po --aur <targets>` prints, read without root: repo
//! packages first, then AUR builds, each after what it needs.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use toolbox_plan::{InvalidName, PackageName};
use toolbox_text::OutsideText;

/// Why a package is in the order, in paru's words `TARGET`, `DEP` and `MAKE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Target,
    Dependency,
    /// Needed only to build, removed again after the build.
    Make,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoInstall {
    pub role: Role,
    pub name: PackageName,
}

/// One package of a build. A pkgbase making several packages gets one per package.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AurBuild {
    pub role: Role,
    pub base: PackageName,
    pub name: PackageName,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AurOrder {
    pub repo: Vec<RepoInstall>,
    /// In build order.
    pub builds: Vec<AurBuild>,
    /// Asked for or needed, but found in no repo and not in the AUR.
    pub missing: Vec<OutsideText>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidOrder {
    #[error("paru printed an incomplete order line: {0}")]
    Line(OutsideText),
    #[error(transparent)]
    Name(#[from] InvalidName),
}

/// Reads the output of `paru -Po`. Lines of other kinds are skipped, as a
/// newer paru may print some. A `REPO`, `AUR` or `MISSING` line that is not
/// whole fails the read.
///
/// # Errors
///
/// [`InvalidOrder`] for a cut line or a name pacman would refuse.
pub fn parse_order(text: &str) -> Result<AurOrder, InvalidOrder> {
    let mut order = AurOrder::default();

    for line in text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let whole = || InvalidOrder::Line(toolbox_text::plain(line));
        match fields.as_slice() {
            ["REPO", role, _repo, name] => order.repo.push(RepoInstall {
                role: role_of(role).ok_or_else(whole)?,
                name: PackageName::new(*name)?,
            }),
            ["AUR", role, base, name] => order.builds.push(AurBuild {
                role: role_of(role).ok_or_else(whole)?,
                base: PackageName::new(*base)?,
                name: PackageName::new(*name)?,
            }),
            ["MISSING", name] => order.missing.push(toolbox_text::plain(name)),
            ["REPO" | "AUR" | "MISSING", ..] => return Err(whole()),
            _ => {}
        }
    }

    Ok(order)
}

fn role_of(word: &str) -> Option<Role> {
    match word {
        "TARGET" => Some(Role::Target),
        "DEP" => Some(Role::Dependency),
        "MAKE" => Some(Role::Make),
        _ => None,
    }
}
