//! Turns the tokens into egui's style, so plain egui widgets already look
//! like the app's own: no outlines, depth from the surface steps, round
//! corners, quiet scroll bars and short animations.

mod scrollbar;
#[cfg(test)]
mod tests;

use egui::style::{ScrollAnimation, ScrollStyle};
use egui::{Color32, Context, CornerRadius, Shadow, Stroke, Style, Theme, vec2};
#[cfg(any(test, feature = "testing"))]
pub(crate) use scrollbar::REST_WIDTH;
pub(crate) use scrollbar::{MIN_LENGTH, STRIP};
pub use scrollbar::{show_scroll, show_scroll_hidden, show_scroll_rows};

use crate::palette::{self, Palette};
use crate::tokens::{self, Tokens};

/// Rows and item highlights, and the hover fill behind them: Darkly's
/// `CornerRadius` of 9 (darkly.kcfg:57), which its item views draw
/// (darklystyle.cpp:4753).
pub const RADIUS: u8 = 9;
/// Buttons, dropdowns and the select.
///
/// Darkly's `CornerRadius` less the two-pixel frame (darklyhelper.cpp:829).
/// Text fields take [`FIELD_RADIUS`]. Filters keep [`PILL_RADIUS`], so a
/// control reads apart from a chip or the search that narrows a list.
pub const CONTROL_RADIUS: u8 = 7;
/// Section cards and frames, and the Console's terminal: Darkly's frame,
/// `CornerRadius` less one (darklyhelper.cpp:466).
pub const CARD_RADIUS: u8 = 8;
/// Popovers, menus and sheets: Darkly's `CornerRadius` (darkly.kcfg:57).
pub const POPOVER_RADIUS: u8 = 9;
/// Text fields and spin boxes: Darkly's line edit, `CornerRadius` less one
/// (darklyhelper.cpp:1045).
pub(crate) const FIELD_RADIUS: u8 = 8;
/// Tabs: the corners of an unchosen tab. The chosen tab is a pill of `RADIUS`.
pub(crate) const TAB_RADIUS: u8 = 8;
/// Check boxes: half of 9, rounded (darklyhelper.cpp:1275).
pub(crate) const CHECK_RADIUS: u8 = 5;
/// Tool buttons at rest: 9 less half a frame pen, 8.5 rounded up to 9
/// (darklyhelper.cpp:908). Sunken ones take 9.
pub(crate) const TOOL_BUTTON_RADIUS: u8 = 9;
/// The tile behind a row's icon.
pub(crate) const TILE_RADIUS: u8 = 6;
/// The tile behind a section's icon.
pub const SMALL_TILE_RADIUS: u8 = 5;
/// The least height of a card row's line and of a banner's action row.
pub const PILL_HEIGHT: f32 = 32.0;
/// The round ends of the search field and segmented controls, half of
/// [`PILL_HEIGHT`].
pub const PILL_RADIUS: u8 = 16;
pub const SIDEBAR_WIDTH: f32 = 248.0;
/// Kirigami's `Units.gridUnit` at the default 10 pt font: the font's line
/// height, rounded up to even.
pub const GRID_UNIT: f32 = 18.0;
/// Kirigami's `Units.smallSpacing`.
pub(crate) const SMALL_SPACING: f32 = 4.0;
/// Kirigami's `Units.largeSpacing`.
pub(crate) const LARGE_SPACING: f32 = 8.0;
/// How long hovers and switches take to settle, in seconds.
pub const ANIMATION_TIME: f32 = 0.12;

/// Sets both themes' styles from their desktop palettes, so a switch
/// between dark and light needs no second call.
pub fn apply(ctx: &Context, dark: &Palette, light: &Palette) {
    let glass = translucent(ctx);
    let tokens_of = |palette: &Palette, theme: Theme| {
        let tokens = Tokens::derive(palette, theme);
        if glass { tokens.glass() } else { tokens }
    };

    let dark_tokens = tokens_of(dark, Theme::Dark);
    let light_tokens = tokens_of(light, Theme::Light);
    let still = reduced_motion(ctx);

    for (theme, tokens) in [(Theme::Dark, &dark_tokens), (Theme::Light, &light_tokens)] {
        ctx.style_mut_of(theme, |style| {
            write(style, tokens);

            if still {
                style.animation_time = 0.0;
                style.scroll_animation = ScrollAnimation::none();
            }
        });
    }

    tokens::store(ctx, dark_tokens, light_tokens);
    palette::store(ctx, *dark, *light);
}

/// Follows the desktop's wish for less motion: hovers, switches and
/// scrolling to a place then take no time. Applies to the palettes in use
/// and every later [`apply`].
pub fn set_reduced_motion(ctx: &Context, on: bool) {
    set_switch(ctx, reduced_motion_id(), on);
}

pub fn reduced_motion(ctx: &Context) -> bool {
    switch(ctx, reduced_motion_id())
}

/// The window's background is translucent at [`crate::tokens::WINDOW_OPACITY`]
/// while on, so the blurred desktop shows through it. Applies to the palettes
/// in use and to every later [`apply`].
pub fn set_translucent(ctx: &Context, on: bool) {
    set_switch(ctx, translucent_id(), on);
}

pub fn translucent(ctx: &Context) -> bool {
    switch(ctx, translucent_id())
}

/// Stores a switch and applies the palettes in use again, when it changed.
fn set_switch(ctx: &Context, id: egui::Id, on: bool) {
    if switch(ctx, id) == on {
        return;
    }

    ctx.data_mut(|data| data.insert_temp(id, on));
    let (dark, light) = palette::both(ctx);
    apply(ctx, &dark, &light);
}

fn switch(ctx: &Context, id: egui::Id) -> bool {
    ctx.data(|data| data.get_temp(id)).unwrap_or(false)
}

fn translucent_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::theme::translucent")
}

fn reduced_motion_id() -> egui::Id {
    egui::Id::unique("toolbox_gui_kit::theme::reduced_motion")
}

fn write(style: &mut Style, tokens: &Tokens) {
    write_visuals(style, tokens);

    let sizes = tokens.sizes;
    let spacing = &mut style.spacing;
    spacing.item_spacing = vec2(sizes.control_gap, sizes.caption_gap);
    spacing.button_padding = vec2(14.0, 7.0);
    spacing.interact_size = vec2(40.0, PILL_HEIGHT);
    spacing.menu_margin = sizes.list_padding;
    spacing.window_margin = sizes.popover_padding;

    // egui keeps the strip and the drag, its bar width is the strip so the
    // drag covers all of it. Its own bar stays invisible, `show_scroll`
    // paints Darkly's handle and groove over it.
    spacing.scroll = ScrollStyle {
        floating: true,
        bar_width: STRIP,
        floating_width: STRIP,
        floating_allocated_width: STRIP,
        handle_min_length: MIN_LENGTH,
        bar_inner_margin: 0.0,
        bar_outer_margin: 0.0,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.0,
        dormant_handle_opacity: 0.0,
        active_handle_opacity: 0.0,
        interact_handle_opacity: 0.0,
        foreground_color: false,
        ..ScrollStyle::floating()
    };

    style.interaction.selectable_labels = false;
    style.interaction.tooltip_delay = 0.4;

    // Long enough to move from icon to icon along the collapsed sidebar
    // without waiting for each tooltip again.
    style.interaction.tooltip_grace_time = 0.5;
    style.animation_time = ANIMATION_TIME;
    style.scroll_animation = ScrollAnimation::default();
}

fn write_visuals(style: &mut Style, tokens: &Tokens) {
    let visuals = &mut style.visuals;
    visuals.weak_text_color = Some(tokens.secondary);

    // egui fades a disabled control as a whole. Its own 0.5 leaves a
    // label at 2.5:1, 0.75 keeps it above 3:1 in light themes.
    visuals.disabled_alpha = 0.75;
    visuals.panel_fill = tokens.window_glass;
    visuals.window_fill = tokens.raised;
    visuals.window_stroke = Stroke::new(1.0, tokens.outline);
    visuals.window_corner_radius = CornerRadius::same(POPOVER_RADIUS);
    visuals.menu_corner_radius = CornerRadius::same(POPOVER_RADIUS);
    visuals.window_shadow = shadow(tokens, 10, 30);
    visuals.popup_shadow = shadow(tokens, 6, 20);
    visuals.extreme_bg_color = tokens.field;
    visuals.faint_bg_color = tokens.field;
    visuals.code_bg_color = tokens.field;
    visuals.hyperlink_color = tokens.accent_text;
    visuals.error_fg_color = tokens.error;
    visuals.warn_fg_color = tokens.warning;

    // The selection is the solid highlight with its own text, as Breeze's
    // item views and text selections paint it. egui draws a selected
    // widget's text in the stroke's colour.
    visuals.selection.bg_fill = tokens.accent_tint;
    visuals.selection.stroke = Stroke::new(1.0, tokens.on_tint);
    visuals.text_cursor.stroke = Stroke::new(2.0, tokens.accent);
    visuals.striped = false;
    visuals.slider_trailing_fill = true;

    // A field or check box on a card or the window needs an edge, its fill
    // alone is too close to both (WCAG 1.4.11).
    let edge = field_edge(tokens);
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = tokens.card;
    widgets.noninteractive.weak_bg_fill = tokens.card;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, tokens.outline);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.text);
    widgets.noninteractive.corner_radius = CornerRadius::same(CARD_RADIUS);

    for (state, fill) in [
        (&mut widgets.inactive, tokens.field),
        (&mut widgets.hovered, tokens.button_hover),
        (&mut widgets.active, tokens.button_pressed),
        (&mut widgets.open, tokens.button_hover),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, edge);
        state.fg_stroke = Stroke::new(1.0, tokens.text);
        state.corner_radius = CornerRadius::same(CONTROL_RADIUS);
        state.expansion = 0.0;
    }
}

const fn shadow(tokens: &Tokens, offset: i8, blur: u8) -> Shadow {
    Shadow {
        offset: [0, offset],
        blur,
        spread: 0,
        color: tokens.shadow,
    }
}

/// The quietest share of the text colour over the card that stands out
/// 3:1 from the card and the window alike.
fn field_edge(tokens: &Tokens) -> Color32 {
    (6_u8..=20)
        .map(|step| tokens::mix(tokens.card, tokens.text, f32::from(step) / 20.0))
        .find(|&edge| {
            [tokens.window, tokens.card]
                .iter()
                .all(|&surface| tokens::contrast(edge, surface) >= tokens::DIM_CONTRAST)
        })
        .unwrap_or(tokens.text)
}
