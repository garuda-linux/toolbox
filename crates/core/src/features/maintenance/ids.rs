//! The Fluent ids of an operation's name, detail, queue button and question.
//! They are literals, so grep and the catalog checks see each one.

use super::Operation;

impl Operation {
    /// The id of its name on the Common tab.
    pub const fn text_id(self) -> &'static str {
        match self {
            Self::RefreshKeyring => "maintenance-op-refresh-keyring",
            Self::RemoveDatabaseLock => "maintenance-op-remove-database-lock",
            Self::UpgradeSystem => "maintenance-op-upgrade-system",
            Self::RemoveOrphans => "maintenance-op-remove-orphans",
            Self::CleanPackageCache => "maintenance-op-clean-package-cache",
        }
    }

    pub const fn detail_id(self) -> &'static str {
        match self {
            Self::RefreshKeyring => "maintenance-op-refresh-keyring-detail",
            Self::RemoveDatabaseLock => "maintenance-op-remove-database-lock-detail",
            Self::UpgradeSystem => "maintenance-op-upgrade-system-detail",
            Self::RemoveOrphans => "maintenance-op-remove-orphans-detail",
            Self::CleanPackageCache => "maintenance-op-clean-package-cache-detail",
        }
    }

    pub const fn queue_id(self) -> &'static str {
        match self {
            Self::RefreshKeyring => "maintenance-op-refresh-keyring-queue",
            Self::RemoveDatabaseLock => "maintenance-op-remove-database-lock-queue",
            Self::UpgradeSystem => "maintenance-op-upgrade-system-queue",
            Self::RemoveOrphans => "maintenance-op-remove-orphans-queue",
            Self::CleanPackageCache => "maintenance-op-clean-package-cache-queue",
        }
    }

    pub const fn question_id(self) -> &'static str {
        match self {
            Self::RefreshKeyring => "maintenance-op-refresh-keyring-question",
            Self::RemoveDatabaseLock => "maintenance-op-remove-database-lock-question",
            Self::UpgradeSystem => "maintenance-op-upgrade-system-question",
            Self::RemoveOrphans => "maintenance-op-remove-orphans-question",
            Self::CleanPackageCache => "maintenance-op-clean-package-cache-question",
        }
    }
}
