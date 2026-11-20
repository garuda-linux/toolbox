use super::Builtin;

/// The note of [`Builtin::SetSystemLocale`]. A test holds it to the path of
/// `LOCALE_GEN`, as a const cannot join the two.
pub(super) const SET_LOCALE_WHY: &str = "only when /etc/locale.gen has it enabled";

/// What a part shows of the steps inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// Their commands, as the Console lists them.
    Commands,
    /// Their commands with their notes, as the log and Review show them.
    WithNotes,
}

impl Builtin {
    /// Why the part runs, after `#` in the log and on the Review page.
    pub fn why(&self) -> Option<&'static str> {
        match self {
            Self::RemoveDatabaseLock => Some("only while pacman is not running"),
            Self::PrepareRepairMount => Some("for root alone, nothing may be mounted there"),
            Self::SetSystemLocale(_) => Some(SET_LOCALE_WHY),
            Self::WriteConfigFile(_) | Self::WriteHomeFile(_) => {
                Some(crate::config_file::UNCHANGED_WHY)
            }
            Self::RepairBootLoader { system, work } => crate::action::loader_why(system, work),
            Self::ResolveMerge(_) => Some(crate::MergeResolve::why()),
            Self::SnapshotBeforeMerges => Some("so the merges can be rolled back with snapper"),
            Self::InstallMirrorlist(_)
            | Self::RequireListedShell(_)
            | Self::WriteDnsConfig(_)
            | Self::WriteWifiBackend(_)
            | Self::SetGarudaUpdateFlag { flag: _, on: _ }
            | Self::EditUserConfig { edit: _, on: _ }
            | Self::EditLocaleGen { entry: _, on: _ }
            | Self::RequireOrdinaryUser(_)
            | Self::RequireFreeHome(_, _)
            | Self::RequireSafeHome(_)
            | Self::RequireUserGroup(_)
            | Self::InstallBuilt {
                run: _,
                packages: _,
                reason: _,
            }
            | Self::BuildAur { base: _, run: _ }
            | Self::EditModprobeBlacklist { module: _, on: _ }
            | Self::EditGrubDefaults(_)
            | Self::EditLoaderConf(_)
            | Self::WriteKernelCmdline(_)
            | Self::RequireInstalledSystem(_)
            | Self::RecheckRepairDevices { system: _, work: _ }
            | Self::RebuildInitramfs { root: _ }
            | Self::RebuildUkis { root: _ } => None,
        }
    }
}
