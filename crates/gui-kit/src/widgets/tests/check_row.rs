use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{CheckRowTexts, check_row, check_row_with_line};
use crate::testing::harness;

#[test]
fn a_check_row_is_a_check_box_that_reports_the_click() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, on: &mut bool| {
            if check_row(ui, *on, "Kitty", "~/.config/kitty/kitty.conf", true).clicked() {
                *on ^= true;
            }
        },
        false,
    );

    harness.run();
    let row = harness.get_by_role_and_label(Role::CheckBox, "Kitty");
    assert_eq!(row.accesskit_node().toggled(), Some(Toggled::False));

    row.click();
    harness.run();

    assert!(*harness.state());
    assert_eq!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Kitty")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
}

#[test]
fn a_disabled_check_row_ignores_clicks() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, on: &mut bool| {
            if check_row(ui, *on, "Kitty", "", false).clicked() {
                *on ^= true;
            }
        },
        false,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .click();
    harness.run();

    assert!(!*harness.state());
}

#[test]
fn the_box_starts_where_the_text_fields_above_it_start() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, left: &mut f32| {
            *left = ui.max_rect().left();
            _ = check_row(ui, true, "Administrator", "", true);
        },
        0.0,
    );

    harness.run();
    let accent = crate::tokens::current(&harness.ctx).accent;
    let square = crate::testing::filled_rect(&harness, accent).expect("a checked box");
    let left = *harness.state();
    assert!(
        (square.left() - left).abs() < 0.5,
        "the box starts at {} but the column at {left}",
        square.left()
    );
}

#[test]
fn a_row_with_a_line_shows_it_under_the_label_and_tells_the_whole_list() {
    let full = "~/.config/kitty/kitty.conf\n~/.config/kitty/theme.conf";
    let mut harness = harness(300.0, 120.0).build_ui(move |ui| {
        _ = check_row_with_line(
            ui,
            false,
            CheckRowTexts {
                label: "Kitty",
                line: "Replaces ~/.config/kitty/kitty.conf, ~/.config/kitty/theme.conf",
                full,
            },
            true,
        );
    });

    harness.run();

    let row = harness.get_by_role_and_label(Role::CheckBox, "Kitty");
    assert!(row.rect().height() > 40.0, "no room for the line");
    assert_eq!(row.accesskit_node().description().as_deref(), Some(full));
}
