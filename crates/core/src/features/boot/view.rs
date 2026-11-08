#[cfg(test)]
mod tests;

use serde::Serialize;
use toolbox_plan::{
    DefaultEntry, GrubKey, GrubMenu, GrubTimeout, KernelParameters, MenuEntry, PlymouthTheme,
};

use super::{Flag, PictureSource, VirtualMachine, free_parameters, is_protected, shown};
use crate::domains::{
    CurrentGrub, GrubDefaults, PlymouthState, PlymouthThemes, SdBootKernel, SdBootKernelDomain,
};
use crate::{ChangeSet, Single, Snapshot};

/// The seconds GRUB waits when the file names no timeout.
const DEFAULT_TIMEOUT: i64 = 5;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EntryRow {
    #[serde(rename = "id")]
    pub entry: String,
    pub title: String,
    pub parent: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    Ready,
    Failed,
    Missing,
    Loading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Probed {
    pub grub_failed: bool,
    pub read: bool,
}

/// What the page shows of `/etc/default/grub` with the queue on top. Built
/// once per frame, as `shown` clones the file's state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is its own switch or state of the GRUB file the page shows"
)]
pub struct Page {
    default: String,
    remember: bool,
    timeout: i32,
    timeout_known: bool,
    submenus: bool,
    os_prober: bool,
    parameters_editable: bool,
    parameter_items: Vec<String>,
    flags: Vec<Flag>,
    flags_queued: Vec<&'static str>,
    background: String,
    background_from_theme: bool,
    theme: String,
    sdboot: bool,
}

impl Page {
    pub fn read(snapshot: &Snapshot, changes: &ChangeSet) -> Self {
        if let Some(kernel) = super::kernel(snapshot) {
            return Self::sdboot(kernel, changes);
        }

        let Some(queued) = shown(snapshot, changes) else {
            return Self {
                timeout: clamped(DEFAULT_TIMEOUT),
                ..Self::default()
            };
        };

        let flags = set_flags(queued.parameters.as_ref());
        let current = if changes.is_empty() {
            flags.clone()
        } else {
            let before = shown(snapshot, &ChangeSet::default());

            set_flags(before.as_ref().and_then(|shown| shown.parameters.as_ref()))
        };

        Self {
            flags_queued: differing(&flags, &current),
            remember: queued.remember,
            timeout: clamped(queued.timeout.unwrap_or(DEFAULT_TIMEOUT)),
            timeout_known: queued.timeout.is_some(),
            submenus: queued.submenus,
            os_prober: queued.os_prober,
            parameters_editable: queued.parameters.is_some(),
            parameter_items: items(queued.parameters.as_ref()),
            flags,
            background_from_theme: queued.background_source == PictureSource::Theme,
            background: queued.background,
            theme: queued.theme,
            default: queued.default,
            sdboot: false,
        }
    }

    /// Only the kernel card reads it: systemd-boot has its own menu card.
    fn sdboot(kernel: &SdBootKernel, changes: &ChangeSet) -> Self {
        let current = kernel.cmdline.parameters.as_ref();
        let queued = SdBootKernelDomain::queued_parameters(changes);
        let list = queued.as_ref().or(current);
        let flags = set_flags(list);

        Self {
            timeout: clamped(DEFAULT_TIMEOUT),
            parameters_editable: list.is_some(),
            parameter_items: items(list),
            flags_queued: differing(&flags, &set_flags(current)),
            flags,
            sdboot: true,
            ..Self::default()
        }
    }

    pub fn removable(&self, item: &str) -> bool {
        !(self.sdboot && is_protected(item))
    }

    pub fn flag(&self, flag: Flag) -> bool {
        self.flags.contains(&flag)
    }

    pub const fn remember(&self) -> bool {
        self.remember
    }

    /// Clamped to what the timeout field offers.
    pub const fn timeout(&self) -> i32 {
        self.timeout
    }

    pub const fn timeout_known(&self) -> bool {
        self.timeout_known
    }

    pub const fn submenus(&self) -> bool {
        self.submenus
    }

    pub const fn os_prober(&self) -> bool {
        self.os_prober
    }

    pub const fn parameters_editable(&self) -> bool {
        self.parameters_editable
    }

    pub fn parameter_items(&self) -> &[String] {
        &self.parameter_items
    }

    /// The ids of the switches whose queued command line differs from the
    /// current one, so each switch can mark its own change.
    pub fn flags_queued(&self) -> &[&'static str] {
        &self.flags_queued
    }

    pub fn background(&self) -> &str {
        &self.background
    }

    pub const fn background_from_theme(&self) -> bool {
        self.background_from_theme
    }

    pub fn theme(&self) -> &str {
        &self.theme
    }

    /// The entry the menu's default select shows, empty when GRUB boots
    /// the last one or nothing is known.
    pub fn entry_value(&self, menu: Option<&GrubMenu>) -> String {
        self.listed_entry(menu).map_or_else(
            || {
                if self.boots_last_entry() {
                    String::new()
                } else {
                    self.default.clone()
                }
            },
            |entry| entry.id.as_str().to_owned(),
        )
    }

    pub fn entry_listed(&self, menu: Option<&GrubMenu>) -> bool {
        self.listed_entry(menu).is_some()
    }

    fn boots_last_entry(&self) -> bool {
        self.default == DefaultEntry::Saved.as_str()
    }

    fn listed_entry<'menu>(&self, menu: Option<&'menu GrubMenu>) -> Option<&'menu MenuEntry> {
        menu.and_then(|menu| menu.find(&self.default))
            .filter(|entry| !entry.submenu && !self.boots_last_entry())
    }
}

fn set_flags(list: Option<&KernelParameters>) -> Vec<Flag> {
    Flag::ALL
        .into_iter()
        .filter(|flag| list.is_some_and(|list| flag.is_set(list)))
        .collect()
}

fn items(list: Option<&KernelParameters>) -> Vec<String> {
    list.map_or_else(Vec::new, free_parameters)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn differing(flags: &[Flag], current: &[Flag]) -> Vec<&'static str> {
    Flag::ALL
        .into_iter()
        .filter(|flag| flags.contains(flag) != current.contains(flag))
        .map(Flag::as_str)
        .collect()
}

fn clamped(seconds: i64) -> i32 {
    let bounded = seconds.clamp(GrubTimeout::MIN.into(), GrubTimeout::MAX.into());
    i32::try_from(bounded).unwrap_or(i32::MAX)
}

pub fn state(snapshot: &Snapshot, probed: Probed) -> Readiness {
    if super::grub(snapshot).is_ok() {
        Readiness::Ready
    } else if probed.grub_failed {
        Readiness::Failed
    } else if probed.read {
        Readiness::Missing
    } else {
        Readiness::Loading
    }
}

pub fn virtual_machine(snapshot: &Snapshot) -> bool {
    snapshot
        .part::<VirtualMachine>()
        .is_some_and(|machine| machine.0)
}

pub fn entries(menu: Option<&GrubMenu>) -> Vec<EntryRow> {
    menu.map(|menu| {
        menu.entries
            .iter()
            .filter(|entry| !entry.submenu)
            .map(|entry| EntryRow {
                entry: entry.id.as_str().to_owned(),
                title: entry.title.clone(),
                parent: entry.parent.clone().unwrap_or_default(),
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn parameters_line(snapshot: &Snapshot) -> String {
    if let Some(kernel) = super::kernel(snapshot) {
        return kernel.cmdline.line.clone();
    }

    snapshot
        .part::<CurrentGrub>()
        .map_or("", |grub| grub.0.command_line.as_str())
        .to_owned()
}

pub fn parameters_queued(changes: &ChangeSet) -> bool {
    is_queued(changes, GrubKey::KernelParameters)
        || SdBootKernelDomain::queued_parameters(changes).is_some()
}

pub fn plymouth_installed(snapshot: &Snapshot) -> bool {
    snapshot.part::<PlymouthState>().is_some()
}

pub fn plymouth_themes(snapshot: &Snapshot) -> Vec<String> {
    snapshot
        .part::<PlymouthState>()
        .map(|state| {
            state
                .themes
                .iter()
                .map(PlymouthTheme::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn plymouth_current(snapshot: &Snapshot) -> String {
    snapshot
        .part::<PlymouthState>()
        .and_then(|state| state.current.as_ref())
        .map_or("", PlymouthTheme::as_str)
        .to_owned()
}

pub fn plymouth_theme_queued(changes: &ChangeSet) -> String {
    changes
        .wish::<PlymouthThemes>(&Single)
        .cloned()
        .or_else(|| SdBootKernelDomain::queued_theme(changes))
        .as_ref()
        .map_or("", PlymouthTheme::as_str)
        .to_owned()
}

pub fn plymouth_shown(snapshot: &Snapshot, changes: &ChangeSet) -> String {
    Some(plymouth_theme_queued(changes))
        .filter(|queued| !queued.is_empty())
        .unwrap_or_else(|| plymouth_current(snapshot))
}

pub fn is_queued(changes: &ChangeSet, key: GrubKey) -> bool {
    GrubDefaults::queued(changes, key).is_some()
}
