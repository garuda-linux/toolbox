use egui::accesskit::{Role, Toggled};
use egui::{Color32, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::switch::knobs;
use super::super::{switch, tristate_switch};
use crate::UiExt as _;
use crate::palette::Palette;
use crate::testing::{circle_fills, harness};
use crate::tokens::{self, Tokens, contrast};

fn knob(theme: Theme, on: Option<bool>) -> Color32 {
    let mut harness = harness(200.0, 80.0).build_ui(move |ui| {
        tristate_switch(ui, on, "Wi-Fi");
    });

    harness.ctx.set_theme(theme);
    harness.run();

    *circle_fills(&harness)
        .last()
        .expect("the switch has a knob")
}

#[test]
fn the_knob_keeps_one_colour_on_and_off_where_it_reads_on_the_accent() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let (off, on) = knobs(&tokens, theme == Theme::Dark);
        let reads = [tokens.accent, tokens.accent_hover].map(|fill| contrast(off, fill));

        if reads.iter().all(|ratio| *ratio >= 3.0) {
            assert_eq!(off, on, "{theme:?}");
            assert_eq!(knob(theme, Some(true)), knob(theme, Some(false)));
        } else {
            assert_eq!(on, tokens.on_accent, "{theme:?}");
        }
    }

    let light = Tokens::derive(&Palette::fallback(Theme::Light), Theme::Light);
    let (off, on) = knobs(&light, false);
    assert_eq!(off, on, "the light fallback keeps one knob");
}

#[test]
fn a_disabled_knob_on_a_light_card_keeps_an_edge() {
    let mut harness = harness(200.0, 80.0).build_ui(|ui| {
        ui.enabled_ui(false, |ui| switch(ui, false, "Wi-Fi"));
    });

    harness.ctx.set_theme(Theme::Light);
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let card = tokens.card;
    let edges: Vec<Color32> = crate::testing::painted(&harness)
        .into_iter()
        .filter_map(|shape| {
            if let egui::Shape::Circle(circle) = shape
                && circle.stroke.width > 0.0
            {
                Some(circle.stroke.color)
            } else {
                None
            }
        })
        .collect();

    let edge = *edges.last().expect("the knob has an edge");
    let seen = contrast(card.blend(edge), card);
    assert!(seen >= 2.2, "the knob's edge reads at {seen}:1");
}

#[test]
fn an_unknown_switch_says_mixed_and_rests_its_knob_in_the_middle() {
    let mut harness = harness(200.0, 80.0).build_ui(|ui| {
        tristate_switch(ui, None, "Wi-Fi");
    });

    harness.run();
    let node = harness.get_by_role_and_label(Role::CheckBox, "Wi-Fi");
    assert_eq!(node.accesskit_node().toggled(), Some(Toggled::Mixed));

    let mut centre = None;
    for shape in crate::testing::painted(&harness) {
        if let egui::Shape::Circle(circle) = shape {
            centre = Some(circle.center);
        }
    }

    let rect = node.rect();
    let centre = centre.expect("the switch has a knob");
    assert!(
        (centre.x - rect.center().x).abs() < 0.5,
        "{centre:?} in {rect:?}"
    );
}
