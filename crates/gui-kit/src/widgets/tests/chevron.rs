use std::f32::consts::PI;

use egui::accesskit::Role;
use egui::{Key, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::Select;
use crate::testing::{harness, painted, tinted_rects};
use crate::{theme, tokens};

const OPTIONS: [&str; 2] = ["Name", "Size"];
const FRAME: f32 = 1.0 / 60.0;

fn dropdown(less_motion: bool) -> Harness<'static> {
    let mut harness = harness(400.0, 300.0)
        .with_step_dt(FRAME)
        .with_max_steps(30)
        .build_ui(|ui| {
            _ = Select::new("Sort by", &OPTIONS).selected(Some(0)).show(ui);
        });

    theme::set_reduced_motion(&harness.ctx, less_motion);
    harness.run();
    harness
}

/// How far the chevron turned, from 0 to π, by the top edge of its glyph:
/// a turned glyph is a mesh, one at rest a tinted rectangle.
fn turn(harness: &Harness<'_>) -> f32 {
    // Darkly's arrow of a closed box takes the highlighted text while the
    // list is open (darklystyle.cpp:7817-7820), and at rest it is the
    // mix toward the view colour.
    let tokens = tokens::current(&harness.ctx);
    let rest = tokens::mix(tokens.text, tokens.view, 0.15);

    // A focused box shows its arrow in the colour that reads on the focus fill.
    let inks = [rest, tokens.highlighted_text, tokens.on_focus];
    let turned = painted(harness).into_iter().find_map(|shape| {
        let Shape::Mesh(mesh) = shape else {
            return None;
        };

        match (mesh.vertices.first(), mesh.vertices.get(1)) {
            (Some(first), Some(second)) if inks.contains(&first.color) => {
                Some((second.pos - first.pos).angle().abs())
            }
            (Some(_) | None, Some(_) | None) => None,
        }
    });

    turned.unwrap_or_else(|| {
        assert!(
            inks.iter()
                .any(|ink| !tinted_rects(harness, *ink).is_empty()),
            "no chevron"
        );
        0.0
    })
}

fn open(harness: &mut Harness<'_>) {
    harness
        .get_by_role_and_label(Role::ComboBox, "Sort by")
        .click();
    harness.step();
}

#[test]
fn the_chevron_turns_over_while_the_list_is_open() {
    let mut harness = dropdown(false);
    assert!(turn(&harness) < 0.01, "at rest it points down");

    open(&mut harness);
    let started = turn(&harness);
    assert!(started > 0.0 && started < PI - 0.01, "it turns: {started}");

    harness.run();
    assert!((turn(&harness) - PI).abs() < 0.01, "it points up");

    harness.key_press(Key::Escape);
    harness.run();
    assert!(turn(&harness) < 0.01, "closed, it points down again");
}

#[test]
fn with_less_motion_the_chevron_turns_at_once() {
    let mut harness = dropdown(true);

    open(&mut harness);

    assert!((turn(&harness) - PI).abs() < 0.01);
}

fn open_with(harness: &mut Harness<'_>, key: Key) {
    harness
        .get_by_role_and_label(Role::ComboBox, "Sort by")
        .focus();
    harness.run();
    harness.key_press(key);
    harness.step();
}

#[test]
fn opened_with_space_or_enter_the_chevron_turns_at_once() {
    for key in [Key::Space, Key::Enter] {
        let mut harness = dropdown(false);
        open_with(&mut harness, key);

        assert!((turn(&harness) - PI).abs() < 0.01, "{key:?}");
    }
}

#[test]
fn closed_with_a_key_the_chevron_turns_back_at_once() {
    for key in [Key::Escape, Key::Enter] {
        let mut harness = dropdown(false);
        open_with(&mut harness, Key::Space);
        harness.run();
        harness.key_press(key);
        harness.step();

        assert!(turn(&harness) < 0.01, "{key:?}");
    }
}

#[test]
fn an_arrow_key_inside_the_open_list_leaves_the_chevron_turned() {
    let mut harness = dropdown(false);
    open_with(&mut harness, Key::Space);
    harness.run();
    harness.key_press(Key::ArrowDown);
    harness.step();

    assert!((turn(&harness) - PI).abs() < 0.01);
}

#[test]
fn a_key_during_a_clicked_turn_lets_it_finish_turning() {
    let mut harness = dropdown(false);
    open(&mut harness);
    harness.key_press(Key::ArrowDown);
    harness.step();

    let turned = turn(&harness);
    assert!(turned < PI - 0.01, "still turning: {turned}");

    harness.run();
    assert!((turn(&harness) - PI).abs() < 0.01, "it points up");
}
