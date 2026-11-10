//! A queueable change's wishes onto the queue: the system read the way
//! the command that runs it now reads it, then that command's own rules.

mod reads;
#[cfg(test)]
mod tests;

use toolbox_aur::AurSource;
use toolbox_cli::args::{DkmsBuild, KernelArgs, ParamArgs};
use toolbox_core::domains::BootLoader;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::kernels::kernels_of;
use toolbox_core::features::maintenance::Operations;
use toolbox_core::features::maintenance::page::{task, upgrader};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::UnitScope;
use toolbox_services::catalogs::Catalogs;
use toolbox_system::ProbeError;

pub use self::reads::{LiveReads, SystemReads};
use super::add::Queueable;
use crate::act::{toggle_pages, unit_lookup};
use crate::change::{self, KernelChange, Refused};
use crate::{accounts, boot, locales, network, packages};

/// What a change may read besides the snapshot: the catalogs, the
/// running system and the AUR's copy.
pub struct Sources<'a> {
    pub catalogs: &'a Catalogs,
    pub system: &'a dyn SystemReads,
    pub aur: &'a dyn AurSource,
}

/// # Errors
///
/// [`Refused`] as the command that runs the change now refuses it.
pub fn build(
    change: &Queueable<'_>,
    sources: &Sources<'_>,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let Sources {
        catalogs,
        system,
        aur,
    } = *sources;

    match *change {
        Queueable::Toggle(args, want) => {
            change::toggle_onto(&toggle_pages(catalogs), &args.id, want, snapshot, changes)
        }
        Queueable::App(args, want) => {
            change::app_onto(&catalogs.apps.apps, &args.id, want, snapshot, changes)
        }
        Queueable::Unit(unit_change, args) => {
            let scope = UnitScope::from_user(args.user);
            let (lookup, known) = unit_lookup(args, &system.units(scope));
            change::service_onto(unit_change, scope, &args.unit, lookup, snapshot, changes).map_err(
                |refused| refused.hinted(args.unit.as_str(), known.iter().map(String::as_str)),
            )
        }
        Queueable::Kernel(args, kernel_change) => kernel(args, kernel_change, snapshot, changes),
        Queueable::Package(args, want) => packages::change_onto(args, want, snapshot, changes),
        Queueable::Install(args) => packages::install_onto(args, aur, snapshot, changes),
        Queueable::Task(args) => {
            let upgrader = upgrader(snapshot, system.garuda_update());
            let task = task(&args.task, upgrader)
                .ok_or_else(|| Refused::Unknown(format!("no task is called `{}`", args.task)))?;

            Operations::want(changes, task, snapshot);
            Ok(())
        }
        Queueable::Dkms(args) => dkms(args, system, snapshot, changes),
        Queueable::Locale(args, want) => locales::switch_onto(args, want, snapshot, changes),
        Queueable::SystemLocale(args) => locales::system_locale_onto(args, snapshot, changes),
        Queueable::EditUser(args) => accounts::edit_onto(args, snapshot, changes),
        Queueable::Lock(args, locked) => accounts::lock_onto(args, locked, snapshot, changes),
        Queueable::RemoveUser(args) => accounts::remove_onto(args, snapshot, changes),
        Queueable::Group(args, wanted) => accounts::group_onto(args, wanted, snapshot, changes),
        Queueable::BootSet(args)
            if snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot) =>
        {
            boot::sdboot::set_onto(args, || system.boot_menu(), snapshot, changes)
        }
        Queueable::BootSet(args) => {
            boot::edit_onto(boot::setting(args, snapshot)?, snapshot, changes)
        }
        Queueable::Parameter(args, wish) => parameter(args, wish, snapshot, changes),
        Queueable::Tweak(tweak) => {
            crate::tweaks::onto(tweak, snapshot, changes);
            Ok(())
        }
        Queueable::Driver(args, on) => {
            network::blacklist_onto(args, Toggle::from_on(on.is_on()), snapshot, changes)
        }
    }
}

fn parameter(
    args: &ParamArgs,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let parameter = args.parameter.to_string();
    let edit = match wish {
        Toggle::On => Edit::AddParameter {
            text: parameter,
            confirmed: args.force,
        },
        Toggle::Off => Edit::RemoveParameter(parameter),
    };

    boot::edit_onto(edit, snapshot, changes)
}

fn kernel(
    args: &KernelArgs,
    kernel_change: KernelChange,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let Some((kernels, _)) = kernels_of(snapshot) else {
        return Err(Refused::Unavailable(
            "not available: pacman's databases could not be read".to_owned(),
        ));
    };

    change::kernel_onto(&kernels, &args.kernel, kernel_change, snapshot, changes)
}

fn dkms(
    args: &DkmsBuild,
    system: &dyn SystemReads,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let entries = match system.dkms() {
        Ok(entries) => entries,
        Err(ProbeError::Unavailable) => Vec::new(),
        Err(error) => {
            tracing::warn!("dkms status: {error}");
            return Err(Refused::Unavailable(
                "not available: dkms could not list its modules".to_owned(),
            ));
        }
    };

    boot::build_onto(args, &entries, snapshot, changes)
}
