//! A parameter that opens a root shell at boot waits for a yes.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::boot::edit::Edit;

use super::{edit, grub_system, menu, page, sent};
use crate::boot::{Boot, BootRefusal};
use crate::command::{BootCommand, Command};

fn asked() -> super::Shown {
    let refusal = BootRefusal {
        code: "needs-confirm",
        value: "init=/bin/sh".to_owned(),
    };

    page(grub_system(), Boot::showing(menu(), Some(refusal), None))
}

#[test]
fn a_risky_parameter_is_added_after_its_question() {
    let mut harness = asked();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Add init=/bin/sh?")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Add the parameter")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [edit(Edit::AddParameter {
            text: "init=/bin/sh".to_owned(),
            confirmed: true,
        })]
    );
}

#[test]
fn cancel_drops_the_risky_parameter() {
    let mut harness = asked();

    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(BootCommand::DismissRefusal)]);
}
