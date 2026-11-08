//! Fixed programs with fixed arguments, no shell.

#[cfg(test)]
mod tests;

use super::run;
use crate::procedure::{Builtin, Command, InitramfsRoot, Invocation, Program};
use crate::sdboot::LoaderChange;
use crate::{GRUB_CONFIG, GrubSettings, KernelParameters, PlymouthTheme};

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

/// The initramfs holds the theme, so it is built again. Not with `-R`,
/// which always runs mkinitcpio. The program knows no `--`: the name's
/// rule keeps a `-` from its start.
pub fn set_plymouth_theme(theme: &PlymouthTheme) -> Vec<Invocation> {
    vec![plymouth_theme(theme), system_initramfs()]
}

pub fn plymouth_theme(theme: &PlymouthTheme) -> Invocation {
    run(Command::new(Program::PlymouthSetDefaultTheme).args([theme.as_str()]))
}

pub fn set_kernel_cmdline(parameters: &KernelParameters) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::WriteKernelCmdline(
        parameters.clone(),
    ))]
}

/// Garuda's script hands kernel-install `/boot/initramfs-<kernel>.img`,
/// so a new theme needs that file built first.
pub fn rebuild_ukis(initramfs: bool) -> Vec<Invocation> {
    let ukis = Invocation::Builtin(Builtin::RebuildUkis);

    if initramfs {
        vec![system_initramfs(), ukis]
    } else {
        vec![ukis]
    }
}

const fn system_initramfs() -> Invocation {
    Invocation::Builtin(Builtin::RebuildInitramfs {
        root: InitramfsRoot::System,
    })
}

pub fn set_loader_settings(settings: &LoaderChange) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::EditLoaderConf(
        settings.clone(),
    ))]
}
