//! What the window adds to a page: its icon, the hue of its section, and
//! whether the sidebar lists it this frame. [`PageId`] itself stays free
//! of egui, so the command line can name pages.

#[cfg(test)]
mod tests;

use toolbox_gui_kit::icons::{self, Icon, colour};
use toolbox_gui_kit::tokens::Hue;

use crate::page::{Group, PageId};

/// The sidebar's Search entry. It sits among the pages, so its glyph
/// belongs to no page.
pub const SEARCH_ICON: Icon = icons::EDIT_FIND;

pub trait PageLook {
    fn icon(self) -> Icon;
}

pub trait GroupLook {
    fn hue(self) -> Hue;
}

impl PageLook for PageId {
    fn icon(self) -> Icon {
        match self {
            Self::Home => icons::GO_HOME,
            Self::Setup => colour::SYSTEM_SOFTWARE_INSTALL,
            Self::Components => colour::PREFERENCES_DESKTOP_PERIPHERALS,
            Self::Gaming => colour::PREFERENCES_DESKTOP_GAMING,
            Self::Discover => colour::PLASMADISCOVER,
            Self::Packages => colour::PACKAGE_X_GENERIC,
            Self::Services => colour::PREFERENCES_SYSTEM_SERVICES,
            Self::SystemSettings => colour::PREFERENCES_SYSTEM,
            Self::Kernels => colour::PREFERENCES_SYSTEM_LINUX,
            Self::Locales => colour::PREFERENCES_DESKTOP_LOCALE,
            Self::Users => colour::PREFERENCES_SYSTEM_USERS,
            Self::Configs => colour::PREFERENCES_OTHER,
            Self::Network => colour::PREFERENCES_SYSTEM_NETWORK,
            Self::Boot => colour::PREFERENCES_SYSTEM_STARTUP,
            Self::Maintenance => colour::APPLICATIONS_UTILITIES,
            Self::BootRepair => colour::DRIVE_HARDDISK_ROOT,
            Self::Diagnostics => colour::HWINFO,
            Self::Settings => icons::CONFIGURE,
            Self::Guide => icons::HELP_CONTENTS,
            Self::About => icons::HELP_ABOUT,
        }
    }
}

impl GroupLook for Group {
    fn hue(self) -> Hue {
        match self {
            Self::Start | Self::App => Hue::Accent,
            Self::Install => Hue::Blue,
            Self::System => Hue::Green,
            Self::Care => Hue::Orange,
        }
    }
}

/// Whether the sidebar, the palette and the shortcuts offer `page`. Boot
/// Repair only shows where the app said so this frame: on the live ISO or
/// with `--boot-repair`.
pub fn listed(ctx: &egui::Context, page: PageId) -> bool {
    page != PageId::BootRepair
        || ctx
            .data(|data| data.get_temp::<bool>(boot_repair_id()))
            .unwrap_or(false)
}

pub fn offer_boot_repair(ctx: &egui::Context, offered: bool) {
    ctx.data_mut(|data| data.insert_temp(boot_repair_id(), offered));
}

fn boot_repair_id() -> egui::Id {
    egui::Id::unique("toolbox_gui::page::boot_repair")
}
