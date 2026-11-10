//! The code of a change's sentence. The UI owns the words: `as_str` is
//! the Fluent id after `change-`, and the saved queue never holds it.

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum SummaryCode {
    AccountLock,
    AccountUnlock,
    AurInstall,
    CleanPackageCache,
    DkmsBuild,
    Dns,
    DriverAllow,
    DriverBlacklist,
    FullNameClear,
    FullNameSet,
    GarudaUpdateFlagOff,
    GarudaUpdateFlagOn,
    GroupAdd,
    GroupJoin,
    GroupLeave,
    GroupRemove,
    GrubBackground,
    GrubBackgroundNone,
    GrubDefault,
    GrubDefaultSaved,
    GrubOsProberOff,
    GrubOsProberOn,
    GrubParameters,
    GrubParametersNone,
    GrubSaveDefaultOff,
    GrubSaveDefaultOn,
    GrubSubmenusOff,
    GrubSubmenusOn,
    GrubTheme,
    GrubThemeNone,
    GrubTimeout,
    HblockOff,
    HblockOn,
    HomeFolder,
    LoaderAutoEntriesOff,
    LoaderAutoEntriesOn,
    LoaderAutoEntriesUnset,
    LoaderAutoFirmwareOff,
    LoaderAutoFirmwareOn,
    LoaderAutoFirmwareUnset,
    LoaderDefault,
    LoaderDefaultSaved,
    LoaderDefaultUnset,
    LoaderEditorOff,
    LoaderEditorOn,
    LoaderEditorUnset,
    LoaderTimeout,
    LoaderTimeoutMenu,
    LoaderTimeoutUnset,
    LocaleDisable,
    LocaleEnable,
    LoginShell,
    PackageInstall,
    PackageRemove,
    PlymouthTheme,
    RefreshKeyring,
    RemoveDatabaseLock,
    RemoveOrphans,
    SdbootParameters,
    SdbootParametersNone,
    SystemLocale,
    UnitDisable,
    UnitEnable,
    UnitMask,
    UnitUnmask,
    UpgradeSystem,
    UserConfigOff,
    UserConfigOn,
    UserRemoveKeepHome,
    UserRemoveWithHome,
    UserUnitDisable,
    UserUnitEnable,
    UserUnitMask,
    UserUnitUnmask,
    WifiBackend,
}

impl SummaryCode {
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    /// Applying it takes away something the user has or can reach, so a
    /// queue with one shows before it is applied.
    pub const fn destructive(self) -> bool {
        matches!(
            self,
            Self::DriverBlacklist
                | Self::GroupLeave
                | Self::GroupRemove
                | Self::PackageRemove
                | Self::UnitMask
                | Self::UserRemoveKeepHome
                | Self::UserRemoveWithHome
                | Self::UserUnitMask
        )
    }

    /// The names of the sentence's values, in their order. One value is
    /// called `name`.
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per code, a new code fails to compile until it names its values"
    )]
    pub const fn argument_names(self) -> &'static [&'static str] {
        match self {
            Self::GroupJoin | Self::GroupLeave => &["user", "group"],
            Self::FullNameSet => &["user", "name"],
            Self::HomeFolder => &["user", "folder"],
            Self::LoginShell => &["user", "shell"],
            Self::DkmsBuild => &["module", "kernel"],
            Self::AccountLock
            | Self::AccountUnlock
            | Self::AurInstall
            | Self::CleanPackageCache
            | Self::Dns
            | Self::DriverAllow
            | Self::DriverBlacklist
            | Self::FullNameClear
            | Self::GarudaUpdateFlagOff
            | Self::GarudaUpdateFlagOn
            | Self::GroupAdd
            | Self::GroupRemove
            | Self::GrubBackground
            | Self::GrubBackgroundNone
            | Self::GrubDefault
            | Self::GrubDefaultSaved
            | Self::GrubOsProberOff
            | Self::GrubOsProberOn
            | Self::GrubParameters
            | Self::GrubParametersNone
            | Self::GrubSaveDefaultOff
            | Self::GrubSaveDefaultOn
            | Self::GrubSubmenusOff
            | Self::GrubSubmenusOn
            | Self::GrubTheme
            | Self::GrubThemeNone
            | Self::GrubTimeout
            | Self::HblockOff
            | Self::HblockOn
            | Self::LoaderAutoEntriesOff
            | Self::LoaderAutoEntriesOn
            | Self::LoaderAutoEntriesUnset
            | Self::LoaderAutoFirmwareOff
            | Self::LoaderAutoFirmwareOn
            | Self::LoaderAutoFirmwareUnset
            | Self::LoaderDefault
            | Self::LoaderDefaultSaved
            | Self::LoaderDefaultUnset
            | Self::LoaderEditorOff
            | Self::LoaderEditorOn
            | Self::LoaderEditorUnset
            | Self::LoaderTimeout
            | Self::LoaderTimeoutMenu
            | Self::LoaderTimeoutUnset
            | Self::LocaleDisable
            | Self::LocaleEnable
            | Self::PackageInstall
            | Self::PackageRemove
            | Self::PlymouthTheme
            | Self::RefreshKeyring
            | Self::RemoveDatabaseLock
            | Self::RemoveOrphans
            | Self::SdbootParameters
            | Self::SdbootParametersNone
            | Self::SystemLocale
            | Self::UnitDisable
            | Self::UnitEnable
            | Self::UnitMask
            | Self::UnitUnmask
            | Self::UpgradeSystem
            | Self::UserConfigOff
            | Self::UserConfigOn
            | Self::UserRemoveKeepHome
            | Self::UserRemoveWithHome
            | Self::UserUnitDisable
            | Self::UserUnitEnable
            | Self::UserUnitMask
            | Self::UserUnitUnmask
            | Self::WifiBackend => &["name"],
        }
    }
}
