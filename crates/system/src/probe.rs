//! Fills a [`Snapshot`] from every source at once. A failing probe leaves
//! its own part out and is reported, the others still deliver, so a
//! system without pacman still shows its services.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use toolbox_plan::UserConfigEdit;
use toolbox_state::Snapshot;
use toolbox_state::accounts::CurrentUser;
use toolbox_state::boot::{BootLoader, SdBootKernel};
use toolbox_state::packages::ProtectedPackages;
pub use toolbox_state::probes::{FailedProbes, Probe};

use crate::accounts::{AccountSource, EtcAccounts};
use crate::boot::{BootSource, SystemBoot};
use crate::error::ProbeError;
use crate::host::{Host, HostSource, SystemHost};
use crate::kernels::{KernelSource, SystemKernels};
use crate::locales::{EtcLocales, LocaleSource};
use crate::network::{ModuleSource, SystemModules};
use crate::packages::{Libalpm, LinuxPackageSource, PackageSource, SyncSource};
use crate::settings::{EtcSettings, SettingsSource};
use crate::units::{Systemctl, UnitSource};
use crate::user_configs::{HomeConfigs, UserConfigSource};

/// The ports one snapshot is probed from. Shared pointers, so a clone can
/// move into `spawn_blocking`.
#[derive(Clone)]
pub struct Sources {
    pub packages: Arc<dyn PackageSource>,
    pub sync: Arc<dyn SyncSource>,
    pub linux: Arc<dyn LinuxPackageSource>,
    pub kernels: Arc<dyn KernelSource>,
    pub locales: Arc<dyn LocaleSource>,
    pub units: Arc<dyn UnitSource>,
    pub accounts: Arc<dyn AccountSource>,
    pub host: Arc<dyn HostSource>,
    pub settings: Arc<dyn SettingsSource>,
    pub boot: Arc<dyn BootSource>,
    pub user_configs: Arc<dyn UserConfigSource>,
    /// The kernel's modules and every modprobe blacklist, for the Network
    /// page's drivers and the guard of a restored blacklist change.
    pub modules: Arc<dyn ModuleSource>,
    /// The edits the catalogs offer, so the probe reads only their files.
    pub config_edits: Arc<[UserConfigEdit]>,
    /// The packages the protected catalog keeps, for every removal guard.
    pub protected: ProtectedPackages,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeIssue {
    pub probe: Probe,
    pub error: ProbeError,
}

impl Sources {
    #[must_use = "the sources do nothing until probed"]
    pub fn system() -> Self {
        // One adapter for all three, so they share its read of the sync
        // databases.
        let alpm = Arc::new(Libalpm::system());

        Self {
            packages: Arc::<Libalpm>::clone(&alpm),
            sync: Arc::<Libalpm>::clone(&alpm),
            linux: alpm,
            kernels: Arc::new(SystemKernels::system()),
            locales: Arc::new(EtcLocales::system()),
            units: Arc::new(Systemctl::new()),
            accounts: Arc::new(EtcAccounts::system()),
            host: Arc::new(SystemHost::from_process()),
            settings: Arc::new(EtcSettings::system()),
            boot: Arc::new(SystemBoot::system()),
            user_configs: Arc::new(HomeConfigs::from_env()),
            modules: Arc::new(SystemModules::system()),
            config_edits: Arc::from([]),
            protected: ProtectedPackages::default(),
        }
    }

    #[must_use = "builder methods return the changed sources"]
    pub fn with_config_edits(mut self, edits: Vec<UserConfigEdit>) -> Self {
        self.config_edits = Arc::from(edits);
        self
    }

    #[must_use = "builder methods return the changed sources"]
    pub fn with_protected(mut self, protected: ProtectedPackages) -> Self {
        self.protected = protected;
        self
    }
}

/// A system that lacks the thing is no failure.
pub fn failed_probes(issues: &[ProbeIssue]) -> FailedProbes {
    FailedProbes::new(
        issues
            .iter()
            .filter(|issue| matches!(issue.error, ProbeError::Failed(_)))
            .map(|issue| issue.probe),
    )
}

/// Parts of failed probes stay missing from the snapshot. [`Host`] is
/// always there, with defaults for the facts that failed. Without a user
/// the part [`CurrentUser`] is missing, the `User` issue explains the gap.
pub fn probe_snapshot(sources: &Sources) -> (Snapshot, Vec<ProbeIssue>) {
    let mut snapshot = Snapshot::default();
    let mut issues = Vec::new();
    let host = probe_host(sources.host.as_ref(), &mut issues);

    insert(
        &mut snapshot,
        &mut issues,
        Probe::Packages,
        sources.packages.installed(),
    );
    insert(
        &mut snapshot,
        &mut issues,
        Probe::SyncPackages,
        sources.sync.offered(),
    );
    insert(
        &mut snapshot,
        &mut issues,
        Probe::Units,
        sources.units.enabled_units(),
    );
    insert(
        &mut snapshot,
        &mut issues,
        Probe::UserUnits,
        sources.units.enabled_user_units(),
    );
    insert(
        &mut snapshot,
        &mut issues,
        Probe::MaskedUnits,
        sources.units.masked_units(),
    );
    insert(
        &mut snapshot,
        &mut issues,
        Probe::MaskedUserUnits,
        sources.units.masked_user_units(),
    );

    insert(
        &mut snapshot,
        &mut issues,
        Probe::Accounts,
        sources.accounts.accounts(),
    );

    if let Some(user) = &host.user {
        snapshot.insert(CurrentUser(user.clone()));
    }

    probe_settings(sources, &mut snapshot, &mut issues);
    probe_kernels_and_locales(sources, &mut snapshot, &mut issues);
    probe_boot(sources.boot.as_ref(), &mut snapshot, &mut issues);
    snapshot.insert(host);
    snapshot.insert(sources.protected.clone());
    snapshot.insert(failed_probes(&issues));

    (snapshot, issues)
}

fn probe_settings(sources: &Sources, snapshot: &mut Snapshot, issues: &mut Vec<ProbeIssue>) {
    let accounts = sources.accounts.as_ref();
    let settings = sources.settings.as_ref();

    insert(
        snapshot,
        issues,
        Probe::LoginShells,
        accounts.login_shells(),
    );
    insert(
        snapshot,
        issues,
        Probe::AvailableShells,
        accounts.available_shells(),
    );
    insert(snapshot, issues, Probe::Dns, settings.dns());
    insert(snapshot, issues, Probe::Hblock, settings.hblock());
    insert(
        snapshot,
        issues,
        Probe::WifiBackend,
        settings.wifi_backend(),
    );
    insert(
        snapshot,
        issues,
        Probe::GarudaUpdate,
        settings.garuda_update(),
    );
    insert(
        snapshot,
        issues,
        Probe::ModuleBlacklist,
        settings.module_blacklist(),
    );
    insert(
        snapshot,
        issues,
        Probe::UserConfigs,
        sources.user_configs.states(&sources.config_edits),
    );

    // The page says itself when the drivers could not be read, so a
    // failure is no issue of its own: the part is left out.
    match sources.modules.facts() {
        Ok(facts) => snapshot.insert(facts),
        Err(error) => tracing::debug!("no kernel modules in the snapshot: {error}"),
    }
}

/// `dkms status` is a program that takes its time, so the Kernels page
/// asks it on its own.
fn probe_kernels_and_locales(
    sources: &Sources,
    snapshot: &mut Snapshot,
    issues: &mut Vec<ProbeIssue>,
) {
    let kernels = sources.kernels.as_ref();
    let locales = sources.locales.as_ref();

    insert(
        snapshot,
        issues,
        Probe::LinuxPackages,
        sources.linux.linux_packages(),
    );
    insert(snapshot, issues, Probe::RunningKernel, kernels.running());
    insert(snapshot, issues, Probe::KernelReleases, kernels.releases());
    insert(
        snapshot,
        issues,
        Probe::SupportedLocales,
        locales.supported(),
    );
    insert(snapshot, issues, Probe::EnabledLocales, locales.enabled());
    snapshot.insert(locales.names());
    insert(
        snapshot,
        issues,
        Probe::SystemLocale,
        locales.system_locale(),
    );
}

fn probe_host(source: &dyn HostSource, issues: &mut Vec<ProbeIssue>) -> Host {
    Host {
        user: take(issues, Probe::User, source.user()),
        live_iso: take(issues, Probe::LiveIso, source.is_live_iso()).unwrap_or_default(),
        desktops: take(issues, Probe::Desktops, source.desktops()).unwrap_or_default(),
        edition: take(issues, Probe::Edition, source.edition()).flatten(),
        reboot_pending: take(issues, Probe::RebootPending, source.is_reboot_pending())
            .unwrap_or_default(),
    }
}

fn insert<T>(
    snapshot: &mut Snapshot,
    issues: &mut Vec<ProbeIssue>,
    probe: Probe,
    result: Result<T, ProbeError>,
) where
    T: Send + Sync + 'static,
{
    if let Some(part) = take(issues, probe, result) {
        snapshot.insert(part);
    }
}

fn take<T>(issues: &mut Vec<ProbeIssue>, probe: Probe, result: Result<T, ProbeError>) -> Option<T> {
    result
        .map_err(|error| issues.push(ProbeIssue { probe, error }))
        .ok()
}

/// The Boot options page's parts. A system without GRUB or Plymouth has
/// its part missing, which the page says in words.
fn probe_boot(boot: &dyn BootSource, snapshot: &mut Snapshot, issues: &mut Vec<ProbeIssue>) {
    insert(snapshot, issues, Probe::Grub, boot.grub());
    let plymouth = boot.plymouth();
    let theme = plymouth
        .as_ref()
        .ok()
        .and_then(|state| state.current.clone());

    insert(snapshot, issues, Probe::Plymouth, plymouth);
    let loader = boot.loader();

    snapshot.insert(loader);

    // GRUB keeps its command line in /etc/default/grub instead.
    if loader == BootLoader::SystemdBoot {
        let kernel = boot
            .kernel_cmdline()
            .map(|cmdline| SdBootKernel { cmdline, theme });

        insert(snapshot, issues, Probe::KernelCmdline, kernel);
    }

    insert(
        snapshot,
        issues,
        Probe::VirtualMachine,
        boot.virtual_machine(),
    );
}
