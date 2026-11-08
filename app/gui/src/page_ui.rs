//! What the window adds to a page: its icon, the hue of its section, and
//! whether the sidebar lists it this frame. [`PageId`] itself stays free
//! of egui, so the command line can name pages.

#[cfg(test)]
mod tests;

use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::tokens::Hue;

use crate::page::{Group, PageId};

/// The sidebar's Search entry. It sits among the pages, so its glyph
/// belongs to no page.
pub const SEARCH_ICON: Icon = icons::SYSTEM_SEARCH;

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
            Self::Setup => icons::TOOLS_WIZARD,
            Self::Components => icons::PLUGINS,
            Self::Gaming => icons::INPUT_GAMEPAD,
            Self::Discover => icons::VIEW_LIST_ICONS,
            Self::Packages => icons::REPOSITORY,
            Self::Services => icons::SYSTEM_RUN,
            Self::SystemSettings => icons::VIEW_MEDIA_EQUALIZER,
            Self::Kernels => icons::RUN_BUILD,
            Self::Locales => icons::LANGUAGES,
            Self::Users => icons::USER_OTHERS,
            Self::Configs => icons::DOCUMENT_PROPERTIES,
            Self::Network => icons::NETWORK_CONNECT,
            Self::Boot => icons::DRIVE_HARDDISK_ROOT,
            Self::Maintenance => icons::TOOLS,
            Self::BootRepair => icons::RUN_BUILD_CONFIGURE,
            Self::Diagnostics => icons::OFFICE_REPORT,
            Self::Settings => icons::CONFIGURE,
            Self::Guide => icons::HELP_CONTENTS,
            Self::About => icons::VIEW_CERTIFICATE,
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
