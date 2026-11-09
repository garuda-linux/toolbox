//! Which probes of a snapshot delivered nothing. Pages and the guards of
//! a restored queue read it to tell a part the probes did not read yet
//! from one they could not read.

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Probe {
    Packages,
    SyncPackages,
    PackageGraph,
    Units,
    UserUnits,
    MaskedUnits,
    MaskedUserUnits,
    Accounts,
    LoginShells,
    AvailableShells,
    Dns,
    Hblock,
    WifiBackend,
    GarudaUpdate,
    ModuleBlacklist,
    Grub,
    KernelCmdline,
    Plymouth,
    VirtualMachine,
    UserConfigs,
    LinuxPackages,
    RunningKernel,
    KernelReleases,
    SupportedLocales,
    EnabledLocales,
    SystemLocale,
    User,
    LiveIso,
    Desktops,
    Edition,
    RebootPending,
}

/// A snapshot without this part was not probed yet. A missing part of a
/// probed snapshot is a failed read when its probe is listed here, else
/// the system lacks the thing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FailedProbes(Vec<Probe>);

impl Probe {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl FailedProbes {
    pub fn new<I>(failed: I) -> Self
    where
        I: IntoIterator<Item = Probe>,
    {
        Self(failed.into_iter().collect())
    }

    pub fn contains(&self, probe: Probe) -> bool {
        self.0.contains(&probe)
    }
}
