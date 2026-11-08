use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::{KernelModule, PackageName, ShellPath, UnitName, UserConfigEdit, UserName};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MaskedUnits(pub BTreeSet<UnitName>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MaskedUserUnits(pub BTreeSet<UnitName>);

/// The modules the app's own modprobe file blacklists, each in its `_`
/// spelling.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnBlacklist(pub BTreeSet<KernelModule>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnabledUnits(pub BTreeSet<UnitName>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnabledUserUnits(pub BTreeSet<UnitName>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstalledPackages(pub BTreeSet<PackageName>);

/// Every package the configured sync databases offer, as last synced.
/// Catalog pages hide what can be neither installed nor removed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncPackages(pub BTreeSet<PackageName>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CurrentShells(pub BTreeMap<UserName, ShellPath>);

/// The shells `/etc/shells` lists, in its order. The helper refuses any
/// other, so the page offers only these.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AvailableShells(pub Vec<ShellPath>);

/// Whether each probed edit is on. An edit whose file could not be read
/// is missing, so its change stays queued.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserConfigStates(pub BTreeMap<UserConfigEdit, bool>);
