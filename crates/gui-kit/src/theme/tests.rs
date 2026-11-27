use egui::Theme;
use egui::style::ScrollAnimation;

use super::{ANIMATION_TIME, apply, reduced_motion, set_reduced_motion, set_translucent};
use crate::palette::Palette;
use crate::tokens::{self, Tokens};

fn applied() -> egui::Context {
    let ctx = egui::Context::default();
    apply(
        &ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );
    ctx
}

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
        dark_tokens.button_hover
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
    let ctx = applied();

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
    let ctx = applied();

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
    let ctx = applied();

    for theme in [Theme::Dark, Theme::Light] {
        let interaction = &ctx.style_of(theme).interaction;
        assert!((interaction.tooltip_grace_time - 0.5).abs() < f32::EPSILON);
    }
}

#[test]
fn radii_are_darkly_corner_radius_9_and_its_derived_shapes() {
    // Darkly's CornerRadius of 9 (darkly.kcfg:57), less each shape's frame.
    use super::{
        CARD_RADIUS, CHECK_RADIUS, CONTROL_RADIUS, FIELD_RADIUS, POPOVER_RADIUS, RADIUS,
        TAB_RADIUS, TOOL_BUTTON_RADIUS,
    };

    assert_eq!(RADIUS, 9, "item highlights and rows (darklystyle.cpp:4753)");
    assert_eq!(
        POPOVER_RADIUS, 9,
        "menus and popovers (darklystyle.cpp:4753)"
    );
    assert_eq!(CARD_RADIUS, 8, "card and frame (darklyhelper.cpp:466)");
    assert_eq!(FIELD_RADIUS, 8, "line edits (darklyhelper.cpp:1045)");
    assert_eq!(TAB_RADIUS, 8, "unchosen tabs, every corner");
    assert_eq!(CONTROL_RADIUS, 7, "buttons (darklyhelper.cpp:829)");
    assert_eq!(CHECK_RADIUS, 5, "check boxes (darklyhelper.cpp:1275)");
    assert_eq!(
        TOOL_BUTTON_RADIUS, 9,
        "tool buttons 8.5, rounded up (darklyhelper.cpp:908)"
    );
}

#[test]
fn a_pill_is_round_at_both_ends() {
    let diameter = f32::from(super::PILL_RADIUS) * 2.0;

    assert!((diameter - super::PILL_HEIGHT).abs() < f32::EPSILON);
}

#[test]
fn the_translucent_window_takes_its_panel_fill_and_gives_it_back() {
    let ctx = egui::Context::default();
    let dark = Palette::fallback(Theme::Dark);
    let light = Palette::fallback(Theme::Light);
    apply(&ctx, &dark, &light);
    let opaque = ctx.style_of(Theme::Dark).visuals.panel_fill;
    assert_eq!(opaque.a(), 255);

    set_translucent(&ctx, true);
    let glass = ctx.style_of(Theme::Dark).visuals.panel_fill;
    assert_eq!(glass.a(), 214);
    assert_eq!(
        glass,
        Tokens::derive(&dark, Theme::Dark).glass().window_glass
    );

    set_translucent(&ctx, false);
    assert_eq!(ctx.style_of(Theme::Dark).visuals.panel_fill, opaque);
}
