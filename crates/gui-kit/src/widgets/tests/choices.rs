use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{chips, counted_chips, named_chips, segmented, toggle_chips};
use crate::testing::harness;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    System,
    User,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Filter {
    Disabled,
    Inactive,
}

#[test]
fn chips_report_the_value_clicked() {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, picked: &mut Option<Scope>| {
            let options = [(Scope::System, "System"), (Scope::User, "User")];
            if let Some(value) = chips(ui, &options, Scope::System) {
                *picked = Some(value);
            }
        },
        None,
    );

    harness.run();
    let system = harness.get_by_role_and_label(Role::Button, "System");
    assert_eq!(system.accesskit_node().toggled(), Some(Toggled::True));
    harness.get_by_role_and_label(Role::Button, "User").click();
    harness.run();

    assert_eq!(*harness.state(), Some(Scope::User));
}

#[test]
fn counted_chips_say_their_count() {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, picked: &mut Option<Option<&'static str>>| {
            let options = [(None, "All", None), (Some("audio"), "Audio", Some("2/8"))];
            if let Some(value) = counted_chips(ui, &options, None) {
                *picked = Some(value);
            }
        },
        None,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Audio 2/8")
        .click();
    harness.run();

    assert_eq!(*harness.state(), Some(Some("audio")));
}

#[test]
fn toggle_chips_are_selected_one_by_one() {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, on: &mut Vec<Filter>| {
            let options = [
                (Filter::Disabled, "Show disabled"),
                (Filter::Inactive, "Hide inactive"),
            ];

            if let Some(value) = toggle_chips(ui, &options, |value| on.contains(&value)) {
                on.push(value);
            }
        },
        vec![Filter::Inactive],
    );

    harness.run();
    let inactive = harness.get_by_role_and_label(Role::Button, "Hide inactive");
    assert_eq!(inactive.accesskit_node().toggled(), Some(Toggled::True));
    harness
        .get_by_role_and_label(Role::Button, "Show disabled")
        .click();
    harness.run();

    assert_eq!(*harness.state(), [Filter::Inactive, Filter::Disabled]);
}

fn scopes(current: Scope) -> egui_kittest::Harness<'static, Scope> {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, current: &mut Scope| {
            let options = [(Scope::System, "System"), (Scope::User, "User")];
            if let Some(value) = segmented(ui, "Units", &options, *current) {
                *current = value;
            }
        },
        current,
    );

    harness.run();
    harness
}

#[test]
fn segments_report_the_value_clicked() {
    let mut harness = scopes(Scope::System);
    harness
        .get_by_role_and_label(Role::RadioButton, "User")
        .click();
    harness.run();

    assert_eq!(*harness.state(), Scope::User);
    let user = harness.get_by_role_and_label(Role::RadioButton, "User");
    assert_eq!(user.accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn segments_sit_in_a_named_radio_group() {
    let harness = scopes(Scope::System);
    let group = harness.get_by_role_and_label(Role::RadioGroup, "Units");

    assert!(
        group
            .query_by_role_and_label(Role::RadioButton, "System")
            .is_some()
    );
    assert!(
        group
            .query_by_role_and_label(Role::RadioButton, "User")
            .is_some()
    );
}

#[test]
fn only_the_chosen_segment_is_a_tab_stop() {
    let mut harness = scopes(Scope::User);

    harness.key_press(egui::Key::Tab);
    harness.run();

    assert!(harness.get_by_label("User").is_focused());
}

#[test]
fn arrow_keys_move_the_choice_and_the_focus() {
    let mut harness = scopes(Scope::System);
    harness.key_press(egui::Key::Tab);
    harness.run();

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), Scope::User);
    assert!(harness.get_by_label("User").is_focused());

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), Scope::System, "wraps to the first");
    assert!(harness.get_by_label("System").is_focused());

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(*harness.state(), Scope::User, "wraps to the last");
    assert!(harness.get_by_label("User").is_focused());
}

#[test]
fn the_measured_width_is_the_drawn_one() {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, measured: &mut f32| {
            *measured = super::super::segmented_width(ui, &["System", "User"]);
            segmented(
                ui,
                "Units",
                &[(Scope::System, "System"), (Scope::User, "User")],
                Scope::System,
            );
        },
        0.0,
    );

    harness.run();
    let field = crate::tokens::current(&harness.ctx).field;
    let pill = crate::testing::filled_rect(&harness, field).expect("the pill");

    assert!((pill.width() - *harness.state()).abs() < 0.5, "{pill:?}");
}

#[test]
fn two_groups_with_the_same_labels_keep_apart() {
    let mut harness = harness(400.0, 160.0).build_ui_state(
        |ui, picked: &mut (Scope, Scope)| {
            let options = [(Scope::System, "On"), (Scope::User, "Off")];
            if let Some(value) = segmented(ui, "First", &options, picked.0) {
                picked.0 = value;
            }

            if let Some(value) = segmented(ui, "Second", &options, picked.1) {
                picked.1 = value;
            }
        },
        (Scope::System, Scope::System),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::RadioGroup, "Second")
        .get_by_role_and_label(Role::RadioButton, "Off")
        .click();
    harness.run();

    assert_eq!(*harness.state(), (Scope::System, Scope::User));
}

#[test]
fn named_chip_rows_tell_their_all_chips_apart() {
    let mut harness = harness(400.0, 160.0).build_ui_state(
        |ui, picked: &mut Vec<(&'static str, Option<Scope>)>| {
            let states = [(None, "All"), (Some(Scope::System), "System")];
            if let Some(value) = named_chips(ui, "State", &states, None) {
                picked.push(("State", value));
            }

            let kinds = [(None, "All"), (Some(Scope::User), "User")];
            if let Some(value) = named_chips(ui, "Kind", &kinds, Some(Scope::User)) {
                picked.push(("Kind", value));
            }
        },
        Vec::new(),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Group, "Kind")
        .get_by_label("All")
        .click();
    harness.run();

    assert_eq!(harness.state().as_slice(), [("Kind", None)]);
}
