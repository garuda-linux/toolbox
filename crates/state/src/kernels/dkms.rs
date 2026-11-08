//! What `dkms status` prints, and what it means for one kernel. The
//! output changed over the dkms versions, so one line is any of:
//!
//! ```text
//! nvidia/550.90.07, 6.14.0-arch1-1, x86_64: installed
//! vboxhost/7.1.6_OSE, 6.12.9-1-lts: installed (original_module exists)
//! acpi_call, 1.2.2, 6.14.0-arch1-1, x86_64: built
//! zfs/2.3.1: added
//! nvidia/550.90.07, 6.2.2-zen: broken
//! ```

#[cfg(test)]
mod tests;

use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion, KernelRelease};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DkmsState {
    /// Its source is registered, nothing is built.
    Added,
    Built,
    Installed,
    /// dkms cannot use the module's source any more.
    Broken,
}

/// `added` lines name no kernel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DkmsEntry {
    pub module: DkmsModuleName,
    pub version: DkmsVersion,
    pub kernel: Option<KernelRelease>,
    pub state: DkmsState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleState {
    Installed,
    Built,
    Broken,
    /// dkms knows the module but has nothing for this kernel.
    Missing,
}

/// One module as the Kernels page shows it on a kernel's row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleStatus {
    pub module: DkmsModuleName,
    pub version: DkmsVersion,
    pub state: ModuleState,
}

/// Every line that names a module and a state. Warnings, errors and lines
/// with a name no dkms command would take are skipped.
pub fn parse_status(output: &str) -> Vec<DkmsEntry> {
    output.lines().filter_map(parse_line).collect()
}

/// Every module dkms knows, with its state for the kernel `release`.
///
/// A module installed there in any version is fine. Otherwise the row
/// offers the version dkms lists last, which is the newest one registered.
pub fn modules(release: &KernelRelease, entries: &[DkmsEntry]) -> Vec<ModuleStatus> {
    let mut names: Vec<&DkmsModuleName> = Vec::new();

    for entry in entries {
        if !names.contains(&&entry.module) {
            names.push(&entry.module);
        }
    }

    names
        .into_iter()
        .filter_map(|name| {
            let of_module = || entries.iter().filter(move |entry| &entry.module == name);
            let here = |entry: &&DkmsEntry| entry.kernel.as_ref() == Some(release);
            let installed = of_module()
                .filter(here)
                .find(|entry| entry.state == DkmsState::Installed);

            if let Some(entry) = installed {
                return Some(status(entry, ModuleState::Installed));
            }

            let newest = of_module().next_back()?;
            let state = of_module()
                .filter(|entry| entry.version == newest.version)
                .filter(|entry| here(entry) || entry.kernel.is_none())
                .map(|entry| match entry.state {
                    DkmsState::Broken => ModuleState::Broken,
                    DkmsState::Built => ModuleState::Built,
                    DkmsState::Added | DkmsState::Installed => ModuleState::Missing,
                })
                .max_by_key(|state| match state {
                    ModuleState::Broken => 2_u8,
                    ModuleState::Built => 1_u8,
                    ModuleState::Installed | ModuleState::Missing => 0_u8,
                })
                .unwrap_or(ModuleState::Missing);

            Some(status(newest, state))
        })
        .collect()
}

/// The builds that bring every module of [`modules`] onto the kernel.
pub fn builds(release: &KernelRelease, entries: &[DkmsEntry]) -> Vec<DkmsBuild> {
    modules(release, entries)
        .into_iter()
        .filter(|module| module.state != ModuleState::Installed)
        .map(|module| DkmsBuild {
            module: module.module,
            version: module.version,
            kernel: release.clone(),
        })
        .collect()
}

fn status(entry: &DkmsEntry, state: ModuleState) -> ModuleStatus {
    ModuleStatus {
        module: entry.module.clone(),
        version: entry.version.clone(),
        state,
    }
}

fn parse_line(line: &str) -> Option<DkmsEntry> {
    let (target, status) = line.trim().split_once(": ")?;
    let state = match status.split_whitespace().next()? {
        "added" => DkmsState::Added,
        "built" => DkmsState::Built,
        "installed" => DkmsState::Installed,
        "broken" => DkmsState::Broken,
        _ => return None,
    };

    let mut fields = target.split(',').map(str::trim);
    let first = fields.next()?;
    let (module, version) = match first.split_once('/') {
        Some(pair) => pair,
        None => (first, fields.next()?),
    };

    let kernel = match fields.next() {
        Some(kernel) => Some(KernelRelease::new(kernel).ok()?),
        None => None,
    };

    Some(DkmsEntry {
        module: DkmsModuleName::new(module).ok()?,
        version: DkmsVersion::new(version).ok()?,
        kernel,
        state,
    })
}
