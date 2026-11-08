use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

const BUTTON: &str = "More actions for Clean cache";

/// Which of the menu's two actions were clicked.
#[derive(Default)]
struct Clicked {
    open: bool,
    resting: bool,
}

fn row() -> egui_kittest::Harness<'static, Clicked> {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 300.0).build_ui_state(
        |ui, clicked: &mut Clicked| {
            super::more(ui, "Clean cache", |ui| {
                clicked.open |= super::item(ui, "row-copy-name", true);
                clicked.resting |= super::item(ui, "row-open-website", false);
            });
        },
        Clicked::default(),
    );

    harness.run();
    harness
}

#[test]
fn the_button_names_its_row() {
    let harness = row();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, BUTTON)
            .is_some()
    );
}

#[test]
fn enter_opens_the_menu_and_an_item_runs() {
    let mut harness = row();
    harness.get_by_role_and_label(Role::Button, BUTTON).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    harness
        .get_by_role_and_label(Role::MenuItem, "Copy name")
        .click();
    harness.run();

    assert!(harness.state().open);
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Copy name")
            .is_none()
    );
}

#[test]
fn a_resting_item_does_not_run() {
    let mut harness = row();
    harness.get_by_role_and_label(Role::Button, BUTTON).click();
    harness.run();

    let resting = harness.get_by_role_and_label(Role::MenuItem, "Open website");
    assert!(resting.accesskit_node().is_disabled());
    resting.click();
    harness.run();

    assert!(!harness.state().resting);
}

#[test]
fn escape_closes_the_menu_and_the_button_keeps_the_focus() {
    let mut harness = row();
    harness.get_by_role_and_label(Role::Button, BUTTON).focus();
    harness.run();
    harness.key_press(Key::Space);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Copy name")
            .is_some()
    );

    harness.key_press(Key::Escape);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Copy name")
            .is_none()
    );
    let button = harness.get_by_role_and_label(Role::Button, BUTTON);
    assert!(button.accesskit_node().is_focused());
}

#[test]
fn the_escape_that_closes_the_menu_reaches_nothing_else() {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 300.0).build_ui_state(
        |ui, later: &mut bool| {
            super::more(ui, "Clean cache", |ui| {
                _ = super::item(ui, "row-copy-name", true);
            });
            *later |= ui.input(|input| input.key_pressed(Key::Escape));
        },
        false,
    );

    harness.run();
    harness.get_by_label(BUTTON).click();
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();

    assert!(!*harness.state(), "the Escape went on past the menu");
}
