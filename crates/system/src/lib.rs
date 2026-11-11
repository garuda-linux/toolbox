//! Ports to the running system: one trait per source and the real adapters
//! behind them. [`probe_snapshot`] fills the parts of a
//! [`toolbox_state::Snapshot`] the domains compare with. Features take the
//! traits, so their tests run against fakes.
//!
//! Most adapters block, so the app runs them in `spawn_blocking`.

mod accounts;
mod app_icons;
mod boot;
mod bus;
mod command;
mod config_files;
pub mod diagnostics;
mod error;
mod facts;
mod files;
mod host;
mod kernels;
mod locales;
mod locks;
pub mod mirrors;
mod names;
mod network;
mod notify;
mod packages;
pub mod pkexec;
mod power;
mod probe;
mod repair;
mod settings;
pub mod skel;
mod status;
pub mod systemd;
#[cfg(test)]
mod test_support;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod units;
mod user_configs;

pub use accounts::{AccountSource, EtcAccounts};
pub use app_icons::{AppIconSource, AppStreamIcons, file_path, file_url};
pub use boot::{
    BootMenuSource, BootSource, Menu, MenuRead, SystemBoot, SystemBootMenu, grub_platform,
};
pub use bus::{
    CALL_TIMEOUT, CallError, INTERACTIVE_TIMEOUT, Method, call, call_with_timeout, classify,
    connect,
};
pub use config_files::{
    ConfigFileSource, DryRunEditor, EditFolder, Editing, HelperFetch, ReadProblem,
    SystemConfigFiles, SystemEditor,
};
pub use diagnostics::{DiagnosticSource, HelperReads, SystemDiagnostics};
pub use error::ProbeError;
pub use facts::{
    DiskSpace, FactSources, Facts, MachineSource, PackageCount, SystemMachine, probe_facts,
};
pub use host::{Host, HostRoots, HostSource, ProcessEnv, SystemHost};
pub use kernels::{Dkms, DkmsSource, KernelSource, SystemKernels};
pub use locales::{EtcLocales, LocaleSource};
pub use locks::{HelperLocks, LockProblem, LockSource};
pub use network::{
    Checked, ModuleSource, NetworkActions, NetworkSource, SystemActions, SystemModules,
    SystemNetwork, connectivity,
};
pub use notify::{NOTIFY, Notification, NotifyBody, notify_body};
pub use packages::{
    Libalpm, LinuxPackageSource, OrphanSource, PackageIndexSource, PackageSource, SyncSource,
};
pub use power::{
    INHIBIT, InhibitBody, REBOOT, RebootBody, ShutdownLock, inhibit_body, inhibit_shutdown,
    reboot_body,
};
pub use probe::{FailedProbes, Probe, ProbeIssue, Sources, failed_probes, probe_snapshot};
#[cfg(any(test, feature = "selftest"))]
pub use repair::DryRunRepair;
pub use repair::{RepairSource, SystemRepair};
pub use settings::{EtcSettings, SettingsSource};
pub use status::{
    AurUpdateSource, Checkupdates, GarudaHealth, HealthIssue, HealthLevel, HealthSource,
    MergeSource, Pacdiff, PacmanLog, Paru, PendingUpdate, RecentLogSource, Status, StatusSources,
    UpdateLogSource, UpdateSource, probe_status,
};
pub use toolbox_exec::locate_editor;
pub use units::{FailedUnitSource, Systemctl, UnitSource};
pub use user_configs::{HomeConfigs, UserConfigSource};
