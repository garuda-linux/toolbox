use std::fmt;

use super::{Command, InitramfsRoot, Program};

/// The ways to build every UKI again, in the order they are tried.
pub fn uki_rebuilds() -> [Command; 2] {
    [
        Command::new(Program::KernelInstallUki).arg("add"),
        Command::new(Program::KernelInstall).arg("add-all"),
    ]
}

/// Without udev in a chroot, bootctl and kernel-install cannot always tell
/// the ESP's partition type. A repair checked the device already.
pub fn relaxed(command: Command) -> Command {
    command.env("SYSTEMD_RELAX_ESP_CHECKS", "1")
}

/// `command` as it runs for `root`.
pub fn in_root(command: Command, root: InitramfsRoot) -> Command {
    match root {
        InitramfsRoot::System => command,
        InitramfsRoot::Repair => relaxed(root.wrap(command)),
    }
}

pub(super) fn describe(out: &mut fmt::Formatter<'_>, root: InitramfsRoot) -> fmt::Result {
    let [first, fallback] = uki_rebuilds();
    let name = first.program().name();

    write!(
        out,
        "{}, or {} where {name} is not installed",
        in_root(first, root),
        in_root(fallback, root)
    )
}
