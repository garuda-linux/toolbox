//! The Services page's rows: systemd units with their states.

use toolbox_plan::UnitName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActiveState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    /// systemd cleans the unit's runtime or state folders.
    Maintenance,
    /// A mount unit's images are refreshed in place.
    Refreshing,
    /// A state newer than this app.
    Other,
}

impl ActiveState {
    pub fn parse(text: &str) -> Self {
        match text {
            "active" => Self::Active,
            "reloading" => Self::Reloading,
            "inactive" => Self::Inactive,
            "failed" => Self::Failed,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            "maintenance" => Self::Maintenance,
            "refreshing" => Self::Refreshing,
            _ => Self::Other,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Reloading => "reloading",
            Self::Inactive => "inactive",
            Self::Failed => "failed",
            Self::Activating => "activating",
            Self::Deactivating => "deactivating",
            Self::Maintenance => "maintenance",
            Self::Refreshing => "refreshing",
            Self::Other => "other",
        }
    }
}

/// The state of a unit's file, as `systemctl list-unit-files` shows it.
/// Runtime variants count as their persistent ones, the page offers the
/// same actions for both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileState {
    Enabled,
    Disabled,
    Static,
    Masked,
    Indirect,
    Generated,
    /// Its file lies outside the unit folders and is linked in.
    Linked,
    Alias,
    /// Made at runtime, gone after a restart.
    Transient,
    /// systemd could not read the file.
    Bad,
    /// A state newer than this app.
    Other,
    /// No unit file was listed for the unit.
    Unknown,
}

impl FileState {
    pub fn parse(text: &str) -> Self {
        match text {
            "enabled" | "enabled-runtime" => Self::Enabled,
            "disabled" => Self::Disabled,
            "static" => Self::Static,
            "masked" | "masked-runtime" => Self::Masked,
            "indirect" => Self::Indirect,
            "generated" => Self::Generated,
            "linked" | "linked-runtime" => Self::Linked,
            "alias" => Self::Alias,
            "transient" => Self::Transient,
            "bad" => Self::Bad,
            _ => Self::Other,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Static => "static",
            Self::Masked => "masked",
            Self::Indirect => "indirect",
            Self::Generated => "generated",
            Self::Linked => "linked",
            Self::Alias => "alias",
            Self::Transient => "transient",
            Self::Bad => "bad",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitInfo {
    pub name: UnitName,
    /// From the unit file, so untrusted text.
    pub description: String,
    pub active: ActiveState,
    /// The type specific state (`running`, `exited`, `listening`, …).
    pub sub_state: String,
    pub file_state: FileState,
}
