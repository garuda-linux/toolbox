//! `maintenance reset`: programs' settings put back from /etc/skel, as
//! the Maintenance page's settings reset does. The replaced files stay
//! as backups beside them.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::path::Path;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_catalog::{SKEL_ROOT, SkelCatalog, SkelEntry};
use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, ReadList, ResetArgs, ResetList};
use toolbox_cli::confirm::{Prompt, consent};
use toolbox_cli::output::{Mark, Row, json};
use toolbox_config::Paths;
use toolbox_system::skel::shelf::{Outcome, SkelRow, offered, reset};

use crate::act::run;
use crate::change::Refused;
use crate::out::{Listing, Out, unavailable};
use crate::terminal::Terminal;
use crate::{catalogs, filter};

/// Where the settings come from and where they go.
pub struct Folders<'a> {
    pub skel_root: &'a Path,
    pub home: &'a Path,
}

#[derive(Clone, Serialize)]
struct ResetRow {
    id: String,
    name: String,
    /// The files in the home folder a reset replaces.
    replaces: Vec<String>,
}

#[derive(Serialize)]
struct Done<'a> {
    copied: usize,
    failed: &'a [String],
}

impl filter::Named for ResetRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.id, &self.name]
    }
}

pub fn command(args: &ResetArgs, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let Some(catalog) = catalogs().skel else {
        return unavailable("the settings that can be reset could not be read", out);
    };

    let paths = Paths::from_process();
    let folders = Folders {
        skel_root: Path::new(SKEL_ROOT),
        home: paths.home(),
    };

    match &args.list {
        Some(ResetList::List(read)) => list(read, &catalog, folders.skel_root, out),
        None => {
            let mut terminal = Terminal {
                color: out.settings().color,
            };

            let chosen = chosen(&args.entries, &catalog, folders.skel_root);
            match chosen {
                Ok(entries) => carry_out(&entries, args.confirm, &folders, &mut terminal, out),
                Err(refused) => run(Err(refused), args.confirm, None, runtime, out),
            }
        }
    }
}

fn rows(catalog: &SkelCatalog, skel_root: &Path) -> Vec<ResetRow> {
    offered(catalog, skel_root)
        .into_iter()
        .map(|row: SkelRow| ResetRow {
            id: row.entry,
            name: row.name,
            replaces: row.targets,
        })
        .collect()
}

/// The ids `maintenance reset` takes, for completion.
pub fn ids(catalog: &SkelCatalog, skel_root: &Path) -> Vec<String> {
    rows(catalog, skel_root)
        .into_iter()
        .map(|row| row.id)
        .collect()
}

fn list(read: &ReadList, catalog: &SkelCatalog, skel_root: &Path, out: &mut Out<'_>) -> Exit {
    filter::print_all(read, &rows(catalog, skel_root), out, |rows| Listing {
        title: "Settings to reset",
        summary: Some(rows.len().to_string()),
        head: &[],
        rows: rows
            .iter()
            .map(|row| Row::new(Mark::On, vec![row.id.clone(), row.replaces.join(", ")]))
            .collect(),
        hint: Some("garuda-toolbox maintenance reset <id>...".to_owned()),
    })
}

/// The entries `ids` name, each once, in the order given.
///
/// # Errors
///
/// [`Refused::Unknown`] for an id the list lacks.
fn chosen(
    ids: &[String],
    catalog: &SkelCatalog,
    skel_root: &Path,
) -> Result<Vec<SkelEntry>, Refused> {
    let rows = rows(catalog, skel_root);
    let mut entries: Vec<SkelEntry> = Vec::new();

    for id in ids {
        let offered = rows.iter().any(|row| row.id == *id);
        let Some(entry) = catalog.entry(id).filter(|_entry| offered) else {
            let text = format!("no settings are called `{id}`, see `maintenance reset list`");
            return Err(Refused::Unknown(text).hinted(id, rows.iter().map(|row| row.id.as_str())));
        };

        if !entries.iter().any(|known| known.id() == entry.id()) {
            entries.push(entry.clone());
        }
    }

    Ok(entries)
}

fn carry_out(
    entries: &[SkelEntry],
    confirm: Confirm,
    folders: &Folders<'_>,
    prompt: &mut dyn Prompt,
    out: &mut Out<'_>,
) -> Exit {
    let replaced: Vec<String> = entries
        .iter()
        .flat_map(|entry| {
            entry
                .files()
                .iter()
                .map(|file| format!("~/{}", file.display()))
        })
        .collect();

    if !confirm.json {
        let paint = out.paint();
        writeln!(out, "{}", paint.title("Reset settings", None)).ok();
        for file in &replaced {
            writeln!(out, "{}", paint.item(file)).ok();
        }

        writeln!(
            out,
            "{}",
            paint.hint("Each replaced file is kept as a backup beside it.")
        )
        .ok();
    }

    if confirm.dry_run {
        return Exit::Success;
    }

    if let Some(refused) = consent(prompt, confirm.yes, None).exit() {
        return refused;
    }

    let outcome = reset(entries, folders.skel_root, folders.home);
    report(&outcome, confirm.json, out)
}

fn report(outcome: &Outcome, as_json: bool, out: &mut Out<'_>) -> Exit {
    let exit = if outcome.failed.is_empty() {
        Exit::Success
    } else {
        Exit::Failure
    };

    if as_json {
        let done = Done {
            copied: outcome.copied,
            failed: &outcome.failed,
        };

        writeln!(out, "{}", json(&done)).ok();
        return exit;
    }

    let paint = out.paint();
    if outcome.failed.is_empty() {
        writeln!(
            out,
            "{}",
            paint.item(&format!("Reset {} files.", outcome.copied))
        )
        .ok();
    } else {
        let names = outcome.failed.join(", ");
        writeln!(out, "{}", paint.error(&format!("could not reset: {names}"))).ok();
    }

    exit
}
