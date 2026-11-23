use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Sense, Shape, TextEdit, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::{focus_from_keyboard, from_keys, item_focus_line};
use crate::testing::{click_at, focused, harness};
use crate::tokens;

/// Two buttons and a field. `ask` hands the focus to a button as the
/// program does. `shown` is what each button's fill would show, `ring`
/// what a ring around the focus would.
#[derive(Default)]
struct Row {
    text: String,
    ask: Option<usize>,
    shown: [bool; 2],
    ring: bool,
}

/// Whether the widget that holds the focus got it from the keyboard, as
/// a ring around it would show.
fn visible(ui: &egui::Ui) -> bool {
    ui.memory(egui::Memory::focused)
        .is_some_and(|focused| from_keys(ui, focused))
}

/// A click focuses, but only the keyboard shows where the focus went, as a
/// browser's `:focus-visible` does. A key while nothing holds the focus
/// shows nothing, as the ring follows egui's focus.
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
    assert!(
        !*harness.state(),
        "a window that opens has no focus to show"
    );

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

    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(focused(&harness).as_deref(), Some("First"));
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(focused(&harness).as_deref(), Some("Second"));
    assert!(*harness.state(), "an arrow key moved the focus");
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

fn item(selected: bool) -> Harness<'static, (bool, f32)> {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, (selected, bottom): &mut (bool, f32)| {
            let (rect, response) = ui.allocate_exact_size(vec2(200.0, 30.0), Sense::click());
            response.request_focus();
            item_focus_line(ui, &response, rect, *selected);
            *bottom = rect.max.y;
        },
        (selected, 0.0),
    );

    harness.run();
    harness
}

/// The lines in the Highlight the last frame drew.
fn highlight_lines(harness: &Harness<'_, (bool, f32)>) -> Vec<f32> {
    let highlight = tokens::current(&harness.ctx).accent_tint;
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let Shape::LineSegment { points, stroke } = &clipped.shape
                && stroke.color == highlight
            {
                Some(points[0].y)
            } else {
                None
            }
        })
        .collect()
}

/// Darkly's focused item that is not selected has a 1 px Highlight line
/// along its bottom edge, and no ring (darklystyle.cpp:4183-4222). A
/// selected one has none. The line shows only once a key moved the focus,
/// so a click or the program's focus shows none.
#[test]
fn a_focused_unselected_item_shows_a_highlight_line_at_its_bottom() {
    let mut harness = item(false);
    let (_, bottom) = *harness.state();
    assert_eq!(highlight_lines(&harness), Vec::<f32>::new());

    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(highlight_lines(&harness), vec![bottom - 0.5]);

    let mut selected = item(true);
    selected.key_press(Key::ArrowDown);
    selected.run();
    assert_eq!(highlight_lines(&selected), Vec::<f32>::new());
}

fn row() -> Harness<'static, Row> {
    let mut harness = harness(300.0, 200.0).build_ui_state(
        |ui, row: &mut Row| {
            let buttons = [ui.button("First"), ui.button("Second")];
            ui.add(TextEdit::singleline(&mut row.text).hint_text("Search"));
            if let Some(button) = row.ask.take().and_then(|at| buttons.get(at)) {
                button.request_focus();
            }

            row.shown = buttons
                .each_ref()
                .map(|button| focus_from_keyboard(ui, button));
            row.ring = visible(ui);
        },
        Row::default(),
    );

    harness.run();
    harness
}

fn tab_onto_first(harness: &mut Harness<'static, Row>) {
    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(focused(harness).as_deref(), Some("First"));
}

fn ask_focus(harness: &mut Harness<'static, Row>, at: usize) {
    harness.state_mut().ask = Some(at);
    harness.run();
}

/// The user's report: typing in a search field, then the program putting
/// the focus on a button, lit the button as if a key had moved there.
#[test]
fn a_key_elsewhere_does_not_light_a_focus_the_program_gave() {
    let mut harness = row();
    let field = harness.get_by_role(Role::TextInput).rect().center();
    click_at(&mut harness, field);
    harness.key_press(Key::A);
    harness.event(Event::Text("a".to_owned()));
    harness.run();
    assert_eq!(harness.state().text, "a", "the field took the key");

    ask_focus(&mut harness, 0);
    assert_eq!(focused(&harness).as_deref(), Some("First"));
    assert_eq!(harness.state().shown, [false, false]);
}

#[test]
fn tab_onto_a_button_lights_it() {
    let mut harness = row();
    tab_onto_first(&mut harness);
    assert_eq!(harness.state().shown, [true, false]);
}

/// The light follows egui's focus, so a focus the program moves on takes
/// it along and the program's new focus shows none.
#[test]
fn a_focus_the_program_moves_on_leaves_no_light_behind() {
    let mut harness = row();
    tab_onto_first(&mut harness);

    ask_focus(&mut harness, 1);
    assert_eq!(focused(&harness).as_deref(), Some("Second"));
    assert_eq!(harness.state().shown, [false, false]);

    ask_focus(&mut harness, 0);
    assert_eq!(
        harness.state().shown,
        [false, false],
        "the Tab of before still lit the first button"
    );
}

/// A press on nothing takes the focus with it. A press on the focused
/// button keeps the focus there and still puts the light out.
#[test]
fn a_pointer_press_anywhere_puts_the_light_out() {
    let mut harness = row();
    tab_onto_first(&mut harness);
    click_at(&mut harness, pos2(290.0, 190.0));
    assert_eq!(harness.state().shown, [false, false]);

    tab_onto_first(&mut harness);
    let first = harness.get_by_label("First").rect().center();
    click_at(&mut harness, first);
    assert_eq!(focused(&harness).as_deref(), Some("First"));
    assert_eq!(harness.state().shown, [false, false]);
}

/// Before any input the ring and the fill agree, so a page that opens
/// with the program's focus on a button shows neither.
#[test]
fn before_any_input_the_ring_and_the_fill_agree() {
    let mut harness = row();
    ask_focus(&mut harness, 0);
    assert_eq!(focused(&harness).as_deref(), Some("First"));
    assert_eq!(harness.state().shown, [false, false]);
    assert!(!harness.state().ring);
}
