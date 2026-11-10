//! The kernel's network modules: which of them are loaded and which a
//! modprobe file blacklists.
//!
//! The kernel and modprobe read `-` and `_` in a module name as the same
//! character, so every name here is in its `_` spelling, or a loaded
//! module with a `-` in its file name would show as unloaded.

use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::KernelModule;

const NETWORK_DRIVERS: &str = "kernel/drivers/net/";

/// Network drivers that live elsewhere: built by DKMS or shipped as extra
/// modules.
const ELSEWHERE: [(&str, Category); 4] = [
    ("wl", Category::Wireless),
    ("ndiswrapper", Category::Wireless),
    ("atl2", Category::Ethernet),
    ("r8168", Category::Ethernet),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Ethernet,
    Wireless,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkModule {
    pub name: KernelModule,
    pub category: Category,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlacklistEntry {
    pub module: KernelModule,
    pub file: String,
    /// The app's own file, the only one it changes.
    pub own: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleFacts {
    pub loaded: BTreeSet<KernelModule>,
    pub modules: Vec<NetworkModule>,
    pub blacklists: Vec<BlacklistEntry>,
}

impl Category {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ethernet => "ethernet",
            Self::Wireless => "wireless",
            Self::Other => "other",
        }
    }

    fn of_path(path: &str) -> Self {
        if path.contains("/net/wireless/") {
            Self::Wireless
        } else if path.contains("/net/ethernet/") {
            Self::Ethernet
        } else {
            Self::Other
        }
    }
}

impl crate::Part for ModuleFacts {}

/// The network modules `modules.dep` of a kernel lists: everything below
/// `kernel/drivers/net`, plus the few known ones that live elsewhere.
/// Sorted by name, each once.
pub fn network_modules(modules_dep: &str) -> Vec<NetworkModule> {
    let modules: BTreeMap<KernelModule, Category> = modules_dep
        .lines()
        .filter_map(|line| {
            let path = line.split(':').next()?.trim();
            let name = module_name(path)?;
            let elsewhere = ELSEWHERE
                .iter()
                .find(|(known, _)| *known == name.as_str())
                .map(|(_, category)| *category);

            let category = if path.starts_with(NETWORK_DRIVERS) {
                Category::of_path(path)
            } else {
                elsewhere?
            };

            Some((name, category))
        })
        .collect();

    modules
        .into_iter()
        .map(|(name, category)| NetworkModule { name, category })
        .collect()
}

pub fn loaded_modules(proc_modules: &str) -> BTreeSet<KernelModule> {
    proc_modules
        .lines()
        .filter_map(|line| KernelModule::new(line.split_whitespace().next()?).ok())
        .collect()
}

/// `ath9k` for `kernel/drivers/net/wireless/ath/ath9k/ath9k.ko.zst`.
fn module_name(path: &str) -> Option<KernelModule> {
    let file = path.rsplit('/').next()?;
    let (name, _) = file.split_once(".ko")?;

    KernelModule::new(name).ok()
}
