//! The arguments of the Languages, Boot Options and Settings pages, and
//! of the DKMS builds of the Kernels page.

use clap::{Args, Subcommand, ValueEnum};
use toolbox_plan::{KernelParameter, LocaleName, PackageName};

use super::{Confirm, LocalesList, Read, ReadList};

#[derive(Debug, Subcommand)]
pub enum LocalesCommand {
    /// List the locales: enabled, the system's, and the ones on offer.
    #[command(after_help = "Example:\n  garuda-toolbox locales list --json")]
    List(LocalesList),

    /// Enable a locale, so programs can use it.
    #[command(after_help = "Example:\n  garuda-toolbox locales enable de_DE.UTF-8")]
    Enable(LocaleArgs),

    /// Disable a locale. The system's locale stays.
    #[command(after_help = "Example:\n  garuda-toolbox locales disable fr_FR.UTF-8")]
    Disable(LocaleArgs),

    /// Make a locale the system's language. It is enabled too.
    #[command(
        visible_alias = "set-system",
        after_help = "Example:\n  garuda-toolbox locales system de_DE.UTF-8"
    )]
    System(LocaleArgs),

    /// The translations of installed programs for the enabled locales.
    #[command(
        subcommand,
        after_help = "Examples:\n  garuda-toolbox locales packs list\n  \
                      garuda-toolbox locales packs install firefox-i18n-de"
    )]
    Packs(PacksCommand),
}

#[derive(Debug, Args)]
pub struct LocaleArgs {
    /// The locale, as `locales list` prints it, for example `de_DE.UTF-8`.
    pub locale: LocaleName,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum PacksCommand {
    /// List the language packs and whether each is installed.
    #[command(after_help = "Example:\n  garuda-toolbox locales packs list --json")]
    List(ReadList),

    /// Install a language pack.
    #[command(after_help = "Example:\n  garuda-toolbox locales packs install firefox-i18n-de")]
    Install(PackArgs),

    /// Remove a language pack.
    #[command(after_help = "Example:\n  garuda-toolbox locales packs remove firefox-i18n-de")]
    Remove(PackArgs),
}

#[derive(Debug, Args)]
pub struct PackArgs {
    /// The pack's package, as `locales packs list` prints it.
    pub package: PackageName,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum BootCommand {
    /// The boot menu's settings as they are now.
    #[command(after_help = "Example:\n  garuda-toolbox boot show --json")]
    Show(Read),

    /// The entries of the boot menu, for `boot set default`.
    #[command(after_help = "Example:\n  garuda-toolbox boot entries")]
    Entries(Read),

    /// Change one setting of the boot menu, then regenerate it.
    #[command(after_help = "Examples:\n  garuda-toolbox boot set timeout 3\n  \
                            garuda-toolbox boot set quiet off\n  \
                            garuda-toolbox boot set plymouth bgrt\n  \
                            garuda-toolbox boot set background none")]
    Set(BootSet),

    /// Add or remove a kernel parameter.
    #[command(
        subcommand,
        after_help = "Examples:\n  garuda-toolbox boot param add nowatchdog\n  \
                      garuda-toolbox boot param remove nowatchdog"
    )]
    Param(ParamCommand),
}

#[derive(Debug, Args)]
pub struct BootSet {
    /// What to change.
    #[arg(value_enum)]
    pub key: BootKey,

    /// The new value: an entry id, seconds, on or off, a full path or
    /// `none`, or a Plymouth theme.
    pub value: String,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// A setting of the Boot Options page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum BootKey {
    /// The entry that boots by default, as `boot entries` prints it.
    Default,
    /// Seconds the menu shows, 0 to 300.
    Timeout,
    /// Boot the entry that booted last (on or off).
    Remember,
    /// Group older kernels in a submenu (on or off).
    Submenus,
    /// List other systems in the menu (on or off).
    OsProber,
    /// Hide the kernel's messages while booting (on or off).
    Quiet,
    /// Show a splash screen while booting (on or off).
    Splash,
    /// Turn the CPU mitigations off (on or off).
    MitigationsOff,
    /// Allow the magic `SysRq` keys (on or off).
    Sysrq,
    /// The picture behind the menu: a full path, or `none`.
    Background,
    /// The menu's theme: the full path of a theme.txt, or `none`.
    Theme,
    /// The Plymouth theme shown while booting.
    Plymouth,
}

#[derive(Debug, Subcommand)]
pub enum ParamCommand {
    /// Add a kernel parameter.
    #[command(after_help = "Example:\n  garuda-toolbox boot param add loglevel=3")]
    Add(ParamArgs),

    /// Remove a kernel parameter.
    #[command(after_help = "Example:\n  garuda-toolbox boot param remove nowatchdog")]
    Remove(ParamArgs),
}

#[derive(Debug, Args)]
pub struct ParamArgs {
    /// The parameter, for example `nowatchdog` or `loglevel=3`.
    pub parameter: KernelParameter,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum DkmsCommand {
    /// The modules DKMS builds and the kernels each is built for.
    #[command(
        visible_alias = "status",
        after_help = "Example:\n  garuda-toolbox dkms list --json"
    )]
    List(ReadList),

    /// Build every module a kernel lacks, or the ones of one kernel.
    #[command(after_help = "Examples:\n  garuda-toolbox dkms build\n  \
                            garuda-toolbox dkms build --kernel linux-lts")]
    Build(DkmsBuild),
}

#[derive(Debug, Args)]
pub struct DkmsBuild {
    /// Only the modules of this kernel's package.
    #[arg(long)]
    pub kernel: Option<PackageName>,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Subcommand)]
pub enum SettingsCommand {
    /// Every setting of the app and its value.
    #[command(after_help = "Example:\n  garuda-toolbox settings list --json")]
    List(ReadList),

    /// One setting's value.
    #[command(after_help = "Example:\n  garuda-toolbox settings get language --json")]
    Get(SettingKeyArgs),

    /// Change one setting. The window takes it at its next start.
    #[command(
        after_help = "Examples:\n  garuda-toolbox settings set language de\n  \
                            garuda-toolbox settings set update-check 60\n  \
                            garuda-toolbox settings set start-at-login on\n  \
                            garuda-toolbox settings set dark-scheme nord"
    )]
    Set(SettingSet),
}

#[derive(Debug, Args)]
pub struct SettingKeyArgs {
    #[arg(value_enum)]
    pub key: SettingKey,

    #[command(flatten)]
    pub read: Read,
}

#[derive(Debug, Args)]
pub struct SettingSet {
    #[arg(value_enum)]
    pub key: SettingKey,

    /// The new value, see `settings set --help`.
    pub value: String,
}

/// A setting of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SettingKey {
    /// A language code such as `de`, or `system`.
    Language,
    /// Open the window at login: on or off.
    StartAtLogin,
    /// Minutes between update checks (5 to 1440), or `never`.
    UpdateCheck,
    /// The Services page opens on your own services: on or off.
    UserServices,
    /// Copy a finished report and an upload's link: on or off.
    CopyReports,
    /// The Packages page lists the AUR, downloaded once a day: on or off.
    Aur,
    /// How much the log file holds: error, warn, info, debug, trace.
    LogLevel,
    /// The colours of the dark theme: a dark colour scheme such as `nord`,
    /// or `system` for the desktop's.
    DarkScheme,
    /// The colours of the light theme: a light colour scheme such as
    /// `catppuccin-latte`, or `system` for the desktop's.
    LightScheme,
}
