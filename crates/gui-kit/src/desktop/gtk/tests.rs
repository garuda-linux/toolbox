#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::Color32;

use super::parse;

/// `gtk-4.0/gtk.css` of Sweet-Dark, the GTK theme Garuda's skel names.
/// It defines the GTK 3 names only.
const SWEET_DARK: &str = include_str!("fixtures/theme-gtk-4.0.css");
/// The same theme's `gtk-3.0/gtk.css`.
const SWEET_DARK_GTK3: &str = include_str!("fixtures/theme-gtk-3.0.css");

const WINDOW: Color32 = Color32::from_rgb(0x16, 0x19, 0x25);
const BASE: Color32 = Color32::from_rgb(0x18, 0x1b, 0x28);
const SELECTED: Color32 = Color32::from_rgb(0xc5, 0x0e, 0xd2);

/// [`SWEET_DARK`] edited: each `theme_` name of a role renamed to its
/// libadwaita name.
fn as_libadwaita() -> String {
    [
        ("theme_bg_color", "window_bg_color"),
        ("theme_fg_color", "window_fg_color"),
        ("theme_base_color", "view_bg_color"),
        ("theme_selected_bg_color", "accent_bg_color"),
        ("theme_selected_fg_color", "accent_fg_color"),
    ]
    .into_iter()
    .fold(SWEET_DARK.to_owned(), |css, (gtk3, libadwaita)| {
        css.replace(
            &format!("@define-color {gtk3} "),
            &format!("@define-color {libadwaita} "),
        )
    })
}

#[test]
fn gtk3_theme_names_fill_their_roles() {
    for css in [SWEET_DARK, SWEET_DARK_GTK3] {
        let colors = parse(css);

        assert_eq!(colors.window, Some(WINDOW));
        assert_eq!(colors.text, Some(Color32::from_rgb(0xc3, 0xc7, 0xd1)));
        assert_eq!(colors.view, Some(BASE));
        assert_eq!(colors.accent, Some(SELECTED));
        assert_eq!(colors.on_accent, Some(Color32::from_rgb(0xfe, 0xfe, 0xfe)));
        assert_eq!(colors.error, Some(Color32::from_rgb(0xe6, 0x13, 0x3e)));
        assert_eq!(colors.warning, Some(Color32::from_rgb(0xcc, 0x55, 0x00)));
        assert_eq!(colors.success, Some(Color32::from_rgb(0x00, 0x96, 0xb1)));
    }
}

#[test]
fn libadwaita_names_fill_their_roles() {
    assert_eq!(parse(&as_libadwaita()), parse(SWEET_DARK));
}

#[test]
fn libadwaita_names_win_over_gtk3_names_in_any_order() {
    // A libadwaita window colour that points at the base colour, put
    // after and before the theme's own definitions.
    let libadwaita = "@define-color window_bg_color @theme_base_color;\n";

    for css in [
        format!("{SWEET_DARK}\n{libadwaita}"),
        format!("{libadwaita}{SWEET_DARK}"),
    ] {
        assert_eq!(parse(&css).window, Some(BASE));
    }
}

#[test]
fn short_hex_rgb_rgba_and_references_resolve() {
    // Sweet-Dark defines `wm_highlight` as `rgba(0, 0, 0, 0)`. The other
    // forms are added.
    let colors = parse(&format!(
        "{SWEET_DARK}\n\
         @define-color purple rgb(203, 166, 247);\n\
         @define-color accent_bg_color @purple;\n\
         @define-color window_bg_color #fff;\n\
         @define-color view_bg_color @wm_highlight;\n",
    ));

    assert_eq!(colors.accent, Some(Color32::from_rgb(203, 166, 247)));
    assert_eq!(colors.window, Some(Color32::WHITE));
    assert_eq!(colors.view, Some(Color32::TRANSPARENT));
}

#[test]
fn functions_and_unknown_references_fall_back_to_the_gtk3_names() {
    // Sweet-Dark defines `wm_title` with `shade()`.
    let colors = parse(&format!(
        "{SWEET_DARK}\n\
         @define-color window_bg_color @wm_title;\n\
         @define-color accent_bg_color @nowhere;\n\
         @define-color view_bg_color #12345;\n",
    ));

    assert_eq!(colors.window, Some(WINDOW));
    assert_eq!(colors.accent, Some(SELECTED));
    assert_eq!(colors.view, Some(BASE));
}

#[test]
fn rules_and_comments_around_definitions_are_ignored() {
    // Sweet-Dark with its window colour put in a comment.
    let commented = SWEET_DARK.replace(
        "@define-color theme_bg_color #161925;",
        "/* @define-color theme_bg_color #161925; */",
    );

    assert_ne!(commented, SWEET_DARK);
    assert_eq!(parse(&commented).window, None);
    assert_eq!(parse(&commented).view, Some(BASE));
}
