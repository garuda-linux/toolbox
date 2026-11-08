//! Without `/etc/default/grub` GRUB is not the boot loader, so nothing here is editable.
//! The kernel command line is one variable, so every parameter change queues the whole list.

pub mod edit;
mod flags;
mod picture;
mod recheck;
mod sdboot;
#[cfg(test)]
mod tests;
pub mod view;

use std::sync::LazyLock;

pub use flags::{Flag, free_parameters, is_flag_parameter};
pub use picture::PictureSource;
pub use recheck::{recheck, recheck_grub};
pub use sdboot::{is_protected, kernel, kernel_default};
use toolbox_plan::{
    DefaultEntry, GrubKey, GrubSetting, GrubState, KernelParameter, KernelParameters, MenuEntryId,
    PackageName, PlymouthTheme,
};
pub use toolbox_state::boot::VirtualMachine;

use crate::domains::{
    CurrentGrub, GrubDefaults, InstalledPackages, KernelSetting, Packages, PlymouthState,
    PlymouthThemes, SdBootKernelDomain,
};
use crate::{ChangeSet, Single, Snapshot, Toggle};

#[expect(clippy::expect_used, reason = "a constant name, parsed by a test")]
pub static PLYMOUTH_PACKAGE: LazyLock<PackageName> =
    LazyLock::new(|| PackageName::new("plymouth").expect("`plymouth` is a package name"));

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// No `/etc/default/grub`, so GRUB is not the boot loader.
    Unavailable,
    /// Quotes or a `$` cannot be written back word for word, so the page leaves the line alone.
    CustomCommandLine,
    TooManyParameters,
    UnknownTheme,
    /// systemd-boot without `/etc/kernel/cmdline` or kernel-install.
    NoKernelCmdline,
    /// [`Refusal::CustomCommandLine`] of `/etc/kernel/cmdline`.
    CustomKernelCmdline,
    /// See [`is_protected`].
    ProtectedParameter,
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub default: String,
    /// GRUB boots the entry it booted last.
    pub remember: bool,
    pub timeout: Option<i64>,
    /// `None` when the page leaves the command line alone.
    pub parameters: Option<KernelParameters>,
    pub submenus: bool,
    pub os_prober: bool,
    pub background: String,
    /// The page cannot take the theme's picture away.
    pub background_source: PictureSource,
    pub theme: String,
}

/// # Errors
///
/// [`Refusal::Unavailable`] on a system without `/etc/default/grub`.
pub fn grub(snapshot: &Snapshot) -> Result<&GrubState, Refusal> {
    snapshot
        .part::<CurrentGrub>()
        .map(|current| &current.0)
        .ok_or(Refusal::Unavailable)
}

/// `None` without `/etc/default/grub`.
pub fn shown(snapshot: &Snapshot, changes: &ChangeSet) -> Option<Shown> {
    let state = grub(snapshot).ok()?;
    let mut shown = Shown {
        default: state.default.clone(),
        remember: state.save_default || state.default == DefaultEntry::Saved.as_str(),
        timeout: state.timeout,
        parameters: state.parameters.clone(),
        submenus: state.submenus,
        os_prober: state.os_prober,
        background: state.shown_background().to_owned(),
        background_source: PictureSource::of(state.background_is_from_theme()),
        theme: state.theme.clone(),
    };

    for (_, setting) in changes.queued::<GrubDefaults>() {
        match setting {
            GrubSetting::Default(entry) => entry.as_str().clone_into(&mut shown.default),
            GrubSetting::Timeout(seconds) => shown.timeout = Some(i64::from(seconds.get())),
            GrubSetting::KernelParameters(list) => shown.parameters = Some(list),
            GrubSetting::Submenus(on) => shown.submenus = on,
            GrubSetting::SaveDefault(on) => shown.remember = on,
            GrubSetting::OsProber(on) => shown.os_prober = on,
            GrubSetting::Background(picture) => {
                shown.background_source =
                    PictureSource::of(picture.is_none() && !state.menu_picture.is_empty());
                shown.background = picture
                    .as_ref()
                    .map_or_else(|| state.menu_picture.clone(), ToString::to_string);
            }
            GrubSetting::Theme(theme) => {
                shown.theme = theme.as_ref().map(ToString::to_string).unwrap_or_default();
            }
        }
    }

    Some(shown)
}

/// Takes the queued change back when the file already holds the value.
///
/// # Errors
///
/// [`Refusal::Unavailable`] without `/etc/default/grub`.
pub fn set(
    setting: GrubSetting,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    grub(snapshot)?;
    GrubDefaults::want(changes, setting, snapshot);
    Ok(())
}

/// Off needs an entry to boot again: `fallback` when the file names none.
///
/// # Errors
///
/// [`Refusal::Unavailable`] without `/etc/default/grub`.
pub fn set_remember(
    on: bool,
    fallback: MenuEntryId,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let state = grub(snapshot)?;
    let default = if on {
        DefaultEntry::Saved
    } else {
        MenuEntryId::new(state.default.as_str())
            .ok()
            .filter(|entry| entry.as_str() != DefaultEntry::Saved.as_str())
            .map_or(DefaultEntry::Entry(fallback), DefaultEntry::Entry)
    };

    set(GrubSetting::SaveDefault(on), changes, snapshot)?;
    set(GrubSetting::Default(default), changes, snapshot)
}

/// # Errors
///
/// [`Refusal::Unavailable`] without the file,
/// [`Refusal::CustomCommandLine`] when the page leaves the line alone.
pub fn parameters(snapshot: &Snapshot, changes: &ChangeSet) -> Result<KernelParameters, Refusal> {
    if sdboot::is_sdboot(snapshot) {
        return sdboot::parameters(snapshot, changes);
    }

    let state = grub(snapshot)?;

    if let Some(GrubSetting::KernelParameters(queued)) =
        GrubDefaults::queued(changes, GrubKey::KernelParameters)
    {
        return Ok(queued);
    }

    state.parameters.clone().ok_or(Refusal::CustomCommandLine)
}

/// # Errors
///
/// As [`parameters`], and [`Refusal::TooManyParameters`] for a full list.
pub fn add_parameter(
    parameter: KernelParameter,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let list = parameters(snapshot, changes)?
        .with(parameter)
        .map_err(|_full| Refusal::TooManyParameters)?;

    set_parameters(list, changes, snapshot)
}

/// # Errors
///
/// As [`parameters`], and [`Refusal::ProtectedParameter`] on systemd-boot
/// for one [`is_protected`] names.
pub fn remove_parameter(
    parameter: &str,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let list = parameters(snapshot, changes)?;

    if sdboot::is_sdboot(snapshot) && is_protected(parameter) {
        return Err(Refusal::ProtectedParameter);
    }

    set_parameters(list.without(parameter), changes, snapshot)
}

fn set_parameters(
    list: KernelParameters,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    if sdboot::is_sdboot(snapshot) {
        SdBootKernelDomain::want(changes, KernelSetting::Parameters(list), snapshot);
        Ok(())
    } else {
        set(GrubSetting::KernelParameters(list), changes, snapshot)
    }
}

/// The splash also queues the Plymouth package while it is missing, and takes that back when the splash goes off.
///
/// # Errors
///
/// As [`add_parameter`].
pub fn set_flag(
    flag: Flag,
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let on = wish.is_on();
    let current = parameters(snapshot, changes)?;
    let list = if on {
        // Secondary spellings give way to the canonical one, which stays
        // where `with` found it.
        flag.unified(&current)
            .with(flag.kernel_parameter().clone())
            .map_err(|_full| Refusal::TooManyParameters)?
    } else {
        flag.stripped(&current)
    };

    set_parameters(list, changes, snapshot)?;

    if flag == Flag::Splash {
        want_plymouth(on, changes, snapshot);
    }

    Ok(())
}

/// # Errors
///
/// [`Refusal::UnknownTheme`] unless Plymouth has that theme installed.
pub fn set_plymouth_theme(
    theme: PlymouthTheme,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    if !plymouth_has(&theme, snapshot) {
        return Err(Refusal::UnknownTheme);
    }

    if sdboot::is_sdboot(snapshot) {
        SdBootKernelDomain::want(changes, KernelSetting::PlymouthTheme(theme), snapshot);
    } else {
        changes.want::<PlymouthThemes>(Single, theme, snapshot);
    }

    Ok(())
}

fn plymouth_has(theme: &PlymouthTheme, snapshot: &Snapshot) -> bool {
    snapshot
        .part::<PlymouthState>()
        .is_some_and(|state| state.themes.contains(theme))
}

pub fn plymouth_queued(changes: &ChangeSet) -> bool {
    changes.wish::<Packages>(&PLYMOUTH_PACKAGE) == Some(&Toggle::On)
}

/// Without the installed package list nothing is queued, and the page reports Plymouth as missing.
fn want_plymouth(on: bool, changes: &mut ChangeSet, snapshot: &Snapshot) {
    let Some(installed) = snapshot.part::<InstalledPackages>() else {
        return;
    };

    if installed.0.contains(&*PLYMOUTH_PACKAGE) {
        return;
    }

    if on {
        changes.want::<Packages>(PLYMOUTH_PACKAGE.clone(), Toggle::On, snapshot);
    } else if plymouth_queued(changes) {
        changes.remove::<Packages>(&PLYMOUTH_PACKAGE);
    } else {
        // Nothing of the page's own is queued for the package.
    }
}
