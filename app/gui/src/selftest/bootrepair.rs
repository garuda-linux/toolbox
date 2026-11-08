//! The Boot Repair page in test mode (`--boot-repair`) on the made-up
//! machine of a dry run: it never reads or shows the disks it runs on. A
//! dry run's repair only checks the plan. A locked system is repaired
//! only with its passphrase.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Boot Repair"),
        Step::wait("UEFI"),
        Step::WaitContaining("Test mode"),
        Step::Shot("bootrepair"),
        Step::button("Generate boot menu…"),
        Step::WaitContaining("is written again"),
        Step::button("Generate boot menu"),
        Step::WaitContaining("The repair finished without an error"),
        Step::Shot("bootrepair-done"),
        Step::combo("Root partition"),
        Step::click("/dev/sda3 · 265.5 GiB"),
        Step::wait("Encrypted (LUKS), locked"),
        Step::button("Reinstall GRUB…"),
        Step::disabled_button("Reinstall GRUB"),
        Step::Shot("bootrepair-locked"),
        Step::press(Key::Escape),
    ]
}
