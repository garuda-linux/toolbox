//! System-wide network choices of the System Settings page: the DNS
//! provider NetworkManager uses and its Wi-Fi backend. Both are closed sets,
//! so the files the helper writes for them come from here and never from
//! the app.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// NetworkManager's drop-in folder, relative to `/`.
pub const NETWORKMANAGER_CONF_D: &str = "etc/NetworkManager/conf.d";

/// The name the Electron app used, kept so DNS choices made with it still
/// show.
pub const DNS_CONFIG: &str = "10-garuda-assistant-dns.conf";

pub const WIFI_BACKEND_CONFIG: &str = "20_wifi_backend_toolbox.conf";

/// Written by an older Garuda tool. Switching back to `wpa_supplicant`
/// removes it as well, or it would keep iwd selected.
pub const OLD_WIFI_BACKEND_CONFIG: &str = "20_wifi_backend_rani.conf";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DnsProvider {
    /// Whatever the connection hands out, usually the ISP's resolver.
    Default,
    Google,
    Cloudflare,
    Quad9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WifiBackend {
    Iwd,
    WpaSupplicant,
}

impl DnsProvider {
    pub const ALL: [Self; 4] = [Self::Default, Self::Google, Self::Cloudflare, Self::Quad9];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Google => "google",
            Self::Cloudflare => "cloudflare",
            Self::Quad9 => "quad9",
        }
    }

    /// The resolvers written to NetworkManager, none for the default.
    pub const fn servers(self) -> &'static [&'static str] {
        match self {
            Self::Default => &[],
            Self::Google => &["8.8.8.8"],
            Self::Cloudflare => &["1.1.1.1"],
            Self::Quad9 => &["9.9.9.9"],
        }
    }

    pub fn from_server(server: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.servers().contains(&server))
    }

    /// The content of [`DNS_CONFIG`], `None` when the file should not
    /// exist.
    pub fn config(self) -> Option<String> {
        let servers = self.servers();

        (!servers.is_empty())
            .then(|| format!("[global-dns-domain-*]\nservers={}\n", servers.join(",")))
    }
}

impl WifiBackend {
    pub const ALL: [Self; 2] = [Self::Iwd, Self::WpaSupplicant];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Iwd => "iwd",
            Self::WpaSupplicant => "wpa-supplicant",
        }
    }

    /// The content of [`WIFI_BACKEND_CONFIG`], `None` for NetworkManager's
    /// own default (`wpa_supplicant`), which needs no file.
    pub const fn config(self) -> Option<&'static str> {
        match self {
            Self::Iwd => Some("[device]\nwifi.backend=iwd\n"),
            Self::WpaSupplicant => None,
        }
    }
}
