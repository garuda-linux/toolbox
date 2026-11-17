use toolbox_plan::PackageName;

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reveal {
    /// The details of a package, its AUR side when `aur`.
    Package { name: PackageName, aur: bool },
    /// The repositories' packages, or the AUR's when `aur`, searched.
    Packages { search: String, aur: bool },
}

impl From<Reveal> for Command {
    fn from(reveal: Reveal) -> Self {
        Self::App(AppCommand::Reveal(reveal))
    }
}
