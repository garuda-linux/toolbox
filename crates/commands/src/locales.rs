#[cfg(test)]
mod tests;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{LocaleArgs, LocalesCommand, PackArgs, PacksCommand, ReadList};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::domains::{InstalledPackages, Packages, SyncPackages};
use toolbox_core::features::locales::{
    all_rows, choose_system, language_packs, toggle, wanted_enabled,
};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::Plan;

use super::act::run;
use super::change::{Refused, fresh, page_edit, why};
use super::out::Listing;
use super::{locales, system, unavailable};
use crate::out::Out;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackRow {
    #[serde(rename = "id")]
    pub package: String,
    pub program: String,
    pub locale: String,
    pub installed: bool,
}

pub fn command(command: LocalesCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    // The list reads the system behind its own spinner.
    if let LocalesCommand::List(list) = command {
        return locales(&list, out);
    }

    let snapshot = system();
    match command {
        LocalesCommand::List(list) => locales(&list, out),
        LocalesCommand::Enable(args) => run(
            switch(&args, Toggle::On, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        LocalesCommand::Disable(args) => run(
            switch(&args, Toggle::Off, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        LocalesCommand::System(args) => run(
            system_locale(&args, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        LocalesCommand::Packs(PacksCommand::List(read)) => packs(&read, &snapshot, out),
        LocalesCommand::Packs(PacksCommand::Install(args)) => run(
            pack(&args, Toggle::On, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
        LocalesCommand::Packs(PacksCommand::Remove(args)) => run(
            pack(&args, Toggle::Off, &snapshot),
            args.confirm,
            None,
            runtime,
            out,
        ),
    }
}

/// # Errors
///
/// [`Refused`] for a locale glibc does not offer here, or the system's
/// own one switched off.
pub fn switch(args: &LocaleArgs, want: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| switch_onto(args, want, snapshot, changes))
}

/// # Errors
///
/// As [`switch`].
pub fn switch_onto(
    args: &LocaleArgs,
    want: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let entry = all_rows(snapshot, changes)
        .into_iter()
        .find(|row| row.entry.locale == args.locale)
        .map(|row| row.entry)
        .ok_or_else(|| {
            Refused::Unknown(format!(
                "`{}` is no locale this system offers, see `locales list`",
                args.locale
            ))
            .hinted(
                args.locale.as_str(),
                super::names::locale_names(snapshot)
                    .iter()
                    .map(String::as_str),
            )
        })?;

    page_edit(changes, snapshot, why::locale, |changes, snapshot| {
        toggle(&entry, want, changes, snapshot)
    })
}

/// The system's language, enabled too when it is not yet.
///
/// # Errors
///
/// [`Refused`] for a name that is no locale or one the page refuses.
pub fn system_locale(args: &LocaleArgs, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| system_locale_onto(args, snapshot, changes))
}

/// # Errors
///
/// As [`system_locale`].
pub fn system_locale_onto(
    args: &LocaleArgs,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let locale = args.locale.clone();

    page_edit(changes, snapshot, why::locale, |changes, snapshot| {
        choose_system(locale, changes, snapshot)
    })
}

/// # Errors
///
/// [`Refused::Unknown`] for a package that is no pack of an enabled
/// locale.
pub fn pack(args: &PackArgs, want: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    let known = pack_rows(snapshot)
        .is_some_and(|rows| rows.iter().any(|row| row.package == args.package.as_str()));

    let package = known.then(|| args.package.clone()).ok_or_else(|| {
        Refused::Unknown(format!(
            "`{}` is no language pack here, see `locales packs list`",
            args.package
        ))
        .hinted(
            args.package.as_str(),
            pack_rows(snapshot)
                .iter()
                .flatten()
                .map(|row| row.package.as_str()),
        )
    })?;

    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package, want, snapshot);
    Ok(toolbox_core::plan(&changes))
}

/// The packs of the enabled locales, `None` without pacman's databases.
pub fn pack_rows(snapshot: &Snapshot) -> Option<Vec<PackRow>> {
    let offered = snapshot.part::<SyncPackages>()?;
    let installed = snapshot.part::<InstalledPackages>()?;
    let enabled = wanted_enabled(snapshot, &ChangeSet::default());

    Some(
        language_packs(&enabled, offered, installed)
            .into_iter()
            .map(|pack| PackRow {
                installed: installed.0.contains(&pack.package),
                package: pack.package.as_str().to_owned(),
                program: pack.base.as_str().to_owned(),
                locale: pack.locale.as_str().to_owned(),
            })
            .collect(),
    )
}

fn packs(read: &ReadList, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    match pack_rows(snapshot) {
        Some(rows) => super::filter::print_all(read, &rows, out, pack_listing),
        None => unavailable("pacman's databases could not be read", out),
    }
}

/// The packs, on when installed, with the command that installs the
/// rest.
pub fn pack_listing(rows: &[PackRow]) -> Listing<'_> {
    let installed = rows.iter().filter(|row| row.installed).count();
    let hint = if installed < rows.len() {
        Some("garuda-toolbox locales packs install <package>")
    } else if rows.is_empty() {
        None
    } else {
        Some("garuda-toolbox locales packs remove <package>")
    };

    Listing {
        title: "Language packs",
        summary: Some(format!("{}, {installed} installed", rows.len())),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let (mark, state) = if row.installed {
                    (Mark::On, "installed")
                } else {
                    (Mark::Off, "")
                };

                let cells = [row.package.as_str(), &row.locale, state];
                Row::new(mark, cells.map(str::to_owned).to_vec())
            })
            .collect(),
        hint: hint.map(str::to_owned),
    }
}
