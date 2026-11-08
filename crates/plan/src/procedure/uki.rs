use std::fmt;

use super::{Command, Program};

/// The ways to build every UKI again, in the order they are tried.
pub fn uki_rebuilds() -> [Command; 2] {
    [
        Command::new(Program::KernelInstallUki).arg("add"),
        Command::new(Program::KernelInstall).arg("add-all"),
    ]
}

pub(super) fn describe(out: &mut fmt::Formatter<'_>) -> fmt::Result {
    let [first, fallback] = uki_rebuilds();
    let name = first.program().name();

    write!(out, "{first}, or {fallback} where {name} is not installed")
}
