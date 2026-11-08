//! The Config Files page on the self-test's own catalog and files
//! (`tests/selftest/configs`), so the edit runs on any system. A dry run
//! starts no editor: its stand-in adds a line, which the review shows and
//! the helper only checks. A file of the user is edited the same way and
//! its save, a step of the user, is skipped as well. `/etc/sudoers` there
//! is no text, so its view says so.

use egui::Key;

use super::step::Step;

const fn search(text: &'static str) -> [Step; 3] {
    let [field, typed] = Step::type_into("Search config files", text);
    [field, Step::shortcut(Key::A), typed]
}

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let mut steps = vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Config Files"),
        Step::WaitContaining("GRUB default"),
        Step::Shot("configs"),
        Step::button("Edit"),
        Step::WaitContaining("Save the changes to /etc/default/grub"),
        Step::WaitContaining("Edited by the dry run"),
        Step::Shot("configs-review"),
        Step::button("Save"),
        Step::WaitContaining("Saved /etc/default/grub"),
        Step::Shot("configs-saved"),
    ];

    steps.extend(search("Bash"));
    steps.extend([
        Step::wait("Bash"),
        Step::button("Edit"),
        Step::WaitContaining("Save the changes to ~/.bashrc"),
        Step::WaitContaining("Edited by the dry run"),
        Step::Shot("configs-home-review"),
        Step::button("Save"),
        Step::WaitContaining("Saved ~/.bashrc"),
        Step::Shot("configs-home-saved"),
    ]);

    steps.extend(search("sudoers"));
    steps.extend([
        Step::wait("Sudoers"),
        Step::button("View"),
        Step::WaitContaining("is no text file"),
        Step::Shot("configs-view-refused"),
    ]);

    steps
}
