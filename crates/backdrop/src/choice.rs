//! Which blur the window asks for. Pure, so the compositor's offers can be
//! faked in tests.

use std::fmt;

/// The bit of `ext_background_effect_manager_v1`'s `capability` enum that
/// means the compositor blurs.
pub const BLUR_CAPABILITY: u32 = 1;

/// What the compositor confirms for the window through a protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offers {
    /// `ext_background_effect_manager_v1` with the blur capability.
    pub extension: bool,
    /// `org_kde_kwin_blur_manager`, KWin before Plasma 6.7.
    pub kde: bool,
    /// KWin on X11, which blurs behind a window with its property.
    pub x11_kwin: bool,
}

/// The way the window's background is blurred, or why it is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    /// `ext_background_effect_v1` on the window's surface.
    Extension,
    /// `org_kde_kwin_blur` on the window's surface.
    KdeBlur,
    /// `_KDE_NET_WM_BLUR_BEHIND_REGION` on the window.
    X11Kwin,
    /// Opaque, because no blur is confirmed.
    Opaque,
}

impl Path {
    /// Whether the window's background may be translucent on this path.
    #[must_use = "the answer decides whether the window is translucent"]
    pub const fn translucent(self) -> bool {
        !matches!(self, Self::Opaque)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Extension => "ext-background-effect-v1",
            Self::KdeBlur => "org_kde_kwin_blur",
            Self::X11Kwin => "_KDE_NET_WM_BLUR_BEHIND_REGION",
            Self::Opaque => "opaque",
        })
    }
}

/// The first confirmed path, in the order the protocols are preferred: the
/// standard one, then KWin's own.
#[must_use = "the path decides the window background"]
pub const fn decide(offers: Offers) -> Path {
    if offers.extension {
        Path::Extension
    } else if offers.kde {
        Path::KdeBlur
    } else if offers.x11_kwin {
        Path::X11Kwin
    } else {
        Path::Opaque
    }
}

/// The offers of a Wayland compositor that advertises the `interfaces`.
/// `capabilities` is the flag set the background effect manager sent when
/// it was bound, `None` before that.
#[must_use = "the offers decide the path"]
pub fn wayland_offers(interfaces: &[impl AsRef<str>], capabilities: Option<u32>) -> Offers {
    let has = |name: &str| {
        interfaces
            .iter()
            .any(|interface| interface.as_ref() == name)
    };

    // Both protocols attach to the window's own surface, so the compositor
    // global that makes regions has to be there too.
    let region = has("wl_compositor");
    let blur_capable = capabilities.is_some_and(|flags| flags & BLUR_CAPABILITY != 0);

    Offers {
        extension: region && has("ext_background_effect_manager_v1") && blur_capable,
        kde: region && has("org_kde_kwin_blur_manager"),
        ..Offers::default()
    }
}
