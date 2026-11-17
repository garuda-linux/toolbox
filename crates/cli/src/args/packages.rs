use clap::{Args, Subcommand};
use toolbox_plan::PackageName;

use super::{Confirm, Read};

#[derive(Debug, Subcommand)]
pub enum PackagesCommand {
    /// Search the repository and installed packages by name and
    /// description, or the AUR with `--aur`.
    #[command(
        visible_alias = "list",
        after_help = "Examples:\n  garuda-toolbox packages search editor\n  \
                      garuda-toolbox packages list --installed\n  \
                      garuda-toolbox packages search --orphans --json\n  \
                      garuda-toolbox packages search --aur paru"
    )]
    Search(PackagesSearch),

    /// Show one package: where it comes from, its version, size and
    /// dependencies, and its files once installed.
    #[command(
        after_help = "Examples:\n  garuda-toolbox packages show gimp --json\n  \
                      garuda-toolbox packages show --aur paru-bin"
    )]
    Show(PackageShow),

    /// Install packages from the repositories, or build them from the AUR
    /// with `--aur`.
    #[command(after_help = "Examples:\n  garuda-toolbox packages install gimp\n  \
                      garuda-toolbox packages install --aur paru-bin")]
    Install(PackageInstall),

    /// Remove an installed package with the dependencies nothing else needs.
    #[command(
        visible_alias = "uninstall",
        after_help = "Example:\n  garuda-toolbox packages remove gimp"
    )]
    Remove(PackageArgs),
}

#[derive(Debug, Args)]
pub struct PackagesSearch {
    /// Only the packages whose name or description contains this text, in
    /// any case. Without it every package.
    pub text: Option<String>,

    #[command(flatten)]
    pub only: PackagesOnly,

    /// Search the AUR instead, also its keywords, the exact name first.
    /// Downloads the AUR's package list when the last try is a day old.
    /// Works while the window's AUR setting is off: the flag asks for it.
    #[arg(long, conflicts_with_all = ["installed", "orphans", "optional"])]
    pub aur: bool,

    #[command(flatten)]
    pub read: Read,
}

/// The chip of the window's Packages page a search keeps to, at most one.
#[derive(Debug, Default, Args)]
pub struct PackagesOnly {
    /// Only the installed packages.
    #[arg(long, conflicts_with_all = ["orphans", "optional"])]
    pub installed: bool,

    /// Only the dependencies no installed package needs any more, not
    /// even as an optional dependency.
    #[arg(long, conflicts_with = "optional")]
    pub orphans: bool,

    /// Only the dependencies that installed packages list as optional
    /// and none requires.
    #[arg(long)]
    pub optional: bool,
}

#[derive(Debug, Args)]
pub struct PackageShow {
    /// The package's name, as `packages search` prints it.
    pub package: PackageName,

    /// Show the package of the AUR. Works while the window's AUR setting
    /// is off.
    #[arg(long)]
    pub aur: bool,

    #[command(flatten)]
    pub read: Read,
}

#[derive(Debug, Args)]
pub struct PackageInstall {
    /// The packages' names, as `packages search` prints them.
    #[arg(required = true)]
    pub packages: Vec<PackageName>,

    /// Build the packages from the AUR with paru, as you. paru shows each
    /// build script before it builds. Works while the window's AUR setting
    /// is off: the flag asks for it.
    #[arg(long)]
    pub aur: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct PackageArgs {
    /// The package's name, as `packages search` prints it.
    pub package: PackageName,

    #[command(flatten)]
    pub confirm: Confirm,
}
