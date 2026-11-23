pub const BREEZEDARK_COLORS: &str = include_str!("../../desktop/kde/fixtures/BreezeDark.colors");
pub const INSTALLED_KDEGLOBALS: &str =
    include_str!("../../desktop/kde/fixtures/installed/kdeglobals");
/// `~/.config/kdedefaults/kdeglobals` of a Plasma 6 desktop with a global
/// theme: the layer `KConfig` reads under the user's own `kdeglobals`.
pub const KDEDEFAULTS_KDEGLOBALS: &str =
    include_str!("../../desktop/kde/fixtures/bare-metal/kdedefaults-kdeglobals");
