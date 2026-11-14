//! Where the Boot page's read of the boot menu stands. Page state, so it
//! lives here and not in `toolbox-system`, which only reads the menu.

#[cfg(test)]
mod tests;

use toolbox_plan::sdboot::SdBootMenu;
use toolbox_plan::{GrubMenu, PackageName};
use toolbox_system::MenuRead;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Menu {
    #[default]
    NotAsked,
    Loading,
    Read(MenuRead),
}

impl Menu {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotAsked => "not-asked",
            Self::Loading => "loading",
            Self::Read(MenuRead::Known(_) | MenuRead::SdBoot(_)) => "known",
            Self::Read(MenuRead::Missing) => "missing",
            Self::Read(MenuRead::Refused(refusal)) => refusal.code(),
            Self::Read(MenuRead::Unknown(_) | MenuRead::Unreadable(_) | MenuRead::Cancelled) => {
                "unknown"
            }
        }
    }

    pub const fn known(&self) -> Option<&GrubMenu> {
        match self {
            Self::Read(MenuRead::Known(menu)) => Some(menu),
            Self::NotAsked
            | Self::Loading
            | Self::Read(
                MenuRead::SdBoot(_)
                | MenuRead::Missing
                | MenuRead::Refused(_)
                | MenuRead::Unknown(_)
                | MenuRead::Unreadable(_)
                | MenuRead::Cancelled,
            ) => None,
        }
    }

    pub const fn sdboot(&self) -> Option<&SdBootMenu> {
        match self {
            Self::Read(MenuRead::SdBoot(menu)) => Some(menu),
            Self::NotAsked
            | Self::Loading
            | Self::Read(
                MenuRead::Known(_)
                | MenuRead::Missing
                | MenuRead::Refused(_)
                | MenuRead::Unknown(_)
                | MenuRead::Unreadable(_)
                | MenuRead::Cancelled,
            ) => None,
        }
    }

    /// The id of `kernel`'s menu entry, `""` while unknown.
    pub fn kernel_entry(&self, kernel: &PackageName) -> &str {
        self.known()
            .and_then(|menu| menu.kernel_entry(kernel))
            .map_or("", |entry| entry.id.as_str())
    }
}
