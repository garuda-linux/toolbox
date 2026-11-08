//! A dialog keeps one margin on every side, its Close button included, and
//! the footer's hint sits level with the buttons.

use egui::Id;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::{ButtonKind, DialogWidth, ModalWrapper, confirm_buttons_why};
use crate::testing::{filled_rect, harness};
use crate::tokens;

const HINT: &str = "Enter a user name.";

fn form() -> egui_kittest::Harness<'static> {
    let mut harness = harness(1000.0, 600.0).build_ui(|ui| {
        ModalWrapper::new(Id::unique("form"), "Add a user")
            .title_bar("Close")
            .width(DialogWidth::Medium)
            .show(ui.ctx(), |ui| {
                ui.label("User name");
                confirm_buttons_why(
                    ui,
                    "Cancel",
                    "Add user now",
                    ButtonKind::Primary,
                    Some(HINT),
                )
            });
    });

    harness.run();
    harness
}

#[test]
fn a_dialog_has_the_same_margin_on_every_side() {
    let harness = form();
    let fill = tokens::current(&harness.ctx).raised;
    let dialog = filled_rect(&harness, fill).expect("the dialog paints its frame");
    let close = harness.get_by_role_and_label(Role::Button, "Close").rect();
    let add = harness
        .get_by_role_and_label(Role::Button, "Add user now")
        .rect();

    let label = harness.get_by_label("User name").rect();

    let left = label.left() - dialog.left();
    let sides = [
        ("top", close.top() - dialog.top()),
        ("right of Close", dialog.right() - close.right()),
        ("right of the buttons", dialog.right() - add.right()),
        ("bottom", dialog.bottom() - add.bottom()),
    ];

    for (side, margin) in sides {
        assert!(
            (margin - left).abs() < 1.0,
            "{side}: {margin} against {left} at the left"
        );
    }
}

#[test]
fn the_hint_sits_level_with_the_buttons() {
    let harness = form();
    let hint = harness.get_by_label(HINT).rect();
    let add = harness
        .get_by_role_and_label(Role::Button, "Add user now")
        .rect();

    assert!(
        (hint.center().y - add.center().y).abs() < 1.0,
        "the hint {hint:?} against the button {add:?}"
    );
}
