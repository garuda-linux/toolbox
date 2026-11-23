//! The look while the desktop blurs behind the window, as Zed's Catppuccin
//! Blur theme draws it: one translucent layer, the window's background, with
//! everything drawn over it see-through. Controls, text, selections and popups
//! keep solid fills.

use egui::Color32;

use super::readability::{Ends, fitted, readable};
use super::{SECONDARY_CONTRAST, Tokens, contrast, mix};

/// The alpha of the one translucent layer, the window's background.
///
/// Zed's Catppuccin Mocha (Blur) and Latte (Blur) paint their `background`
/// at `#…d7`, 215 of 255 (84.3 %). 0.84 lands one level under it, at 214 of
/// 255. It is the only layer the glass stacks, so the page, the bands and
/// the sidebar all show this one composite.
pub const WINDOW_OPACITY: f32 = 0.84;
/// The darkest and the lightest backdrop a desktop blur can leave behind a
/// window.
pub(super) const BACKDROPS: [Color32; 2] = [Color32::BLACK, Color32::WHITE];
/// The contrast of the crust under the window. Catppuccin Mocha's crust
/// (#11111b) reads 1.14:1 under its window (#1e1e2e). A light window takes
/// the same step, so the crust stays a shade of the window in both themes.
const CRUST_CONTRAST: f32 = 1.14;
/// The glass fills as shares of their colour, as Zed's `tab` and `ghost_element`
/// colours are: the crust under an active tab (`tab.active_background`, 69 %)
/// and a hover (`ghost_element.hover`, 56 %), and the crust moved toward the
/// accent under a selection (`ghost_element.selected`, 31 %). The button
/// colour under a border (`border`, 8 %).
const PILL_SHARE: f32 = 0.69;
const HOVER_SHARE: f32 = 0.56;
const SELECTION_SHARE: f32 = 0.31;
/// No step over the selection: the selection text reads 4.5:1 on the solid
/// fill in the light theme only up to the selection's own share, so a
/// stronger hover fill would fall below it (32 % 4.42:1, 40 % 3.96:1).
const SELECTED_HOVER_SHARE: f32 = SELECTION_SHARE;
const SEPARATOR_SHARE: f32 = 0.08;
/// The text mixed into a popup's fill under a dropdown entry, a step the
/// popup's own text reads across at 4.5:1 or more.
const POPUP_HOVER_SHARE: f32 = 0.20;

impl Tokens {
    /// The look while the desktop blurs behind the window: the window's
    /// background is the one translucent layer, and the bars, sidebar and
    /// cards draw no fill over it. Their edges are the faint border. Controls,
    /// selections and popups take solid fills, so their text does not depend
    /// on the desktop. `window` stays opaque for frames and footers.
    #[must_use = "the glass tokens are the ones a frame paints with"]
    pub fn glass(self) -> Self {
        let crust = crust(self.window, self.shadow_shade);
        let separator = self.field.gamma_multiply(SEPARATOR_SHARE);
        let selection = mix(crust, self.accent, SELECTION_SHARE);
        let tint_hover = mix(crust, self.accent, SELECTED_HOVER_SHARE);
        let window_glass = self.window.gamma_multiply(WINDOW_OPACITY);

        // The layer is seen over the darkest and the lightest backdrop, so
        // secondary text is fitted against both, as it is on the opaque window.
        let layers = BACKDROPS.map(|backdrop| backdrop.blend(window_glass));
        let ends = Ends::of(&self);

        let secondary = fitted(self.secondary, &layers, SECONDARY_CONTRAST, ends);

        Self {
            window_glass,
            secondary,
            sidebar: Color32::TRANSPARENT,
            sidebar_fade: Color32::TRANSPARENT,
            band: Color32::TRANSPARENT,
            pane: Color32::TRANSPARENT,
            header_scrim: Color32::TRANSPARENT,
            card_glass: Color32::TRANSPARENT,
            card_edge: separator,
            card_shadow: Color32::TRANSPARENT,
            row_divider: separator,
            chosen_tab: crust.gamma_multiply(PILL_SHARE),
            item_hover: crust.gamma_multiply(HOVER_SHARE),
            selection,
            selection_text: readable(self.text, selection, SECONDARY_CONTRAST, ends),
            tint_hover,
            popup_hover: mix(self.raised, self.text, POPUP_HOVER_SHARE),
            frost_raised: self.raised,
            frost_raised_blurred: self.raised,
            separator,
            rule: separator,
            ..self
        }
    }
}

/// The darker base of the window, the crust of the Catppuccin glass themes:
/// the window moved toward the shadow shade until it is [`CRUST_CONTRAST`]
/// from the window.
pub(super) fn crust(window: Color32, shadow: Color32) -> Color32 {
    (0..=100_u8)
        .map(|step| mix(window, shadow, f32::from(step) / 100.0))
        .find(|crust| contrast(*crust, window) >= CRUST_CONTRAST)
        .unwrap_or(shadow)
}
