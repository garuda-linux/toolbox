//! Each page's check of a queue restored from disk. The variants run in
//! their written order, and a new page that queues changes adds its own.

use super::{Recheck, boot, kernels, locales, missing, network, packages, users};
use crate::Domain as _;
use crate::domains::{
    AccountLocks, AurPackages, FullNames, Groups, GrubDefaults, Hblock, HomeFolders, Locales,
    LoginShells, ModuleBlacklist, NewGroups, Packages, PlymouthThemes, RemovedGroups, RemovedUsers,
    SdBootKernelDomain, WifiBackends,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumIter)]
pub enum Guard {
    Users,
    Memberships,
    /// Apart from the users: the lock flags come from the helper once the
    /// Users page asks, long after the account files.
    UserLocks,
    Kernels,
    Locales,
    LanguagePacks,
    Plymouth,
    Grub,
    ModuleBlacklist,
    NewGroups,
    Hblock,
    WifiBackend,
    /// Waits for the AUR's copy, which the window reads only on demand.
    AurPackages,
}

impl Guard {
    /// The domains whose changes it may drop, so a plan of them waits
    /// for it.
    pub const fn domains(self) -> &'static [&'static str] {
        match self {
            Self::Users => &[
                RemovedUsers::ID,
                Groups::ID,
                LoginShells::ID,
                FullNames::ID,
                HomeFolders::ID,
            ],
            Self::Memberships => &[Groups::ID, RemovedGroups::ID],
            Self::UserLocks => &[RemovedUsers::ID, Groups::ID, AccountLocks::ID],
            Self::Kernels | Self::LanguagePacks => &[Packages::ID],
            Self::Locales => &[Locales::ID],
            Self::Plymouth => &[PlymouthThemes::ID, SdBootKernelDomain::ID],
            Self::Grub => &[GrubDefaults::ID, SdBootKernelDomain::ID],
            Self::ModuleBlacklist => &[ModuleBlacklist::ID],
            Self::NewGroups => &[NewGroups::ID],
            Self::Hblock => &[Hblock::ID],
            Self::WifiBackend => &[WifiBackends::ID],
            Self::AurPackages => &[AurPackages::ID],
        }
    }

    pub fn recheck(self) -> Recheck {
        match self {
            Self::Users => users::recheck,
            Self::Memberships => users::recheck_groups,
            Self::UserLocks => users::recheck_locks,
            Self::Kernels => kernels::recheck,
            Self::Locales => locales::recheck,
            Self::LanguagePacks => locales::view::drop_stranded_packs,
            Self::Plymouth => boot::recheck,
            Self::Grub => boot::recheck_grub,
            Self::ModuleBlacklist => network::recheck_blacklist,
            Self::NewGroups => missing::recheck_new_groups,
            Self::Hblock => missing::recheck_hblock,
            Self::WifiBackend => missing::recheck_wifi_backend,
            Self::AurPackages => packages::aur::recheck,
        }
    }
}
