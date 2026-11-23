use egui::{FontFamily, TextStyle};

use super::{
    FontFile, INTER, JETBRAINS_MONO, UiFaces, Weight, add_fallback, apply_sizes, definitions, font,
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
fn without_desktop_fonts_regular_is_inter_at_400_and_bold_at_700() {
    let fonts = definitions(&UiFaces::default(), None);

    assert_eq!(fonts.families[&FontFamily::Proportional], ["Inter 400"]);
    assert_eq!(fonts.families[&Weight::Bold.family()], ["Inter 700"]);
    assert_eq!(
        fonts.families[&FontFamily::Monospace],
        [JETBRAINS_MONO.to_owned(), format!("{INTER} 400")]
    );
}

#[test]
fn the_desktop_regular_face_draws_regular_text_at_its_own_weight() {
    // A KDE user with Inter Bold 10 gets the face at 700 for all body text.
    let ui = UiFaces {
        regular: Some(desktop_font(INTER, 700)),
        bold: None,
    };

    let fonts = definitions(&ui, None);

    assert_eq!(
        fonts.families[&FontFamily::Proportional],
        ["Inter 700", "Inter 400"]
    );
}

#[test]
fn medium_and_semibold_draw_as_regular_text() {
    assert_eq!(Weight::Medium.family(), FontFamily::Proportional);
    assert_eq!(Weight::SemiBold.family(), FontFamily::Proportional);
}

#[test]
fn desktop_faces_go_first_in_the_family_of_their_step() {
    let ui = UiFaces {
        regular: Some(desktop_font("Noto Sans", 400)),
        bold: Some(desktop_font("Noto Sans", 700)),
    };

    let fonts = definitions(&ui, Some(&desktop_font("Hack", 400)));

    assert_eq!(
        fonts.families[&FontFamily::Proportional],
        ["Noto Sans 400", "Inter 400"]
    );
    assert_eq!(
        fonts.families[&Weight::Bold.family()],
        ["Noto Sans 700", "Inter 700"]
    );
    assert_eq!(
        fonts.families[&FontFamily::Monospace],
        [
            "Hack".to_owned(),
            JETBRAINS_MONO.to_owned(),
            format!("{INTER} 400")
        ]
    );
}

#[test]
fn a_desktop_face_of_a_bundled_name_and_weight_does_not_replace_it() {
    let ui = UiFaces {
        regular: None,
        bold: Some(desktop_font(INTER, 700)),
    };

    let fonts = definitions(&ui, None);

    assert_eq!(fonts.families[&Weight::Bold.family()], ["Inter 700"]);
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

/// Kirigami's `Heading.qml` sizes level 1 at 1.35 and level 2 at 1.20 of the
/// default font, which the body size stands for here.
#[test]
fn the_page_title_and_section_headings_are_kirigami_levels_one_and_two() {
    assert!((super::PAGE_TITLE - 1.35).abs() < 0.001);
    assert!((super::SECTION_TITLE - 1.2).abs() < 0.001);
}

fn assert_falls(steps: [f32; 4]) {
    for pair in steps.windows(2) {
        let [larger, smaller] = pair else { continue };
        assert!(larger > smaller, "{steps:?}");
    }
}

#[test]
fn the_heading_scale_falls_from_page_title_to_small() {
    assert_falls([
        super::PAGE_TITLE,
        super::SECTION_TITLE,
        super::BODY,
        super::SMALL,
    ]);
}

#[test]
fn the_type_scale_falls_from_title_to_small() {
    assert_falls([super::TITLE, super::SECTION, super::BODY, super::SMALL]);
}

#[test]
fn a_fallback_goes_last_in_every_family() {
    let ui = UiFaces {
        regular: Some(desktop_font("Noto Sans", 400)),
        bold: None,
    };

    let mut fonts = definitions(&ui, None);
    add_fallback(&mut fonts, &desktop_font("Noto Color Emoji", 400));

    for family in [Weight::Regular.family(), Weight::Bold.family()]
        .into_iter()
        .chain([FontFamily::Monospace])
    {
        assert_eq!(
            fonts.families[&family].last().map(String::as_str),
            Some("Noto Color Emoji 400")
        );
    }
}

#[test]
fn a_fallback_named_like_a_font_in_use_is_left_out() {
    let mut fonts = definitions(&UiFaces::default(), None);
    add_fallback(&mut fonts, &desktop_font(INTER, 400));

    assert_eq!(fonts.families[&FontFamily::Proportional], ["Inter 400"]);
}
