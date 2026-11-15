use egui::Color32;
use egui_kittest::kittest::Queryable as _;

use super::super::{SectionHeading, group_label, menu_heading};
use crate::testing::{fill_under, harness, painted_text, text_colour};
use crate::{icons, tokens};

fn painted(text: &str, draw: impl FnMut(&mut egui::Ui)) -> (Color32, tokens::Tokens) {
    let mut harness = harness(400.0, 200.0).build_ui(draw);
    harness.run();
    let colour = text_colour(&harness, text).expect("the text is painted");

    (colour, tokens::current(&harness.ctx))
}

#[test]
fn a_section_detail_is_secondary_text() {
    let (colour, tokens) = painted("2/8", |ui| {
        SectionHeading::new(icons::AUDIO_VOLUME_HIGH, "Audio", "2/8").show(ui);
    });

    assert_eq!(colour, tokens.secondary);
}

#[test]
fn a_menu_heading_is_secondary_text() {
    let (colour, tokens) = painted("Sort by", |ui| menu_heading(ui, "Sort by"));

    assert_eq!(colour, tokens.secondary);
}

#[test]
fn a_sidebar_group_label_is_secondary_text() {
    let (colour, tokens) = painted("SYSTEM", |ui| group_label(ui, "System", false));

    assert_eq!(colour, tokens.secondary);
}

#[test]
fn a_disabled_button_keeps_its_colours_under_egui_s_fade_alone() {
    use super::super::{ButtonKind, pill_button};

    for kind in [ButtonKind::Primary, ButtonKind::Danger, ButtonKind::Soft] {
        let mut harness = harness(400.0, 200.0).build_ui(move |ui| {
            ui.add_enabled_ui(false, |ui| _ = pill_button(ui, "Add user…", kind));
        });

        harness.run();
        let tokens = tokens::current(&harness.ctx);
        let (ink, fill) = match kind {
            ButtonKind::Primary => (tokens.on_accent, tokens.accent),
            ButtonKind::Danger => (tokens.on_error, tokens.error),
            ButtonKind::Soft | ButtonKind::Ghost => (tokens.text, tokens.field),
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
        ui.add_enabled_ui(false, |ui| {
            _ = pill_button(ui, "Details", ButtonKind::Ghost);
        });
    });

    harness.run();
    let tokens = tokens::current(&harness.ctx);
    let alpha = harness.ctx.global_style().visuals.disabled_alpha;
    let centre = harness.get_by_label("Details").rect().center();
    let under = fill_under(&harness, centre);

    for quiet in [tokens.field, tokens.hover] {
        assert_ne!(under, Some(quiet.gamma_multiply(alpha)));
        assert_ne!(under, Some(quiet));
    }
}
