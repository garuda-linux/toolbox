//! The arguments of the Network, Config Files, Boot Repair and Setup
//! Assistant pages.

use clap::{Args, Subcommand, ValueEnum};
use toolbox_plan::{DEFAULT_HOPS, DEFAULT_PING_TIMEOUT, DEFAULT_PINGS, HostName, KernelModule};

use super::change::id;
use super::{Confirm, Read, ReadList};

const MAX_PINGS: i64 = 100;
const MAX_PING_TIMEOUT_SECONDS: i64 = 60;
const MAX_HOPS: i64 = 64;

#[derive(Debug, Subcommand)]
pub enum NetworkCommand {
    /// The connection, the radios and the network services.
    #[command(
        visible_alias = "status",
        after_help = "Example:\n  garuda-toolbox network show --json"
    )]
    Show(Read),

    /// Switch a radio on or off.
    #[command(after_help = "Examples:\n  garuda-toolbox network radio wifi on\n  \
                            garuda-toolbox network radio flight on")]
    Radio(RadioArgs),

    /// The network drivers and whether each is blacklisted.
    #[command(
        subcommand,
        after_help = "Examples:\n  garuda-toolbox network drivers list\n  \
                      garuda-toolbox network drivers blacklist r8169"
    )]
    Drivers(DriversCommand),

    /// Ask geoip.kde.org for the public address. Only this command
    /// contacts it, nothing is kept.
    #[command(after_help = "Example:\n  garuda-toolbox network public-address")]
    PublicAddress(Read),

    /// Ping a host, in the terminal.
    #[command(after_help = "Example:\n  garuda-toolbox network ping garudalinux.org --count 4")]
    Ping(PingArgs),

    /// Trace the route to a host, in the terminal.
    #[command(after_help = "Example:\n  garuda-toolbox network traceroute garudalinux.org")]
    Traceroute(TraceArgs),
}

#[derive(Debug, Args)]
pub struct RadioArgs {
    #[arg(value_enum)]
    pub radio: Radio,

    #[arg(value_enum)]
    pub state: OnOff,
}

/// A radio of the Network page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Radio {
    Wifi,
    Bluetooth,
    /// Flight mode: every radio off.
    Flight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

impl OnOff {
    pub const fn on(self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Debug, Subcommand)]
pub enum DriversCommand {
    /// List the network drivers.
    #[command(after_help = "Example:\n  garuda-toolbox network drivers list --json")]
    List(ReadList),

    /// Keep a driver from loading at the next start.
    #[command(
        visible_alias = "block",
        after_help = "Example:\n  garuda-toolbox network drivers blacklist r8169"
    )]
    Blacklist(ModuleArgs),

    /// Let a blacklisted driver load again.
    #[command(
        visible_alias = "unblock",
        after_help = "Example:\n  garuda-toolbox network drivers allow r8169"
    )]
    Allow(ModuleArgs),
}

#[derive(Debug, Args)]
pub struct ModuleArgs {
    /// The kernel module, as `network drivers list` prints it.
    pub module: KernelModule,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct PingArgs {
    /// A host name or an address.
    pub host: HostName,

    /// How many pings.
    #[arg(long, default_value_t = DEFAULT_PINGS, value_parser = clap::value_parser!(u32).range(1..=MAX_PINGS))]
    pub count: u32,

    /// Seconds to wait for each answer.
    #[arg(
        long,
        default_value_t = DEFAULT_PING_TIMEOUT,
        value_parser = clap::value_parser!(u32).range(1..=MAX_PING_TIMEOUT_SECONDS)
    )]
    pub timeout: u32,
}

#[derive(Debug, Args)]
pub struct TraceArgs {
    /// A host name or an address.
    pub host: HostName,

    /// The most hops.
    #[arg(long, default_value_t = DEFAULT_HOPS, value_parser = clap::value_parser!(u32).range(1..=MAX_HOPS))]
    pub max_hops: u32,
}

#[derive(Debug, Subcommand)]
pub enum ConfigsCommand {
    /// List the config files the page offers.
    #[command(after_help = "Example:\n  garuda-toolbox configs list --json")]
    List(ReadList),

    /// Print a config file's text.
    #[command(after_help = "Example:\n  garuda-toolbox configs show pacman-config")]
    Show(ConfigArgs),

    /// Edit a config file in your editor, then review and save it.
    #[command(after_help = "Example:\n  EDITOR=nano garuda-toolbox configs edit pacman-config")]
    Edit(ConfigEdit),
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// The file's key, as `configs list` prints it.
    pub key: String,
}

#[derive(Debug, Args)]
pub struct ConfigEdit {
    /// The file's key, as `configs list` prints it.
    pub key: String,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum BootRepairCommand {
    /// The installed systems and boot targets found, on the live ISO.
    #[command(
        visible_alias = "status",
        after_help = "Example:\n  garuda-toolbox bootrepair show --json"
    )]
    Show(Read),

    /// Build the boot files of an installed system again: GRUB's menu, or
    /// systemd-boot's kernel images.
    #[command(after_help = "Example:\n  garuda-toolbox bootrepair regenerate /dev/nvme0n1p2")]
    Regenerate(RepairArgs),

    /// Install the boot loader of an installed system again, GRUB or
    /// systemd-boot, then build its boot files.
    #[command(
        after_help = "Example:\n  garuda-toolbox bootrepair reinstall /dev/nvme0n1p2 \
                            --target /dev/nvme0n1p1"
    )]
    Reinstall(ReinstallArgs),
}

#[derive(Debug, Args)]
pub struct RepairArgs {
    /// The installed system's device, as `bootrepair show` prints it.
    pub system: String,

    /// A locked system's passphrase is asked on the terminal without
    /// echo, or read from the first line of standard input with this.
    #[arg(long)]
    pub passphrase_stdin: bool,

    /// Run on an installed system too, for testing. The helper still
    /// refuses a disk the running system uses.
    #[arg(long)]
    pub boot_repair: bool,

    /// Outside the live ISO, run the chosen system's own grub-install,
    /// mkinitcpio and bootctl as root. Anyone who could write to that
    /// disk could put code there.
    #[arg(long)]
    pub trust_disk: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct ReinstallArgs {
    #[command(flatten)]
    pub repair: RepairArgs,

    /// The EFI system partition, or the whole disk on a BIOS machine.
    /// Without it the usual one.
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SetupCommand {
    /// The steps of the Setup Assistant and their choices.
    #[command(after_help = "Example:\n  garuda-toolbox setup list --json")]
    List(ReadList),

    /// Turn several choices on at once, as the assistant's summary does.
    #[command(after_help = "Example:\n  garuda-toolbox setup apply firefox strawberry-player")]
    Apply(SetupApply),
}

#[derive(Debug, Args)]
pub struct SetupApply {
    /// The choices' ids, as `setup list` prints them.
    #[arg(required = true, value_parser = id)]
    pub ids: Vec<String>,

    #[command(flatten)]
    pub confirm: Confirm,
}
