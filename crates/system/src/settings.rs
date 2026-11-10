//! System-wide choices of the System Settings page, read from the files
//! the root helper writes: NetworkManager's drop-ins, hblock's header in
//! `/etc/hosts` and garuda-update's config. All of them are world readable
//! on a standard install, so no root is needed to show them.

#[cfg(test)]
mod tests;
mod wifi;

use std::io;
use std::path::{Path, PathBuf};

use toolbox_plan::{
    DNS_CONFIG, DnsProvider, GARUDA_UPDATE_CONFIG, GarudaUpdateFlag, MODPROBE_BLACKLIST,
    NETWORKMANAGER_CONF_D, blacklisted_modules,
};
use toolbox_state::parts::OwnBlacklist;
use toolbox_state::settings::{CurrentDns, CurrentWifiBackend, GarudaUpdateConfig, HblockState};

use crate::error::{ProbeError, from_io};
use crate::files;

const BLOCKED_DOMAINS: &str = "Blocked domains:";
const HOSTS: &str = "etc/hosts";

pub trait SettingsSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without NetworkManager's drop-in folder,
    /// [`ProbeError::Failed`] when the drop-in cannot be read.
    fn dns(&self) -> Result<CurrentDns, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the hosts file cannot be read.
    fn hblock(&self) -> Result<HblockState, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without NetworkManager's drop-in folder,
    /// [`ProbeError::Failed`] when it cannot be listed.
    fn wifi_backend(&self) -> Result<CurrentWifiBackend, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without garuda-update's folder or when
    /// the config is not readable, [`ProbeError::Failed`] when reading
    /// fails otherwise.
    fn garuda_update(&self) -> Result<GarudaUpdateConfig, ProbeError>;

    /// The modules the app's own modprobe file blacklists. None without
    /// the file, which the helper only creates with its first entry.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the file cannot be read.
    fn module_blacklist(&self) -> Result<OwnBlacklist, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcSettings {
    root: PathBuf,
}

impl EtcSettings {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/")
    }

    /// Reads `etc/…` below `root`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn conf_d(&self) -> PathBuf {
        self.root.join(NETWORKMANAGER_CONF_D)
    }
}

impl SettingsSource for EtcSettings {
    fn dns(&self) -> Result<CurrentDns, ProbeError> {
        let conf_d = self.conf_d();

        if !conf_d.is_dir() {
            return Err(ProbeError::Unavailable);
        }

        let config = files::read_optional(&conf_d.join(DNS_CONFIG))?;

        Ok(CurrentDns(
            config.as_deref().map_or(DnsProvider::Default, parse_dns),
        ))
    }

    fn hblock(&self) -> Result<HblockState, ProbeError> {
        let hosts = files::read_optional(&self.root.join(HOSTS))?;

        Ok(HblockState(hosts.as_deref().is_some_and(blocks_domains)))
    }

    fn wifi_backend(&self) -> Result<CurrentWifiBackend, ProbeError> {
        wifi::backend(&self.root).map(CurrentWifiBackend)
    }

    /// A missing file in an existing folder means every switch is off, the
    /// helper creates the file on the first change.
    fn garuda_update(&self) -> Result<GarudaUpdateConfig, ProbeError> {
        let path = self.root.join(GARUDA_UPDATE_CONFIG);

        match files::read_lossy(&path) {
            Ok(config) => Ok(GarudaUpdateConfig(GarudaUpdateFlag::enabled_in(&config))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if path.parent().is_some_and(Path::is_dir) {
                    Ok(GarudaUpdateConfig::default())
                } else {
                    Err(ProbeError::Unavailable)
                }
            }
            Err(error) => Err(from_io(path.display(), error)),
        }
    }

    fn module_blacklist(&self) -> Result<OwnBlacklist, ProbeError> {
        let config = files::read_optional(&self.root.join(MODPROBE_BLACKLIST))?;

        Ok(OwnBlacklist(
            config
                .as_deref()
                .map(blacklisted_modules)
                .unwrap_or_default(),
        ))
    }
}

/// The provider of the first `servers=` entry. Anything else (a server of
/// no known provider, no server line) shows as the default, which the
/// page can switch from.
fn parse_dns(config: &str) -> DnsProvider {
    let server = config.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;

        (key.trim() == "servers").then(|| {
            value
                .split([',', ';'])
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        })
    });

    let provider = server.as_deref().and_then(DnsProvider::from_server);

    if provider.is_none() {
        tracing::debug!(
            "the DNS drop-in names no known provider: {}",
            server.as_deref().unwrap_or("no server")
        );
    }

    provider.unwrap_or(DnsProvider::Default)
}

/// hblock writes `# Blocked domains: <count>` into its header.
fn blocks_domains(hosts: &str) -> bool {
    hosts.lines().any(|line| {
        line.trim_start()
            .strip_prefix('#')
            .and_then(|comment| comment.trim_start().strip_prefix(BLOCKED_DOMAINS))
            .and_then(|count| count.trim().parse::<u64>().ok())
            .is_some_and(|count| count > 0)
    })
}
