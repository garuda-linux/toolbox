//! `garuda-toolbox queue`: the window's queue on the command line. While
//! the window runs, every edit goes to it over the session bus and it
//! queues the change as a click would. Otherwise the command edits
//! `changes.json`, which the window restores at its next start.

mod add;
mod build;
mod ready;
mod said;
mod show;
mod store;
#[cfg(test)]
mod tests;

use std::io::Write as _;

use tokio::runtime::Runtime;
use toolbox_aur::{AurSource, Cache};
use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, QueueAdd, QueueCommand, Read};
use toolbox_cli::confirm::consent;
use toolbox_cli::output::json;
use toolbox_config::Paths;
use toolbox_core::features::Unchecked;
use toolbox_core::{ChangeSet, Owners, SavedChanges, Snapshot, Summary, domains, plan, planned};
use toolbox_plan::{PlannedAction, StepId};
use toolbox_services::catalogs::Catalogs;
use toolbox_services::queue::edit::QueueEdit;

use self::said::{changes, done, say, waiting, words};
use self::show::Unpicked;
use self::store::{Problem, Store};
use super::change::Refused;
use super::out::write_listing;
use super::terminal::Terminal;
use super::{apply, catalogs, root, snapshot, spinner};
use crate::out::Out;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failed {
    /// The change's own command would refuse it the same way.
    Refused(Refused),
    Unpicked(Unpicked),
    Store(Problem),
    Nothing,
}

#[derive(Clone, Copy)]
struct Context<'a> {
    store: &'a dyn Store,
    catalogs: &'a Catalogs,
    system: &'a Snapshot,
    runtime: &'a Runtime,
    /// Read only by an AUR install, which may download once a day.
    aur: &'a dyn AurSource,
}

pub fn command(command: QueueCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let quiet = matches!(&command, QueueCommand::Show(read) if read.json);
    let catalogs = catalogs();
    let system = spinner::while_reading(quiet || out.quiet(), || snapshot(&catalogs));
    let path = Paths::from_process().staged_changes_file();

    let store = match store::open(runtime.handle(), path, out.paint()) {
        Ok(store) => store,
        Err(problem) => return Failed::Store(problem).say(out),
    };

    let aur = Cache::system(Paths::from_process().aur_dir());
    let context = Context {
        store: &*store,
        catalogs: &catalogs,
        system: &system,
        runtime,
        aur: &aur,
    };

    match command {
        QueueCommand::Show(read) => show(&*store, &system, read, out),
        QueueCommand::Add(args) => add(&args, &context, out),
        QueueCommand::Remove(args) => {
            match remove_change(&*store, &system, &args.change, words()) {
                Ok(left) => done(&format!("Taken back, {}", waiting(left)), &*store, out),
                Err(failed) => failed.say(out),
            }
        }
        QueueCommand::Clear(args) => {
            if let Some(exit) = consent(
                &mut Terminal {
                    color: out.settings().color,
                },
                args.yes,
                None,
            )
            .exit()
            {
                return exit;
            }

            match clear_all(&*store) {
                Ok(count) => done(&format!("Took back {}", changes(count)), &*store, out),
                Err(failed) => failed.say(out),
            }
        }
        QueueCommand::Apply(confirm) => apply_queue(confirm, &context, out),
    }
}

/// The queue as the window would show it against `snapshot`, after every
/// page's guard.
pub fn restored(saved: &SavedChanges, snapshot: &Snapshot) -> ChangeSet {
    let mut changes = restore(saved, snapshot);
    Unchecked::all().recheck(&mut changes, snapshot);
    changes
}

/// The queue `saved` against `snapshot`, before any page's guard.
fn restore(saved: &SavedChanges, snapshot: &Snapshot) -> ChangeSet {
    let mut changes = ChangeSet::default();
    match domains::all() {
        Ok(domains) => {
            for problem in domains.restore(&mut changes, saved, snapshot) {
                tracing::warn!("dropped a queued change: {problem}");
            }
        }
        Err(error) => tracing::error!("the change domains are broken: {error}"),
    }

    changes
}

/// # Errors
///
/// [`Failed`] for a refused change, one the queue has already, or a queue
/// that did not take it.
pub fn add_change(
    store: &dyn Store,
    snapshot: &Snapshot,
    build: impl FnOnce(&mut ChangeSet) -> Result<(), Refused>,
) -> Result<usize, Failed> {
    let saved = store.saved().map_err(Failed::Store)?;
    let before = restored(&saved, snapshot);
    let mut after = restored(&saved, snapshot);
    build(&mut after).map_err(Failed::Refused)?;

    edit(store, &QueueEdit::between(&before, &after))?;
    Ok(after.len())
}

/// # Errors
///
/// [`Failed`] for no such change or a queue that did not take it back.
pub fn remove_change(
    store: &dyn Store,
    snapshot: &Snapshot,
    selector: &str,
    words: impl Fn(&Summary) -> String,
) -> Result<usize, Failed> {
    let changes = restored(&store.saved().map_err(Failed::Store)?, snapshot);
    let rows = show::rows(&changes.views(), words);
    let picked = show::pick(&rows, selector).map_err(Failed::Unpicked)?;

    edit(store, &QueueEdit::removing(picked))?;
    Ok(rows.len() - 1)
}

/// Takes back every change, also those the system fulfils by now.
///
/// # Errors
///
/// [`Failed::Nothing`] for an empty queue, or a queue that did not take
/// it back.
pub fn clear_all(store: &dyn Store) -> Result<usize, Failed> {
    let changes = restored(&store.saved().map_err(Failed::Store)?, &Snapshot::default());
    let removals = changes
        .views()
        .into_iter()
        .map(toolbox_services::queue::edit::Removal::of)
        .collect();

    edit(store, &QueueEdit::removing(removals))?;
    Ok(changes.len())
}

/// After a run, as the window does: each change the plan of `owners`
/// carried out learns how it went from `ended_well`. What `after` shows
/// done and its plan finished leaves the queue, and the one-off work
/// whose actions are in `done`, the succeeded steps.
///
/// # Errors
///
/// [`Failed::Store`] when the queue could not be written.
pub fn settle(
    store: &dyn Store,
    after: &Snapshot,
    owners: &Owners,
    ended_well: impl Fn(StepId) -> bool,
    done: &[PlannedAction],
) -> Result<(), Failed> {
    let saved = store.saved().map_err(Failed::Store)?;
    let before = restored(&saved, &Snapshot::default());
    let mut left = restored(&saved, &Snapshot::default());
    left.finished(owners, ended_well);
    left.reconcile(after);
    left.drop_done_one_offs(done);

    match edit(store, &QueueEdit::between(&before, &left)) {
        Ok(()) | Err(Failed::Nothing) => Ok(()),
        Err(failed) => Err(failed),
    }
}

fn edit(store: &dyn Store, edit: &QueueEdit) -> Result<(), Failed> {
    if edit.is_empty() {
        return Err(Failed::Nothing);
    }

    store.edit(edit).map_err(Failed::Store)
}

fn add(args: &QueueAdd, context: &Context<'_>, out: &mut Out<'_>) -> Exit {
    let Context {
        store,
        catalogs,
        system,
        runtime,
        aur,
    } = *context;

    let command = match args.command() {
        Ok(command) => command,
        Err(error) => {
            error.print().ok();
            return Exit::Usage;
        }
    };

    let change = match add::queueable(&command) {
        Ok(change) => change,
        Err(not) => {
            let (error, hint) = not.words();
            say(out, error, hint);
            return Exit::Usage;
        }
    };

    if let Some(exit) = change.typed().and_then(|name| {
        consent(
            &mut Terminal {
                color: out.settings().color,
            },
            false,
            Some(&name),
        )
        .exit()
    }) {
        return exit;
    }

    let added = add_change(store, system, |changes| {
        let sources = build::Sources {
            catalogs,
            runtime,
            aur,
        };

        build::build(&change, &sources, system, changes)
    });

    match added {
        Ok(count) => done(&format!("Queued, {}", waiting(count)), store, out),
        Err(failed) => failed.say(out),
    }
}

fn show(store: &dyn Store, system: &Snapshot, read: Read, out: &mut Out<'_>) -> Exit {
    let saved = match store.saved() {
        Ok(saved) => saved,
        Err(problem) => return Failed::Store(problem).say(out),
    };

    let changes = restored(&saved, system);
    let rows = show::rows(&changes.views(), words());
    let plan = plan(&changes);

    if read.json {
        let steps = plan
            .steps()
            .iter()
            .map(|step| step.action.command_lines().join(" "))
            .collect();

        write!(
            out,
            "{}",
            json(&show::Shown {
                changes: &rows,
                steps
            })
        )
        .ok();
        return Exit::Success;
    }

    let mut listing = show::listing(&rows, store.in_window());
    let hint = listing.hint.take();
    let exit = write_listing(listing, out);
    if !plan.steps().is_empty() {
        write!(out, "\n{}", apply::planned(out.paint(), &plan)).ok();
    }

    if let Some(hint) = hint {
        writeln!(out, "\n{}", out.paint().hint(&hint)).ok();
    }

    exit
}

/// The window's queue belongs to the window, so a change never runs twice.
fn apply_queue(confirm: Confirm, context: &Context<'_>, out: &mut Out<'_>) -> Exit {
    let Context {
        store,
        catalogs,
        system,
        runtime,
        aur,
    } = *context;

    if store.in_window() {
        say(
            out,
            "the window holds the queue, so only the window applies it",
            "apply it in the window, or close the window",
        );
        return Exit::Unavailable;
    }

    let saved = match store.saved() {
        Ok(saved) => saved,
        Err(problem) => return Failed::Store(problem).say(out),
    };

    let reads = ready::SystemReads {
        color: out.settings().color,
        aur,
    };

    let changes = match ready::checked(&saved, system, &reads) {
        Ok(changes) => changes,
        Err(refusal) => return ready::refused(refusal, out),
    };

    let (planned, owners) = planned(&changes);
    let ended = apply::carry_out_steps(
        planned.clone(),
        confirm,
        None,
        runtime.handle(),
        root::launcher(out.settings().color),
        &mut Terminal {
            color: out.settings().color,
        },
        out,
    );

    let exit = ended.exit;
    let succeeded: Vec<PlannedAction> = planned
        .steps()
        .iter()
        .filter(|step| ended.succeeded.contains(&step.id))
        .map(|step| step.action.clone())
        .collect();

    let ran = !confirm.dry_run && !matches!(exit, Exit::Declined | Exit::NeedsYes | Exit::Usage);
    if ran
        && let Err(failed) = settle(
            store,
            &snapshot(catalogs),
            &owners,
            |step| ended.ended_well(step),
            &succeeded,
        )
    {
        failed.say(out);
    }

    exit
}
