use std::collections::BTreeSet;

use toolbox_plan::{DnsProvider, GarudaUpdateFlag, WifiBackend};

/// The provider the toolbox's NetworkManager drop-in names, the default
/// without one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentDns(pub DnsProvider);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentWifiBackend(pub WifiBackend);

/// Whether `/etc/hosts` blocks any domain, as hblock's header counts them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HblockState(pub bool);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GarudaUpdateConfig(pub BTreeSet<GarudaUpdateFlag>);

impl crate::Part for CurrentDns {}

impl crate::Part for CurrentWifiBackend {}

impl crate::Part for GarudaUpdateConfig {}

impl crate::Part for HblockState {}
