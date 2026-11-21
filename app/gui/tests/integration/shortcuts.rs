use egui::Key;
use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::{App, Setup};

use crate::common::{focus_terminal, launch, launch_in, palette, press_ctrl as press};
use crate::shell::SETUP;

const PALETTE: &str = "Command palette";

#[test]
fn ctrl_b_toggles_the_sidebar() {
    let mut harness = launch(Setup::default());

    press(&mut harness, Key::B);
    assert!(harness.query_by_label("Expand sidebar").is_some());

    press(&mut harness, Key::B);
    assert!(harness.query_by_label("Collapse sidebar").is_some());
}

#[test]
fn ctrl_and_a_digit_opens_that_page() {
    let mut harness = launch(Setup::default());

    press(&mut harness, Key::Num3);

    assert_eq!(harness.query_all_by_label("Components").count(), 2);
}

#[test]
fn ctrl_j_toggles_the_console() {
    let mut harness = launch(Setup::default());

    press(&mut harness, Key::J);
    assert!(
        harness
            .query_by_label("No changes are being applied")
            .is_some()
    );

    press(&mut harness, Key::J);
    assert!(
        harness
            .query_by_label("No changes are being applied")
            .is_none()
    );
}

#[test]
fn the_palette_opens_a_page_by_typing_part_of_its_name() {
    let mut harness = launch(Setup::default());

    palette(&mut harness, "setup as");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.query_all_by_label("Setup Assistant").count(), 2);
    assert!(harness.query_by_label(SETUP).is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, PALETTE)
            .is_none()
    );
}

#[test]
fn the_palette_runs_a_command() {
    let mut harness = launch(Setup::default());

    palette(&mut harness, "console");
    harness.key_press(Key::Enter);
    harness.run();

    assert!(
        harness
            .query_by_label("No changes are being applied")
            .is_some()
    );
}

#[test]
fn escape_closes_the_palette() {
    let mut harness = launch(Setup::default());

    press(&mut harness, Key::K);
    harness.key_press(Key::Escape);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, PALETTE)
            .is_none()
    );
}

#[test]
fn ctrl_f_puts_the_cursor_into_the_page_search() {
    // The search shows once a list exists, so the units are read first.
    let (mut harness, _runtime) = crate::services::services();

    press(&mut harness, Key::F);

    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, "Search services")
            .is_focused()
    );
}

#[test]
fn arrow_keys_move_the_palette_row_that_enter_takes() {
    let mut harness = launch(Setup::default());

    palette(&mut harness, "ser");
    harness.key_press(Key::ArrowDown);
    harness.run();

    assert_eq!(marked_rows(&harness), ["Users and Groups"]);
    assert!(
        harness
            .get_by_role_and_label(Role::TextInput, PALETTE)
            .is_focused(),
        "the typing goes on"
    );

    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.query_all_by_label("Users and Groups").count(), 2);
}

fn marked_rows(harness: &Harness<'_, App>) -> Vec<String> {
    harness
        .get_by_role_and_label(Role::Dialog, PALETTE)
        .query_all_by_role(Role::Button)
        .filter(|row| row.accesskit_node().toggled() == Some(Toggled::True))
        .filter_map(|row| row.accesskit_node().label())
        .collect()
}

fn palette_field(harness: &Harness<'_, App>) -> egui::Rect {
    harness
        .get_by_role_and_label(Role::TextInput, PALETTE)
        .rect()
}

/// The palette hangs from the top, so a shorter list leaves the field
/// where the eye is.
#[test]
fn the_palette_field_stays_put_as_the_list_shortens() {
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::K);
    let full = palette_field(&harness);

    harness
        .get_by_role_and_label(Role::TextInput, PALETTE)
        .type_text("zzzz");
    harness.run();

    assert_eq!(palette_field(&harness), full);
}

#[test]
fn the_palette_lists_pages_and_commands_under_their_labels() {
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::K);

    let dialog = harness.get_by_role_and_label(Role::Dialog, PALETTE);
    let labels: Vec<String> = dialog
        .query_all_by_role(Role::Label)
        .filter_map(|node| node.accesskit_node().value())
        .collect();

    assert!(labels.iter().any(|label| label == "Pages"), "{labels:?}");
    assert!(labels.iter().any(|label| label == "Commands"), "{labels:?}");
}

/// The shortcut shows as one cap per key, and a screen reader still hears
/// it after the name.
#[test]
fn a_palette_shortcut_shows_as_keycaps() {
    let mut harness = launch(Setup::default());
    palette(&mut harness, "console");

    let named = harness
        .query_all_by_label_contains("Toggle console")
        .filter_map(|node| node.accesskit_node().label())
        .any(|label| label.starts_with("Toggle console ") && label.ends_with('J'));

    assert!(named, "the row's name ends in its shortcut");
    assert!(toolbox_gui_kit::testing::painted_text(&harness, "J").is_some());
}

#[test]
fn a_hovered_palette_row_is_the_one_marked_until_a_key_moves_the_mark() {
    let mut harness = launch(Setup::default());
    palette(&mut harness, "ser");

    harness
        .get_by_role_and_label(Role::Dialog, PALETTE)
        .get_by_role_and_label(Role::Button, "Users and Groups")
        .hover();
    harness.run();
    assert_eq!(marked_rows(&harness), ["Users and Groups"]);

    harness.key_press(Key::ArrowUp);
    harness.run();
    assert_ne!(
        marked_rows(&harness),
        ["Users and Groups"],
        "a resting pointer leaves the key's mark"
    );
}

#[test]
fn the_console_header_names_how_to_leave_its_terminal() {
    let hint = "Ctrl+Tab leaves the terminal.";
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::J);
    assert!(toolbox_gui_kit::testing::painted_text(&harness, hint).is_some());

    harness
        .get_by_role_and_label(Role::Terminal, "Console output")
        .click();
    harness.run();

    assert!(toolbox_gui_kit::testing::painted_text(&harness, hint).is_some());
    assert_eq!(
        harness
            .get_by_role_and_label(Role::Terminal, "Console output")
            .accesskit_node()
            .description()
            .as_deref(),
        Some(hint)
    );
}

/// nano in the Console cuts a line with Ctrl+K and vim pages with Ctrl+B.
#[test]
fn app_shortcuts_leave_the_keys_to_the_focused_terminal() {
    let mut harness = launch(Setup::default());
    focus_terminal(&mut harness);

    press(&mut harness, Key::K);
    press(&mut harness, Key::B);

    assert!(harness.query_by_label(PALETTE).is_none());
    assert!(harness.query_by_label("Collapse sidebar").is_some());
}

#[test]
fn a_narrow_window_starts_with_the_sidebar_folded_and_ctrl_b_unfolds_it() {
    let mut harness = launch_in(
        toolbox_gui_kit::testing::harness(600.0, 600.0),
        Setup::default(),
    );

    assert!(harness.query_by_label("Expand sidebar").is_some());

    press(&mut harness, Key::B);
    assert!(harness.query_by_label("Collapse sidebar").is_some());
}

#[test]
fn f1_opens_the_guide_on_the_page_it_was_pressed_on() {
    let mut harness = launch(Setup {
        open: Some(toolbox_gui::page::PageId::Kernels),
        ..Setup::default()
    });

    harness.key_press(Key::F1);
    harness.run();

    assert!(
        harness.query_all_by_role(Role::Heading).any(|node| node
            .accesskit_node()
            .label()
            .as_deref()
            == Some("Install a kernel"))
    );
    assert!(harness.query_by_label("Open Kernels").is_some());
}

#[cfg(debug_assertions)]
#[test]
fn f12_flips_the_debug_overlay_also_over_a_dialog() {
    use toolbox_gui::debug_overlay::showing;

    let mut harness = launch(Setup::default());
    harness.key_press(Key::F12);
    harness.run();
    assert!(showing(&harness.ctx));

    press(&mut harness, Key::K);
    harness.key_press(Key::F12);
    harness.run();
    assert!(!showing(&harness.ctx), "the palette is open");
}

#[cfg(debug_assertions)]
#[test]
fn a_focused_terminal_keeps_f12() {
    let mut harness = launch(Setup::default());
    focus_terminal(&mut harness);

    harness.key_press(Key::F12);
    harness.run();

    assert!(!toolbox_gui::debug_overlay::showing(&harness.ctx));
}
