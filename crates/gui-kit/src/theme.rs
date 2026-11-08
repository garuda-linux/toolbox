//! Turns the tokens into egui's style, so plain egui widgets already look
//! like the app's own: no outlines, depth from the surface steps, round
//! corners, quiet scroll bars and short animations.

#[cfg(test)]
mod tests;

use egui::style::{ScrollAnimation, ScrollStyle};
use egui::{Color32, Context, CornerRadius, Shadow, Stroke, Style, Theme, vec2};

use crate::palette::{self, Palette};
use crate::tokens::{self, Tokens};

/// Buttons, fields, chips and rows.
pub const RADIUS: u8 = 10;
/// Section cards and the Console's terminal.
pub const CARD_RADIUS: u8 = 14;
/// Popovers, menus and sheets.
pub const POPOVER_RADIUS: u8 = 16;
/// The tile behind a row's icon, [`crate::tokens::Sizes::icon_tile`].
pub const TILE_RADIUS: u8 = 10;
/// The tile behind a section's icon, [`crate::tokens::Sizes::icon_tile_small`].
pub const SMALL_TILE_RADIUS: u8 = 8;
/// Buttons, the select, the search field and segmented controls.
pub const PILL_HEIGHT: f32 = 34.0;
/// Their round ends, half of [`PILL_HEIGHT`].
pub const PILL_RADIUS: u8 = 17;
pub const SIDEBAR_WIDTH: f32 = 248.0;
/// The column the pages' content keeps to on wide windows.
pub const CONTENT_WIDTH: f32 = 880.0;
/// How long hovers and switches take to settle, in seconds.
pub const ANIMATION_TIME: f32 = 0.12;

/// Sets both themes' styles from their desktop palettes, so a switch
/// between dark and light needs no second call.
pub fn apply(ctx: &Context, dark: &Palette, light: &Palette) {
    let dark_tokens = Tokens::derive(dark, Theme::Dark);
    let light_tokens = Tokens::derive(light, Theme::Light);
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
    if reduced_motion(ctx) == on {
        return;
    }

    ctx.data_mut(|data| data.insert_temp(reduced_motion_id(), on));
    let (dark, light) = palette::both(ctx);
    apply(ctx, &dark, &light);
}

pub fn reduced_motion(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp(reduced_motion_id()))
        .unwrap_or(false)
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
    spacing.interact_size = vec2(40.0, 30.0);
    spacing.menu_margin = sizes.list_padding;
    spacing.window_margin = sizes.popover_padding;
    let handle = handle_opacity(tokens);
    spacing.scroll = ScrollStyle {
        bar_width: 8.0,
        floating_width: 6.0,
        floating_allocated_width: 0.0,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.0,
        dormant_handle_opacity: 0.0,
        active_handle_opacity: handle,
        interact_handle_opacity: (handle + 0.2).min(1.0),
        foreground_color: true,
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
    visuals.panel_fill = tokens.window;
    visuals.window_fill = tokens.raised;
    visuals.window_stroke = Stroke::new(1.0, tokens.outline);
    visuals.window_corner_radius = CornerRadius::same(POPOVER_RADIUS);
    visuals.menu_corner_radius = CornerRadius::same(CARD_RADIUS);
    visuals.window_shadow = shadow(tokens, 10, 30);
    visuals.popup_shadow = shadow(tokens, 6, 20);
    visuals.extreme_bg_color = tokens.field;
    visuals.faint_bg_color = tokens.field;
    visuals.code_bg_color = tokens.field;
    visuals.hyperlink_color = tokens.accent_text;
    visuals.error_fg_color = tokens.error;
    visuals.warn_fg_color = tokens.warning;
    visuals.selection.bg_fill = tokens.accent_soft;
    visuals.selection.stroke = Stroke::new(2.0, tokens.accent_ring);
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
    widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS);

    for (state, fill) in [
        (&mut widgets.inactive, tokens.field),
        (&mut widgets.hovered, tokens.hover),
        (&mut widgets.active, tokens.pressed),
        (&mut widgets.open, tokens.hover),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, edge);
        state.fg_stroke = Stroke::new(1.0, tokens.text);
        state.corner_radius = CornerRadius::same(RADIUS);
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

/// Scroll bars that stay drawn at rest.
///
/// For a view whose content runs past its edge and only scrolling shows
/// the rest, such as a report with long lines. Elsewhere the bars float
/// and show only while the pointer is near.
pub const fn show_scroll_bars(style: &mut Style) {
    let scroll = &mut style.spacing.scroll;
    scroll.dormant_handle_opacity = scroll.active_handle_opacity;
    scroll.floating_allocated_width = scroll.bar_width;
}

/// The handle's share of the text colour: the least that still stands out
/// 3:1 from the window and the cards, so the bar is quiet but seen.
fn handle_opacity(tokens: &Tokens) -> f32 {
    (6_u8..=20)
        .map(|step| f32::from(step) / 20.0)
        .find(|&share| {
            [tokens.window, tokens.card].iter().all(|&surface| {
                let handle = tokens::mix(surface, tokens.text, share);
                tokens::contrast(handle, surface) >= tokens::DIM_CONTRAST
            })
        })
        .unwrap_or(1.0)
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
