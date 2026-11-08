//! The app's pages, in sidebar order, in one enum.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// One page of the app. The queue and the console are panels, not pages.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    strum::IntoStaticStr,
    strum::EnumString,
)]
#[strum(serialize_all = "kebab-case")]
pub enum PageId {
    #[default]
    Home,
    Setup,
    Components,
    Gaming,
    Discover,
    Packages,
    Services,
    SystemSettings,
    Kernels,
    Locales,
    Users,
    Configs,
    Network,
    Boot,
    Maintenance,
    BootRepair,
    Diagnostics,
    Settings,
    Guide,
    About,
}

/// The sidebar's sections, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Start,
    Install,
    System,
    Care,
    App,
}

impl PageId {
    pub const ALL: [Self; 20] = [
        Self::Home,
        Self::Setup,
        Self::Components,
        Self::Gaming,
        Self::Discover,
        Self::Packages,
        Self::Services,
        Self::SystemSettings,
        Self::Kernels,
        Self::Locales,
        Self::Users,
        Self::Configs,
        Self::Network,
        Self::Boot,
        Self::Maintenance,
        Self::BootRepair,
        Self::Diagnostics,
        Self::Settings,
        Self::Guide,
        Self::About,
    ];

    pub const fn text_id(self) -> &'static str {
        match self {
            Self::Home => "page-home",
            Self::Setup => "page-setup",
            Self::Components => "page-components",
            Self::Gaming => "page-gaming",
            Self::Discover => "page-discover",
            Self::Packages => "page-packages",
            Self::Services => "page-services",
            Self::SystemSettings => "page-system-settings",
            Self::Kernels => "page-kernels",
            Self::Locales => "page-locales",
            Self::Users => "page-users",
            Self::Configs => "page-configs",
            Self::Network => "page-network",
            Self::Boot => "page-boot",
            Self::Maintenance => "page-maintenance",
            Self::BootRepair => "page-boot-repair",
            Self::Diagnostics => "page-diagnostics",
            Self::Settings => "page-settings",
            Self::Guide => "page-guide",
            Self::About => "page-about",
        }
    }

    /// The page's word for other starts, desktop actions and the command
    /// line.
    pub fn slug(self) -> &'static str {
        self.into()
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        slug.parse().ok()
    }

    /// The user guide's page on this page, under `docs/src`, and the
    /// heading it starts at. F1 opens it.
    pub const fn guide_topic(self) -> (&'static str, Option<&'static str>) {
        match self {
            Self::Home => ("home.md", None),
            Self::Setup => ("setup-assistant.md", None),
            Self::Components => ("components.md", None),
            Self::Gaming => ("gaming.md", None),
            Self::Discover => ("discover.md", None),
            Self::Packages => ("packages.md", None),
            Self::Services => ("services.md", None),
            Self::SystemSettings => ("system-settings.md", None),
            Self::Kernels => ("kernels.md", None),
            Self::Locales => ("locales.md", None),
            Self::Users => ("users.md", None),
            Self::Configs => ("config-files.md", None),
            Self::Network => ("network.md", None),
            Self::Boot => ("boot-options.md", None),
            Self::Maintenance => ("maintenance.md", None),
            Self::BootRepair => ("boot-repair.md", None),
            Self::Diagnostics => ("diagnostics.md", None),
            Self::Settings => ("settings.md", Some("settings")),
            Self::Guide => ("guide.md", None),
            Self::About => ("settings.md", Some("about")),
        }
    }

    /// The page a guide page is about, for its Open button. Settings and
    /// About share one guide page, which opens Settings.
    pub fn of_guide_file(file: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .filter(|page| *page != Self::Guide && *page != Self::About)
            .find(|page| page.guide_topic().0 == file)
    }

    pub const fn group(self) -> Group {
        match self {
            Self::Home => Group::Start,
            Self::Setup | Self::Components | Self::Gaming | Self::Discover | Self::Packages => {
                Group::Install
            }
            Self::Services
            | Self::SystemSettings
            | Self::Kernels
            | Self::Locales
            | Self::Users
            | Self::Configs
            | Self::Network
            | Self::Boot => Group::System,
            Self::Maintenance | Self::BootRepair | Self::Diagnostics => Group::Care,
            Self::Settings | Self::Guide | Self::About => Group::App,
        }
    }
}

impl Group {
    /// The Fluent id of the section heading. The first and last sections
    /// have none.
    pub const fn heading_id(self) -> Option<&'static str> {
        match self {
            Self::Start | Self::App => None,
            Self::Install => Some("group-install"),
            Self::System => Some("group-system"),
            Self::Care => Some("group-care"),
        }
    }
}
