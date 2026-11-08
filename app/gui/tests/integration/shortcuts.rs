use egui::accesskit::{Role, Toggled};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::{App, Setup};

use crate::common::{launch, launch_in, palette};
use crate::shell::SETUP;

const PALETTE: &str = "Command palette";

fn press(harness: &mut Harness<'_, App>, key: Key) {
    harness.key_press_modifiers(Modifiers::COMMAND, key);
    harness.run();
}

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
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::Num7);

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

    let marked: Vec<String> = harness
        .get_by_role_and_label(Role::Dialog, PALETTE)
        .query_all_by_role(Role::Button)
        .filter(|row| row.accesskit_node().toggled() == Some(Toggled::True))
        .filter_map(|row| row.accesskit_node().label())
        .collect();

    assert_eq!(marked, ["Users and Groups"]);
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

#[test]
fn the_console_shows_how_to_leave_its_terminal_while_it_has_focus() {
    let hint = "Ctrl+Tab leaves the terminal.";
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::J);
    assert!(toolbox_gui_kit::testing::painted_text(&harness, hint).is_none());

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
    press(&mut harness, Key::J);
    harness
        .get_by_role_and_label(Role::Terminal, "Console output")
        .click();
    harness.run();

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
