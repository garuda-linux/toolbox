//! The Setup Assistant's commands: its choices are switches of its own
//! catalog, and the summary turns several on in one plan.

#[cfg(test)]
mod tests;

use std::io::Write;

use tokio::runtime::Runtime;
use toolbox_catalog::ToggleCatalog;
use toolbox_cli::Exit;
use toolbox_cli::args::{SetupCommand, ToggleList, TogglePage};
use toolbox_core::features::toggles;
use toolbox_core::{ChangeSet, Snapshot, Toggle, plan};
use toolbox_plan::Plan;

use super::act::run;
use super::change::Refused;
use super::{catalogs, snapshot, toggle};

pub fn command(command: SetupCommand, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    match command {
        SetupCommand::List(read) => toggle(
            &ToggleList {
                page: Some(TogglePage::Setup),
                on: false,
                off: false,
                read,
            },
            out,
        ),
        SetupCommand::Apply(args) => {
            let catalogs = catalogs();
            let planned = match catalogs.setup.catalog.as_ref() {
                Ok(catalog) => apply(catalog, &args.ids, &snapshot(&catalogs)),
                Err(error) => Err(Refused::Rule(format!(
                    "the Setup Assistant's catalog is unusable: {}",
                    error.detail
                ))),
            };

            run(planned, args.confirm, None, runtime, out)
        }
    }
}

/// # Errors
///
/// [`Refused`] for an id the assistant does not offer, or a choice the
/// page refuses.
pub fn apply(
    catalog: &ToggleCatalog,
    ids: &[String],
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    let mut changes = ChangeSet::default();
    for id in ids {
        if catalog.entry(id).is_none() {
            return Err(Refused::Unknown(format!(
                "the Setup Assistant offers no `{id}`, see `setup list`"
            )));
        }

        toggles::toggle(catalog, id, Toggle::On, snapshot, &mut changes).map_err(|refusal| {
            Refused::Rule(format!(
                "the page refuses `{id}` ({}), see the window for why",
                refusal.as_str()
            ))
        })?;
    }

    Ok(plan(&changes))
}
