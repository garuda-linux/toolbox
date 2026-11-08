use egui::Id;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{ButtonKind, ModalWrapper, confirm_buttons};
use crate::testing::harness;

fn page() -> Harness<'static, bool> {
    let mut harness = harness(600.0, 400.0).build_ui_state(
        |ui, open: &mut bool| {
            if ui.button("Remove all…").clicked() {
                *open = true;
            }

            _ = ui.button("Something else");
            if !*open {
                return;
            }

            let outcome = ModalWrapper::new(Id::unique("ask"), "Remove all changes")
                .title_bar("Close")
                .show(ui.ctx(), |ui| {
                    confirm_buttons(ui, "Keep", "Remove all", ButtonKind::Danger)
                });

            if outcome.closed || outcome.inner.is_some() {
                *open = false;
            }
        },
        false,
    );

    harness.run();
    harness
}

fn focused(harness: &Harness<'static, bool>) -> Option<String> {
    let node = harness
        .query_all(egui_kittest::kittest::by())
        .find(|node| node.accesskit_node().is_focused())?;

    node.accesskit_node().label()
}

fn open_by_keyboard(harness: &mut Harness<'static, bool>) {
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(focused(harness).as_deref(), Some("Remove all…"));
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.run();
}

fn focused_any(harness: &Harness<'static, bool>) -> bool {
    harness
        .query_all(egui_kittest::kittest::by())
        .any(|node| node.accesskit_node().is_focused())
}

fn tab(harness: &mut Harness<'static, bool>) -> Option<String> {
    harness.key_press(egui::Key::Tab);
    harness.run();
    focused(harness)
}

#[test]
fn a_dialog_takes_the_focus_without_putting_it_on_a_control() {
    let mut harness = page();
    open_by_keyboard(&mut harness);

    let now = focused(&harness);
    assert!(focused_any(&harness), "nothing took the focus");
    assert!(
        !matches!(
            now.as_deref(),
            Some("Close" | "Keep" | "Remove all" | "Remove all…" | "Something else")
        ),
        "a control took the focus when the dialog opened: {now:?}"
    );
}

#[test]
fn tab_from_an_opened_dialog_reaches_close_then_cycles_its_controls() {
    let mut harness = page();
    open_by_keyboard(&mut harness);

    let order: Vec<_> = std::iter::repeat_with(|| tab(&mut harness))
        .take(4)
        .collect();

    assert_eq!(
        order,
        ["Close", "Keep", "Remove all", "Close"].map(|name| Some(name.to_owned())),
        "Tab must not stop on the dialog itself again"
    );
}

#[test]
fn shift_tab_from_an_opened_dialog_stays_in_it() {
    let mut harness = page();
    open_by_keyboard(&mut harness);

    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
    harness.run();

    let now = focused(&harness);
    assert!(
        matches!(now.as_deref(), Some("Close" | "Keep" | "Remove all")),
        "Shift+Tab left the dialog: {now:?}"
    );
}

#[test]
fn closing_a_dialog_gives_the_focus_back_to_its_opener() {
    let mut harness = page();
    open_by_keyboard(&mut harness);

    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.run();

    assert!(!*harness.state(), "Escape left the dialog open");
    assert_eq!(focused(&harness).as_deref(), Some("Remove all…"));
}
