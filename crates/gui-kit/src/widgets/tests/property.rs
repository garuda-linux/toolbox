use egui::accesskit::Role;
use egui::{OutputCommand, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};

use super::super::{PropertyTexts, number_field, property_list};
use crate::testing::harness;
use crate::toasts::{ToastTexts, Toasts};

fn copy(label: &str) -> String {
    format!("Copy {label}")
}

fn texts() -> PropertyTexts<'static> {
    PropertyTexts {
        copy: &copy,
        copied: "Copied",
    }
}

fn properties() -> Harness<'static, Toasts> {
    let mut harness = harness(600.0, 300.0).build_ui_state(
        |ui, toasts: &mut Toasts| {
            property_list(ui, texts(), |list| {
                list.row_copied("Kernel", "6.14.0-arch3-1");
                list.row("Running since", "3 days");
                list.row_with("Retries", None, |value| {
                    let mut retries = 3_u32;
                    number_field(value, &mut retries, 0..=10, 72.0, ("Retries", "Retries"));
                });
            });

            let page = ui.max_rect();
            toasts.show(
                ui.ctx(),
                page,
                ToastTexts {
                    name: "Notification",
                },
            );
        },
        Toasts::default(),
    );

    harness.run();
    harness
}

#[test]
fn the_values_of_one_list_line_up_after_the_widest_label() {
    let harness = properties();

    let short = harness.get_by_label("6.14.0-arch3-1");
    let long = harness.get_by_label("3 days");
    let widest = harness.get_by_label("Running since").rect();

    assert!((short.rect().left() - long.rect().left()).abs() < 0.5);
    assert!(long.rect().left() > widest.right());
}

#[test]
fn a_value_is_named_by_its_row_label() {
    let harness = properties();

    let value = harness.get_by_label("6.14.0-arch3-1");

    assert_eq!(value.accesskit_node().label().as_deref(), Some("Kernel"));
}

#[test]
fn a_widget_value_is_named_by_its_row_label() {
    let harness = properties();

    assert!(
        harness
            .query_by_role_and_label(Role::SpinButton, "Retries")
            .is_some()
    );
}

#[test]
fn copy_puts_the_value_on_the_clipboard_and_says_so() {
    let mut harness = properties();

    harness.get_by_label("Kernel").hover();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Copy Kernel")
        .click();
    harness.step();

    let copied = harness.output().platform_output.commands.iter().any(
        |command| matches!(command, OutputCommand::CopyText(text) if text == "6.14.0-arch3-1"),
    );

    assert!(copied);

    harness.run();
    assert_eq!(harness.state().texts(), ["Copied"]);
}

#[test]
fn the_list_in_both_themes() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [
        (Theme::Dark, "widgets/properties_dark"),
        (Theme::Light, "widgets/properties_light"),
    ] {
        let mut harness = properties();
        harness.ctx.set_theme(theme);
        harness.get_by_label("Running since").hover();
        harness.run_steps(4);
        results.add(harness.try_snapshot(name));
    }
}

#[test]
fn a_plain_fact_has_no_copy_button() {
    let harness = properties();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Copy Running since")
            .is_none()
    );
}

#[test]
fn a_plain_fact_takes_the_same_room_as_one_with_a_copy_button() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        property_list(ui, texts(), |list| {
            list.row_copied("Version", "0.1.0");
            list.row("Licence", "GPL");
            list.row_copied("Kernel", "6.14");
        });
    });

    harness.run();

    let top = |label: &str| harness.get_by_label(label).rect().top();
    let first = top("Licence") - top("Version");
    let second = top("Kernel") - top("Licence");

    assert!((first - second).abs() < 1.0, "{first} then {second}");
}

#[test]
fn a_plain_fact_wraps_only_at_the_right_of_the_list() {
    let words = "ab ".repeat(60);
    let mut harness = harness(400.0, 300.0).build_ui(move |ui| {
        property_list(ui, texts(), |list| {
            list.row_copied("Version", "0.1.0");
            list.row("Notes", words.trim_end());
        });
    });

    harness.run();

    let value = harness.get_by_label("ab ".repeat(60).trim_end()).rect();
    let right = harness.ctx.content_rect().right() - 8.0;

    assert!(
        value.right() > right - 24.0,
        "the value ends at {}",
        value.right()
    );
}

#[test]
fn a_label_centres_on_its_value_control() {
    let harness = properties();

    let label = harness.get_by_role_and_label(Role::Label, "Retries").rect();

    let spin = harness
        .get_by_role_and_label(Role::SpinButton, "Retries")
        .rect();

    assert!(
        (label.center().y - spin.center().y).abs() < 1.0,
        "the label at {label:?}, its control at {spin:?}"
    );
}

#[test]
fn a_label_starts_on_the_line_of_its_text() {
    let harness = properties();

    let label = harness.get_by_label("Running since").rect();
    let value = harness.get_by_label("3 days").rect();

    assert!(
        (label.top() - value.top()).abs() < 0.5,
        "the label at {label:?}, its value at {value:?}"
    );
}

#[test]
fn the_copy_button_centres_on_the_line_of_its_value() {
    let harness = properties();

    let value = harness.get_by_label("6.14.0-arch3-1").rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Copy Kernel")
        .rect();

    assert!(
        (value.center().y - button.center().y).abs() < 1.0,
        "the value at {value:?}, its button at {button:?}"
    );
}
