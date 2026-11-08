use egui::Theme;
use egui::style::ScrollAnimation;

use super::{ANIMATION_TIME, apply, reduced_motion, set_reduced_motion};
use crate::palette::Palette;
use crate::tokens::{self, Tokens};

#[test]
fn both_themes_take_their_tokens() {
    let ctx = egui::Context::default();
    let dark = Palette::fallback(Theme::Dark);
    let light = Palette::fallback(Theme::Light);
    apply(&ctx, &dark, &light);

    let dark_tokens = Tokens::derive(&dark, Theme::Dark);
    let dark_style = ctx.style_of(Theme::Dark);
    assert_eq!(dark_style.visuals.panel_fill, dark_tokens.window);
    assert_eq!(dark_style.visuals.window_fill, dark_tokens.card);
    assert_eq!(
        dark_style.visuals.widgets.hovered.bg_fill,
        dark_tokens.hover
    );
    assert_eq!(
        dark_style.visuals.override_text_color, None,
        "it would hide the selected text colour"
    );

    let light_tokens = Tokens::derive(&light, Theme::Light);
    assert_eq!(
        ctx.style_of(Theme::Light).visuals.panel_fill,
        light_tokens.window
    );
    assert_eq!(tokens::current(&ctx), dark_tokens, "egui starts dark");
}

#[test]
fn fields_show_a_boundary_that_stands_out_three_to_one_without_growing() {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );

    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let widgets = ctx.style_of(theme).visuals.widgets.clone();
        for state in [widgets.inactive, widgets.hovered, widgets.active] {
            assert!((state.bg_stroke.width - 1.0).abs() < f32::EPSILON);
            for surface in [tokens.card, tokens.window] {
                assert!(
                    tokens::contrast(state.bg_stroke.color, surface) >= 3.0,
                    "{theme:?}: a field's edge is lost on its surface"
                );
            }

            assert!(state.expansion.abs() < f32::EPSILON);
        }
    }

    let style = ctx.style_of(Theme::Dark);
    assert!(style.spacing.scroll.floating);
    assert!(!style.interaction.selectable_labels);
}

#[test]
fn reduced_motion_stops_every_animation_in_both_themes() {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );

    set_reduced_motion(&ctx, true);

    for theme in [Theme::Dark, Theme::Light] {
        let style = ctx.style_of(theme);
        assert_eq!(style.animation_time, 0.0);
        assert_eq!(style.scroll_animation, ScrollAnimation::none());
    }

    assert!(reduced_motion(&ctx));
}

#[test]
fn reduced_motion_outlasts_a_new_palette_and_can_be_taken_back() {
    let ctx = egui::Context::default();
    let dark = Palette::fallback(Theme::Dark);
    let light = Palette::fallback(Theme::Light);
    set_reduced_motion(&ctx, true);

    apply(&ctx, &dark, &light);
    assert_eq!(ctx.style_of(Theme::Dark).animation_time, 0.0);

    set_reduced_motion(&ctx, false);
    let style = ctx.style_of(Theme::Dark);
    assert_eq!(style.animation_time, ANIMATION_TIME);
    assert_eq!(style.scroll_animation, ScrollAnimation::default());
}

#[test]
fn tooltips_stay_up_while_the_pointer_moves_along_the_sidebar() {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );

    for theme in [Theme::Dark, Theme::Light] {
        let interaction = &ctx.style_of(theme).interaction;
        assert!((interaction.tooltip_grace_time - 0.5).abs() < f32::EPSILON);
    }
}

#[test]
fn a_shown_scroll_bar_stays_drawn_at_rest_and_keeps_its_own_room() {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );
    let mut style = (*ctx.style_of(Theme::Dark)).clone();
    super::show_scroll_bars(&mut style);

    let scroll = style.spacing.scroll;
    assert!(scroll.dormant_handle_opacity > 0.0, "it hides at rest");
    assert!(
        scroll.floating_allocated_width >= scroll.floating_width,
        "the bar would cover the last characters"
    );
}

#[test]
fn every_scroll_handle_is_one_quiet_look_that_still_reaches_three_to_one() {
    let ctx = egui::Context::default();
    let palettes = (
        Palette::fallback(Theme::Dark),
        Palette::fallback(Theme::Light),
    );

    apply(&ctx, &palettes.0, &palettes.1);

    for (theme, palette) in [(Theme::Dark, &palettes.0), (Theme::Light, &palettes.1)] {
        let tokens = Tokens::derive(palette, theme);
        let page = (*ctx.style_of(theme)).clone();
        let mut shown = page.clone();
        super::show_scroll_bars(&mut shown);

        let near = page.spacing.scroll.active_handle_opacity;
        assert!(
            (shown.spacing.scroll.dormant_handle_opacity - near).abs() < f32::EPSILON,
            "{theme:?}: a shown bar looks other than a page's"
        );
        assert!(
            near < 0.8,
            "{theme:?}: the handle is as loud as text ({near})"
        );

        for surface in [tokens.window, tokens.card] {
            // egui blends the faded text over the surface in gamma space.
            let handle = tokens::mix(surface, tokens.text, near);
            let ratio = tokens::contrast(handle, surface);
            assert!(
                ratio >= 3.0,
                "{theme:?}: handle at {ratio:.2}:1 on {surface:?}"
            );
        }
    }
}

#[test]
fn a_pill_is_round_at_both_ends() {
    let diameter = f32::from(super::PILL_RADIUS) * 2.0;

    assert!((diameter - super::PILL_HEIGHT).abs() < f32::EPSILON);
}
