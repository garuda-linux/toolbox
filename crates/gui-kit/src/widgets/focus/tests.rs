use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, TextEdit};
use egui_kittest::kittest::Queryable as _;

use super::visible;
use crate::testing::harness;

/// A click focuses as before, but only the keyboard shows where the
/// focus went, as a browser's `:focus-visible` does.
#[test]
fn a_click_hides_the_ring_and_a_key_brings_it_back() {
    let mut harness = harness(300.0, 200.0).build_ui_state(
        |ui, shown: &mut bool| {
            drop(ui.button("First"));
            drop(ui.button("Second"));
            *shown = visible(ui);
        },
        false,
    );

    harness.run();
    assert!(*harness.state(), "a window that opens shows it");

    harness.get_by_label("First").click();
    harness.run();
    assert!(!*harness.state());

    harness.run();
    assert!(!*harness.state(), "until a key moves the focus");

    harness.key_press(Key::Tab);
    harness.run();
    assert!(*harness.state());

    harness.get_by_label("Second").click();
    harness.run();
    assert!(!*harness.state());

    harness.key_press(Key::ArrowDown);
    harness.run();
    assert!(*harness.state());
}

/// The flag follows the pointer from the start of every frame, not only
/// once a ring would draw, so a control that takes the focus on the press
/// shows none while the button is held. A text edit is such a control.
#[test]
fn a_press_that_focuses_hides_the_ring_in_every_frame() {
    let mut harness = harness(300.0, 200.0).build_ui_state(
        |ui, (text, focused, rings): &mut (String, usize, usize)| {
            // The ring is asked before the text edit, from the focus it
            // held when the frame began.
            let id = ui.make_persistent_id("field");
            if ui.memory(|memory| memory.has_focus(id)) {
                *focused += 1;
                *rings += usize::from(visible(ui));
            }

            ui.add(TextEdit::singleline(text).id(id).hint_text("Name"));
        },
        (String::new(), 0, 0),
    );

    harness.run();
    let at = harness.get_by_role(Role::TextInput).rect().center();
    let press = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };

    // A field takes the focus on the press. The frames while the button
    // is held have no pointer event.
    harness.event(Event::PointerMoved(at));
    harness.event(press(true));
    for _ in 0..4_u8 {
        harness.step();
    }

    harness.event(press(false));
    harness.step();

    let (_, focused, rings) = *harness.state();
    assert!(focused > 0, "the click focused the field");
    assert_eq!(rings, 0);
}
