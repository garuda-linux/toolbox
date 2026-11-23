use egui::accesskit::Role;
use egui::epaint::ColorMode;
use egui::{Color32, Key, Shape, TextStyle};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::buttons::BUTTON_EXTRA_HEIGHT;
use super::{SpinTexts, spin_field};
use crate::testing::{click_at, filled_rects, focused, harness, painted_text_within};
use crate::tokens;

const WIDTH: f32 = 140.0;
const NAME: &str = "Menu timeout in seconds";
const LESS: &str = "One second less";
const MORE: &str = "One second more";

fn shown(value: u32) -> Harness<'static, u32> {
    let mut harness = harness(300.0, 80.0).build_ui_state(
        |ui, value: &mut u32| {
            let texts = SpinTexts {
                salt: NAME,
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

/// The field's height, a push button's: the text line and the button's
/// extra height, as `push_height` takes them.
fn field_height(harness: &Harness<'_, u32>) -> f32 {
    let font = harness.ctx.global_style().text_styles[&TextStyle::Body].clone();
    harness
        .ctx
        .fonts_mut(|fonts| fonts.row_height(&font))
        .round()
        + BUTTON_EXTRA_HEIGHT
}

/// Darkly's spin box: one field, the number on its left and the arrows
/// in a 20 px column, up above down, at the right end of the fill.
#[test]
fn the_arrows_stack_in_a_column_inside_the_one_field() {
    let harness = shown(5);
    let less = harness.get_by_role_and_label(Role::Button, LESS).rect();
    let more = harness.get_by_role_and_label(Role::Button, MORE).rect();
    let number = harness.get_by_role_and_label(Role::SpinButton, NAME).rect();
    let input = tokens::current(&harness.ctx).input;

    let frame = filled_rects(&harness, input)
        .into_iter()
        .find(|fill| fill.contains_rect(less) && fill.contains_rect(more))
        .expect("one fill holds both arrows");

    assert!((frame.width() - (WIDTH - 4.0)).abs() < 0.5, "{frame:?}");
    assert!((frame.height() - (field_height(&harness) - 4.0)).abs() < 0.5);
    assert!((less.width() - 20.0).abs() < 0.5, "{less:?}");
    assert!((more.width() - 20.0).abs() < 0.5, "{more:?}");
    assert!((more.height() - 10.0).abs() < 0.5, "{more:?}");
    assert!((less.height() - 10.0).abs() < 0.5, "{less:?}");
    assert!((less.top() - more.bottom()).abs() < 0.5, "up is above down");
    assert!(
        (less.right() - frame.right()).abs() < 0.5,
        "the column sits at the fill's end"
    );
    assert!(
        number.right() <= more.left() + 0.5,
        "the number {number:?} lies left of the arrows {more:?}"
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

/// The strokes of the arrows the last frame drew, less then more.
fn arrow_inks(harness: &Harness<'_, u32>) -> Vec<Color32> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let Shape::Path(path) = &clipped.shape
                && !path.closed
                && let ColorMode::Solid(colour) = path.stroke.color
            {
                Some(colour)
            } else {
                None
            }
        })
        .collect()
}

/// Darkly's arrows rest in the text shaded toward the base
/// (darklyhelper.cpp:56, 190), and one under the pointer takes the view
/// focus colour (darklystyle.cpp:8323).
#[test]
fn an_arrow_rests_shaded_and_takes_the_focus_colour_under_the_pointer() {
    let mut harness = shown(5);
    let tokens = tokens::current(&harness.ctx);
    let rest = tokens::mix(tokens.text, tokens.view, 0.15);
    assert_eq!(arrow_inks(&harness), vec![rest, rest]);

    let more_at = harness.get_by_role_and_label(Role::Button, MORE).rect();
    harness.hover_at(more_at.center());
    harness.run();

    assert_eq!(arrow_inks(&harness), vec![rest, tokens.decoration_focus]);
}

/// An arrow that cannot step further is drawn in the disabled text
/// (darklystyle.cpp:8337).
#[test]
fn an_arrow_at_its_limit_draws_in_the_dim_text() {
    let harness = shown(10);
    let tokens = tokens::current(&harness.ctx);

    assert_eq!(arrow_inks(&harness)[1], tokens.dim);
}

/// Tab reaches the arrows, so a focused one shows it as the pointer does,
/// with no ring (ui-rules.md: keyboard focus is a fill or a line). A
/// click keeps the focus but hides it, so the arrow rests once the
/// pointer leaves.
#[test]
fn an_arrow_shows_the_focus_of_a_key_and_hides_it_after_a_click() {
    let mut harness = shown(5);
    let tokens = tokens::current(&harness.ctx);
    let rest = tokens::mix(tokens.text, tokens.view, 0.15);

    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(arrow_inks(&harness), vec![tokens.decoration_focus, rest]);

    let at = harness
        .get_by_role_and_label(Role::Button, LESS)
        .rect()
        .center();

    click_at(&mut harness, at);
    harness.hover_at(egui::pos2(290.0, 70.0));
    harness.run();

    assert_eq!(
        focused(&harness).as_deref(),
        Some(LESS),
        "the click keeps the focus"
    );
    assert_eq!(arrow_inks(&harness), vec![rest, rest], "after the click");
}

/// An arrow at its limit does nothing, so a screen reader hears it
/// disabled.
#[test]
fn an_arrow_at_its_limit_is_spoken_disabled() {
    let harness = shown(10);

    let more = harness.get_by_role_and_label(Role::Button, MORE);
    let less = harness.get_by_role_and_label(Role::Button, LESS);
    assert!(more.accesskit_node().is_disabled());
    assert!(!less.accesskit_node().is_disabled());
}
