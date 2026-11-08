use std::fs;

use toolbox_plan::{ConfigText, Editor};

use super::{DryRunEditor, Editing as _, SystemEditor};
use crate::EditFolder;

#[test]
fn the_system_needs_an_installed_editor() {
    assert_eq!(SystemEditor.editor(Some(Editor::Nano)), Some(Editor::Nano));
    assert_eq!(SystemEditor.editor(None), None);
}

#[test]
fn the_system_leaves_the_copy_to_the_editor() {
    let folder = tempfile::tempdir().unwrap();
    let copies = EditFolder::new(folder.path());
    let text = ConfigText::new("first\n").unwrap();
    let copy = copies.create("fstab", &text).unwrap();

    SystemEditor.after_edit(&copies, &copy).unwrap();
    assert_eq!(fs::read_to_string(copy.as_str()).unwrap(), "first\n");
}

#[test]
fn the_dry_run_needs_no_program_and_falls_back_to_micro() {
    assert_eq!(DryRunEditor.editor(Some(Editor::Nano)), Some(Editor::Nano));
    assert_eq!(DryRunEditor.editor(None), Some(Editor::Micro));
}

#[test]
fn the_dry_run_changes_the_copy_itself() {
    let folder = tempfile::tempdir().unwrap();
    let copies = EditFolder::new(folder.path());
    let text = ConfigText::new("first\n").unwrap();
    let copy = copies.create("fstab", &text).unwrap();

    DryRunEditor.after_edit(&copies, &copy).unwrap();

    let edited = fs::read_to_string(copy.as_str()).unwrap();
    assert!(edited.starts_with("first\n"), "{edited}");
    assert_ne!(edited, "first\n");
}
