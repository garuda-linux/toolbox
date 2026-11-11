//! The ports the app gets at start: the real ones, or the stand-ins of a
//! dry run where a port would change the machine or leave it.

mod samples;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_aur::Cache;
use toolbox_catalog::{ConfigFileCatalog, LoadProblem, SKEL_ROOT};
use toolbox_config::Paths;
use toolbox_core::ChangeSet;
use toolbox_gui::apply::{self, ApplyHandle, HelperLauncher, Launcher as _};
use toolbox_gui::boot::BootSetup;
use toolbox_gui::boot::portal::PortalChooser;
use toolbox_gui::bootrepair::BootRepairSetup;
use toolbox_gui::catalogs::{DEVELOPMENT, catalog_dir};
use toolbox_gui::command::CommandSender;
use toolbox_gui::configs::ConfigsSetup;
use toolbox_gui::diagnostics::{DiagnosticsLink, DryRunTransport};
use toolbox_gui::launch::{DryRunLauncher, Launcher, SystemLauncher};
use toolbox_gui::maintenance::MaintenanceSetup;
use toolbox_gui::network::{DryRunActions, DryRunAddress, NetworkSetup};
use toolbox_gui::packages::PackagesSetup;
use toolbox_gui::pictures::{FALLBACK_THEME, Pictures, SystemPictures};
use toolbox_gui::power::{DryRunPower, Logind, Power};
use toolbox_gui::queue::{Queue, Restored};
use toolbox_gui::reader::ReaderHandle;
use toolbox_gui::share::ShareLink;
use toolbox_gui::users::UsersSetup;
use toolbox_news::{Feeds, News, NewsSource};
use toolbox_plan::Toggle;
use toolbox_plan::protocol::Mode;
use toolbox_privatebin::{Https, Transport};
use toolbox_services::address::{AddressSource, KdeGeoIp};
use toolbox_system::mirrors::RateMirrors;
use toolbox_system::{
    ConfigFileSource, DiagnosticSource as _, DryRunEditor, EditFolder, Editing, HelperLocks,
    Libalpm, NetworkActions, NetworkSource, PacmanLog, SystemActions, SystemBootMenu,
    SystemConfigFiles, SystemDiagnostics, SystemEditor, SystemNetwork, SystemRepair,
};

const SELF_TEST_CONFIGS: &str = "GARUDA_TOOLBOX_SELFTEST_CONFIGS";

pub struct HelperServices {
    pub apply: ApplyHandle,
    pub diagnostics: DiagnosticsLink,
    pub output_share: ShareLink,
    pub boot: BootSetup,
    pub users: UsersSetup,
    pub configs: ConfigsSetup,
    pub dry_run: bool,
}

/// What restarts the machine and starts the installer. A dry run touches
/// neither, so its self-test can press every button. A started program
/// that fails tells of it through `commands`.
pub fn machine(
    handle: &Handle,
    mode: Mode,
    commands: CommandSender,
    ctx: &egui::Context,
) -> (Arc<dyn Power>, Arc<dyn Launcher>) {
    let dry_run = mode.is_dry_run();
    if dry_run {
        return (Arc::new(DryRunPower), Arc::new(DryRunLauncher));
    }

    (
        Arc::new(Logind::new(handle.clone())),
        Arc::new(SystemLauncher::new(handle.clone(), commands, ctx.clone())),
    )
}

/// A self-test build's dry run ranks a fixed list, so its self-test asks
/// no mirror. Ranking writes nothing, so a release's dry run ranks too.
pub fn maintenance(handle: &Handle, mode: Mode) -> MaintenanceSetup {
    let dry_run = mode.is_dry_run();
    let mirrors = dry_run
        .then(samples::mirrors)
        .flatten()
        .unwrap_or_else(|| Arc::new(RateMirrors::system()));

    MaintenanceSetup {
        runtime: handle.clone(),
        mirrors,
        home: Paths::from_process().home().to_path_buf(),
        skel_root: PathBuf::from(SKEL_ROOT),
    }
}

pub fn helper_services(handle: &Handle, ctx: &egui::Context) -> HelperServices {
    let launcher = Arc::new(HelperLauncher::from_process());
    let dry_run = launcher.mode() == Mode::DryRun;
    let apply_launcher: Arc<dyn apply::Launcher> = Arc::<HelperLauncher>::clone(&launcher);
    let wake = toolbox_gui::waking::repaint(ctx.clone());
    let apply = apply::spawn(handle, wake.clone(), apply_launcher);

    // The collectors read the real system in a dry run too: they change
    // nothing. Only the upload goes nowhere then.
    let transport: Arc<dyn Transport> = if dry_run {
        Arc::new(DryRunTransport)
    } else {
        Arc::new(Https::new())
    };

    let source = Arc::new(SystemDiagnostics::system(Arc::<HelperLauncher>::clone(
        &launcher,
    )));

    // A few short files, read once before the first frame.
    let known = source.known();
    let output_share =
        toolbox_gui::share::spawn(handle, wake.clone(), Arc::clone(&transport), known);

    let diagnostics = toolbox_gui::diagnostics::spawn(handle, wake, source, transport);

    let users = UsersSetup {
        runtime: handle.clone(),
        locks: Arc::new(HelperLocks(Arc::<HelperLauncher>::clone(&launcher))),
    };

    let configs = configs(handle, Arc::<HelperLauncher>::clone(&launcher), dry_run);

    let boot = BootSetup {
        runtime: handle.clone(),
        menu: Arc::new(SystemBootMenu::system(launcher)),
        chooser: Arc::new(PortalChooser::new(handle.clone())),
    };

    HelperServices {
        apply,
        diagnostics,
        output_share,
        boot,
        users,
        configs,
        dry_run,
    }
}

/// The page reads the real machine in a dry run too. Only the radio
/// switches and the address request are stand-ins then.
pub fn network(handle: &Handle, mode: Mode) -> NetworkSetup {
    let dry_run = mode.is_dry_run();
    let source: Arc<dyn NetworkSource> = Arc::new(SystemNetwork::system());
    let (actions, address): (Arc<dyn NetworkActions>, Arc<dyn AddressSource>) = if dry_run {
        (Arc::new(DryRunActions), Arc::new(DryRunAddress))
    } else {
        (
            Arc::new(SystemActions::new(Arc::clone(&source))),
            Arc::new(KdeGeoIp::new()),
        )
    };

    NetworkSetup {
        runtime: handle.clone(),
        source,
        actions,
        address,
    }
}

/// The look may never change, as without a session bus, so the pictures
/// start from the fallback theme.
pub fn pictures(ctx: &egui::Context) -> Pictures {
    let pictures = Pictures::spawn(ctx.clone(), SystemPictures::system());
    pictures.set_theme(FALLBACK_THEME);
    pictures
}

/// The Config Files page reads the real system, or the self-test's own
/// catalog and files when `GARUDA_TOOLBOX_SELFTEST_CONFIGS` names them.
/// Its copies live in the user's runtime folder, and `$VISUAL` and
/// `$EDITOR` only pick one of the editors the app knows. A dry run starts
/// no editor and opens no app.
fn configs(handle: &Handle, launcher: Arc<HelperLauncher>, dry_run: bool) -> ConfigsSetup {
    let variable = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
    let (source, dir): (Arc<dyn ConfigFileSource>, PathBuf) =
        match self_test_configs(|key| std::env::var_os(key), DEVELOPMENT) {
            Some(dir) => {
                tracing::info!("the config files of the self-test in {}", dir.display());
                let files = SystemConfigFiles::new(dir.join("root"), dir.join("home"), launcher);
                (Arc::new(files), dir)
            }
            None => (
                Arc::new(SystemConfigFiles::system(launcher)),
                catalog_dir(|key| std::env::var_os(key)),
            ),
        };

    let editing: Arc<dyn Editing> = if dry_run {
        Arc::new(DryRunEditor)
    } else {
        Arc::new(SystemEditor)
    };

    ConfigsSetup {
        runtime: handle.clone(),
        source,
        catalog: ConfigFileCatalog::load(&dir)
            .inspect_err(|error| tracing::warn!("the config files are unavailable: {error}"))
            .map_err(|error| LoadProblem::from(&error)),
        copies: EditFolder::from_env(
            |key| std::env::var_os(key),
            &Paths::from_process().cache_dir(),
        ),
        editing,
        visual: variable("VISUAL"),
        editor: variable("EDITOR"),
        dry_run,
    }
}

/// The self-test's folder of config files. A release never reads it, so
/// no variable points the page at other files than the system's.
fn self_test_configs(
    lookup: impl Fn(&str) -> Option<OsString>,
    development: bool,
) -> Option<PathBuf> {
    lookup(SELF_TEST_CONFIGS)
        .filter(|_| development)
        .map(PathBuf::from)
}

/// The disks of the machine, or a sample machine in a self-test build's
/// dry run, whose page must neither depend on the disks it runs on nor
/// show them. Reading them writes nothing, so a release's dry run reads
/// them too.
pub fn repair(handle: &Handle, mode: Mode, test: Toggle) -> BootRepairSetup {
    let forced = test.is_on();
    let dry_run = mode.is_dry_run();
    let source = dry_run
        .then(samples::repair)
        .flatten()
        .unwrap_or_else(|| Arc::new(SystemRepair::system()));

    BootRepairSetup {
        runtime: handle.clone(),
        source,
        forced,
    }
}

/// The page reads libalpm and pacman's log, which change nothing, so a
/// dry run reads the real machine too. The AUR is a download, which a
/// self-test build's dry run answers with a sample.
pub fn packages(handle: &Handle, mode: Mode) -> PackagesSetup {
    let dry_run = mode.is_dry_run();
    let aur = dry_run
        .then(samples::aur)
        .flatten()
        .unwrap_or_else(|| Arc::new(Cache::system(Paths::from_process().aur_dir())));

    PackagesSetup {
        runtime: handle.clone(),
        source: Arc::new(Libalpm::system()),
        log: Arc::new(PacmanLog::system()),
        aur,
    }
}

/// Garuda's and Arch's news. They are a download, which a self-test
/// build's dry run answers with a sample.
fn news(dry_run: bool) -> Arc<dyn NewsSource> {
    dry_run
        .then(samples::news)
        .flatten()
        .unwrap_or_else(|| Arc::new(Feeds::system(&Paths::from_process().news_dir())))
}

/// The news, read once at start and on Read again. The cache decides
/// whether that asks the sites.
pub fn news_reader(handle: &Handle, ctx: &egui::Context, mode: Mode) -> ReaderHandle<News> {
    let dry_run = mode.is_dry_run();
    let source = news(dry_run);
    toolbox_gui::reader::spawn_fallible(
        handle,
        toolbox_gui::waking::repaint(ctx.clone()),
        "the news",
        move || source.news(),
    )
}

/// The queue restored from `file`, written there after every change. A
/// failed write wakes the window, which tells of it.
pub fn kept_queue(
    (changes, restored): (ChangeSet, Restored),
    file: PathBuf,
    handle: &Handle,
    ctx: &egui::Context,
) -> Queue {
    Queue::with_file(
        changes,
        file,
        handle,
        toolbox_gui::waking::repaint(ctx.clone()),
        restored,
    )
}
