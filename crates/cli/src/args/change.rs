//! The arguments of the commands that change the system. Names are only
//! shaped here, the app checks them against the system and the catalogs.

use clap::{Args, Subcommand, ValueEnum};
use toolbox_plan::{PackageName, UnitName};

use super::{Confirm, KernelsList, ReadList};

/// Longer ids name no entry of a catalog, unit or package.
const ID_MAX: usize = 128;

/// One service and where it lives.
#[derive(Debug, Args)]
pub struct UnitArgs {
    /// The unit, for example `bluetooth.service`.
    pub unit: UnitName,

    /// One of your own services instead of the system's.
    #[arg(long)]
    pub user: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// One catalog entry by its id, as `toggle list` or `apps list` prints it.
#[derive(Debug, Args)]
pub struct EntryArgs {
    /// The id, as the list command prints it.
    #[arg(value_parser = id)]
    pub id: String,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum MaintenanceCommand {
    /// List the tasks and repairs and whether each can run here.
    #[command(after_help = "Example:\n  garuda-toolbox maintenance list")]
    List(ReadList),

    /// Run a task or repair now, as its Run button does.
    #[command(
        after_help = "Examples:\n  garuda-toolbox maintenance run clean-package-cache\n  \
                            garuda-toolbox maintenance run remote-full-fix --yes"
    )]
    Run(TaskArgs),
}

#[derive(Debug, Args)]
pub struct TaskArgs {
    /// The task, as `maintenance list` prints it. A repair that reinstalls
    /// or wipes asks for its name to be typed, or takes `--yes`.
    #[arg(value_parser = id)]
    pub task: String,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum KernelsCommand {
    /// List the kernels: installed, running, available.
    #[command(after_help = "Example:\n  garuda-toolbox kernels list --json")]
    List(KernelsList),

    /// Install a kernel. It is used after the next restart.
    #[command(after_help = "Example:\n  garuda-toolbox kernels install linux-lts")]
    Install(KernelArgs),

    /// Remove an installed kernel. The running one stays.
    #[command(
        visible_alias = "uninstall",
        after_help = "Example:\n  garuda-toolbox kernels remove linux-zen"
    )]
    Remove(KernelArgs),

    /// Install the headers of an installed kernel, for DKMS modules.
    #[command(after_help = "Example:\n  garuda-toolbox kernels headers linux")]
    Headers(KernelArgs),

    /// Boot an installed kernel by default, through the boot menu.
    #[command(
        visible_alias = "set-default",
        after_help = "Example:\n  garuda-toolbox kernels default linux-lts"
    )]
    Default(KernelArgs),
}

#[derive(Debug, Args)]
pub struct KernelArgs {
    /// The kernel's package, as `kernels list` prints it.
    pub kernel: PackageName,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// What a services command does to its unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum UnitChange {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Mask,
    Unmask,
}

/// An id of a unit, a package or a catalog entry: letters, digits and the
/// marks such names use, never a space or a shell character.
pub(super) fn id(text: &str) -> Result<String, String> {
    let valid = (1..=ID_MAX).contains(&text.len())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.@:+".contains(&byte));

    if valid {
        Ok(text.to_owned())
    } else {
        Err(format!(
            "`{text}` is no id: letters, digits and - _ . @ : +, at most {ID_MAX}"
        ))
    }
}
