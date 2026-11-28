//! The portal's settings as the look takes them. The portal itself is
//! `toolbox_system::LookPortal`.

use toolbox_gui_kit::desktop::look::Portal;
use toolbox_system::PortalSettings;

/// A setting the portal lacks stays unset, and an accent outside the unit
/// range counts as unset too.
pub fn look(settings: PortalSettings) -> Portal {
    Portal {
        dark: settings.dark,
        accent: settings
            .accent
            .and_then(|(red, green, blue)| Portal::accent_from_unit_rgb(red, green, blue)),
        animations: settings.animations,
    }
}
