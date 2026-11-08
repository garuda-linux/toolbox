use egui::{FontFamily, TextStyle};

use super::{
    FontFile, INTER, JETBRAINS_MONO, Weight, add_fallback, apply_sizes, definitions, font,
    has_weight_axis,
};

fn desktop_font(name: &str, weight: u16) -> FontFile {
    FontFile {
        name: name.to_owned(),
        data: include_bytes!("../../assets/fonts/InterVariable.ttf").to_vec(),
        index: 0,
        weight,
    }
}

#[test]
fn without_desktop_fonts_inter_carries_every_weight() {
    let fonts = definitions(&[], None);

    assert_eq!(fonts.families[&FontFamily::Proportional], [INTER]);
    assert_eq!(fonts.families[&Weight::Medium.family()], ["Inter 500"]);
    assert_eq!(fonts.families[&Weight::SemiBold.family()], ["Inter 600"]);
    assert_eq!(fonts.families[&Weight::Bold.family()], ["Inter 700"]);
    assert_eq!(
        fonts.families[&FontFamily::Monospace],
        [JETBRAINS_MONO, INTER]
    );
}

#[test]
fn desktop_fonts_go_first_in_the_family_of_their_weight() {
    let fonts = definitions(
        &[
            desktop_font("Noto Sans", 400),
            desktop_font("Noto Sans", 700),
        ],
        Some(&desktop_font("Hack", 400)),
    );

    assert_eq!(
        fonts.families[&FontFamily::Proportional],
        ["Noto Sans", INTER]
    );
    assert_eq!(
        fonts.families[&Weight::Bold.family()],
        ["Noto Sans 700", "Inter 700"]
    );
    assert_eq!(
        fonts.families[&Weight::Medium.family()],
        ["Inter 500"],
        "a weight the desktop font lacks stays Inter"
    );
    assert_eq!(
        fonts.families[&FontFamily::Monospace],
        ["Hack", JETBRAINS_MONO, INTER]
    );
}

#[test]
fn a_desktop_font_named_like_a_bundled_one_does_not_replace_it() {
    let fonts = definitions(&[desktop_font(INTER, 400)], None);

    assert_eq!(fonts.families[&FontFamily::Proportional], [INTER]);
}

#[test]
fn css_weights_round_to_the_nearest_weight() {
    assert_eq!(Weight::nearest(300), Weight::Regular);
    assert_eq!(Weight::nearest(500), Weight::Medium);
    assert_eq!(Weight::nearest(600), Weight::SemiBold);
    assert_eq!(Weight::nearest(800), Weight::Bold);
}

#[test]
fn variable_fonts_have_a_weight_axis() {
    let inter = include_bytes!("../../assets/fonts/InterVariable.ttf");

    assert!(has_weight_axis(inter, 0));
    assert!(!has_weight_axis(b"not a font", 0));
}

#[test]
fn desktop_point_sizes_become_text_style_sizes() {
    let ctx = egui::Context::default();
    apply_sizes(&ctx, Some(12.0), Some(9.0));

    let style = ctx.global_style();
    assert!((style.text_styles[&TextStyle::Body].size - 16.0).abs() < 0.01);
    assert!((style.text_styles[&TextStyle::Monospace].size - 12.0).abs() < 0.01);
    assert!(style.text_styles[&TextStyle::Heading].size > 16.0);
    assert!(style.text_styles[&TextStyle::Small].size < 16.0);

    let title = font(&style, Weight::Bold, super::TITLE);
    assert!(16.0_f32.mul_add(-super::TITLE, title.size).abs() < 0.01);
    assert_eq!(title.family, Weight::Bold.family());
}

#[test]
fn at_the_default_size_titles_are_four_points_and_sections_two_over_the_old_scale() {
    let ctx = egui::Context::default();
    apply_sizes(&ctx, None, None);
    let style = ctx.global_style();
    let body = style.text_styles[&TextStyle::Body].size;

    let title = font(&style, Weight::Bold, super::TITLE).size;
    let section = font(&style, Weight::Bold, super::SECTION).size;
    assert!(
        (body.mul_add(-2.0, title) - 4.0).abs() < 0.2,
        "title {title}, body {body}"
    );
    assert!(
        (body.mul_add(-1.25, section) - 2.0).abs() < 0.2,
        "section {section}, body {body}"
    );
}

#[test]
fn a_fallback_goes_last_in_every_family() {
    let mut fonts = definitions(&[desktop_font("Noto Sans", 400)], None);
    add_fallback(&mut fonts, &desktop_font("Noto Color Emoji", 400));

    for family in Weight::ALL
        .map(Weight::family)
        .into_iter()
        .chain([FontFamily::Monospace])
    {
        assert_eq!(
            fonts.families[&family].last().map(String::as_str),
            Some("Noto Color Emoji")
        );
    }
}

#[test]
fn a_fallback_named_like_a_font_in_use_is_left_out() {
    let mut fonts = definitions(&[], None);
    add_fallback(&mut fonts, &desktop_font(INTER, 400));

    assert_eq!(fonts.families[&FontFamily::Proportional], [INTER]);
}
