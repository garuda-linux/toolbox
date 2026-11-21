use egui::accesskit::Role;
use egui::{Key, Modifiers, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::chips;
use crate::testing::{focused, harness};

const OPTIONS: [(Scope, &str); 3] = [
    (Scope::One, "One"),
    (Scope::Two, "Two"),
    (Scope::Three, "Three"),
];

const MANY: [(Scope, &str); 6] = [
    (Scope::One, "One"),
    (Scope::Two, "Two"),
    (Scope::Three, "Three"),
    (Scope::One, "Four"),
    (Scope::Two, "Five"),
    (Scope::Three, "Six"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    One,
    Two,
    Three,
}

/// A button before the chips and one after, so Tab can be followed through
/// the group. The picked chip is kept in the state.
fn group() -> Harness<'static, Option<Scope>> {
    group_with(Scope::One)
}

fn group_with(selected: Scope) -> Harness<'static, Option<Scope>> {
    let mut harness = harness(400.0, 200.0).build_ui_state(
        move |ui: &mut Ui, picked: &mut Option<Scope>| {
            _ = ui.button("Before");
            if let Some(value) = chips(ui, "chips", &OPTIONS, selected).value {
                *picked = Some(value);
            }

            _ = ui.button("After");
        },
        None,
    );

    harness.run();
    harness
}

fn narrow() -> Harness<'static, Option<Scope>> {
    let mut harness = harness(160.0, 200.0).build_ui_state(
        |ui: &mut Ui, picked: &mut Option<Scope>| {
            if let Some(value) = chips(ui, "chips", &MANY, Scope::One).value {
                *picked = Some(value);
            }
        },
        None,
    );

    harness.run();
    harness
}

fn press(harness: &mut Harness<'static, Option<Scope>>, key: Key) {
    harness.key_press(key);
    harness.run();
}

#[test]
fn tab_reaches_one_stop_in_a_chip_group() {
    let mut harness = group();

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("Before"));

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("One"));

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("After"));
}

#[test]
fn arrow_keys_move_the_focus_along_the_chips() {
    let mut harness = group();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);

    press(&mut harness, Key::ArrowRight);
    assert_eq!(focused(&harness).as_deref(), Some("Two"));

    press(&mut harness, Key::ArrowDown);
    assert_eq!(focused(&harness).as_deref(), Some("Three"));

    press(&mut harness, Key::ArrowRight);
    assert_eq!(
        focused(&harness).as_deref(),
        Some("Three"),
        "the last chip stops the focus"
    );

    press(&mut harness, Key::Home);
    assert_eq!(focused(&harness).as_deref(), Some("One"));

    press(&mut harness, Key::End);
    assert_eq!(focused(&harness).as_deref(), Some("Three"));
}

#[test]
fn enter_and_space_pick_the_focused_chip() {
    let mut harness = group();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);

    press(&mut harness, Key::ArrowRight);
    press(&mut harness, Key::Enter);
    assert_eq!(*harness.state(), Some(Scope::Two));

    press(&mut harness, Key::ArrowRight);
    press(&mut harness, Key::Space);
    assert_eq!(*harness.state(), Some(Scope::Three));
}

#[test]
fn the_tab_stop_of_a_group_is_its_selected_chip() {
    let mut harness = group_with(Scope::Three);

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("Before"));

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("Three"));
}

#[test]
fn leaving_the_group_puts_its_stop_back_on_the_selected_chip() {
    let mut harness = group();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::ArrowRight);

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("After"));

    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(focused(&harness).as_deref(), Some("One"));
}

#[test]
fn a_chip_group_wider_than_its_room_wraps_onto_a_second_line() {
    let harness = narrow();
    let first = harness.get_by_role_and_label(Role::Button, "One").rect();
    let fourth = harness.get_by_role_and_label(Role::Button, "Four").rect();
    let last = harness.get_by_role_and_label(Role::Button, "Six").rect();

    assert!(
        (first.height() - 32.0).abs() < 0.5,
        "a chip is one pill high, got {}",
        first.height()
    );
    assert!(
        (first.min.y - fourth.min.y).abs() > 1.0,
        "the chips past the room's width start a second line, at {} and {}",
        first.min.y,
        fourth.min.y
    );
    assert!(
        last.max.x <= 160.5,
        "every chip of the group stays inside the room, the last ends at {}",
        last.max.x
    );
}

#[test]
fn the_first_chip_keeps_its_whole_hit_area() {
    let mut harness = narrow();
    let first = harness.get_by_role_and_label(Role::Button, "One").rect();
    assert!(
        first.width() > 40.0,
        "the first chip is whole, got {} wide",
        first.width()
    );

    harness.get_by_role_and_label(Role::Button, "One").click();
    harness.run();
    assert_eq!(*harness.state(), Some(Scope::One));
}
