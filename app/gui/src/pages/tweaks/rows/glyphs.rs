//! The glyph on each lead row's tile. A glyph means one row, so no two rows
//! share one and none is a section's or a page's.

use toolbox_gui_kit::icons::{self, Icon};
use toolbox_plan::GarudaUpdateFlag;

use super::Choice;

impl Choice {
    pub const fn glyph(self) -> Icon {
        match self {
            Self::Shell => icons::DIALOG_SCRIPTS,
            Self::Dns => icons::NETWORK_SERVER,
            Self::Wifi => icons::NETWORK_WIRELESS_SIGNAL_GOOD,
            Self::Hblock => icons::DIALOG_CANCEL,
        }
    }
}

pub(super) const fn flag_glyph(flag: GarudaUpdateFlag) -> Icon {
    match flag {
        GarudaUpdateFlag::SkipMirrorlist => icons::VIEW_LIST_DETAILS,
        GarudaUpdateFlag::UpdateAur => icons::APPLICATIONS_DEVELOPMENT,
        GarudaUpdateFlag::PacmanNoconfirm => icons::DIALOG_OK,
        GarudaUpdateFlag::NoCheckSpace => icons::DRIVE_HARDDISK,
    }
}
