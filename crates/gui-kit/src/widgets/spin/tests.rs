use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::{SpinTexts, spin_field};
use crate::testing::{filled_rects, harness, painted_text_within};
use crate::theme::PILL_HEIGHT;

const WIDTH: f32 = 140.0;
const NAME: &str = "Menu timeout in seconds";
const LESS: &str = "One second less";
const MORE: &str = "One second more";

fn shown(value: u32) -> Harness<'static, u32> {
    let mut harness = harness(300.0, 80.0).build_ui_state(
        |ui, value: &mut u32| {
            let texts = SpinTexts {
                name: NAME,
                unit: "s",
                less: LESS,
                more: MORE,
            };

            spin_field(ui, value, 0..=10, WIDTH, texts);
        },
        value,
    );

    harness.run();
    harness
}

fn press(harness: &mut Harness<'static, u32>, label: &str) {
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run();
}

#[test]
fn the_steppers_sit_inside_the_one_fill_at_its_ends() {
    let harness = shown(5);
    let less = harness.get_by_role_and_label(Role::Button, LESS).rect();
    let more = harness.get_by_role_and_label(Role::Button, MORE).rect();
    let number = harness.get_by_role_and_label(Role::SpinButton, NAME).rect();
    let field = crate::tokens::current(&harness.ctx).field;

    let fill = filled_rects(&harness, field)
        .into_iter()
        .find(|fill| fill.contains_rect(less) && fill.contains_rect(more))
        .expect("one fill holds both steppers");

    assert!((fill.width() - WIDTH).abs() < 0.5, "{fill:?}");
    assert!((fill.height() - PILL_HEIGHT).abs() < 0.5, "{fill:?}");
    assert!((less.left() - fill.left()).abs() < 0.5, "{less:?} {fill:?}");
    assert!(
        (more.right() - fill.right()).abs() < 0.5,
        "{more:?} {fill:?}"
    );
    assert!(
        less.right() <= number.left() + 0.5 && number.right() <= more.left() + 0.5,
        "the number {number:?} lies between {less:?} and {more:?}"
    );
}

#[test]
fn the_number_and_its_unit_are_painted_between_the_steppers() {
    let harness = shown(5);
    let number = harness.get_by_role_and_label(Role::SpinButton, NAME).rect();

    assert!(painted_text_within(&harness, "5", number).is_some());
    assert!(painted_text_within(&harness, "s", number).is_some());
}

#[test]
fn the_steppers_step_by_one_and_stop_at_the_range() {
    let mut harness = shown(9);

    press(&mut harness, MORE);
    assert_eq!(*harness.state(), 10);
    press(&mut harness, MORE);
    assert_eq!(*harness.state(), 10);

    let mut low = shown(0);
    press(&mut low, LESS);
    assert_eq!(*low.state(), 0);
    press(&mut low, MORE);
    assert_eq!(*low.state(), 1);
}

#[test]
fn a_typed_number_counts_once_enter_commits_it() {
    let mut harness = shown(5);

    harness
        .get_by_role_and_label(Role::SpinButton, NAME)
        .click();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::SpinButton, NAME)
        .type_text("7");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(*harness.state(), 7);
}

#[test]
fn the_steppers_are_reached_with_the_keyboard() {
    let mut harness = shown(5);

    harness.key_press(Key::Tab);
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(*harness.state(), 6, "Tab reaches less, the number, more");
}
