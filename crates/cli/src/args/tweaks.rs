//! The System Tweaks page's choices beside the login shell, which
//! `users edit --shell` sets.

use clap::builder::{PossibleValuesParser, TypedValueParser as _};
use clap::{Args, Subcommand};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, WifiBackend};

use super::net::OnOff;
use super::{Confirm, ReadList};

#[derive(Debug, Subcommand)]
pub enum TweaksCommand {
    /// List the DNS servers, the Wi-Fi backend, hblock and the options of
    /// garuda-update, each as it is now.
    #[command(after_help = "Example:\n  garuda-toolbox tweaks list --json")]
    List(ReadList),

    /// Choose the DNS servers NetworkManager uses.
    #[command(after_help = "Example:\n  garuda-toolbox tweaks dns cloudflare")]
    Dns(DnsArgs),

    /// Choose the program NetworkManager runs Wi-Fi with.
    #[command(after_help = "Example:\n  garuda-toolbox tweaks wifi-backend iwd")]
    WifiBackend(WifiBackendArgs),

    /// Block ads and trackers system-wide with hblock.
    #[command(after_help = "Example:\n  garuda-toolbox tweaks hblock on")]
    Hblock(HblockArgs),

    /// Switch an option of garuda-update.
    #[command(after_help = "Example:\n  garuda-toolbox tweaks update-flag update-aur on")]
    UpdateFlag(UpdateFlagArgs),
}

#[derive(Debug, Args)]
pub struct DnsArgs {
    #[arg(value_parser = PossibleValuesParser::new(DnsProvider::ALL.map(DnsProvider::as_str))
        .try_map(|name: String| named(&DnsProvider::ALL, DnsProvider::as_str, &name)))]
    pub provider: DnsProvider,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct WifiBackendArgs {
    #[arg(value_parser = PossibleValuesParser::new(WifiBackend::ALL.map(WifiBackend::as_str))
        .try_map(|name: String| named(&WifiBackend::ALL, WifiBackend::as_str, &name)))]
    pub backend: WifiBackend,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct HblockArgs {
    pub state: OnOff,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct UpdateFlagArgs {
    #[arg(value_parser = PossibleValuesParser::new(GarudaUpdateFlag::ALL.map(GarudaUpdateFlag::as_str))
        .try_map(|name: String| named(&GarudaUpdateFlag::ALL, GarudaUpdateFlag::as_str, &name)))]
    pub flag: GarudaUpdateFlag,

    pub state: OnOff,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// The value called `name`. The possible values were checked first, so
/// the error stays a fallback.
fn named<T>(all: &[T], as_str: fn(T) -> &'static str, name: &str) -> Result<T, String>
where
    T: Copy,
{
    all.iter()
        .copied()
        .find(|value| as_str(*value) == name)
        .ok_or_else(|| format!("no value is called `{name}`"))
}
