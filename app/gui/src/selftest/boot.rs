//! The Boot options page on the real system: its settings where GRUB or
//! systemd-boot is the boot loader, a word that it is not on other
//! systems. A dry run queues nothing here, the kittest tests cover the
//! edits.

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Boot Options"),
        Step::WaitEither("Boot menu", "GRUB is not the boot loader of this system"),
        Step::WaitEither(
            "Parameters passed to the kernel",
            "GRUB is not the boot loader of this system",
        ),
        Step::Shot("boot"),
    ]
}
