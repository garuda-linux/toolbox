use egui::accesskit::Role;
use egui::{Key, Modifiers, Ui};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};

use super::super::chips;
use crate::testing::harness;

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
    let mut harness = harness(400.0, 200.0).build_ui_state(
        |ui: &mut Ui, picked: &mut Option<Scope>| {
            _ = ui.button("Before");
            if let Some(value) = chips(ui, &OPTIONS, Scope::One).value {
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
            if let Some(value) = chips(ui, &MANY, Scope::One).value {
                *picked = Some(value);
            }
        },
        None,
    );

    harness.run();
    harness
}

fn focused(harness: &Harness<'static, Option<Scope>>) -> Option<String> {
    let node = harness
        .query_all(egui_kittest::kittest::by())
        .find(|node| node.accesskit_node().is_focused())?;

    node.accesskit_node().label()
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
fn shift_tab_leaves_the_group_from_the_moved_stop() {
    let mut harness = group();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::ArrowRight);

    press(&mut harness, Key::Tab);
    assert_eq!(focused(&harness).as_deref(), Some("After"));

    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(focused(&harness).as_deref(), Some("Two"));
}

#[test]
fn a_chip_line_too_long_for_its_width_scrolls_on_one_line() {
    let harness = narrow();
    let first = harness.get_by_role_and_label(Role::Button, "One").rect();
    let last = harness.get_by_role_and_label(Role::Button, "Six").rect();

    assert!(
        last.min.x >= 160.0,
        "the last chip starts beyond the row's edge, at {}",
        last.min.x
    );
    assert!(
        (first.height() - 32.0).abs() < 0.5,
        "a chip is one pill high, got {}",
        first.height()
    );
    assert!(
        (first.min.y - last.min.y).abs() < 0.5,
        "the chips share one line"
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

#[test]
fn the_keyboard_scrolls_the_line_to_the_focused_chip() {
    let mut harness = narrow();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::End);

    let last = harness.get_by_role_and_label(Role::Button, "Six").rect();
    assert!(
        last.max.x <= 160.5,
        "the focused last chip scrolls into view, its right edge at {}",
        last.max.x
    );
}

#[test]
fn a_chip_line_shows_its_fades_and_the_focus_ring_whole() {
    let mut results = SnapshotResults::new();
    let mut harness = narrow();
    press(&mut harness, Key::Tab);
    results.add(harness.try_snapshot("widgets/chip_line_start"));

    press(&mut harness, Key::End);
    results.add(harness.try_snapshot("widgets/chip_line_end"));
}
