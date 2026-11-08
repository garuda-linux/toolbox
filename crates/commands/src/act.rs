//! The changing commands' side that reads the system and talks to the
//! terminal: the plan comes from [`super::change`], the run from
//! [`super::apply`].

#[cfg(test)]
mod tests;

use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_catalog::ToggleCatalog;
use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, EntryArgs, KernelArgs, ReadList, TaskArgs, UnitArgs, UnitChange};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::Toggle;
use toolbox_core::features::kernels::kernels_of;
use toolbox_core::features::maintenance::page::{
    AUR_UPGRADE, OPERATION_KEYS, asks_first, repairs, upgrader,
};
use toolbox_core::features::services::{asks_first as asks_first_unit, critical};
use toolbox_plan::{Plan, UnitOperation, UnitScope};
use toolbox_services::catalogs::Catalogs;
use toolbox_services::programs::Programs;
use toolbox_services::units::{Systemd, UnitLister as _};

use super::change::{self, KernelChange, Refused, UnitLookup};
use super::out::Listing;
use super::terminal::Terminal;
use super::{apply, catalogs, snapshot, system, unavailable};

#[derive(Clone, Serialize)]
struct TaskRow {
    id: String,
    available: bool,
    asks_first: bool,
}

impl super::filter::Named for TaskRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.id]
    }
}

pub fn toggle(args: &EntryArgs, want: Toggle, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let catalogs = catalogs();
    let planned = change::toggle(
        &toggle_pages(&catalogs),
        &args.id,
        want,
        &snapshot(&catalogs),
    );

    run(planned, args.confirm, None, runtime, out)
}

pub fn toggle_pages(catalogs: &Catalogs) -> Vec<&ToggleCatalog> {
    [
        &catalogs.components,
        &catalogs.settings,
        &catalogs.gaming,
        &catalogs.setup,
    ]
    .into_iter()
    .filter_map(|toggles| toggles.catalog.as_ref().ok())
    .collect()
}

pub fn app(args: &EntryArgs, want: Toggle, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let catalogs = catalogs();
    let planned = change::app(&catalogs.apps.apps, &args.id, want, &snapshot(&catalogs));
    run(planned, args.confirm, None, runtime, out)
}

/// A service changed, its state read first so a start of a running unit
/// is refused as the page does.
pub fn service(
    change: UnitChange,
    args: &UnitArgs,
    runtime: &Runtime,
    out: &mut dyn Write,
) -> Exit {
    let scope = UnitScope::from_user(args.user);
    let (lookup, known) = unit_lookup(args, runtime);

    // What the session lives on wants its name typed, as the window asks.
    let asks = match change {
        UnitChange::Stop => asks_first_unit(UnitOperation::Stop, &args.unit),
        UnitChange::Restart => asks_first_unit(UnitOperation::Restart, &args.unit),
        UnitChange::Mask => critical(&args.unit),
        UnitChange::Start
        | UnitChange::Reload
        | UnitChange::Enable
        | UnitChange::Disable
        | UnitChange::Unmask => false,
    };

    let planned = change::service(change, scope, &args.unit, lookup, &system())
        .map_err(|refused| refused.hinted(args.unit.as_str(), known.iter().map(String::as_str)));

    run(
        planned,
        args.confirm,
        asks.then_some(args.unit.as_str()),
        runtime,
        out,
    )
}

/// The unit names are for a hint at a typo.
pub fn unit_lookup(args: &UnitArgs, runtime: &Runtime) -> (UnitLookup, Vec<String>) {
    let listed = runtime.block_on(Systemd.list(UnitScope::from_user(args.user)));
    let known = listed
        .iter()
        .flat_map(|units| units.iter())
        .map(|unit| unit.name.as_str().to_owned())
        .collect();

    let lookup = match &listed {
        Ok(units) => units
            .iter()
            .find(|unit| unit.name == args.unit)
            .map_or(UnitLookup::Missing, |unit| UnitLookup::Found(unit.active)),
        Err(error) => {
            tracing::warn!("systemd's units could not be read: {error}");
            UnitLookup::Unread
        }
    };

    (lookup, known)
}

pub fn kernel(
    args: &KernelArgs,
    change: KernelChange,
    runtime: &Runtime,
    out: &mut dyn Write,
) -> Exit {
    let snapshot = system();
    let Some((kernels, _)) = kernels_of(&snapshot) else {
        return unavailable("pacman's databases could not be read", out);
    };

    let planned = change::kernel(&kernels, &args.kernel, change, &snapshot);
    run(planned, args.confirm, None, runtime, out)
}

pub fn task(args: &TaskArgs, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let snapshot = system();
    let found = Programs::check().garuda_update;
    match change::task(&args.task, upgrader(&snapshot, found)) {
        Ok((plan, typed)) => run(Ok(plan), args.confirm, typed.as_deref(), runtime, out),
        Err(refused) => run(Err(refused), args.confirm, None, runtime, out),
    }
}

pub fn tasks(read: &ReadList, out: &mut dyn Write) -> Exit {
    let exists = |path: &Path| path.exists();
    let common = OPERATION_KEYS
        .into_iter()
        .chain([AUR_UPGRADE])
        .map(|id| TaskRow {
            id: id.to_owned(),
            available: true,
            asks_first: false,
        });

    let rows: Vec<TaskRow> = common
        .chain(repairs(exists).into_iter().map(|row| TaskRow {
            asks_first: asks_first(&row.repair),
            id: row.repair,
            available: row.available,
        }))
        .collect();

    super::filter::print_all(read, &rows, out, task_listing)
}

fn task_listing(rows: &[TaskRow]) -> Listing<'_> {
    let available = rows.iter().filter(|row| row.available).count();
    let summary = format!("{}, {available} available", rows.len());
    let rows = rows
        .iter()
        .map(|row| {
            let (mark, note) = match (row.available, row.asks_first) {
                (false, _) => (Mark::Off, "not available"),
                (true, true) => (Mark::On, "asks first"),
                (true, false) => (Mark::On, ""),
            };

            Row::new(mark, vec![row.id.clone(), note.to_owned()])
        })
        .collect();

    Listing {
        title: "Maintenance tasks",
        summary: Some(summary),
        head: &[],
        rows,
        hint: Some("garuda-toolbox maintenance run <task>".to_owned()),
    }
}

pub fn run(
    planned: Result<Plan, Refused>,
    confirm: Confirm,
    typed: Option<&str>,
    runtime: &Runtime,
    out: &mut dyn Write,
) -> Exit {
    match planned {
        Ok(plan) => apply::carry_out(
            plan,
            confirm,
            typed,
            runtime.handle(),
            super::root::launcher(),
            &mut Terminal,
            out,
        ),
        Err(Refused::NothingToDo(_)) if confirm.json => {
            write!(out, "{}", apply::nothing_lines()).ok();
            Exit::Success
        }
        Err(refused) if confirm.json => {
            write!(out, "{}", apply::error_line(refused.message(), None)).ok();
            refused.exit()
        }
        Err(refused) => {
            writeln!(out, "{}", refused.line(crate::paint())).ok();
            refused.exit()
        }
    }
}
