//! Fixed programs with fixed arguments, no shell.

#[cfg(test)]
mod tests;

use super::run;
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{GRUB_CONFIG, GrubSettings, PlymouthTheme};

pub fn set_grub_settings(settings: &GrubSettings) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::EditGrubDefaults(
        settings.clone(),
    ))]
}

/// What Arch's `update-grub` runs. Its failure fails the step: the menu
/// then still is the old one.
pub fn generate_grub_config() -> Vec<Invocation> {
    vec![run(
        Command::new(Program::GrubMkconfig).args(["-o".to_owned(), format!("/{GRUB_CONFIG}")])
    )]
}

/// `-R` builds the initramfs again, which holds the theme. The program
/// knows no `--`: the name's rule keeps a `-` from its start.
pub fn set_plymouth_theme(theme: &PlymouthTheme) -> Vec<Invocation> {
    vec![run(
        Command::new(Program::PlymouthSetDefaultTheme).args(["-R", theme.as_str()])
    )]
}
