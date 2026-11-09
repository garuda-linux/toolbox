//! Fake ports for the probe tests: each answers what the test put in.

use std::sync::Arc;

use toolbox_plan::{
    DnsProvider, GrubState, KernelModule, PlymouthTheme, UserConfigEdit, UserName, WifiBackend,
};
use toolbox_state::accounts::Accounts;
use toolbox_state::boot::{BootLoader, CurrentGrub, KernelCmdline, PlymouthState, VirtualMachine};
use toolbox_state::kernels::{KernelReleases, LinuxPackages, RunningKernel};
use toolbox_state::locales::{CurrentSystemLocale, EnabledLocales, LocaleNames, SupportedLocales};
use toolbox_state::network::ModuleFacts;
use toolbox_state::packages::PackageGraph;
use toolbox_state::parts::{
    AvailableShells, CurrentShells, EnabledUnits, EnabledUserUnits, InstalledPackages, MaskedUnits,
    MaskedUserUnits, OwnBlacklist, SyncPackages, UserConfigStates,
};
use toolbox_state::settings::{CurrentDns, CurrentWifiBackend, GarudaUpdateConfig, HblockState};

use super::{accounts, kernels, offered, units};
use crate::probe::Sources;
use crate::{
    AccountSource, BootSource, HostSource, KernelSource, LinuxPackageSource, LocaleSource,
    ModuleSource, PackageSource, ProbeError, SettingsSource, SyncSource, UnitSource,
    UserConfigSource,
};

pub struct FakePackages(pub Result<InstalledPackages, ProbeError>);

pub struct FakeSync(pub Result<SyncPackages, ProbeError>);

pub struct FakeUnits {
    pub system: Result<EnabledUnits, ProbeError>,
    pub user: Result<EnabledUserUnits, ProbeError>,
    pub masked: Result<MaskedUnits, ProbeError>,
    pub masked_user: Result<MaskedUserUnits, ProbeError>,
}

pub struct FakeAccounts(pub Result<Accounts, ProbeError>);

pub struct FakeSettings {
    pub dns: Result<CurrentDns, ProbeError>,
    pub garuda_update: Result<GarudaUpdateConfig, ProbeError>,
}

/// Answers the kernel packages and the running kernel. No kernel has a
/// modules folder.
pub struct FakeKernels {
    pub linux: Result<LinuxPackages, ProbeError>,
    pub running: Result<RunningKernel, ProbeError>,
}

pub struct FakeLocales(pub Result<EnabledLocales, ProbeError>);

pub struct FakeConfigs;

pub struct FakeHost {
    pub user: Result<UserName, ProbeError>,
    pub live_iso: Result<bool, ProbeError>,
    pub desktops: Result<Vec<String>, ProbeError>,
    pub edition: Result<Option<String>, ProbeError>,
    pub reboot_pending: Result<bool, ProbeError>,
}

impl LinuxPackageSource for FakeKernels {
    fn linux_packages(&self) -> Result<LinuxPackages, ProbeError> {
        self.linux.clone()
    }
}

impl KernelSource for FakeKernels {
    fn running(&self) -> Result<RunningKernel, ProbeError> {
        self.running.clone()
    }

    fn releases(&self) -> Result<KernelReleases, ProbeError> {
        Ok(KernelReleases::default())
    }
}

impl LocaleSource for FakeLocales {
    fn supported(&self) -> Result<SupportedLocales, ProbeError> {
        Ok(SupportedLocales::default())
    }

    fn enabled(&self) -> Result<EnabledLocales, ProbeError> {
        self.0.clone()
    }

    fn system_locale(&self) -> Result<CurrentSystemLocale, ProbeError> {
        Ok(CurrentSystemLocale::default())
    }

    fn names(&self) -> LocaleNames {
        LocaleNames::default()
    }
}

impl PackageSource for FakePackages {
    fn installed(&self) -> Result<InstalledPackages, ProbeError> {
        self.0.clone()
    }

    /// Fails as the installed list does, since both read the same database.
    fn graph(&self) -> Result<PackageGraph, ProbeError> {
        self.0.clone().map(|_| PackageGraph::default())
    }
}

impl SyncSource for FakeSync {
    fn offered(&self) -> Result<SyncPackages, ProbeError> {
        self.0.clone()
    }
}

impl UnitSource for FakeUnits {
    fn enabled_units(&self) -> Result<EnabledUnits, ProbeError> {
        self.system.clone()
    }

    fn enabled_user_units(&self) -> Result<EnabledUserUnits, ProbeError> {
        self.user.clone()
    }

    fn masked_units(&self) -> Result<MaskedUnits, ProbeError> {
        self.masked.clone()
    }

    fn masked_user_units(&self) -> Result<MaskedUserUnits, ProbeError> {
        self.masked_user.clone()
    }
}

impl AccountSource for FakeAccounts {
    fn accounts(&self) -> Result<Accounts, ProbeError> {
        self.0.clone()
    }

    fn login_shells(&self) -> Result<CurrentShells, ProbeError> {
        Ok(CurrentShells::default())
    }

    fn available_shells(&self) -> Result<AvailableShells, ProbeError> {
        Ok(AvailableShells::default())
    }
}

impl SettingsSource for FakeSettings {
    fn dns(&self) -> Result<CurrentDns, ProbeError> {
        self.dns.clone()
    }

    fn hblock(&self) -> Result<HblockState, ProbeError> {
        Ok(HblockState(true))
    }

    fn wifi_backend(&self) -> Result<CurrentWifiBackend, ProbeError> {
        Ok(CurrentWifiBackend(WifiBackend::Iwd))
    }

    fn garuda_update(&self) -> Result<GarudaUpdateConfig, ProbeError> {
        self.garuda_update.clone()
    }

    fn module_blacklist(&self) -> Result<OwnBlacklist, ProbeError> {
        Ok(OwnBlacklist(KernelModule::new("wl").into_iter().collect()))
    }
}

impl UserConfigSource for FakeConfigs {
    fn states(&self, edits: &[UserConfigEdit]) -> Result<UserConfigStates, ProbeError> {
        Ok(UserConfigStates(
            edits.iter().map(|edit| (edit.clone(), true)).collect(),
        ))
    }
}

impl HostSource for FakeHost {
    fn user(&self) -> Result<UserName, ProbeError> {
        self.user.clone()
    }

    fn is_live_iso(&self) -> Result<bool, ProbeError> {
        self.live_iso.clone()
    }

    fn desktops(&self) -> Result<Vec<String>, ProbeError> {
        self.desktops.clone()
    }

    fn edition(&self) -> Result<Option<String>, ProbeError> {
        self.edition.clone()
    }

    fn is_reboot_pending(&self) -> Result<bool, ProbeError> {
        self.reboot_pending.clone()
    }
}

/// A GRUB system with Plymouth on real hardware.
pub struct FakeBoot;

impl BootSource for FakeBoot {
    fn grub(&self) -> Result<CurrentGrub, ProbeError> {
        Ok(CurrentGrub(GrubState::read("GRUB_TIMEOUT=7\n")))
    }

    fn plymouth(&self) -> Result<PlymouthState, ProbeError> {
        Ok(PlymouthState {
            themes: PlymouthTheme::new("bgrt").into_iter().collect(),
            current: PlymouthTheme::new("bgrt").ok(),
        })
    }

    fn virtual_machine(&self) -> Result<VirtualMachine, ProbeError> {
        Ok(VirtualMachine(false))
    }

    fn loader(&self) -> Result<BootLoader, ProbeError> {
        Ok(BootLoader::Grub)
    }

    fn kernel_cmdline(&self) -> Result<KernelCmdline, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

pub struct FakeModules;

impl ModuleSource for FakeModules {
    fn facts(&self) -> Result<ModuleFacts, ProbeError> {
        Ok(ModuleFacts::default())
    }
}

pub fn sources(packages: FakePackages, host: FakeHost) -> Sources {
    Sources {
        packages: Arc::new(packages),
        sync: Arc::new(FakeSync(Ok(offered()))),
        linux: Arc::new(kernels()),
        kernels: Arc::new(kernels()),
        locales: Arc::new(FakeLocales(Ok(EnabledLocales::default()))),
        units: Arc::new(units()),
        accounts: Arc::new(FakeAccounts(Ok(accounts()))),
        host: Arc::new(host),
        settings: Arc::new(FakeSettings {
            dns: Ok(CurrentDns(DnsProvider::Quad9)),
            garuda_update: Ok(GarudaUpdateConfig::default()),
        }),
        boot: Arc::new(FakeBoot),
        user_configs: Arc::new(FakeConfigs),
        modules: Arc::new(FakeModules),
        config_edits: Arc::from([]),
    }
}
