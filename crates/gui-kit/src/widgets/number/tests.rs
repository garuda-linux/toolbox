use egui::accesskit::Role;
use egui::{Key, TextEdit};
use egui_kittest::kittest::Queryable as _;

use super::super::{number_field, number_field_with_unit, text_field_with_unit};
use super::committed;
use crate::testing::{harness, painted_text, text_colour};

const WIDTH: f32 = 72.0;
/// The boot timeout's field, which holds "300 s".
const UNIT_WIDTH: f32 = 72.0;

fn shown(value: u32) -> egui_kittest::Harness<'static, u32> {
    let mut harness = harness(300.0, 80.0).build_ui_state(
        |ui, value: &mut u32| {
            number_field(ui, value, 1..=30, WIDTH, "Pings");
        },
        value,
    );

    harness.run();
    harness
}

fn type_and_press(harness: &mut egui_kittest::Harness<'static, u32>, text: &str, key: Key) {
    harness
        .get_by_role_and_label(Role::TextInput, "Pings")
        .click();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Pings")
        .type_text(text);
    harness.run();
    harness.key_press(key);
    harness.run();
}

#[test]
fn a_typed_number_is_held_to_the_range_and_bad_text_keeps_the_value() {
    assert_eq!(committed("12", &(1..=30)), Some(12));
    assert_eq!(committed("99", &(1..=30)), Some(30));
    assert_eq!(committed("0", &(1..=30)), Some(1));
    assert_eq!(committed("", &(1..=30)), None);
    assert_eq!(committed("99999999999999", &(1..=30)), Some(30));
}

#[test]
fn the_digits_end_at_the_right_of_the_field() {
    let harness = shown(5);
    let field = harness
        .get_by_role_and_label(Role::TextInput, "Pings")
        .rect();

    let digits = painted_text(&harness, "5").expect("the number is painted");
    let right = digits.pos.x + digits.galley.rect.right();
    let padding = harness.ctx.global_style().spacing.button_padding.x;

    assert!((field.width() - WIDTH).abs() < 1.0, "{}", field.width());
    assert!(
        (field.right() - padding - right).abs() < 1.5,
        "the digits end at {right}, the field at {}",
        field.right()
    );
}

#[test]
fn enter_commits_the_number_held_to_its_range() {
    let mut harness = shown(5);
    type_and_press(&mut harness, "99", Key::Enter);

    assert_eq!(*harness.state(), 30);
}

#[test]
fn letters_are_not_typed_and_escape_keeps_the_value() {
    let mut harness = shown(5);
    type_and_press(&mut harness, "1x2", Key::Enter);
    assert_eq!(*harness.state(), 12);

    type_and_press(&mut harness, "7", Key::Escape);
    assert_eq!(*harness.state(), 12);
}

fn with_unit(value: u32, width: f32) -> egui_kittest::Harness<'static, u32> {
    let mut harness = harness(300.0, 80.0).build_ui_state(
        move |ui, value: &mut u32| {
            number_field_with_unit(ui, value, 0..=300, width, ("Pings", "s"));
        },
        value,
    );

    harness.run();
    harness
}

fn painted_at<State>(harness: &egui_kittest::Harness<'_, State>, text: &str) -> egui::Rect {
    let shape = painted_text(harness, text).expect("the text is painted");
    shape.galley.rect.translate(shape.pos.to_vec2())
}

fn assert_centred_group<State>(
    harness: &egui_kittest::Harness<'_, State>,
    name: &str,
    (digits, unit): (&str, &str),
) {
    let field = harness.get_by_role_and_label(Role::TextInput, name).rect();
    let digits = painted_at(harness, digits);
    let unit = painted_at(harness, unit);
    let gap = unit.left() - digits.right();
    let group = f32::midpoint(digits.left(), unit.right());

    assert!(gap > 0.0 && gap < 12.0, "{digits:?} then {unit:?}");
    assert!(
        (group - field.center().x).abs() < 1.0,
        "{digits:?} {unit:?} in {field:?}"
    );
}

#[test]
fn a_number_and_its_unit_are_one_group_centred_in_the_field() {
    for (value, digits) in [(5, "5"), (300, "300")] {
        let harness = with_unit(value, UNIT_WIDTH);
        assert_centred_group(&harness, "Pings", (digits, "s"));
    }

    let harness = with_unit(5, UNIT_WIDTH);
    assert_eq!(
        text_colour(&harness, "s"),
        Some(crate::tokens::current(&harness.ctx).secondary)
    );
}

#[test]
fn a_text_field_with_a_unit_centres_the_two_as_well() {
    let mut harness = harness(300.0, 80.0).build_ui(|ui| {
        let mut text = "60".to_owned();
        let edit = TextEdit::singleline(&mut text).desired_width(WIDTH);
        let response = text_field_with_unit(ui, edit, "min");
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label("Minutes"));
    });

    harness.run();
    assert_centred_group(&harness, "Minutes", ("60", "min"));
}

#[test]
fn a_field_with_a_unit_still_takes_only_the_number() {
    let mut harness = with_unit(5, UNIT_WIDTH);
    type_and_press(&mut harness, "12", Key::Enter);

    assert_eq!(*harness.state(), 12);
}
