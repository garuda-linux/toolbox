use egui::{Key, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::segmented;
use crate::palette::Palette;
use crate::testing::{filled_rect, harness};
use crate::theme;
use crate::tokens::Tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    System,
    User,
}

fn scopes() -> Harness<'static, Scope> {
    let mut harness = harness(400.0, 120.0)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(60)
        .build_ui_state(
            |ui, current: &mut Scope| {
                let options = [(Scope::System, "System"), (Scope::User, "User")];
                if let Some(value) = segmented(ui, "Units", &options, *current) {
                    *current = value;
                }
            },
            Scope::System,
        );

    // The harness turns egui's own motion off, these tests watch it.
    harness
        .ctx
        .all_styles_mut(|style| style.animation_time = theme::ANIMATION_TIME);
    harness.run();
    harness
}

fn pill_left(harness: &Harness<'static, Scope>) -> f32 {
    let raised = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark).raised;
    filled_rect(harness, raised).expect("the pill").left()
}

fn left_of(harness: &Harness<'static, Scope>, label: &str) -> f32 {
    harness.get_by_label(label).rect().left()
}

#[test]
fn a_click_slides_the_pill_to_the_segment() {
    let mut harness = scopes();
    let system = left_of(&harness, "System");
    let user = left_of(&harness, "User");
    assert!((pill_left(&harness) - system).abs() < 0.5);

    harness.get_by_label("User").click();
    harness.step();
    harness.step();
    harness.step();

    let moving = pill_left(&harness);
    assert!(moving > system + 0.5 && moving < user - 0.5, "{moving}");

    harness.run();
    assert!((pill_left(&harness) - user).abs() < 0.5);
}

#[test]
fn an_arrow_key_moves_the_pill_at_once() {
    let mut harness = scopes();
    let user = left_of(&harness, "User");

    harness.key_press(Key::Tab);
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.step();
    harness.step();

    assert!((pill_left(&harness) - user).abs() < 0.5);
}
