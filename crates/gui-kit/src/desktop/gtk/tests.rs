#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::Color32;

use super::parse;

#[test]
fn libadwaita_names_fill_their_roles() {
    let colors = parse(
        "/* Catppuccin */\n\
         @define-color accent_bg_color #cba6f7;\n\
         @define-color accent_fg_color #11111b;\n\
         @define-color window_bg_color #1e1e2e;\n\
         @define-color window_fg_color #cdd6f4;\n\
         @define-color view_bg_color #181825;\n\
         @define-color error_color #f38ba8;\n\
         @define-color warning_color #f9e2af;\n\
         @define-color success_color #a6e3a1;\n",
    );

    assert_eq!(colors.window, Some(Color32::from_rgb(0x1e, 0x1e, 0x2e)));
    assert_eq!(colors.text, Some(Color32::from_rgb(0xcd, 0xd6, 0xf4)));
    assert_eq!(colors.view, Some(Color32::from_rgb(0x18, 0x18, 0x25)));
    assert_eq!(colors.accent, Some(Color32::from_rgb(0xcb, 0xa6, 0xf7)));
    assert_eq!(colors.on_accent, Some(Color32::from_rgb(0x11, 0x11, 0x1b)));
    assert_eq!(colors.error, Some(Color32::from_rgb(0xf3, 0x8b, 0xa8)));
    assert_eq!(colors.warning, Some(Color32::from_rgb(0xf9, 0xe2, 0xaf)));
    assert_eq!(colors.success, Some(Color32::from_rgb(0xa6, 0xe3, 0xa1)));
}

#[test]
fn gtk3_theme_names_are_read_when_libadwaita_names_are_missing() {
    let colors = parse(
        "@define-color theme_bg_color #ffffff;\n\
         @define-color theme_fg_color #000000;\n\
         @define-color theme_selected_bg_color #3584e4;\n",
    );

    assert_eq!(colors.window, Some(Color32::WHITE));
    assert_eq!(colors.text, Some(Color32::BLACK));
    assert_eq!(colors.accent, Some(Color32::from_rgb(0x35, 0x84, 0xe4)));
}

#[test]
fn libadwaita_names_win_over_gtk3_names_in_any_order() {
    let colors = parse(
        "@define-color window_bg_color #111111;\n\
         @define-color theme_bg_color #222222;\n",
    );

    assert_eq!(colors.window, Some(Color32::from_rgb(0x11, 0x11, 0x11)));
}

#[test]
fn short_hex_rgb_rgba_and_references_resolve() {
    let colors = parse(
        "@define-color purple rgb(203, 166, 247);\n\
         @define-color accent_bg_color @purple;\n\
         @define-color window_bg_color #fff;\n\
         @define-color view_bg_color rgba(0, 0, 0, 0.5);\n",
    );

    assert_eq!(colors.accent, Some(Color32::from_rgb(203, 166, 247)));
    assert_eq!(colors.window, Some(Color32::WHITE));
    assert_eq!(
        colors.view,
        Some(Color32::from_rgba_unmultiplied(0, 0, 0, 128))
    );
}

#[test]
fn functions_and_unknown_references_are_skipped() {
    let colors = parse(
        "@define-color window_bg_color mix(#000, #fff, 0.5);\n\
         @define-color accent_bg_color @nowhere;\n\
         @define-color view_bg_color #12345;\n",
    );

    assert_eq!(colors.window, None);
    assert_eq!(colors.accent, None);
    assert_eq!(colors.view, None);
}

#[test]
fn rules_and_comments_around_definitions_are_ignored() {
    let colors = parse(
        "window { background: red; }\n\
         /* @define-color window_bg_color #000000; */\n\
         @define-color window_fg_color #eeeeee;",
    );

    assert_eq!(colors.window, None);
    assert_eq!(colors.text, Some(Color32::from_rgb(0xee, 0xee, 0xee)));
}
