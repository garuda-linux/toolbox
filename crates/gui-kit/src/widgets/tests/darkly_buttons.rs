//! Darkly's push and flat buttons, checked on the pure formulas in both
//! themes and on the shapes a button paints.

use egui::{Key, Modifiers, Theme};
use egui_kittest::kittest::Queryable as _;

use super::super::buttons::ButtonKind;
use super::super::buttons::look::{press_overlay, push_blur, push_fill, push_width, tool_fill};
use super::super::press::Pointer;
use super::super::{pill_button, sidebar_button};
use crate::desktop::kde::parse;
use crate::icons;
use crate::palette::Palette;
use crate::testing::desktop::kde::{BREEZEDARK_COLORS, INSTALLED_KDEGLOBALS};
use crate::testing::{fill_under, harness};
use crate::tokens::{Tokens, contrast, darkly};

pub(super) fn tokens(theme: Theme) -> Tokens {
    Tokens::derive(&Palette::fallback(theme), theme)
}

#[test]
fn a_hovered_push_button_is_its_colour_lightened_by_five_percent() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);
        assert_eq!(
            push_fill(&tokens, tokens.input, Pointer::Hovered, false),
            tokens.button_hover
        );
        assert_eq!(
            push_fill(&tokens, tokens.default_button, Pointer::Hovered, false),
            darkly::lighter(tokens.default_button, 1.05)
        );
    }
}

#[test]
fn a_pressed_push_button_is_the_focus_colour_darkened_by_ten_percent() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);
        let pressed = darkly::darker(tokens.decoration_focus, 1.1);
        assert_eq!(tokens.button_pressed, pressed);
        for focused in [false, true] {
            assert_eq!(
                push_fill(&tokens, tokens.field, Pointer::Pressed, focused),
                pressed
            );
        }
    }
}

#[test]
fn a_focused_push_button_is_filled_with_the_focus_colour() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);
        assert_eq!(
            push_fill(&tokens, tokens.field, Pointer::Rest, true),
            tokens.focus_fill
        );
        assert_eq!(
            push_fill(&tokens, tokens.default_button, Pointer::Rest, true),
            tokens.focus_fill
        );
    }
}

#[test]
fn the_text_on_a_focus_fill_reads_at_4_5_at_rest_and_under_the_pointer() {
    let captures = [INSTALLED_KDEGLOBALS, BREEZEDARK_COLORS]
        .map(|capture| parse(capture).colors.over(Palette::fallback(Theme::Dark)));

    let palettes = [
        (Theme::Dark, Palette::fallback(Theme::Dark)),
        (Theme::Light, Palette::fallback(Theme::Light)),
        (Theme::Dark, captures[0]),
        (Theme::Dark, captures[1]),
    ];

    for (theme, palette) in palettes {
        let tokens = Tokens::derive(&palette, theme);
        for pointer in [Pointer::Rest, Pointer::Hovered] {
            let fill = push_fill(&tokens, tokens.field, pointer, true);
            let ratio = contrast(tokens.on_focus, fill);
            assert!(
                ratio >= 4.5,
                "{theme:?}: text {:?} on {fill:?} under {pointer:?} is {ratio:.2}:1, shadow {:?}, window {:?}",
                tokens.on_focus,
                tokens.shadow_shade,
                tokens.window
            );
        }
    }
}

#[test]
fn a_push_button_is_at_least_85_wide_and_keeps_a_given_width() {
    assert_eq!(push_width(20.0, 0.0), 85.0, "80 minimum plus the 5 frame");
    assert_eq!(push_width(100.0, 0.0), 117.0, "text plus 12 margin plus 5");
    assert_eq!(push_width(20.0, 100.0), 100.0);
}

#[test]
fn a_push_button_blurs_3_at_rest_6_under_focus_or_pointer_and_2_pressed() {
    assert_eq!(push_blur(Pointer::Rest, false), 3);
    assert_eq!(push_blur(Pointer::Rest, true), 6);
    assert_eq!(push_blur(Pointer::Hovered, false), 6);
    assert_eq!(push_blur(Pointer::Pressed, false), 2);
}

#[test]
fn the_top_highlight_is_the_light_end_at_30_of_255_on_a_dark_window_only() {
    let dark = tokens(Theme::Dark);
    assert_eq!(dark.button_top_light.a(), 30);

    let light = tokens(Theme::Light);
    assert_eq!(light.button_top_light.a(), 0);
}

#[test]
fn the_press_ripple_is_the_focus_colour_darkened_by_200_at_half() {
    let tokens = tokens(Theme::Dark);
    assert_eq!(
        press_overlay(&tokens),
        darkly::darker(tokens.decoration_focus, 2.0).gamma_multiply(0.5)
    );
}

#[test]
fn a_flat_button_is_text_at_a_tenth_under_the_pointer_and_a_fifth_pressed() {
    let tokens = tokens(Theme::Light);
    assert_eq!(
        tool_fill(&tokens, Pointer::Hovered, false),
        Some(tokens.text.gamma_multiply(0.1))
    );
    assert_eq!(
        tool_fill(&tokens, Pointer::Pressed, false),
        Some(tokens.text.gamma_multiply(0.2))
    );
    assert_eq!(
        tool_fill(&tokens, Pointer::Rest, true),
        Some(tokens.focus_fill)
    );
    assert_eq!(tool_fill(&tokens, Pointer::Rest, false), None);
}

#[test]
fn a_flat_button_is_a_28_pixel_square_round_a_16_pixel_icon() {
    let mut harness = harness(200.0, 80.0).build_ui(|ui| {
        sidebar_button(ui, icons::VIEW_REFRESH, "Refresh");
    });

    harness.run();
    let rect = harness.get_by_label("Refresh").rect();
    assert_eq!(rect.width(), rect.height());
    assert_eq!(rect.width(), 28.0);
}

/// Two push buttons and two flat ones, for the focus fill of each kind.
fn focus_pair() -> egui_kittest::Harness<'static> {
    harness(300.0, 200.0).build_ui(|ui| {
        _ = pill_button(ui, "First push", ButtonKind::Soft);
        _ = pill_button(ui, "Second push", ButtonKind::Soft);
        sidebar_button(ui, icons::VIEW_REFRESH, "First flat");
        sidebar_button(ui, icons::VIEW_REFRESH, "Second flat");
    })
}

/// The face's fill near its left edge, where no glyph or icon paints over it.
fn fill_at(harness: &egui_kittest::Harness<'_>, label: &str) -> Option<egui::Color32> {
    let rect = harness.get_by_label(label).rect();
    fill_under(harness, egui::pos2(rect.left() + 4.0, rect.center().y))
}

#[test]
fn a_click_focuses_a_button_without_its_focus_fill() {
    // The Tab count reaches the first button of each kind.
    for (first, tabs) in [("First push", 1_usize), ("First flat", 3_usize)] {
        let mut harness = focus_pair();
        harness.run();
        let focus = crate::tokens::current(&harness.ctx).focus_fill;

        for _ in 0..tabs {
            harness.key_press(Key::Tab);
            harness.run();
            harness.run();
        }

        assert_eq!(
            fill_at(&harness, first),
            Some(focus),
            "a key shows the fill on {first}"
        );

        harness.get_by_label(first).click();
        harness.run();
        harness.run();
        assert!(
            harness.get_by_label(first).is_focused(),
            "the click keeps the focus on {first}"
        );
        assert_ne!(
            fill_at(&harness, first),
            Some(focus),
            "a click shows no fill on {first}"
        );
    }
}

#[test]
fn a_focus_the_program_gives_shows_no_fill_until_a_key_moves_it() {
    for (given, before) in [("Second push", "First push"), ("Second flat", "First flat")] {
        let mut harness = focus_pair();
        harness.run();
        let focus = crate::tokens::current(&harness.ctx).focus_fill;

        harness.get_by_label(given).focus();
        harness.run();
        harness.run();
        assert!(harness.get_by_label(given).is_focused());
        assert_ne!(
            fill_at(&harness, given),
            Some(focus),
            "no key yet on {given}"
        );

        harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
        harness.run();
        harness.run();
        assert!(
            harness.get_by_label(before).is_focused(),
            "Shift+Tab moves to {before}"
        );
        assert_eq!(
            fill_at(&harness, before),
            Some(focus),
            "a key shows the fill on {before}"
        );
    }
}
