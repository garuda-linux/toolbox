use clap::{Args, Subcommand, ValueEnum};
use toolbox_plan::PageName;
use toolbox_state::diagnostics::Selection;

use super::change::{EntryArgs, UnitArgs, UnitChange};
use super::{Confirm, ReadList, name};

#[derive(Debug, Args)]
pub struct Diagnostics {
    /// The report: inxi, garuda-health, systemd-analyze, journalctl,
    /// pacman, dmesg, or all.
    #[arg(value_parser = report)]
    pub report: Report,

    /// Upload the report as an encrypted paste and print its link.
    #[arg(long)]
    pub upload: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// A report of the diagnostics page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report(pub Selection);

#[derive(Debug, Subcommand)]
pub enum ServicesCommand {
    /// List the services and their state.
    #[command(after_help = "Example:\n  garuda-toolbox services list --user")]
    List(ServicesList),

    /// Start a service now. Refused when it runs already.
    #[command(after_help = "Example:\n  garuda-toolbox services start bluetooth.service")]
    Start(UnitArgs),

    /// Stop a service now. Refused when it is not running.
    #[command(after_help = "Example:\n  garuda-toolbox services stop bluetooth.service")]
    Stop(UnitArgs),

    /// Restart a service now.
    #[command(after_help = "Example:\n  garuda-toolbox services restart sshd.service")]
    Restart(UnitArgs),

    /// Reload a service's configuration now.
    #[command(after_help = "Example:\n  garuda-toolbox services reload sshd.service")]
    Reload(UnitArgs),

    /// Start a service at boot.
    #[command(after_help = "Example:\n  garuda-toolbox services enable sshd.service --yes")]
    Enable(UnitArgs),

    /// Stop starting a service at boot.
    #[command(after_help = "Example:\n  garuda-toolbox services disable sshd.service")]
    Disable(UnitArgs),

    /// Mask a service, so nothing can start it.
    #[command(after_help = "Example:\n  garuda-toolbox services mask cups.service")]
    Mask(UnitArgs),

    /// Unmask a masked service.
    #[command(after_help = "Example:\n  garuda-toolbox services unmask cups.service")]
    Unmask(UnitArgs),
}

impl ServicesCommand {
    /// The change and its unit.
    ///
    /// # Errors
    ///
    /// The list's arguments for `list`, which changes nothing.
    pub const fn change(&self) -> Result<(UnitChange, &UnitArgs), &ServicesList> {
        Ok(match self {
            Self::List(list) => return Err(list),
            Self::Start(unit) => (UnitChange::Start, unit),
            Self::Stop(unit) => (UnitChange::Stop, unit),
            Self::Restart(unit) => (UnitChange::Restart, unit),
            Self::Reload(unit) => (UnitChange::Reload, unit),
            Self::Enable(unit) => (UnitChange::Enable, unit),
            Self::Disable(unit) => (UnitChange::Disable, unit),
            Self::Mask(unit) => (UnitChange::Mask, unit),
            Self::Unmask(unit) => (UnitChange::Unmask, unit),
        })
    }
}

#[derive(Debug, Args)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is a flag of its own on the command line"
)]
pub struct ServicesList {
    /// Your own services instead of the system's.
    #[arg(long)]
    pub user: bool,

    /// Only the running services.
    #[arg(long, conflicts_with = "failed")]
    pub running: bool,

    /// Only the failed services.
    #[arg(long)]
    pub failed: bool,

    /// Only the services that start at boot.
    #[arg(long)]
    pub enabled: bool,

    #[command(flatten)]
    pub read: ReadList,
}

#[derive(Debug, Subcommand)]
pub enum ToggleCommand {
    /// List the switches and whether each is on.
    #[command(after_help = "Example:\n  garuda-toolbox toggle list --page gaming")]
    List(ToggleList),

    /// Turn a switch on, with what it needs.
    #[command(after_help = "Example:\n  garuda-toolbox toggle on pipewire-support")]
    On(EntryArgs),

    /// Turn a switch off.
    #[command(after_help = "Example:\n  garuda-toolbox toggle off bluetooth-support --yes")]
    Off(EntryArgs),
}

#[derive(Debug, Args)]
pub struct ToggleList {
    /// Only the switches of this page.
    #[arg(long, value_enum)]
    pub page: Option<TogglePage>,

    /// Only the switches that are on.
    #[arg(long, conflicts_with = "off")]
    pub on: bool,

    /// Only the switches that are off.
    #[arg(long)]
    pub off: bool,

    #[command(flatten)]
    pub read: ReadList,
}

/// The pages made of switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TogglePage {
    Components,
    #[value(name = "system-settings", alias = "tweaks")]
    Tweaks,
    Gaming,
    Setup,
}

#[derive(Debug, Subcommand)]
pub enum AppsCommand {
    /// List the apps of the catalog and whether each is installed.
    #[command(after_help = "Example:\n  garuda-toolbox apps list --category internet")]
    List(AppsList),

    /// Install an app of the catalog.
    #[command(after_help = "Example:\n  garuda-toolbox apps install firefox")]
    Install(EntryArgs),

    /// Remove an app of the catalog.
    #[command(
        visible_alias = "uninstall",
        after_help = "Example:\n  garuda-toolbox apps remove firefox --yes"
    )]
    Remove(EntryArgs),
}

#[derive(Debug, Args)]
pub struct AppsList {
    /// Only the apps of this category.
    #[arg(long, value_name = "CATEGORY", value_parser = name)]
    pub category: Option<PageName>,

    /// Only the installed apps.
    #[arg(long)]
    pub installed: bool,

    #[command(flatten)]
    pub read: ReadList,
}

#[derive(Debug, Args)]
pub struct Update {
    /// Also update the AUR packages, with paru as you.
    #[arg(long)]
    pub aur: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

fn report(text: &str) -> Result<Report, String> {
    Selection::from_id(text)
        .map(Report)
        .ok_or_else(|| format!("`{text}` is no report, see `garuda-toolbox diagnostics --help`"))
}
