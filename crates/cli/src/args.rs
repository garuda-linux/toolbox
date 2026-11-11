//! What the one binary accepts.
//!
//! Every value with a value object is checked here, before anything runs.
//! Config file keys and boot repair devices stay text: the commands look
//! them up among what the system offers.

#[cfg(test)]
mod tests;

mod accounts;
mod change;
mod confirm_of;
mod group;
mod guide;
mod list;
mod net;
mod packages;
mod pages;
mod queue;
mod status;
mod system;
mod tools;
mod tweaks;
mod window;

pub use accounts::{
    AddUser, EditUser, GroupArgs, GroupsCommand, PasswordArgs, PasswordSource, RemoveUser,
    UserArgs, UsersCommand, UsersList,
};
pub use change::{
    EntryArgs, EntryShow, KernelArgs, KernelsCommand, MaintenanceCommand, ResetArgs, ResetList,
    TaskArgs, UnitArgs, UnitChange, UnitShow,
};
use clap::{Args, Parser, Subcommand};
pub use group::{Defaulted, Group};
pub use guide::{GuideCommand, GuideShow};
pub use list::{KernelsList, LocalesList, ReadList, Search};
pub use net::{
    BootRepairCommand, ConfigArgs, ConfigEdit, ConfigsCommand, DriversCommand, ModuleArgs,
    NetworkCommand, OnOff, PingArgs, Radio, RadioArgs, ReinstallArgs, RepairArgs, SetupApply,
    SetupCommand, TraceArgs,
};
pub use packages::{PackageArgs, PackageInstall, PackageShow, PackagesCommand, PackagesSearch};
pub use pages::{
    AppsCommand, AppsList, Diagnostics, ServicesCommand, ServicesList, ServicesRead, ToggleCommand,
    ToggleList, TogglePage, Update,
};
pub use queue::{QueueAdd, QueueClear, QueueCommand, QueueRemove};
pub use status::{Interval, StatusArgs};
pub use system::{
    BootCommand, BootKey, BootSet, DkmsBuild, DkmsCommand, LocaleArgs, LocalesCommand, PackArgs,
    PacksCommand, ParamArgs, ParamCommand, SettingKey, SettingKeyArgs, SettingSet, SettingsCommand,
};
use toolbox_plan::{MAX_PAGE_NAME_BYTES, PageName};
pub use tools::{Completions, Help};
pub use tweaks::{DnsArgs, HblockArgs, TweaksCommand, UpdateFlagArgs, WifiBackendArgs};

use crate::output::ColorMode;

const ABOUT: &str = "Maintain and configure Garuda Linux and other Arch based systems.";

pub const EXAMPLES: &str = "\
Examples:
  garuda-toolbox                          open the window
  garuda-toolbox --page maintenance       open the window on a page
  garuda-toolbox status                   what needs attention
  garuda-toolbox update --yes             update the system without asking";

#[derive(Debug, Parser)]
#[command(
    name = "garuda-toolbox",
    styles = crate::output::Paint::help(),
    version,
    about = ABOUT,
    after_help = EXAMPLES,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(flatten)]
    pub window: Window,

    /// Print the programs' own words under an error.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Print long text directly instead of through `$PAGER`.
    #[arg(long, global = true)]
    pub no_pager: bool,

    /// Colour the output: auto, always or never.
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true, value_name = "WHEN")]
    pub color: ColorMode,

    /// Cut long cells to this many columns.
    #[arg(long, global = true, value_name = "COLUMNS", value_parser = columns)]
    pub width: Option<usize>,

    /// Leave out the spinner and the hints.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// How the window opens. A running window takes these over.
#[derive(Debug, Default, Args)]
pub struct Window {
    /// Open the window on this page (see `garuda-toolbox pages`).
    #[arg(long, value_name = "PAGE", value_parser = name)]
    pub page: Option<PageName>,

    /// Run this action of the page once it is open (for example a report
    /// on the diagnostics page).
    #[arg(long, value_name = "ACTION", value_parser = name, requires = "page")]
    pub action: Option<PageName>,

    /// Open the window on the Setup Assistant.
    #[arg(long, conflicts_with = "page")]
    pub setup_assistant: bool,

    /// Show Boot Repair on an installed system too, for testing it. The
    /// helper still refuses to write to a disk the running system uses.
    #[arg(long)]
    pub boot_repair: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show what needs attention: updates, files to merge, health, restart.
    #[command(
        display_order = 1,
        after_help = "Examples:\n  garuda-toolbox status --json\n  garuda-toolbox status --brief"
    )]
    Status(StatusArgs),

    /// Check what the toolbox needs and say what is missing or broken.
    #[command(
        display_order = 20,
        after_help = "Examples:\n  garuda-toolbox doctor\n  garuda-toolbox doctor --json"
    )]
    Doctor(Read),

    /// Read the user guide in the terminal.
    #[command(
        display_order = 21,
        after_help = "Examples:\n  garuda-toolbox guide\n  garuda-toolbox guide show kernels"
    )]
    Guide(Group<GuideCommand>),

    /// List the pages of the window, for `--page`.
    #[command(display_order = 2, after_help = "Example:\n  garuda-toolbox pages")]
    Pages(Read),

    /// Collect a report for a forum post or a bug report.
    #[command(
        display_order = 17,
        after_help = "Examples:\n  garuda-toolbox diagnostics inxi\n  \
                            garuda-toolbox diagnostics all --upload"
    )]
    Diagnostics(Diagnostics),

    /// List, start, stop and enable services, the system's and your own.
    #[command(
        display_order = 6,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox services list --user\n  \
                      garuda-toolbox services restart sshd.service"
    )]
    Services(ServicesCommand),

    /// List and switch Components, Gaming, System Tweaks and the Setup
    /// Assistant.
    #[command(
        display_order = 4,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox toggle list --page gaming\n  \
                      garuda-toolbox toggle on pipewire-support"
    )]
    Toggle(ToggleCommand),

    /// List, install and remove the apps of the App Catalog.
    #[command(
        display_order = 5,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox apps list --category internet\n  \
                      garuda-toolbox apps install firefox"
    )]
    Apps(AppsCommand),

    /// Search, show, install and remove the packages of the repositories
    /// and the AUR.
    #[command(
        display_order = 5,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox packages search editor\n  \
                      garuda-toolbox packages install gimp"
    )]
    Packages(PackagesCommand),

    /// List, install and remove kernels and their headers.
    #[command(
        display_order = 7,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox kernels list --json\n  \
                      garuda-toolbox kernels install linux-lts"
    )]
    Kernels(KernelsCommand),

    /// List and run the tasks and repairs of the Maintenance page.
    #[command(
        display_order = 16,
        subcommand,
        after_help = "Examples:\n  garuda-toolbox maintenance list\n  \
                      garuda-toolbox maintenance run clean-package-cache"
    )]
    Maintenance(MaintenanceCommand),

    /// Show and build the DKMS modules of each kernel.
    #[command(
        display_order = 8,
        subcommand,
        after_help = "Example:\n  garuda-toolbox dkms build"
    )]
    Dkms(DkmsCommand),

    /// Turn locales on or off and set the system language.
    #[command(
        display_order = 9,
        subcommand,
        after_help = "Example:\n  garuda-toolbox locales system de_DE.UTF-8"
    )]
    Locales(LocalesCommand),

    /// Add, change and remove accounts.
    #[command(
        display_order = 10,
        subcommand,
        after_help = "Example:\n  garuda-toolbox users add ada"
    )]
    Users(UsersCommand),

    /// Add and remove groups.
    #[command(
        display_order = 11,
        subcommand,
        after_help = "Example:\n  garuda-toolbox groups add media"
    )]
    Groups(GroupsCommand),

    /// Set the boot menu: default entry, timeout, kernel parameters, looks.
    #[command(
        display_order = 14,
        after_help = "Examples:\n  garuda-toolbox boot\n  garuda-toolbox boot set timeout 3"
    )]
    Boot(Group<BootCommand>),

    /// Show and edit config files, the system's and your own.
    #[command(
        display_order = 12,
        subcommand,
        after_help = "Example:\n  garuda-toolbox configs edit pacman-config"
    )]
    Configs(ConfigsCommand),

    /// Show the connection and switch radios, drivers and network tests.
    #[command(
        display_order = 13,
        after_help = "Examples:\n  garuda-toolbox network\n  garuda-toolbox network radio wifi off"
    )]
    Network(Group<NetworkCommand>),

    /// Choose the DNS servers, the Wi-Fi backend, hblock and garuda-update's
    /// options, as the System Tweaks page does.
    #[command(
        display_order = 13,
        after_help = "Examples:\n  garuda-toolbox tweaks\n  garuda-toolbox tweaks dns cloudflare"
    )]
    Tweaks(Group<TweaksCommand>),

    /// Repair the boot of an installed system, from the live ISO.
    #[command(
        display_order = 18,
        subcommand,
        name = "bootrepair",
        visible_alias = "boot-repair",
        after_help = "Example:\n  garuda-toolbox bootrepair show"
    )]
    BootRepair(BootRepairCommand),

    /// List and pick the Setup Assistant's apps.
    #[command(
        display_order = 3,
        subcommand,
        after_help = "Example:\n  garuda-toolbox setup list"
    )]
    Setup(SetupCommand),

    /// Show and change the app's own settings.
    #[command(
        display_order = 19,
        after_help = "Examples:\n  garuda-toolbox settings\n  garuda-toolbox settings set aur on"
    )]
    Settings(Group<SettingsCommand>),

    /// Update the system now, and the AUR packages with `--aur`.
    #[command(
        display_order = 15,
        after_help = "Examples:\n  garuda-toolbox update\n  \
                            garuda-toolbox update --aur --yes"
    )]
    Update(Update),

    /// Queue changes as the window does, then apply them as one plan.
    /// While the window runs, its own queue.
    #[command(
        display_order = 21,
        after_help = "Examples:\n  garuda-toolbox queue add apps install firefox\n  \
                      garuda-toolbox queue\n  garuda-toolbox queue apply"
    )]
    Queue(Group<QueueCommand>),

    /// Print the completion hook for a shell, which asks this program for
    /// the names at each Tab.
    #[command(
        display_order = 22,
        after_help = "Examples:\n  garuda-toolbox completions bash > ~/.local/share/bash-completion/completions/garuda-toolbox\n  \
                      garuda-toolbox completions zsh > ~/.zfunc/_garuda-toolbox"
    )]
    Completions(Completions),

    /// Explain a command of this program.
    #[command(
        display_order = 23,
        after_help = "Examples:\n  garuda-toolbox help\n  garuda-toolbox help users add"
    )]
    Help(Help),
}

/// The output of a command that only reads.
#[derive(Clone, Copy, Debug, Default, Args)]
pub struct Read {
    /// Print JSON instead of text, for scripts.
    #[arg(long)]
    pub json: bool,
}

/// The question before a change.
#[derive(Clone, Copy, Debug, Default, Args)]
pub struct Confirm {
    /// Do not ask, carry the change out. Needed without a terminal.
    #[arg(long, short = 'y')]
    pub yes: bool,

    /// Print the plan and stop, without asking and without changing
    /// anything.
    #[arg(long, conflicts_with = "yes")]
    pub dry_run: bool,

    /// Print JSON Lines instead of text, for scripts. Needs `--yes` or
    /// `--dry-run`.
    #[arg(long)]
    pub json: bool,
}

impl Confirm {
    /// `--json` without `--yes` or `--dry-run`: the question cannot be
    /// asked, as its answer would land among the JSON lines.
    pub const fn cannot_ask(self) -> bool {
        self.json && !self.yes && !self.dry_run
    }
}

/// A page, an action or a category, by the rule the session bus applies
/// to a later start's wish too.
fn name(text: &str) -> Result<PageName, String> {
    PageName::new(text).map_err(|_invalid| {
        format!("`{text}` is no name: small letters and hyphens, at most {MAX_PAGE_NAME_BYTES}")
    })
}

fn columns(text: &str) -> Result<usize, String> {
    text.parse()
        .ok()
        .filter(|cols| *cols > 0)
        .ok_or_else(|| format!("`{text}` is no width: a number of columns, 1 or more"))
}
