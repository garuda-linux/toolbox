use std::fmt;

use super::{Command, Program};
use crate::repair::{REPAIR_ROOT, RepairMount};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitramfsRoot {
    System,
    /// Through `chroot`: the installed system's own programs and hooks
    /// build it, not those of the live ISO.
    Repair,
}

impl InitramfsRoot {
    pub const fn folder(self) -> &'static str {
        match self {
            Self::System => "",
            Self::Repair => REPAIR_ROOT,
        }
    }

    pub fn wrap(self, command: Command) -> Command {
        match self {
            Self::System => command,
            Self::Repair => Command::new(Program::Chroot)
                .arg(RepairMount::Root.to_string())
                .arg(command.program().path())
                .args(command.arguments().iter().cloned()),
        }
    }
}

/// The ways to build every initramfs again, in the order they are tried.
/// Garuda builds them with dracut, plain Arch with mkinitcpio.
pub fn initramfs_rebuilds() -> [Command; 2] {
    [
        Command::new(Program::DracutRebuild),
        Command::new(Program::Mkinitcpio).args(["-P"]),
    ]
}

pub(super) fn describe(out: &mut fmt::Formatter<'_>, root: InitramfsRoot) -> fmt::Result {
    let [first, fallback] = initramfs_rebuilds();
    let name = first.program().name();

    write!(
        out,
        "{}, or {} where {name} is not installed",
        root.wrap(first),
        root.wrap(fallback)
    )
}
