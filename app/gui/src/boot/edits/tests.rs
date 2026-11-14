use toolbox_core::features::boot::edit::Edit;
use toolbox_core::{ChangeSet, Snapshot};

use crate::boot::Boot;
use crate::command::{AppCommand, BootCommand};

/// Without `/etc/default/grub` there is nothing to edit.
#[test]
fn an_edit_without_a_known_menu_is_refused_with_its_value() {
    let mut boot = Boot::default();

    let handled = boot.edit(
        Edit::AddParameter {
            text: "quiet".to_owned(),
            confirmed: false,
        },
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.edit.is_none());
    let refusal = boot.refusal().unwrap();
    assert_eq!(refusal.subject, "quiet");

    boot.handle(
        &AppCommand::Boot(BootCommand::DismissRefusal),
        &ChangeSet::default(),
        &Snapshot::default(),
    );
    assert!(boot.refusal().is_none());
}
