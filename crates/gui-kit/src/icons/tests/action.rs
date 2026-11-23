use std::sync::Arc;

use egui::Color32;

use super::super::{GO_HOME, HELP_ABOUT, ThemeIcons};
use super::uri_of;

/// Tela circle has `help-about` only as a colour app icon: a blue disc
/// with a white mark. A mask would make both white, a disc of the tint.
const TELA_CIRCLE_HELP_ABOUT: &str =
    include_str!("../fixtures/installed/help-about-tela-circle-dark-16.svg");

#[test]
fn an_action_icon_leaves_a_colour_file_of_the_theme_to_the_bundled_one() {
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(HELP_ABOUT, Arc::from(TELA_CIRCLE_HELP_ABOUT.as_bytes()));

    let image = HELP_ABOUT.image(Some(&theme), Color32::WHITE);

    assert_eq!(uri_of(&image), "bytes://icons/bundled/help-about.svg");
}

#[test]
fn an_action_icon_takes_a_theme_file_drawn_in_the_text_colour() {
    let drawn = r#"<svg viewBox="0 0 16 16"><style>.ColorScheme-Text { color: #aaaaaa; } .ColorScheme-Highlight { color: #5294e2; }</style><path class="ColorScheme-Text" fill="currentColor" d="M1 1h8v8H1z"/></svg>"#;
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(GO_HOME, Arc::from(drawn.as_bytes()));

    let image = GO_HOME.image(Some(&theme), Color32::WHITE);

    assert_eq!(
        uri_of(&image),
        "bytes://icons/theme/Tela-circle-dark/go-home.svg"
    );
}
