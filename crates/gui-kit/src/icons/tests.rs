mod action;

use std::collections::BTreeSet;
use std::sync::Arc;

use egui::{Color32, ImageSource};

use super::{ALL, GO_HOME, Icon, ThemeIcons, colour};

const TWO_COLOURS: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
<style>.ColorScheme-Text { color: #232629; }</style>
<path style="fill:currentColor" class="ColorScheme-Text" d="M1 1h8v8H1z"/>
<path fill="#27ae60" fill-opacity="0.5" d="M9 9h6v6H9z"/>
<path style="fill:url(#abc)" d="M0 0h1v1H0z"/>
</svg>"##;

const TELA_CIRCLE_INSTALL: &str =
    include_str!("fixtures/installed/system-software-install-tela-circle-dark-22.svg");

fn uri_of(image: &egui::Image<'_>) -> String {
    match image.source(&egui::Context::default()) {
        ImageSource::Bytes { uri, bytes: _ } | ImageSource::Uri(uri) => uri.into_owned(),
        ImageSource::Texture(_) => String::new(),
    }
}

#[test]
fn the_theme_in_use_is_kept_on_the_context() {
    let ctx = egui::Context::default();
    assert_eq!(super::theme_in_use(&ctx), None);

    let theme = Arc::new(ThemeIcons::new("Papirus"));
    super::use_theme(&ctx, Some(Arc::clone(&theme)));
    assert_eq!(super::theme_in_use(&ctx), Some(theme));

    super::use_theme(&ctx, None);
    assert_eq!(super::theme_in_use(&ctx), None);
}

#[test]
fn every_icon_asks_the_theme_for_its_own_file() {
    let files: BTreeSet<_> = ALL.iter().map(Icon::theme_name).collect();

    assert_eq!(files.len(), ALL.len());
    for icon in ALL {
        let text = std::str::from_utf8(icon.bundled()).unwrap();
        assert!(text.contains("<svg"), "{} is no SVG", icon.name());
    }
}

#[test]
fn every_bundled_icon_decodes_through_the_installed_loader() {
    let ctx = egui::Context::default();
    crate::install(&ctx);

    for icon in ALL {
        let uri = uri_of(&icon.image(None, Color32::WHITE));
        ctx.include_bytes(uri.clone(), icon.bundled());
        let poll = ctx.try_load_image(&uri, egui::SizeHint::default());

        let error = poll.as_ref().err().map(ToString::to_string);
        assert!(
            matches!(poll, Ok(egui::load::ImagePoll::Ready { image: _ })),
            "{} does not decode: {error:?}",
            icon.name()
        );
    }
}

#[test]
fn without_a_theme_icon_the_bundled_one_is_drawn() {
    let image = GO_HOME.image(None, Color32::WHITE);

    assert_eq!(uri_of(&image), "bytes://icons/bundled/go-home.svg");
}

#[test]
fn a_theme_icon_is_drawn_instead_of_the_bundled_one() {
    let mut theme = ThemeIcons::new("Papirus");
    theme.insert(GO_HOME, Arc::from(b"<svg/>".as_slice()));

    let image = GO_HOME.image(Some(&theme), Color32::WHITE);

    assert_eq!(uri_of(&image), "bytes://icons/theme/Papirus/go-home.svg");
}

#[test]
fn a_theme_without_this_icon_falls_back_to_the_bundled_one() {
    let theme = ThemeIcons::new("Papirus");

    let image = GO_HOME.image(Some(&theme), Color32::WHITE);

    assert_eq!(uri_of(&image), "bytes://icons/bundled/go-home.svg");
}

#[test]
fn icons_are_found_by_name() {
    assert_eq!(super::named("go-home"), Some(GO_HOME));
    assert_eq!(super::named("no-such-icon"), None);
}

#[test]
fn a_name_shared_by_a_glyph_and_a_colour_icon_finds_the_glyph() {
    let found = super::named("network-wired").expect("network-wired is bundled");

    assert!(found.is_glyph());
    assert_ne!(found, colour::NETWORK_WIRED);
}

#[test]
fn a_glyph_and_a_colour_icon_of_one_name_ask_for_different_files() {
    assert_eq!(super::NETWORK_WIRED.name(), colour::NETWORK_WIRED.name());
    assert_eq!(super::NETWORK_WIRED.theme_name(), "network-wired-symbolic");
    assert_eq!(colour::NETWORK_WIRED.theme_name(), "network-wired");
    assert_ne!(
        super::NETWORK_WIRED.bundled(),
        colour::NETWORK_WIRED.bundled()
    );
}

#[test]
fn tinted_glyphs_come_from_breeze_dark() {
    for icon in ALL.iter().filter(|icon| icon.is_glyph()) {
        let text = std::str::from_utf8(icon.bundled()).unwrap();

        assert!(
            !text.contains("#232629"),
            "{} has Breeze's dark ink, which the tint turns near black",
            icon.name()
        );
    }
}

#[test]
fn a_row_draws_the_colour_icon_of_its_glyph() {
    assert_eq!(super::NETWORK_WIRED.for_row(), colour::NETWORK_WIRED);
    assert_eq!(super::TEXT_X_GENERIC.for_row(), colour::TEXT_X_GENERIC);
    assert_eq!(super::GLOBE.for_row(), colour::INTERNET_WEB_BROWSER);
}

/// KDE's settings use a colour icon for these names, where Breeze has only a
/// glyph.
#[test]
fn a_row_takes_the_colour_icon_kde_settings_use_where_breeze_has_only_a_glyph() {
    assert_eq!(
        super::AUDIO_VOLUME_HIGH.for_row(),
        colour::PREFERENCES_DESKTOP_SOUND
    );
    assert_eq!(super::TOOLS.for_row(), colour::APPLICATIONS_UTILITIES);
    assert_eq!(
        super::SPEEDOMETER.for_row(),
        colour::UTILITIES_SYSTEM_MONITOR
    );
}

#[test]
fn a_row_keeps_a_glyph_breeze_has_no_colour_file_for() {
    assert_eq!(super::CHECKMARK.for_row(), super::CHECKMARK);
    assert_eq!(super::VCS_DIFF.for_row(), super::VCS_DIFF);
}

/// "Reset Snapper" and "Remove database lock" revert something, so their
/// rows keep the revert glyph rather than a hard drive.
#[test]
fn a_revert_row_keeps_its_own_glyph() {
    assert_eq!(super::DOCUMENT_REVERT.for_row(), super::DOCUMENT_REVERT);
    assert_eq!(super::DRIVE_HARDDISK.for_row(), colour::DRIVE_HARDDISK);
}

#[test]
fn every_row_mapping_lands_on_a_colour_icon() {
    for icon in ALL.iter().filter(|icon| icon.is_glyph()) {
        let row = icon.for_row();

        if row != *icon {
            assert!(!row.is_glyph(), "{} maps to a glyph", icon.name());
        }
    }
}

#[test]
fn a_glyph_theme_file_is_a_mask_that_keeps_only_shape_and_alpha() {
    let mut theme = ThemeIcons::new("breeze");
    theme.insert(super::SECURITY_HIGH, Arc::from(TWO_COLOURS.as_bytes()));

    let file = &theme.files[super::SECURITY_HIGH.theme_name()];
    let text = std::str::from_utf8(&file.svg).unwrap();

    assert!(file.tinted);
    let painted: Vec<String> = colours(text)
        .into_iter()
        .filter(|hex| hex != "abc")
        .collect();

    assert_eq!(painted, ["ffffff", "ffffff", "ffffff"], "{text}");
    assert!(text.contains("fill-opacity=\"0.5\""), "alpha stays");
    assert!(text.contains("url(#abc)"), "references stay");
}

#[test]
fn a_colour_theme_file_keeps_its_text_parts_open_for_the_ink() {
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(
        colour::SYSTEM_SOFTWARE_INSTALL,
        Arc::from(TELA_CIRCLE_INSTALL.as_bytes()),
    );

    let file = &theme.files[colour::SYSTEM_SOFTWARE_INSTALL.theme_name()];
    let text = std::str::from_utf8(&file.svg).unwrap();

    assert!(!file.tinted);
    assert!(text.contains("currentColor"), "{text}");
    assert!(
        text.contains(".ColorScheme-Text { color:#aaaaaa; }"),
        "{text}"
    );
}

#[test]
fn the_ink_takes_the_text_class_and_current_colour_and_keeps_other_classes() {
    let red = Color32::RED;

    let inked = super::ink::inked(TWO_COLOURS, red).unwrap();

    assert!(
        inked.contains(".ColorScheme-Text { color: #ff0000; }"),
        "{inked}"
    );
    assert!(inked.contains(r#"style="fill:#ff0000"#), "{inked}");
    assert!(inked.contains(r##"fill="#27ae60""##), "{inked}");
    assert!(!inked.contains("currentColor"), "{inked}");
}

#[test]
fn a_file_with_no_text_part_is_not_rewritten() {
    let plain = r##"<svg><path fill="#27ae60" d="M0 0h1v1H0z"/></svg>"##;

    assert_eq!(super::ink::inked(plain, Color32::RED), None);
}

#[test]
fn a_current_colour_part_of_another_class_keeps_that_class_colour() {
    let highlight = r#"<svg><style>.ColorScheme-Highlight { color:#5294e2; }</style><path class="ColorScheme-Highlight" style="fill:currentColor" d="M0 0h1v1H0z"/></svg>"#;

    let inked = super::ink::inked(highlight, Color32::RED).unwrap();

    assert_eq!(colours(&inked), ["5294e2", "5294e2"], "{inked}");
}

#[test]
fn the_same_colour_icon_in_two_inks_has_two_uris() {
    let red = colour::COMPUTER.image(None, Color32::RED);
    let blue = colour::COMPUTER.image(None, Color32::BLUE);

    assert_ne!(uri_of(&red), uri_of(&blue));
    assert!(uri_of(&red).ends_with("ff0000.svg"), "{}", uri_of(&red));
}

#[test]
fn a_theme_colour_icon_takes_the_ink_under_its_own_uri() {
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(
        colour::SYSTEM_SOFTWARE_INSTALL,
        Arc::from(TELA_CIRCLE_INSTALL.as_bytes()),
    );

    let image = colour::SYSTEM_SOFTWARE_INSTALL.image(Some(&theme), Color32::RED);

    assert_eq!(
        uri_of(&image),
        "bytes://icons/theme/Tela-circle-dark/system-software-install-ff0000.svg"
    );
}

#[test]
fn a_colour_icon_is_never_tinted_and_takes_the_ink_for_its_text_parts() {
    let image = colour::COMPUTER.image(None, Color32::BLACK);

    assert_eq!(image.image_options().tint, Color32::WHITE);
    assert_eq!(uri_of(&image), "bytes://icons/bundled/computer-000000.svg");
}

#[test]
fn a_bundled_colour_icon_without_a_text_part_keeps_its_own_uri() {
    let image = colour::SYSTEM_SOFTWARE_INSTALL.image(None, Color32::BLACK);

    assert_eq!(
        uri_of(&image),
        "bytes://icons/bundled/system-software-install.svg"
    );
}

#[test]
fn a_colour_icon_takes_the_theme_file_when_the_theme_has_one() {
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(colour::DIALOG_ERROR, Arc::from(b"<svg/>".as_slice()));

    let image = colour::DIALOG_ERROR.image(Some(&theme), Color32::BLACK);

    assert_eq!(
        uri_of(&image),
        "bytes://icons/theme/Tela-circle-dark/dialog-error.svg"
    );
    assert_eq!(image.image_options().tint, Color32::WHITE);
}

#[test]
fn every_tinted_glyph_renders_in_its_ink_alone() {
    let ctx = egui::Context::default();
    crate::install(&ctx);
    let size = egui::SizeHint::Size {
        width: 64,
        height: 64,
        maintain_aspect_ratio: true,
    };

    for icon in ALL.iter().filter(|icon| icon.is_glyph()) {
        let uri = uri_of(&icon.image(None, Color32::WHITE));
        ctx.include_bytes(uri.clone(), icon.bundled());
        let Ok(egui::load::ImagePoll::Ready { image }) = ctx.try_load_image(&uri, size) else {
            panic!("{} does not decode", icon.name());
        };

        // Edge pixels of little alpha lose their colour to rounding.
        let stray = image
            .pixels
            .iter()
            .map(Color32::to_srgba_unmultiplied)
            .find(|[red, green, blue, alpha]| {
                *alpha >= 96 && [red, green, blue].iter().any(|channel| **channel < 0xf0)
            });

        assert_eq!(stray, None, "{} paints a colour of its own", icon.name());
    }
}

#[test]
fn a_glyph_takes_the_text_and_accent_of_both_themes() {
    let mut theme = ThemeIcons::new("breeze");
    theme.insert(GO_HOME, Arc::from(TWO_COLOURS.as_bytes()));

    for side in [egui::Theme::Dark, egui::Theme::Light] {
        let tokens = crate::tokens::Tokens::derive(&crate::Palette::fallback(side), side);
        for tint in [tokens.text, tokens.secondary, tokens.accent] {
            for icon in ALL.iter().filter(|icon| icon.is_glyph()) {
                assert_eq!(icon.image(None, tint).image_options().tint, tint);
            }

            assert_eq!(GO_HOME.image(Some(&theme), tint).image_options().tint, tint);
        }
    }
}

fn colours(svg: &str) -> Vec<String> {
    svg.split('#')
        .skip(1)
        .filter_map(|rest| {
            let hex: String = rest.chars().take_while(char::is_ascii_hexdigit).collect();
            let after = rest.chars().nth(hex.len());
            let ends = after.is_none_or(|next| !next.is_ascii_alphanumeric());
            ([3, 6].contains(&hex.len()) && ends).then(|| hex.to_ascii_lowercase())
        })
        .collect()
}

#[test]
fn one_colour_glyphs_carry_no_colour_besides_their_ink() {
    let coloured: Vec<(&str, Vec<String>)> = ALL
        .iter()
        .filter(|icon| icon.is_glyph())
        .filter_map(|icon| {
            let text = std::str::from_utf8(icon.bundled()).unwrap();
            let fixed: Vec<String> = colours(text)
                .into_iter()
                .filter(|colour| colour != "fcfcfc")
                .collect();

            (!fixed.is_empty()).then_some((icon.name(), fixed))
        })
        .collect();

    assert!(coloured.is_empty(), "{coloured:?}");
}

#[test]
fn a_status_part_takes_the_colour_its_class_sets_in_a_theme() {
    let status = r#"<svg><style>.ColorScheme-NeutralText { color: #f67400; }</style><path class="ColorScheme-NeutralText" fill="currentColor"/></svg>"#;
    let inked = super::ink::inked(status, Color32::RED).unwrap();

    assert!(inked.contains(r##"fill="#f67400"/>"##), "{inked}");
}

#[test]
fn a_bundled_warning_is_drawn_in_its_class_colour() {
    let warning = std::str::from_utf8(colour::DIALOG_WARNING.bundled()).unwrap();

    assert!(warning.contains(r##"fill="#f67400""##), "{warning}");
}
