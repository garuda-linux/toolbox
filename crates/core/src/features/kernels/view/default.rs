#[cfg(test)]
mod tests;

use toolbox_plan::sdboot::SdBootMenu;
use toolbox_plan::{DefaultEntry, GrubMenu, GrubState, PackageName};

use crate::features::kernels::{Kernel, KernelReleases};

/// The kernel of the entry `GRUB_DEFAULT` names, resolved as the Boot
/// page resolves it. `saved` keeps its entry in grubenv, which is not
/// read, so it names none.
pub fn grub(state: &GrubState, menu: &GrubMenu, kernels: &[Kernel]) -> Option<PackageName> {
    if state.default == DefaultEntry::Saved.as_str() {
        return None;
    }

    let entry = menu.find(&state.default).filter(|entry| !entry.submenu)?;

    // The advanced id names the kernel of its entry. Other entries, such as
    // the top-level one, name the kernel their `linux` line boots.
    let by_id = kernels
        .iter()
        .filter(|kernel| kernel.is_installed())
        .find(|kernel| {
            menu.kernel_entry(&kernel.name)
                .is_some_and(|own| own.id == entry.id)
        });

    by_id
        .or_else(|| {
            kernels
                .iter()
                .filter(|kernel| kernel.is_installed())
                .find(|kernel| Some(&kernel.name) == entry.kernel.as_ref())
        })
        .map(|kernel| kernel.name.clone())
}

/// The installed kernel whose release is the one of the entry `bootctl`
/// marks. The release comes from the entry's own `.uname`.
pub fn sdboot(
    menu: &SdBootMenu,
    releases: &KernelReleases,
    kernels: &[Kernel],
) -> Option<PackageName> {
    let default = menu.default.as_ref()?;
    let release = menu
        .entries
        .iter()
        .find(|entry| &entry.id == default)?
        .uname
        .as_ref()?;

    kernels
        .iter()
        .filter(|kernel| kernel.is_installed())
        .find(|kernel| releases.0.get(&kernel.name) == Some(release))
        .map(|kernel| kernel.name.clone())
}
