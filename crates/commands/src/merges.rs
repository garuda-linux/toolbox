//! `merges`: whole-file choices only. Picking single changes is the
//! window's.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{MergeResolveArgs, MergesCommand, ReadList};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::domains::MergeCopies;
use toolbox_core::features::configs::Diff;
use toolbox_core::features::merges::{self, Standing, choice, standing};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{ConfigFilePath, ConfigText, MergeCopy, Resolution};
use toolbox_system::{MergePair, MergeSource as _, Pacdiff, SystemConfigFiles, read_pair};
use toolbox_text::OutsideText;

use crate::act::run;
use crate::change::{Refused, fresh};
use crate::configs::read_problem;
use crate::names::NoHelper;
use crate::out::{self, Listing, Out};
use crate::{filter, system, unavailable};

/// The state of a copy the toolbox never opens, such as one of the
/// password hashes.
const TERMINAL_ONLY: &str = "merge it in a terminal";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MergeRow {
    #[serde(serialize_with = "toolbox_text::serialize_text")]
    pub copy: Result<MergeCopy, OutsideText>,
    /// `None` for a copy the toolbox never opens.
    #[serde(serialize_with = "serialize_file")]
    pub file: Option<ConfigFilePath>,
    pub kind: String,
    pub state: String,
    #[serde(skip)]
    pub unread: bool,
}

impl filter::Named for MergeRow {
    fn names(&self) -> Vec<&str> {
        vec![
            toolbox_text::checked_text(&self.copy),
            self.file.as_ref().map_or("", ConfigFilePath::as_str),
        ]
    }
}

/// The file of a copy, `""` for a copy the toolbox never opens.
#[expect(
    clippy::ref_option,
    reason = "serde hands a field over by reference, so the option cannot be taken by value"
)]
fn serialize_file<S>(file: &Option<ConfigFilePath>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(file.as_ref().map_or("", ConfigFilePath::as_str))
}

pub fn command(command: &MergesCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let helped = SystemConfigFiles::system(Arc::new(crate::root::helper(out.settings())));

    match command {
        MergesCommand::List(read) => list(read, out),
        MergesCommand::Show(args) => show(&read_pair(&helped, &args.copy, &uncancelled()), out),
        MergesCommand::Resolve(args) => {
            let pair = read_pair(&helped, &args.copy, &uncancelled());
            let snapshot = system();
            let planned = fresh(|changes| onto(args, &pair, &snapshot, changes));

            run(planned, args.confirm, None, runtime, out)
        }
    }
}

/// # Errors
///
/// [`Refused`] for a copy that does not wait, files that could not be
/// read, and `--take-new` where only removing the copy makes sense.
pub fn onto(
    args: &MergeResolveArgs,
    pair: &MergePair,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let copy = &args.copy;

    if snapshot
        .part::<MergeCopies>()
        .is_some_and(|waiting| !waiting.0.contains(copy))
    {
        return Err(Refused::Unknown(format!(
            "{copy} does not wait for a merge\ngaruda-toolbox merges list"
        )));
    }

    let (copied, current) = texts(pair).map_err(|(what, raw)| {
        Refused::Unavailable(if raw.is_empty() {
            what
        } else {
            format!("{what}: {raw}")
        })
    })?;

    let resolution = if args.how.keep {
        Resolution::KeepCurrent
    } else {
        taken(copy, current, copied)?
    };

    merges::queue(
        changes,
        copy.clone(),
        choice(resolution, current, copied),
        snapshot,
    );
    Ok(())
}

/// One row per path `pacdiff -o` printed, read without a password: a
/// file only root may read shows as not read.
pub fn rows(paths: &[PathBuf], read: &dyn Fn(&MergeCopy) -> MergePair) -> Vec<MergeRow> {
    paths
        .iter()
        .map(|path| {
            let shown = path.display().to_string();

            match MergeCopy::new(shown.as_str()) {
                Ok(copy) => {
                    let pair = read(&copy);
                    MergeRow {
                        file: Some(copy.original().clone()),
                        kind: copy.kind().as_str().to_owned(),
                        state: state(&pair),
                        unread: texts(&pair).is_err(),
                        copy: Ok(copy),
                    }
                }
                Err(_withheld) => MergeRow {
                    copy: Err(toolbox_text::plain(&shown)),
                    file: None,
                    kind: String::new(),
                    state: TERMINAL_ONLY.to_owned(),
                    unread: false,
                },
            }
        })
        .collect()
}

fn taken(copy: &MergeCopy, current: Option<&str>, copied: &str) -> Result<Resolution, Refused> {
    match standing(copy, current, copied) {
        Standing::OlderSave => Err(Refused::Rule(format!(
            "{copy} is an older save, which can only be removed\nuse --keep"
        ))),
        Standing::Same => Err(Refused::Rule(format!(
            "{copy} is the same as {}\nuse --keep",
            copy.original()
        ))),
        Standing::Differs { changes: _ } | Standing::CurrentGone => Ok(Resolution::TakeCopy),
    }
}

fn texts(pair: &MergePair) -> Result<(&str, Option<&str>), (String, String)> {
    let copied = pair.copied.as_ref().map_err(read_problem)?;
    let current = pair.current.as_ref().map_err(read_problem)?;

    Ok((copied.as_str(), current.as_ref().map(ConfigText::as_str)))
}

fn state(pair: &MergePair) -> String {
    let Ok((copied, current)) = texts(pair) else {
        return "not read, only root may read it".to_owned();
    };

    match standing(&pair.copy, current, copied) {
        Standing::Differs { changes: 1 } => "1 change".to_owned(),
        Standing::Differs { changes } => format!("{changes} changes"),
        Standing::Same => "same as the file".to_owned(),
        Standing::CurrentGone => "the file is gone".to_owned(),
        Standing::OlderSave => "older save".to_owned(),
    }
}

fn list(read: &ReadList, out: &mut Out<'_>) -> Exit {
    let paths = match Pacdiff::new().waiting() {
        Ok(paths) => paths,
        Err(error) => return unavailable(&format!("pacdiff could not list them: {error}"), out),
    };

    let source = SystemConfigFiles::system(NoHelper);
    let rows = rows(&paths, &|copy| read_pair(&source, copy, &uncancelled()));

    filter::print_all(read, &rows, out, listing)
}

fn listing(rows: &[MergeRow]) -> Listing<'_> {
    Listing {
        title: "Configuration files to merge",
        summary: Some(rows.len().to_string()),
        head: &["COPY", "FILE", "STATE"],
        rows: rows
            .iter()
            .map(|row| {
                let mark = if row.unread {
                    Mark::Unknown
                } else {
                    Mark::Blank
                };

                Row::new(
                    mark,
                    vec![
                        toolbox_text::checked_text(&row.copy).to_owned(),
                        row.file
                            .as_ref()
                            .map_or_else(String::new, ToString::to_string),
                        row.state.clone(),
                    ],
                )
            })
            .collect(),
        hint: Some("garuda-toolbox merges show <copy>".to_owned()),
    }
}

fn show(pair: &MergePair, out: &mut Out<'_>) -> Exit {
    let (copied, current) = match texts(pair) {
        Ok(texts) => texts,
        Err((what, raw)) => {
            out::failure(out, &what, &raw);
            return Exit::Failure;
        }
    };

    let copy = &pair.copy;
    let file = copy.original();
    let text = match current {
        Some(current) if current == copied => format!("{copy} is the same as {file}\n"),
        Some(current) => format!(
            "--- {file}\n+++ {copy}\n{}",
            Diff::new(current, copied).plain()
        ),
        None => format!("{file} is gone, {copy} holds:\n{copied}"),
    };

    write!(out, "{}", OutsideText::keep_lines(&text)).ok();
    Exit::Success
}

fn uncancelled() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
