//! The plans behind the changing commands: the same queue domains, page
//! rules and planner the window uses, without the window. No I/O.

mod service;
#[cfg(test)]
mod tests;

use toolbox_catalog::ToggleCatalog;
use toolbox_cli::Exit;
use toolbox_core::domains::Packages;
use toolbox_core::features::discover::Apps;
use toolbox_core::features::kernels::{Kernel, lacks_headers};
use toolbox_core::features::maintenance::page::{asks_first, run_now_plan};
use toolbox_core::features::toggles;
use toolbox_core::{ChangeSet, Snapshot, Toggle, plan};
use toolbox_plan::{PackageName, Plan, Upgrader};

pub use self::service::{UnitLookup, service, service_onto};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    Unknown(String),
    /// The page would refuse the same click.
    Rule(String),
    NothingToDo(String),
    Unavailable(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelChange {
    Install,
    Remove,
    Headers,
}

impl Refused {
    pub const fn exit(&self) -> Exit {
        match self {
            Self::Unknown(_) | Self::Unavailable(_) => Exit::Unavailable,
            Self::Rule(_) => Exit::PageRule,
            Self::NothingToDo(_) => Exit::Success,
        }
    }

    pub fn line(&self, paint: toolbox_cli::output::Paint) -> String {
        match self {
            Self::NothingToDo(message) => message.clone(),
            Self::Unknown(message) | Self::Rule(message) | Self::Unavailable(message) => {
                match message.split_once('\n') {
                    Some((error, hint)) => format!("{}\n{}", paint.error(error), paint.hint(hint)),
                    None => paint.error(message),
                }
            }
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn hinted<'a>(self, wanted: &str, known: impl IntoIterator<Item = &'a str>) -> Self {
        let Some(hint) = toolbox_cli::suggest::did_you_mean(wanted, known) else {
            return self;
        };

        match self {
            Self::Unknown(message) => Self::Unknown(format!("{message}\n{hint}")),
            Self::Rule(message) => Self::Rule(format!("{message}\n{hint}")),
            other @ (Self::NothingToDo(_) | Self::Unavailable(_)) => other,
        }
    }

    pub(super) fn page_rule(code: &str) -> Self {
        Self::Rule(format!(
            "the page refuses this ({code}), see the window for why"
        ))
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Unknown(message)
            | Self::Rule(message)
            | Self::NothingToDo(message)
            | Self::Unavailable(message) => message,
        }
    }
}

/// A switch of one of `catalogs` on or off, with what it needs and what
/// conflicts with it, as a click on the page would queue it.
///
/// # Errors
///
/// [`Refused`] for an unknown id or a click the page would refuse.
pub fn toggle(
    catalogs: &[&ToggleCatalog],
    id: &str,
    want: Toggle,
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    fresh(|changes| toggle_onto(catalogs, id, want, snapshot, changes))
}

/// [`toggle`] onto `changes`, as the page queues it onto its queue.
///
/// # Errors
///
/// [`Refused`] for an unknown id or a click the page would refuse.
pub fn toggle_onto(
    catalogs: &[&ToggleCatalog],
    id: &str,
    want: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let catalog = catalogs
        .iter()
        .find(|catalog| catalog.entry(id).is_some())
        .ok_or_else(|| {
            Refused::Unknown(format!("no switch is called `{id}`, see `toggle list`")).hinted(
                id,
                catalogs
                    .iter()
                    .flat_map(|catalog| catalog.entries().map(toolbox_catalog::ToggleEntry::id)),
            )
        })?;

    toggles::toggle(catalog, id, want, snapshot, changes).map_err(|refusal| match refusal {
        toggles::Refusal::Kernel(kept, kernel) => {
            crate::packages::kept_package(&kernel, kept.into())
        }
        toggles::Refusal::Protected(why, package) => crate::packages::kept_package(
            &package,
            toolbox_core::features::packages::queue::Refusal::Protected(why),
        ),
        other @ (toggles::Refusal::Unknown
        | toggles::Refusal::Unmet
        | toggles::Refusal::Account(..)) => Refused::page_rule(other.as_str()),
    })
}

/// # Errors
///
/// [`Refused::Unknown`] for an id the catalog lacks.
pub fn app(apps: &Apps, id: &str, want: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| app_onto(apps, id, want, snapshot, changes))
}

/// # Errors
///
/// [`Refused::Unknown`] for an id the catalog lacks.
pub fn app_onto(
    apps: &Apps,
    id: &str,
    want: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    if apps.toggle(id, want, snapshot, changes) {
        Ok(())
    } else {
        Err(
            Refused::Unknown(format!("the catalog has no app `{id}`, see `apps list`"))
                .hinted(id, apps.all().map(|hit| hit.entry.id())),
        )
    }
}

/// A kernel installed or removed, or its headers installed. The kernel
/// that runs and the last one stay.
///
/// # Errors
///
/// [`Refused`] for a package that is no kernel, a removal the Kernels
/// page refuses, or headers of a kernel that is not installed.
pub fn kernel(
    kernels: &[Kernel],
    name: &PackageName,
    change: KernelChange,
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    fresh(|changes| kernel_onto(kernels, name, change, snapshot, changes))
}

/// [`kernel`] onto `changes`, where a queued install counts as installed.
///
/// # Errors
///
/// As [`kernel`].
pub fn kernel_onto(
    kernels: &[Kernel],
    name: &PackageName,
    change: KernelChange,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let kernel = kernels
        .iter()
        .find(|kernel| kernel.name == *name)
        .ok_or_else(|| {
            Refused::Unknown(format!("`{name}` is no kernel, see `kernels list`")).hinted(
                name.as_str(),
                kernels.iter().map(|kernel| kernel.name.as_str()),
            )
        })?;

    match change {
        KernelChange::Install => {
            changes.want::<Packages>(kernel.name.clone(), Toggle::On, snapshot);
        }
        KernelChange::Remove => {
            // The same rules as the Kernels page, also when nobody knows
            // which kernel runs or this is the last one.
            toolbox_core::features::kernels::may_remove(&kernel.name, changes, snapshot)
                .map_err(|refusal| crate::packages::kept_package(name, refusal.into()))?;
            changes.want::<Packages>(kernel.name.clone(), Toggle::Off, snapshot);
        }
        KernelChange::Headers if kernel.installed.is_none() => {
            return Err(Refused::Rule(format!("{name} is not installed")));
        }
        KernelChange::Headers if !lacks_headers(kernel, changes) => {
            return Err(Refused::NothingToDo(format!(
                "Nothing to do: the headers of {name} are installed."
            )));
        }
        KernelChange::Headers => {
            changes.want::<Packages>(kernel.headers.clone(), Toggle::On, snapshot);
        }
    }

    Ok(())
}

/// The name to type for a task that reinstalls or wipes.
///
/// # Errors
///
/// [`Refused::Unknown`] for an id the page has no Run button for.
pub fn task(id: &str, upgrader: Upgrader) -> Result<(Plan, Option<String>), Refused> {
    let plan = run_now_plan(id, upgrader).ok_or_else(|| {
        Refused::Unknown(format!("no task is called `{id}`, see `maintenance list`"))
            .hinted(id, super::names::task_ids().iter().map(String::as_str))
    })?;

    Ok((plan, asks_first(id).then(|| id.to_owned())))
}

/// # Errors
///
/// [`Refused::Rule`] with the code `edit` refused with.
pub fn page_edit<E>(
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
    code: impl FnOnce(E) -> &'static str,
    edit: impl FnOnce(&mut ChangeSet, &Snapshot) -> Result<(), E>,
) -> Result<(), Refused> {
    edit(changes, snapshot).map_err(|refusal| Refused::page_rule(code(refusal)))
}

/// The plan of a change built onto an empty queue, for a command that
/// carries it out now.
///
/// # Errors
///
/// What `build` refused.
pub fn fresh(build: impl FnOnce(&mut ChangeSet) -> Result<(), Refused>) -> Result<Plan, Refused> {
    let mut changes = ChangeSet::default();
    build(&mut changes)?;

    Ok(plan(&changes))
}
