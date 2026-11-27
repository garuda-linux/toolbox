//! The token set of one theme. The parent module derives it, every widget reads it.

use egui::Color32;

use super::{CodeInks, Roles, Sizes, Tiles};
use crate::terminal_colors::Ansi;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub window: Color32,
    /// The window's background behind the page. Translucent with the glass on,
    /// so the desktop shows through it, else the same as `window`.
    pub window_glass: Color32,
    /// A band laid on the window: the window's own colour, or nothing with
    /// the glass on, as the translucent layer under it already shows.
    pub band: Color32,
    /// The header's scrim on the window, so sidebar and header read as one surface.
    pub sidebar: Color32,
    /// The fill the cut edges of the sidebar fade into: the sidebar's, or none
    /// with the glass. An opaque fade would show as a strip on the translucent
    /// layer, so a glass cut is a hard edge with its hairline.
    pub sidebar_fade: Color32,
    /// The full text colour, as Darkly's sidebar entries take it.
    pub sidebar_text: Color32,
    /// The status bar and the Console sheet: the window colour, or nothing
    /// with the glass.
    pub pane: Color32,
    /// Darkly's group box fill, the window mixed toward the view by 0.3.
    pub card: Color32,
    /// The card fill of a card widget: the card, or nothing over the glass.
    pub card_glass: Color32,
    /// The 1 px edge of a card: Darkly's frame outline, or the faint border
    /// with the glass.
    pub card_edge: Color32,
    /// The shadow under a card: Darkly's group box shadow, or none with the
    /// glass, as a shadow would darken the layer it lies on.
    pub card_shadow: Color32,
    /// The line between the rows of a card: the text mixed into the card, or
    /// the faint border with the glass.
    pub row_divider: Color32,
    pub raised: Color32,
    pub field: Color32,
    /// The chosen tab's pill: the input fill, or the crust over the glass.
    pub chosen_tab: Color32,
    /// Darkly's line edit fill, the view colour (`QPalette::Base`,
    /// darklystyle.cpp:4097). Push buttons and dropdowns take
    /// [`Self::input`].
    pub view: Color32,
    /// The fill of a text field: the view colour, stepped toward the text
    /// as a card is where the view is the window colour.
    pub input: Color32,
    /// A Soft button's fill on a dialog. The field fill all but melts into
    /// a dark dialog, a white one already sets it apart.
    pub field_on_raised: Color32,
    /// A button under the pointer: the text field fill [`Self::input`]
    /// lightened by 5 %.
    pub button_hover: Color32,
    /// A button pressed: the view's focus colour darkened by 10 %.
    pub button_pressed: Color32,
    /// The default button's grey: the text field fill [`Self::input`] mixed
    /// toward the view by 0.7.
    pub default_button: Color32,
    /// A list item under the pointer: the accent at a fifth of its opacity.
    pub item_hover: Color32,
    /// A dropdown entry under the pointer. On a popup, which is opaque with
    /// the glass, it is a solid step over the popup's fill.
    pub popup_hover: Color32,
    /// A selected item under the pointer: the selection lightened by 10 %.
    pub tint_hover: Color32,
    pub outline: Color32,
    /// The hairline of a bar or a band on the window: `outline`, or the
    /// separator's faint share with the glass.
    pub rule: Color32,
    /// Darkly's frame outline of a card: the window mixed toward the text by a quarter
    /// (darklyhelper.cpp:127).
    pub card_outline: Color32,
    /// Darkly's separator: the rule under a toolbar and the footer's line.
    /// The text at 16 of 255 on a dark window, the shadow shade at 16 on a light one.
    pub separator: Color32,
    /// The sidebar's edge: the shadow shade at 50 of 255, at 20 on a light window.
    pub sidebar_edge: Color32,
    /// KDE's shadow shade of the window, the Qt palette's Shadow role, which
    /// Darkly's literal black maps to.
    pub shadow_shade: Color32,
    /// At least 3:1 against the field fill, so a button's shape reads on a banner.
    pub rim: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub tint_pressed: Color32,
    pub accent_soft: Color32,
    /// The selection: the solid accent, as Qt's highlight.
    pub accent_tint: Color32,
    /// A selected row or sidebar entry: the selection tint, or the accent
    /// at a third over the glass.
    pub selection: Color32,
    /// The text on a selection: the selection text, or the theme's text over
    /// the glass.
    pub selection_text: Color32,
    /// The selection's text as the desktop names it, Darkly's `HighlightedText`,
    /// or the theme's text or window where that misses 3:1 on the accent.
    pub highlighted_text: Color32,
    /// The selection's text, the same as [`Self::highlighted_text`].
    pub on_tint: Color32,
    pub on_accent: Color32,
    /// Darkly's view hover, a menu item under the pointer, the accent where
    /// the desktop names none.
    pub decoration_hover: Color32,
    /// Darkly's view focus, the fill of a focused button, the accent where
    /// the desktop names none.
    pub decoration_focus: Color32,
    /// The focus fill of a control that carries text: the focus colour, or
    /// darkened until text reads 4.5:1 on it.
    pub focus_fill: Color32,
    /// The text on a focus fill, at 4.5:1 on the fill at rest and under the
    /// pointer.
    pub on_focus: Color32,
    /// The tooltip's fill and text, the raised surface and the text where
    /// the desktop names none.
    pub tooltip: Color32,
    pub tooltip_text: Color32,
    /// The accent as it is, with no move to a contrast target, for the
    /// focus line `widgets::edge_ring` draws.
    pub accent_ring: Color32,
    /// Links and figures in the accent, moved to 4.5:1.
    pub accent_text: Color32,
    pub success: Color32,
    pub warning: Color32,
    /// Kirigami's active text and the Complementary set, see [`roles`].
    pub roles: Roles,
    pub error: Color32,
    pub top_light: Color32,
    /// Darkly's top highlight on a push button: the light end at 30 of 255
    /// on a dark window (darklyhelper.h:247), clear on a light one.
    pub button_top_light: Color32,
    pub shimmer: Color32,
    pub terminal: Color32,
    /// Thin neutral shadow shade or text, so the blur casts no tint and output stays readable.
    pub terminal_glass: Color32,
    /// Program colours, moved in lightness until they read on the terminal and its glass.
    pub terminal_ansi: Ansi,
    pub code: CodeInks,
    /// The pane colour, see-through only as far as text on it stays readable.
    pub frost: Color32,
    /// Thin neutral shadow shade or text, so the blur shows and casts no tint.
    pub header_scrim: Color32,
    /// No blur lies behind them, so what is under shows only as a faint ghost.
    pub frost_raised: Color32,
    /// Thinner than `frost_raised`, as the blur under it shows through.
    pub frost_raised_blurred: Color32,
    /// Over the blurred window under a dialog.
    pub scrim: Color32,
    pub shadow: Color32,
    /// The shadow under a control, the shadow shade at 40 of 255 (darklyhelper.cpp:592).
    pub control_shadow: Color32,
    pub tiles: Tiles,
    pub sizes: Sizes,
}
