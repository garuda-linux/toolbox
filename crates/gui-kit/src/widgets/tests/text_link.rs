use egui::accesskit::Role;
use egui::{Key, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{counted_disclosure, disclosure, minor_heading, text_button, text_link};
use crate::palette::Palette;
use crate::testing::{harness, text_colour};
use crate::tokens;

fn link_shown() -> Harness<'static, bool> {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, clicked: &mut bool| {
            if text_link(ui, "gegl", "Open gegl").clicked() {
                *clicked = true;
            }
        },
        false,
    );

    harness.run();
    harness
}

#[test]
fn a_text_link_is_a_link_with_its_own_name_and_reports_the_click() {
    let mut harness = link_shown();

    harness
        .get_by_role_and_label(Role::Link, "Open gegl")
        .click();
    harness.run();

    assert!(*harness.state());
}

#[test]
fn a_text_link_answers_the_keyboard() {
    let mut harness = link_shown();

    harness
        .get_by_role_and_label(Role::Link, "Open gegl")
        .focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert!(*harness.state());
}

fn folding(ui: &mut Ui, open: &mut bool) {
    if disclosure(ui, "Files (3)", *open).clicked() {
        *open = !*open;
    }
}

#[test]
fn a_disclosure_says_whether_it_is_open_and_flips_on_a_click() {
    let mut harness = harness(300.0, 120.0).build_ui_state(folding, false);

    harness.run();
    let folded = harness.get_by_role_and_label(Role::Button, "Files (3)");
    assert_eq!(folded.accesskit_node().data().is_expanded(), Some(false));
    folded.click();
    harness.run();

    assert!(*harness.state());
    let open = harness.get_by_role_and_label(Role::Button, "Files (3)");
    assert_eq!(open.accesskit_node().data().is_expanded(), Some(true));
}

#[test]
fn an_inline_button_is_a_button_with_its_own_name() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, clicked: &mut bool| {
            if text_button(ui, "+3 more", "Show 3 more").clicked() {
                *clicked = true;
            }
        },
        false,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Show 3 more")
        .click();
    harness.run();

    assert!(*harness.state());
}

#[test]
fn a_minor_heading_is_a_third_level_heading_with_its_count() {
    let harness = harness(300.0, 120.0).build_ui(|ui| minor_heading(ui, "Depends on", "12"));

    let heading = harness.get_by_role_and_label(Role::Heading, "Depends on");
    assert_eq!(heading.accesskit_node().data().level(), Some(3));
    assert!(harness.query_by_label("12").is_some());
}

#[test]
fn a_counted_disclosure_shows_its_count_as_a_minor_heading_does() {
    let harness = harness(300.0, 120.0).build_ui(|ui| {
        counted_disclosure(ui, "Files", "4", false);
    });

    let button = harness.get_by_role_and_label(Role::Button, "Files 4");
    assert_eq!(button.accesskit_node().data().is_expanded(), Some(false));

    let tokens = tokens::current(&harness.ctx);
    assert_eq!(text_colour(&harness, "Files"), Some(tokens.text));
    assert_eq!(text_colour(&harness, "4"), Some(tokens.secondary));
}

#[test]
fn the_tinted_text_reads_on_its_tints_in_both_themes() {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let tokens = tokens::Tokens::derive(&Palette::fallback(theme), theme);

        for fill in [tokens.accent_tint, tokens.tint_pressed] {
            let ratio = tokens::contrast(tokens.on_tint, fill);
            assert!(ratio >= 4.5, "{theme:?}: {ratio}");
        }
    }
}
