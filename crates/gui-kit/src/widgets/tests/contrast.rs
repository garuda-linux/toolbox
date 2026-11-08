use egui::Color32;

use super::super::{SectionHeading, group_label, menu_heading};
use crate::testing::{harness, text_colour};
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
fn a_disabled_button_of_any_kind_reads_as_a_quiet_one() {
    use super::super::{ButtonKind, pill_button};

    for kind in [ButtonKind::Primary, ButtonKind::Danger, ButtonKind::Soft] {
        let mut harness = harness(400.0, 200.0).build_ui(move |ui| {
            ui.add_enabled_ui(false, |ui| _ = pill_button(ui, "Add user…", kind));
        });

        harness.run();
        let tokens = tokens::current(&harness.ctx);
        let text =
            crate::testing::painted_text(&harness, "Add user…").expect("the text is painted");

        let drawn = tokens::mix(
            tokens.field,
            text.fallback_color.to_opaque(),
            f32::from(text.fallback_color.a()) / 255.0,
        );

        let ratio = tokens::contrast(drawn, tokens.field);
        assert!(ratio >= 3.0, "{kind:?} label {ratio:.2}:1 on its fill");
    }
}
