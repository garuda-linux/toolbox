use egui::Color32;
use egui_kittest::kittest::Queryable as _;

use super::super::{SectionHeading, group_label, menu_heading};
use crate::testing::{fill_under, harness, painted_text, text_colour};
use crate::{UiExt as _, tokens};

fn painted(text: &str, draw: impl FnMut(&mut egui::Ui)) -> (Color32, tokens::Tokens) {
    let mut harness = harness(400.0, 200.0).build_ui(draw);
    harness.run();
    let colour = text_colour(&harness, text).expect("the text is painted");

    (colour, tokens::current(&harness.ctx))
}

#[test]
fn a_section_detail_is_secondary_text() {
    let (colour, tokens) = painted("2/8", |ui| {
        SectionHeading::new("Audio", "2/8").show(ui);
    });

    assert_eq!(colour, tokens.secondary);
}

#[test]
fn a_menu_heading_is_secondary_text() {
    let (colour, tokens) = painted("Sort by", |ui| menu_heading(ui, "Sort by"));

    assert_eq!(colour, tokens.secondary);
}

#[test]
fn a_sidebar_group_label_takes_the_full_sidebar_text() {
    // The sidebar's text is the full text colour, as Darkly's sidebar entries take it.
    let (colour, tokens) = painted("System", |ui| group_label(ui, "System", false));

    assert_eq!(colour, tokens.sidebar_text);
}

#[test]
fn a_sidebar_group_label_is_a_heading_heard_as_written() {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| group_label(ui, "System", false));
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Heading, "System")
            .is_some()
    );
    assert!(harness.query_by_label("SYSTEM").is_none());
}

#[test]
fn a_disabled_button_keeps_its_colours_under_egui_s_fade_alone() {
    use super::super::{ButtonKind, pill_button};

    for kind in [ButtonKind::Primary, ButtonKind::Danger, ButtonKind::Soft] {
        let mut harness = harness(400.0, 200.0).build_ui(move |ui| {
            ui.enabled_ui(false, |ui| _ = pill_button(ui, "Add user…", kind));
        });

        harness.run();
        let tokens = tokens::current(&harness.ctx);

        // Breeze draws every push button's text in the normal text colour, and
        // a Primary button fills with the accent blended into the field.
        let ink = tokens.text;
        let fill = if kind == ButtonKind::Primary {
            tokens.default_button
        } else {
            tokens.input
        };

        let alpha = harness.ctx.global_style().visuals.disabled_alpha;
        let text = painted_text(&harness, "Add user…").expect("the text is painted");
        let centre = harness.get_by_label("Add user…").rect().center();

        assert_eq!(
            text.fallback_color,
            ink.gamma_multiply(alpha),
            "{kind:?}: the label fades once"
        );
        assert_eq!(
            fill_under(&harness, centre),
            Some(fill.gamma_multiply(alpha)),
            "{kind:?}: the fill fades once"
        );
    }
}

#[test]
fn a_disabled_ghost_button_stays_without_a_fill() {
    use super::super::{ButtonKind, pill_button};

    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        ui.enabled_ui(false, |ui| {
            _ = pill_button(ui, "Details", ButtonKind::Ghost);
        });
    });

    harness.run();
    let tokens = tokens::current(&harness.ctx);
    let alpha = harness.ctx.global_style().visuals.disabled_alpha;
    let centre = harness.get_by_label("Details").rect().center();
    let under = fill_under(&harness, centre);

    for quiet in [tokens.field, tokens.button_hover] {
        assert_ne!(under, Some(quiet.gamma_multiply(alpha)));
        assert_ne!(under, Some(quiet));
    }
}

#[test]
fn a_toned_tag_is_a_checked_chip_in_its_tone() {
    use super::super::Tone;
    use super::super::press::Pointer;
    use crate::palette::Palette;

    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let tokens = tokens::Tokens::derive(&Palette::fallback(theme), theme);
        let look = Tone::Warning.look(&tokens, Pointer::Rest);

        assert_eq!(look.fill, tokens.warning.gamma_multiply(0.2), "{theme:?}");
        assert_eq!(look.frame, tokens.warning.gamma_multiply(0.9), "{theme:?}");
        assert_eq!(
            Tone::Neutral.look(&tokens, Pointer::Rest).fill,
            Color32::TRANSPARENT,
            "{theme:?}"
        );
    }
}
