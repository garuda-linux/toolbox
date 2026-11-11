//! The names tab completion offers and "did you mean" picks from, read
//! from the live system.
//!
//! [`of`] is the [`toolbox_cli::complete::Source`] the binary hands to
//! the completion. It reads only what the user may read: no helper, no
//! root, no password dialog. A part that cannot be read gives no names.
//! The functions below it pick the names out of what was read.

#[cfg(test)]
mod tests;

mod aur;
mod cache;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use toolbox_catalog::{
    ColourSchemeCatalog, ConfigFileCatalog, SKEL_ROOT, SchemeMode, ToggleCatalog,
};
use toolbox_cli::args::{BootKey, SettingKey};
use toolbox_cli::complete::Names;
use toolbox_core::domains::AvailableShells;
use toolbox_core::features::boot::shown;
use toolbox_core::features::boot::view::entries;
use toolbox_core::features::discover::Apps;
use toolbox_core::features::maintenance::page::{AUR_UPGRADE, OPERATION_KEYS, repairs};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{ConfigFilePath, GRUB_CONFIG, GrubMenu, GrubPlatform, PackageName, UnitScope};
use toolbox_services::catalogs::{Catalogs, catalog_dir};
use toolbox_services::configs::specs;
use toolbox_services::page::PageId;
use toolbox_services::units::{Systemd, UnitLister as _};
use toolbox_system::{
    AccountSource as _, BootSource as _, ConfigFileSource as _, EtcAccounts, EtcLocales,
    HelperFetch, Libalpm, LinuxPackageSource as _, LocaleSource as _, ModuleSource as _,
    PackageSource as _, ProbeError, SyncSource as _, SystemBoot, SystemConfigFiles, SystemModules,
};

use super::locales::{PackRow, pack_rows};
use super::{guide, lists};

/// A completion that waits longer than this for systemd gives no names.
const SYSTEMD_WAIT: Duration = Duration::from_secs(2);

/// The config files' listing never reads a file through it.
struct NoHelper;

impl HelperFetch for NoHelper {
    fn fetch_command(&self, _path: &ConfigFilePath) -> Command {
        Command::new("/usr/bin/false")
    }
}

/// The names of a kind as the system has them now. A list from the last
/// ten seconds stands in, since a shell may ask once per Tab. The AUR's
/// list stands in until its copy changes.
pub fn of(names: Names) -> Vec<String> {
    if names == Names::AurPackages {
        return aur::of();
    }

    let now = cache::now();
    if let Some(found) = cache::lookup(names, now) {
        return found;
    }

    let found = read(names);
    cache::store(names, now, &found);
    found
}

fn read(names: Names) -> Vec<String> {
    match names {
        Names::Services { user } => services(UnitScope::from_user(user)),
        Names::Switches => switch_ids(&toggle_catalogs(&catalogs())),
        Names::Apps => app_ids(&catalogs().apps.apps),
        Names::Kernels => kernel_names(&with(Libalpm::system().linux_packages())),
        Names::Locales => locale_names(&with(EtcLocales::system().supported())),
        Names::Users => user_names(&with(EtcAccounts::system().accounts())),
        Names::Groups => group_names(&with(EtcAccounts::system().accounts())),
        Names::ConfigFiles => config_keys(),
        Names::Tasks => task_ids(),
        Names::BootEntries => std::fs::read_to_string(Path::new("/").join(GRUB_CONFIG))
            .map(|config| entry_ids(&config, toolbox_system::grub_platform(Path::new("/"))))
            .unwrap_or_default(),
        Names::Drivers => driver_names(&with(SystemModules::system().facts())),
        Names::Pages => page_names(),
        Names::AppCategories => app_categories(),
        Names::Shells => EtcAccounts::system()
            .available_shells()
            .map(|shells| shell_names(&shells))
            .unwrap_or_default(),
        Names::OnOff => ["on", "off"].map(str::to_owned).to_vec(),
        Names::LogLevels => ["error", "warn", "info", "debug", "trace"]
            .map(str::to_owned)
            .to_vec(),
        Names::BootKeys => keys::<BootKey>(),
        Names::SettingKeys => keys::<SettingKey>(),
        Names::ColourSchemes { dark } => scheme_names(&catalogs().colour_schemes, dark),
        Names::BootValues | Names::SettingValues => Vec::new(),
        Names::OfferedPackages => Libalpm::system()
            .offered()
            .map(|offered| package_names(&offered.0))
            .unwrap_or_default(),
        Names::InstalledPackages => Libalpm::system()
            .installed()
            .map(|installed| package_names(&installed.0))
            .unwrap_or_default(),
        Names::AurPackages => aur::of(),
        Names::LanguagePacks { installed } => {
            pack_names(&pack_rows(&packs()).unwrap_or_default(), installed)
        }
        Names::GuidePages => guide_pages(),
        Names::SetupChoices => catalogs()
            .setup
            .catalog
            .as_ref()
            .map(|setup| switch_ids(&[setup]))
            .unwrap_or_default(),
        Names::KernelParameters => parameter_names(&with(SystemBoot::system().grub())),
        Names::ResetEntries => catalogs()
            .skel
            .map(|catalog| super::reset::ids(&catalog, Path::new(SKEL_ROOT)))
            .unwrap_or_default(),
    }
}

pub fn package_names(packages: &BTreeSet<PackageName>) -> Vec<String> {
    packages
        .iter()
        .map(|package| package.as_str().to_owned())
        .collect()
}

pub fn pack_names(rows: &[PackRow], installed: bool) -> Vec<String> {
    rows.iter()
        .filter(|row| row.installed == installed)
        .map(|row| row.package.clone())
        .collect()
}

pub fn guide_pages() -> Vec<String> {
    toolbox_guide::embedded()
        .pages()
        .iter()
        .map(|page| guide::name_of(page).to_owned())
        .collect()
}

pub fn parameter_names(snapshot: &Snapshot) -> Vec<String> {
    shown(snapshot, &ChangeSet::default())
        .and_then(|now| now.parameters)
        .map(|parameters| parameters.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

/// What the language packs are found from: the databases and the
/// locales.
fn packs() -> Snapshot {
    let alpm = Libalpm::system();
    let locales = EtcLocales::system();
    let mut snapshot = Snapshot::default();

    if let Ok(offered) = alpm.offered() {
        snapshot.insert(offered);
    }

    if let Ok(installed) = alpm.installed() {
        snapshot.insert(installed);
    }

    if let Ok(supported) = locales.supported() {
        snapshot.insert(supported);
    }

    if let Ok(enabled) = locales.enabled() {
        snapshot.insert(enabled);
    }

    snapshot
}

pub fn switch_ids(catalogs: &[&ToggleCatalog]) -> Vec<String> {
    catalogs
        .iter()
        .flat_map(|catalog| catalog.entries().map(|entry| entry.id().to_owned()))
        .collect()
}

fn app_ids(apps: &Apps) -> Vec<String> {
    apps.all().map(|hit| hit.entry.id().to_owned()).collect()
}

/// The App Catalog's categories, the file name of each catalog.
pub fn app_categories() -> Vec<String> {
    toolbox_catalog::APPS
        .iter()
        .filter_map(|path| path.rsplit('/').next()?.strip_suffix(".toml"))
        .map(str::to_owned)
        .collect()
}

/// The shells `/etc/shells` allows, as a full path.
pub fn shell_names(shells: &AvailableShells) -> Vec<String> {
    shells
        .0
        .iter()
        .map(|shell| shell.as_str().to_owned())
        .collect()
}

fn keys<T>() -> Vec<String>
where
    T: clap::ValueEnum,
{
    T::value_variants()
        .iter()
        .filter_map(clap::ValueEnum::to_possible_value)
        .map(|value| value.get_name().to_owned())
        .collect()
}

pub fn kernel_names(snapshot: &Snapshot) -> Vec<String> {
    lists::kernels(snapshot)
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.name)
        .collect()
}

/// The locales as the locales commands take them, `de_DE.UTF-8`.
pub fn locale_names(snapshot: &Snapshot) -> Vec<String> {
    toolbox_core::features::locales::all_rows(snapshot, &ChangeSet::default())
        .into_iter()
        .map(|row| row.entry.locale.to_string())
        .collect()
}

pub fn user_names(snapshot: &Snapshot) -> Vec<String> {
    let rows = lists::users(snapshot).unwrap_or_default();

    lists::people(rows, false)
        .0
        .into_iter()
        .map(|row| row.name)
        .collect()
}

pub fn group_names(snapshot: &Snapshot) -> Vec<String> {
    lists::groups(snapshot)
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.name)
        .collect()
}

/// Every task and repair `maintenance run` takes, whether or not its
/// program is installed.
pub fn task_ids() -> Vec<String> {
    OPERATION_KEYS
        .into_iter()
        .chain([AUR_UPGRADE, crate::mirrors::RANK_MIRRORS])
        .map(str::to_owned)
        .chain(repairs(|_| true).into_iter().map(|row| row.repair))
        .collect()
}

pub fn entry_ids(config: &str, platform: GrubPlatform) -> Vec<String> {
    entries(Some(&GrubMenu::for_platform(config, platform)))
        .into_iter()
        .map(|row| row.entry)
        .collect()
}

fn driver_names(snapshot: &Snapshot) -> Vec<String> {
    snapshot
        .part()
        .map(|facts| toolbox_core::features::network::driver_rows(facts, &ChangeSet::default()))
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.name.as_str().to_owned())
        .collect()
}

pub fn page_names() -> Vec<String> {
    PageId::ALL
        .iter()
        .map(|page| page.slug().to_owned())
        .collect()
}

fn toggle_catalogs(catalogs: &Catalogs) -> Vec<&ToggleCatalog> {
    [
        &catalogs.components,
        &catalogs.settings,
        &catalogs.gaming,
        &catalogs.setup,
    ]
    .into_iter()
    .filter_map(|toggles| toggles.catalog.as_ref().ok())
    .collect()
}

/// A snapshot of the one part that was read, else an empty one.
fn with<T>(found: Result<T, ProbeError>) -> Snapshot
where
    T: toolbox_state::Part,
{
    let mut snapshot = Snapshot::default();
    if let Ok(part) = found {
        snapshot.insert(part);
    }

    snapshot
}

fn catalogs() -> Catalogs {
    Catalogs::load(&catalog_dir(|key| std::env::var_os(key)))
}

fn services(scope: UnitScope) -> Vec<String> {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return Vec::new();
    };

    let listed =
        runtime.block_on(async { tokio::time::timeout(SYSTEMD_WAIT, Systemd.list(scope)).await });

    match listed {
        Ok(Ok(units)) => lists::services(&units)
            .into_iter()
            .map(|row| row.name)
            .collect(),
        Ok(Err(_)) | Err(_) => Vec::new(),
    }
}

pub fn config_keys() -> Vec<String> {
    let Ok(catalog) = ConfigFileCatalog::load(&catalog_dir(|key| std::env::var_os(key))) else {
        return Vec::new();
    };

    let listing = SystemConfigFiles::system(NoHelper).listing(&specs(&catalog));
    super::configs::files(&catalog, &listing)
        .into_iter()
        .map(|row| row.key)
        .collect()
}

/// `system` and the colour schemes of dark or light mode, as
/// `settings set` takes them.
pub fn scheme_names(catalog: &ColourSchemeCatalog, dark: bool) -> Vec<String> {
    let mode = if dark {
        SchemeMode::Dark
    } else {
        SchemeMode::Light
    };

    std::iter::once("system".to_owned())
        .chain(catalog.of(mode).map(|scheme| scheme.id().to_string()))
        .collect()
}
