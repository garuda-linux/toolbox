use std::sync::Arc;

use fluent_templates::LanguageIdentifier;
use tokio::sync::watch;
use toolbox_core::ChangeSet;
use toolbox_news::News;
use toolbox_system::{Facts, Status};

use super::PreferencesSetup;
use crate::apply::ApplyHandle;
use crate::boot::BootSetup;
use crate::bootrepair::BootRepairSetup;
use crate::catalogs::Catalogs;
use crate::command::CommandBus;
use crate::configs::ConfigsSetup;
use crate::desktop::DesktopLook;
use crate::diagnostics::DiagnosticsLink;
use crate::i18n;
use crate::instance::Arrivals;
use crate::launch::Launcher;
use crate::maintenance::MaintenanceSetup;
use crate::network::NetworkSetup;
use crate::notifications::Notifier;
use crate::packages::PackagesSetup;
use crate::page::PageId;
use crate::pages::kernels::view::Dkms;
use crate::pictures::Pictures;
use crate::power::Power;
use crate::programs::Programs;
use crate::queue::Queue;
use crate::reader::ReaderHandle;
use crate::share::ShareLink;
use crate::snapshot::SnapshotHandle;
use crate::units::UnitsHandle;
use crate::users::UsersSetup;

/// The version tests show on About, so a release changes no snapshot.
pub const FIXED_VERSION: &str = "1.2.3";

/// Tests build it by hand, so they depend on neither the desktop nor the environment.
#[derive(Debug)]
pub struct Setup {
    /// The desktop's look, newest first. `None` keeps the fallback.
    pub look: Option<watch::Receiver<DesktopLook>>,
    pub system_language: LanguageIdentifier,
    /// The queued changes, read before the window opens.
    pub queue: Queue,
    /// Saved changes that could not be restored, named on Home until dismissed.
    pub queue_lost: usize,
    /// The apply service. `None` leaves Apply without effect.
    pub apply: Option<ApplyHandle>,
    /// Off in tests, which have no display.
    pub system_clipboard: bool,
    pub notifier: Option<Arc<dyn Notifier>>,
    /// Reads the system. `None` leaves the queue unchecked.
    pub snapshot: Option<SnapshotHandle>,
    /// Asks dkms for its modules. `None` leaves them unknown.
    pub dkms: Option<ReaderHandle<Dkms>>,
    /// Checks updates, files to merge, the last update and garuda-health
    /// for Home. `None` leaves Home without a status.
    pub status: Option<ReaderHandle<Status>>,
    /// Reads the packages, the kernel, the disk and the checks besides the
    /// updates for Home. `None` leaves them unknown.
    pub facts: Option<ReaderHandle<Facts>>,
    /// Reads the forum's announcements for Home. `None` shows none.
    pub news: Option<ReaderHandle<News>>,
    pub programs: Option<ReaderHandle<Programs>>,
    /// `None` leaves Restart without effect.
    pub power: Option<Arc<dyn Power>>,
    pub launcher: Option<Arc<dyn Launcher>>,
    pub units: Option<UnitsHandle>,
    /// `None` shows a glyph for each.
    pub pictures: Option<Pictures>,
    pub catalogs: Catalogs,
    /// The session's `XDG_CURRENT_DESKTOP`, split at its colons.
    pub desktops: Vec<String>,
    /// The version About shows. The default is fixed for tests, so a
    /// release changes no snapshot.
    pub version: &'static str,
    /// The helper runs in dry-run mode: nothing is changed, the status bar
    /// says so.
    pub dry_run: bool,
    /// The Boot options page's menu reads and file dialog. `None` does
    /// neither.
    pub boot: Option<BootSetup>,
    /// The Network page's reads and acts. `None` reads nothing.
    pub network: Option<NetworkSetup>,
    /// The Packages page's reads. `None` reads nothing.
    pub packages: Option<PackagesSetup>,
    pub users: Option<UsersSetup>,
    pub configs: Option<ConfigsSetup>,
    pub boot_repair: Option<BootRepairSetup>,
    /// The page a start option opens, such as `--setup-assistant`.
    pub open: Option<PageId>,
    /// The Diagnostics page's service. `None` collects nothing.
    pub diagnostics: Option<DiagnosticsLink>,
    /// Uploads the Console's output. `None` shares nothing.
    pub output_share: Option<ShareLink>,
    /// The command bus, made before the window when a later start may
    /// post to it. `None` makes a new one.
    pub commands: Option<CommandBus>,
    /// Where later starts arrive. `None` when this start runs alone.
    pub arrivals: Option<Arc<Arrivals>>,
    /// The Maintenance page's ranking and reset. `None` does neither.
    pub maintenance: Option<MaintenanceSetup>,
    pub preferences: PreferencesSetup,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            look: None,
            system_language: i18n::FALLBACK,
            queue: Queue::in_memory(ChangeSet::default()),
            queue_lost: 0,
            apply: None,
            system_clipboard: false,
            notifier: None,
            snapshot: None,
            dkms: None,
            status: None,
            facts: None,
            news: None,
            programs: None,
            power: None,
            launcher: None,
            units: None,
            pictures: None,
            catalogs: Catalogs::default(),
            desktops: Vec::new(),
            version: FIXED_VERSION,
            dry_run: false,
            maintenance: None,
            diagnostics: None,
            output_share: None,
            commands: None,
            arrivals: None,
            boot: None,
            network: None,
            packages: None,
            users: None,
            configs: None,
            boot_repair: None,
            open: None,
            preferences: PreferencesSetup::default(),
        }
    }
}
