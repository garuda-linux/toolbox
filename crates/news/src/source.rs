//! The two feeds the news reads: Garuda's forum announcements and Arch
//! Linux's news. Both are fixed addresses, nothing is built from input.

use serde::{Deserialize, Serialize};

/// The Garuda forum category whose posts ask for a step of the user,
/// as Arch's titles say "manual intervention".
const GARUDA_MAINTENANCE: &str = "Announcements - Maintenance";

const ARCH_MANUAL: &str = "manual intervention";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    #[default]
    Garuda,
    Arch,
}

impl Source {
    pub const ALL: [Self; 2] = [Self::Garuda, Self::Arch];

    pub const fn feed(self) -> &'static str {
        match self {
            // The category's own feed. `/c/announcements.rss` redirects here.
            Self::Garuda => "https://forum.garudalinux.org/c/announcements/16.rss",
            Self::Arch => "https://archlinux.org/feeds/news/",
        }
    }

    pub const fn page(self) -> &'static str {
        match self {
            Self::Garuda => "https://forum.garudalinux.org/c/announcements/16",
            Self::Arch => "https://archlinux.org/news/",
        }
    }

    pub const fn post_prefix(self) -> &'static str {
        match self {
            Self::Garuda => "https://forum.garudalinux.org/",
            Self::Arch => "https://archlinux.org/news/",
        }
    }

    /// Stable code, for file names and text ids.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Garuda => "garuda",
            Self::Arch => "arch",
        }
    }

    pub fn asks_for_a_step(self, title: &str, categories: &[String]) -> bool {
        match self {
            Self::Garuda => categories
                .iter()
                .any(|category| category.trim() == GARUDA_MAINTENANCE),
            Self::Arch => title.to_lowercase().contains(ARCH_MANUAL),
        }
    }
}
