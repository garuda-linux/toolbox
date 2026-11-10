use std::collections::BTreeSet;
use std::sync::Arc;

use egui::{Color32, ImageSource};

use super::{ALL, GO_HOME, Icon, ThemeIcons};

const TWO_COLOURS: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
<style>.ColorScheme-Text { color: #232629; }</style>
<path style="fill:currentColor" class="ColorScheme-Text" d="M1 1h8v8H1z"/>
<path fill="#27ae60" fill-opacity="0.5" d="M9 9h6v6H9z"/>
<path style="fill:url(#abc)" d="M0 0h1v1H0z"/>
</svg>"##;

/// The icons that keep their own colours, each with the reason.
const FULL_COLOUR: [(&str, &str); 2] = [
    (
        "preferences-virtualization-container",
        "Breeze has no symbolic variant",
    ),
    ("system-software-install", "Breeze has no symbolic variant"),
];

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
fn every_bundled_icon_is_an_svg_with_a_unique_name() {
    let names: BTreeSet<_> = ALL.iter().map(Icon::name).collect();

    assert_eq!(names.len(), ALL.len());
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
    theme.insert(GO_HOME.name(), Arc::from(b"<svg/>".as_slice()));

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
fn symbolic_theme_glyphs_take_the_tint_and_coloured_ones_keep_theirs() {
    let mut theme = ThemeIcons::new("Tela-circle-dark");
    theme.insert(
        GO_HOME.name(),
        Arc::from(&br#"<svg><path fill="currentColor"/></svg>"#[..]),
    );
    theme.insert(
        super::SYSTEM_SOFTWARE_INSTALL.name(),
        Arc::from(&br##"<svg><path fill="#3daee9"/></svg>"##[..]),
    );
    let accent = crate::Palette::fallback(egui::Theme::Dark).accent;

    let home = GO_HOME.image(Some(&theme), accent);
    let games = super::SYSTEM_SOFTWARE_INSTALL.image(Some(&theme), accent);

    assert_eq!(home.image_options().tint, accent);
    assert_eq!(games.image_options().tint, Color32::WHITE);
}

#[test]
fn state_glyphs_keep_the_bundled_glyph_under_a_theme() {
    let state = [
        super::HELP_ABOUT,
        super::DIALOG_WARNING,
        super::DIALOG_ERROR,
        super::DIALOG_POSITIVE,
    ];

    let mut theme = ThemeIcons::new("Tela-circle-dark");
    for icon in state {
        theme.insert(icon.name(), Arc::from(b"<svg/>".as_slice()));
    }

    for icon in state {
        let image = icon.image(Some(&theme), Color32::WHITE);

        assert_eq!(
            uri_of(&image),
            format!("bytes://icons/bundled/{}.svg", icon.name()),
        );
    }
}

#[test]
fn bluetooth_and_the_shield_are_symbolic_glyphs() {
    for name in ["preferences-system-bluetooth", "security-high"] {
        let icon = super::named(name).unwrap_or_else(|| panic!("{name} is not bundled"));

        assert!(icon.tinted, "{name} keeps its own colours");
    }
}

#[test]
fn tinted_glyphs_come_from_breeze_dark() {
    for icon in ALL.iter().filter(|icon| icon.tinted) {
        let text = std::str::from_utf8(icon.bundled()).unwrap();

        assert!(
            !text.contains("#232629"),
            "{} has Breeze's dark ink, which the tint turns near black",
            icon.name()
        );
    }
}

#[test]
fn glyphs_ask_the_theme_for_their_symbolic_name() {
    assert_eq!(GO_HOME.theme_name().as_deref(), Some("go-home-symbolic"));
    assert_eq!(
        super::SYSTEM_SOFTWARE_INSTALL.theme_name().as_deref(),
        Some("system-software-install")
    );
    assert_eq!(super::DIALOG_ERROR.theme_name(), None);
}

#[test]
fn information_has_one_glyph_the_i() {
    assert_eq!(super::named("dialog-information"), None);
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
        .filter(|icon| icon.tinted)
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
fn a_theme_glyph_is_a_mask_that_keeps_only_shape_and_alpha() {
    let mut theme = ThemeIcons::new("breeze");
    theme.insert(
        super::SECURITY_HIGH.name(),
        Arc::from(TWO_COLOURS.as_bytes()),
    );

    let file = &theme.files[super::SECURITY_HIGH.name()];
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
fn a_coloured_theme_icon_keeps_its_colours() {
    let coloured = r##"<svg><path fill="#232629"/><path fill="#27ae60"/></svg>"##;
    let mut theme = ThemeIcons::new("breeze");
    theme.insert(
        super::SYSTEM_SOFTWARE_INSTALL.name(),
        Arc::from(coloured.as_bytes()),
    );

    let file = &theme.files[super::SYSTEM_SOFTWARE_INSTALL.name()];
    let text = std::str::from_utf8(&file.svg).unwrap();

    assert!(!file.tinted);
    assert_eq!(colours(text), ["232629", "27ae60"]);
}

#[test]
fn every_icon_is_a_tinted_glyph_or_a_named_full_colour_exception() {
    for icon in ALL {
        let exception = FULL_COLOUR.iter().any(|(name, _)| *name == icon.name());

        assert!(
            icon.tinted != exception,
            "{} is tinted: {}, listed as full colour: {exception}",
            icon.name(),
            icon.tinted
        );
    }
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

    for icon in ALL.iter().filter(|icon| icon.tinted) {
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
    theme.insert(GO_HOME.name(), Arc::from(TWO_COLOURS.as_bytes()));

    for side in [egui::Theme::Dark, egui::Theme::Light] {
        let tokens = crate::tokens::Tokens::derive(&crate::Palette::fallback(side), side);
        for tint in [tokens.text, tokens.secondary, tokens.accent] {
            for icon in ALL.iter().filter(|icon| icon.tinted) {
                assert_eq!(icon.image(None, tint).image_options().tint, tint);
            }

            assert_eq!(GO_HOME.image(Some(&theme), tint).image_options().tint, tint);
        }
    }
}

fn ink_of(icon: Icon) -> Vec<bool> {
    let ctx = egui::Context::default();
    crate::install(&ctx);
    let uri = uri_of(&icon.image(None, Color32::WHITE));
    ctx.include_bytes(uri.clone(), icon.bundled());
    let size = egui::SizeHint::Size {
        width: 64,
        height: 64,
        maintain_aspect_ratio: true,
    };

    let Ok(egui::load::ImagePoll::Ready { image }) = ctx.try_load_image(&uri, size) else {
        panic!("{} does not decode", icon.name());
    };

    assert_eq!(image.size, [64, 64], "{} is not square", icon.name());
    image.pixels.iter().map(|pixel| pixel.a() >= 96).collect()
}

fn parts(ink: &[bool]) -> usize {
    let mut seen = vec![false; ink.len()];
    let mut count = 0;
    for start in 0..ink.len() {
        if !ink[start] || seen[start] {
            continue;
        }

        count += 1;
        let mut open = vec![start];
        seen[start] = true;
        while let Some(at) = open.pop() {
            let (x, y) = (at % 64, at / 64);
            let near = [(x > 0, at.wrapping_sub(1)), (x < 63, at + 1)];
            let above = [(y > 0, at.wrapping_sub(64)), (y < 63, at + 64)];
            for (inside, next) in near.into_iter().chain(above) {
                if inside && ink[next] && !seen[next] {
                    seen[next] = true;
                    open.push(next);
                }
            }
        }
    }

    count
}

#[test]
fn state_glyphs_are_their_symbol_alone_without_a_shape_around_it() {
    let symbols = [
        (super::DIALOG_ERROR, "a cross", 1),
        (super::DIALOG_WARNING, "an exclamation mark", 2),
        (super::DIALOG_POSITIVE, "a check", 1),
        (super::HELP_ABOUT, "an i", 2),
    ];

    for (icon, symbol, strokes) in symbols {
        let ink = ink_of(icon);
        let inked = ink.iter().filter(|inked| **inked).count();

        assert!(
            inked * 5 < ink.len(),
            "{} covers {inked} of {} pixels",
            icon.name(),
            ink.len()
        );
        assert_eq!(parts(&ink), strokes, "{} is not {symbol}", icon.name());
    }
}
