//! The glass look while the desktop blurs behind the window, as Zed's Catppuccin
//! Blur theme draws it: one translucent layer, the window's background, with
//! everything drawn over it seeing through. Controls, selections and popups
//! take solid fills, and cards take no fill of their own.

use egui::{Color32, Theme};

use super::super::glass::{BACKDROPS, WINDOW_OPACITY, crust};
use super::super::{Tokens, contrast, mix};
use super::cards::captured;
use super::every_tokens;
use super::frost::laid_on;
use crate::palette::Palette;
use crate::testing::desktop::kde::INSTALLED_KDEGLOBALS;

/// The alpha a share of opacity gives, rounded as `gamma_multiply` rounds.
fn alpha_of(share: f32) -> f32 {
    255.0 * share
}

/// What a fill over the glass shows of the desktop behind the window: the
/// fill lies on the translucent window, which lies on the backdrop.
fn through_window(fill: Color32, backdrop: Color32, tokens: &Tokens) -> Color32 {
    laid_on(fill, laid_on(tokens.window_glass, backdrop))
}

#[test]
fn the_window_is_translucent_at_zeds_background_alpha() {
    // Zed's Catppuccin Blur `background` is `#…d7`, 215 of 255.
    assert!((WINDOW_OPACITY - 0.84).abs() < f32::EPSILON);
    assert!((alpha_of(WINDOW_OPACITY) - 215.0).abs() <= 1.0);

    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        let window = f32::from(glass.window_glass.a());
        assert!(
            (window - alpha_of(WINDOW_OPACITY)).abs() <= 1.0,
            "{theme:?}: {window}"
        );
    }
}

#[test]
fn the_one_layer_is_the_window_at_its_opacity() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();
        assert_eq!(
            glass.window_glass,
            tokens.window.gamma_multiply(WINDOW_OPACITY),
            "{theme:?}"
        );
    }
}

#[test]
fn the_sidebar_takes_no_fill_of_its_own_so_it_shares_the_window_layer() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        assert_eq!(glass.sidebar, Color32::TRANSPARENT, "{theme:?}");
        assert_eq!(
            tokens.sidebar_fade, tokens.sidebar,
            "{theme:?}: opaque cut edges fade into the sidebar"
        );
        assert_eq!(
            glass.sidebar_fade,
            Color32::TRANSPARENT,
            "{theme:?}: a glass cut edge has no fade"
        );
    }
}

#[test]
fn the_sidebar_and_the_page_area_are_the_one_layer_over_any_backdrop() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        for backdrop in BACKDROPS {
            let layer = backdrop.blend(glass.window_glass);
            let page = layer.blend(glass.band);
            let side = layer.blend(glass.sidebar);
            assert_eq!(page, layer, "{theme:?} over {backdrop:?}");
            assert_eq!(side, layer, "{theme:?} over {backdrop:?}");
        }
    }
}

#[test]
fn the_bars_and_the_header_draw_no_fill_over_the_layer() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        assert_eq!(glass.band, Color32::TRANSPARENT, "{theme:?}");
        assert_eq!(glass.pane, Color32::TRANSPARENT, "{theme:?}");
        assert_eq!(glass.header_scrim, Color32::TRANSPARENT, "{theme:?}");
        assert_eq!(glass.card_shadow, Color32::TRANSPARENT, "{theme:?}");
    }
}

#[test]
fn the_window_colour_used_under_footers_stays_opaque() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        assert_eq!(glass.window, tokens.window, "{theme:?}");
        assert_eq!(glass.window.a(), 255, "{theme:?}");
    }
}

#[test]
fn controls_popups_and_outlines_stay_opaque() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();

        for (name, before, after) in [
            ("field", tokens.field, glass.field),
            ("input", tokens.input, glass.input),
            ("terminal", tokens.terminal, glass.terminal),
            ("text", tokens.text, glass.text),
            ("button_hover", tokens.button_hover, glass.button_hover),
            ("outline", tokens.outline, glass.outline),
            ("card_outline", tokens.card_outline, glass.card_outline),
            ("raised", tokens.raised, glass.raised),
        ] {
            assert_eq!(after, before, "{name} in {theme:?}");
            assert_eq!(after.a(), 255, "{name} in {theme:?}");
        }
    }
}

#[test]
fn popups_take_the_solid_raised_fill_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();
        assert_eq!(glass.frost_raised, glass.raised, "{theme:?}");
        assert_eq!(glass.frost_raised_blurred, glass.raised, "{theme:?}");
    }
}

#[test]
fn cards_draw_no_fill_and_no_shadow_with_the_glass_and_keep_an_edge() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.card_glass, tokens.card,
            "{theme:?}: opaque cards are the card"
        );
        assert_eq!(
            tokens.card_edge, tokens.card_outline,
            "{theme:?}: opaque cards keep the outline"
        );
        assert_eq!(
            tokens.card_shadow, tokens.control_shadow,
            "{theme:?}: opaque cards keep the shadow"
        );

        let glass = tokens.glass();
        assert_eq!(glass.card_glass, Color32::TRANSPARENT, "{theme:?}");
        assert_eq!(glass.card_edge, glass.separator, "{theme:?}");
        assert_eq!(
            glass.card_edge,
            tokens.field.gamma_multiply(0.08),
            "{theme:?}"
        );
    }
}

#[test]
fn the_rows_of_a_card_are_divided_by_the_faint_border_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();
        assert_eq!(glass.row_divider, glass.separator, "{theme:?}");
        assert_eq!(
            tokens.row_divider,
            mix(tokens.card, tokens.text, crate::tokens::ROW_DIVIDER_SHARE),
            "{theme:?}: opaque dividers are unchanged"
        );
    }
}

#[test]
fn the_opaque_tokens_are_untouched_by_the_glass() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(tokens.window.a(), 255, "{theme:?}");
        assert_eq!(tokens.sidebar.a(), 255, "{theme:?}");
    }
}

#[test]
fn the_bands_on_the_window_are_see_through_only_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.band, tokens.window,
            "{theme:?}: opaque bands are the window"
        );
        assert_eq!(tokens.glass().band, Color32::TRANSPARENT, "{theme:?}");
    }
}

#[test]
fn the_chosen_tab_is_the_crust_at_69_percent_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.chosen_tab, tokens.input,
            "{theme:?}: opaque keeps the field"
        );

        let glass = tokens.glass();
        let crust = crust(tokens.window, tokens.shadow_shade);
        assert_eq!(glass.chosen_tab, crust.gamma_multiply(0.69), "{theme:?}");
        assert!(
            (f32::from(glass.chosen_tab.a()) - alpha_of(0.69)).abs() <= 1.0,
            "{theme:?}"
        );
    }
}

#[test]
fn hovers_are_the_crust_at_56_percent_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.item_hover,
            tokens.accent.gamma_multiply(0.2),
            "{theme:?}: the opaque hover is unchanged"
        );

        let glass = tokens.glass();
        let crust = crust(tokens.window, tokens.shadow_shade);
        assert_eq!(glass.item_hover, crust.gamma_multiply(0.56), "{theme:?}");
        assert!(
            (f32::from(glass.item_hover.a()) - alpha_of(0.56)).abs() <= 1.0,
            "{theme:?}"
        );
    }
}

#[test]
fn selections_are_the_crust_moved_31_percent_toward_the_accent_and_solid() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.selection, tokens.accent_tint,
            "{theme:?}: opaque keeps the tint"
        );
        assert_eq!(tokens.selection_text, tokens.on_tint, "{theme:?}");

        let glass = tokens.glass();
        let crust = crust(tokens.window, tokens.shadow_shade);
        assert_eq!(
            glass.selection,
            mix(crust, tokens.accent, 0.31),
            "{theme:?}"
        );
        assert_eq!(glass.selection.a(), 255, "{theme:?}");
    }
}

/// A stronger step under the pointer breaks the 4.5:1 of the selection's
/// text in the light theme, so the selected hover keeps the selection's share.
#[test]
fn a_selected_row_under_the_pointer_keeps_the_selection_and_is_solid() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();
        let crust = crust(tokens.window, tokens.shadow_shade);
        assert_eq!(
            glass.tint_hover,
            mix(crust, tokens.accent, 0.31),
            "{theme:?}"
        );
        assert_eq!(glass.tint_hover, glass.selection, "{theme:?}");
        assert_eq!(glass.tint_hover.a(), 255, "{theme:?}");
    }
}

#[test]
fn separators_are_the_button_colour_at_8_percent_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        let glass = tokens.glass();
        assert_eq!(
            glass.separator,
            tokens.field.gamma_multiply(0.08),
            "{theme:?}"
        );
        assert!(
            (f32::from(glass.separator.a()) - alpha_of(0.08)).abs() <= 1.0,
            "{theme:?}"
        );
    }
}

#[test]
fn the_hairlines_of_the_bands_are_the_separator_with_the_glass_and_the_outline_without() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.rule, tokens.outline,
            "{theme:?}: opaque hairlines keep the outline"
        );
        assert_eq!(tokens.glass().rule, tokens.glass().separator, "{theme:?}");
    }
}

#[test]
fn the_crust_of_the_captured_mocha_is_its_crust() {
    let palette = captured(INSTALLED_KDEGLOBALS);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    // Catppuccin Mocha's crust is #11111b.
    let found = crust(tokens.window, tokens.shadow_shade);
    assert!(found.r().abs_diff(17) <= 2, "{found:?}");
    assert!(found.g().abs_diff(17) <= 2, "{found:?}");
    assert!(found.b().abs_diff(27) <= 2, "{found:?}");
}

/// Every text colour on every glass surface, over the darkest and the
/// lightest backdrop, at 4.5:1. The shortfalls are listed together, so one
/// run shows them all.
#[test]
fn text_reads_on_every_glass_surface_over_the_darkest_and_lightest_backdrop() {
    let mut short = Vec::new();

    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme).glass();

        let surfaces = [
            ("window", tokens.window_glass),
            ("chosen tab", tokens.chosen_tab),
            ("hover", tokens.item_hover),
        ];

        for (name, fill) in surfaces {
            for (ink_name, ink) in [("text", tokens.text), ("secondary", tokens.secondary)] {
                for backdrop in BACKDROPS {
                    let ratio = contrast(ink, through_window(fill, backdrop, &tokens));
                    if ratio < 4.5 {
                        short.push(format!(
                            "{theme:?} {name} {ink_name} over {backdrop:?}: {ratio:.2}:1"
                        ));
                    }
                }
            }
        }
    }

    assert!(short.is_empty(), "below 4.5:1:\n{}", short.join("\n"));
}

/// The selection and its hover are solid, so the text on them is checked on
/// the fill alone, at rest and under the pointer.
#[test]
fn the_selection_text_reads_on_the_selection_and_its_hover() {
    let mut short = Vec::new();

    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme).glass();

        for (name, fill) in [
            ("selection", tokens.selection),
            ("selected hover", tokens.tint_hover),
        ] {
            let ratio = contrast(tokens.selection_text, fill);
            if ratio < 4.5 {
                short.push(format!("{theme:?} {name}: {ratio:.2}:1"));
            }
        }
    }

    assert!(short.is_empty(), "below 4.5:1:\n{}", short.join("\n"));
}

/// Opaque popups keep the item hover, the glass gives an entry under the
/// pointer a solid step over the popup, which the popup text reads across.
#[test]
fn dropdown_entries_step_solidly_over_the_popup_with_the_glass() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(tokens.popup_hover, tokens.item_hover, "{theme:?}");

        let glass = tokens.glass();
        assert_eq!(
            glass.popup_hover,
            mix(glass.raised, glass.text, 0.2),
            "{theme:?}"
        );
        assert_eq!(glass.popup_hover.a(), 255, "{theme:?}");
        let ratio = contrast(glass.text, glass.popup_hover);
        assert!(ratio >= 4.5, "{theme:?}: {ratio:.2}");
    }
}
