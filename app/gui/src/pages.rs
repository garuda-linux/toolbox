//! The pages in the middle of the window.
//!
//! Each one is a struct with the state that belongs to it alone (a
//! search, a selection) and a `ui` method. Pages read the system and the
//! queue and send commands for every change, so the app is the one place
//! that changes its data.

pub mod about;
pub mod boot;
pub mod bootrepair;
mod catalog;
mod components;
pub mod configs;
pub mod diagnostics;
pub mod discover;
mod gaming;
pub mod guide;
mod header;
pub mod home;
pub mod kernels;
pub mod locales;
pub mod maintenance;
pub mod network;
pub mod packages;
pub mod rows;
pub mod services;
pub mod setting_row;
pub mod settings;
pub mod setup;
pub mod toggles;
pub mod tweaks;
pub mod users;

use std::collections::HashMap;
use std::sync::Arc;

use about::AboutPage;
use boot::BootPage;
use bootrepair::BootRepairPage;
use components::ComponentsPage;
use configs::ConfigsPage;
use diagnostics::DiagnosticsPage;
use discover::DiscoverPage;
use egui::Ui;
use gaming::GamingPage;
use guide::GuidePage;
use home::{HomeFacts, HomePage};
use kernels::KernelsPage;
use kernels::view::DkmsReading;
use locales::LocalesPage;
use maintenance::MaintenancePage;
use network::NetworkPage;
use packages::PackagesPage;
use services::ServicesPage;
use settings::SettingsPage;
use setup::SetupPage;
use toolbox_core::features::console::ending::Ending;
use toolbox_core::features::{kernels as kernels_core, toggles as toggles_core};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{BodyShape, Tone, banner};
use toolbox_plan::{PackageName, UnitScope};
use tweaks::TweaksPage;
use users::UsersPage;

use crate::catalogs::{CatalogId, Catalogs};
use crate::command::CommandSender;
use crate::desktop::icon_files::AppIcons;
use crate::page::PageId;
use crate::page_service::{Provides, Services};
use crate::pictures::Pictures;
use crate::settings::SettingsView;
use crate::units::Units;

/// What every page reads, built each frame.
pub struct PageContext<'app> {
    /// Shared, so a page tells a new snapshot from the last by identity.
    pub snapshot: &'app Arc<Snapshot>,
    /// A snapshot came in. Before that nothing is read, which is no failure.
    pub probed: bool,
    pub changes: &'app ChangeSet,
    pub revision: u64,
    pub catalogs: &'app Catalogs,
    pub refusals: &'app Refusals,
    pub dkms: &'app DkmsReading,
    /// The Console shows, so a page need not offer to show it.
    pub console_open: bool,
    /// A plan runs, so buttons that would start one wait.
    pub busy: bool,
    pub units: &'app Units,
    pub home: HomeFacts<'app>,
    /// Each page asks [`PageContext::service`] for its own.
    pub services: &'app Services,
    pub plans: Plans,
    /// The session's `XDG_CURRENT_DESKTOP`, for entries of one desktop.
    pub desktops: &'app [String],
    /// The version About shows. Tests show a fixed one, so a release
    /// changes no snapshot.
    pub version: &'static str,
    /// The icon theme's pictures for the catalogs' apps, once read.
    pub app_icons: Option<&'app AppIcons>,
    /// The Discover apps' pictures, looked up as rows show. `None` in tests.
    pub pictures: Option<&'app Pictures>,
    pub commands: &'app CommandSender,
    /// What the Settings page shows.
    pub preferences: SettingsView<'app>,
}

impl PageContext<'_> {
    /// The page's own service, such as `context.service::<Boot>()`.
    pub fn service<T>(&self) -> &T
    where
        Services: Provides<T>,
    {
        self.services.service()
    }
}

/// The last refused click of each page, until dismissed or a later click
/// on that page is queued.
#[derive(Debug, Default)]
pub struct Refusals {
    pub catalogs: HashMap<CatalogId, toggles_core::Refusal>,
    pub kernel: Option<KernelRefusal>,
    pub locale: Option<LocaleRefusal>,
    pub network: Option<NetworkRefusal>,
    pub packages: Option<PackagesRefusal>,
    pub users: Option<UsersRefusal>,
    pub configs: Option<ConfigsRefusal>,
    pub boot_repair: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelRefusal {
    pub refusal: kernels_core::Refusal,
    pub kernel: PackageName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleRefusal {
    pub code: &'static str,
    pub locale: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plans {
    pub finished: u64,
    pub last_succeeded: bool,
    pub last_ending: Ending,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigsRefusal {
    pub code: &'static str,
    pub subject: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsersRefusal {
    pub code: &'static str,
    pub subject: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagesRefusal {
    pub refusal: toolbox_core::features::packages::queue::Refusal,
    pub package: PackageName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkRefusal {
    pub code: &'static str,
    pub driver: String,
}

/// A page as the shell draws it: the header stays at the top while the
/// body scrolls below it.
pub trait PageView {
    /// A page without a header starts its body at the top.
    fn header(&mut self, _ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>);

    /// Another page opened. A question still open here would greet the
    /// user on return with a confirm for what they left behind.
    fn left(&mut self, _ctx: &egui::Context) {}

    /// What stays below the scrolling body, such as a wizard's Back and Next.
    fn footer(&mut self, _ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        false
    }

    /// When the queued changes show. A page with its own way to apply
    /// hides them, as their Apply would be a second one.
    fn queue_shown(&self) -> QueueShown {
        QueueShown::Always
    }

    /// Which view of the page shows, such as its tab or a wizard's step.
    /// Another one starts at the top of the page.
    fn place(&self) -> u64 {
        0
    }

    fn shape(&self) -> BodyShape {
        BodyShape::Column
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueShown {
    Always,
    Never,
    /// Only while their review is open, which the page itself asks for.
    WhileReviewing,
}

impl QueueShown {
    pub const fn shows(self, reviewing: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::WhileReviewing => reviewing,
        }
    }
}

#[derive(Debug, Default)]
pub struct Pages {
    about: AboutPage,
    boot: BootPage,
    components: ComponentsPage,
    diagnostics: DiagnosticsPage,
    discover: DiscoverPage,
    gaming: GamingPage,
    guide: GuidePage,
    home: HomePage,
    kernels: KernelsPage,
    locales: LocalesPage,
    network: NetworkPage,
    packages: PackagesPage,
    users: UsersPage,
    configs: ConfigsPage,
    boot_repair: BootRepairPage,
    setup: SetupPage,
    maintenance: MaintenancePage,
    services: ServicesPage,
    settings: SettingsPage,
    tweaks: TweaksPage,
}

impl Pages {
    pub fn view(&mut self, page: PageId) -> &mut dyn PageView {
        match page {
            PageId::Components => &mut self.components,
            PageId::Diagnostics => &mut self.diagnostics,
            PageId::Discover => &mut self.discover,
            PageId::Gaming => &mut self.gaming,
            PageId::Home => &mut self.home,
            PageId::Kernels => &mut self.kernels,
            PageId::Locales => &mut self.locales,
            PageId::Maintenance => &mut self.maintenance,
            PageId::Network => &mut self.network,
            PageId::Packages => &mut self.packages,
            PageId::Users => &mut self.users,
            PageId::Configs => &mut self.configs,
            PageId::BootRepair => &mut self.boot_repair,
            PageId::Setup => &mut self.setup,
            PageId::SystemSettings => &mut self.tweaks,
            PageId::Services => &mut self.services,
            PageId::Settings => &mut self.settings,
            PageId::Guide => &mut self.guide,
            PageId::About => &mut self.about,
            PageId::Boot => &mut self.boot,
        }
    }

    pub const fn open_services_on(&mut self, scope: UnitScope) {
        self.services.show_scope(scope);
    }

    /// Opens Discover's search on `title`, for an app picked in the palette.
    pub fn reveal_app(&mut self, title: &str) {
        self.discover.reveal(title);
    }

    pub fn show_guide(&mut self, file: &str, anchor: Option<&str>) {
        self.guide.show(file, anchor);
    }

    /// Opens Maintenance on `tab`, for an entry picked in the palette.
    pub const fn show_maintenance_tab(&mut self, tab: maintenance::Tab) {
        self.maintenance.show_tab(tab);
    }
}

/// A [`PageView::place`] from any value that tells views apart.
pub fn place_of(view: &impl std::hash::Hash) -> u64 {
    use std::hash::{BuildHasher as _, BuildHasherDefault, DefaultHasher};
    BuildHasherDefault::<DefaultHasher>::default().hash_one(view)
}

/// The one banner of a page whose rows wait while a plan runs, so the
/// faded rows and their resting switches have a reason on screen.
pub fn plan_running(ui: &mut Ui, context: &PageContext<'_>) {
    if !context.busy {
        return;
    }

    let text = crate::i18n::text(ui.ctx(), "common-plan-running");
    banner(ui, Tone::Neutral, &text, &[]);
    ui.add_space(ui.tokens().sizes.card_gap);
}

pub fn heading(ui: &mut Ui, icon: toolbox_gui_kit::icons::Icon, id: &str) {
    let title = crate::i18n::text(ui.ctx(), id);
    toolbox_gui_kit::widgets::SectionHeading::new(icon, &title, "").show(ui);
    ui.add_space(ui.tokens().sizes.heading_gap);
}
