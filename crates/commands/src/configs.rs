//! The Config Files page's commands.
//!
//! Every edit works on a private copy in the user's editor, right in this
//! terminal. The changes are shown, and a yes writes them: a root-owned
//! file through garuda-toolbox-helper, a file of the user as the user.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_catalog::{ConfigFileCatalog, ConfigSection};
use toolbox_cli::Exit;
use toolbox_cli::args::{ConfigEdit, ConfigsCommand, ReadList};
use toolbox_cli::output::Row;
use toolbox_cli::suggest::did_you_mean;
use toolbox_core::features::configs::session::EditedFile;
use toolbox_core::features::configs::{
    Diff, Listing, RowKind, Target, choose_editor, rows, target, view_target, write_plan,
};
use toolbox_plan::{ConfigText, EditCopy, Editor};
use toolbox_services::catalogs::catalog_dir;
use toolbox_services::configs::specs;
use toolbox_system::{ConfigFileSource, EditFolder, ReadProblem, SystemConfigFiles};

use super::act::run;
use super::out::{self, Listing as Shown};
use super::unavailable;
use crate::out::Out;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileRow {
    #[serde(rename = "id")]
    pub key: String,
    pub path: String,
    pub name: String,
    pub admin: bool,
    pub read_only: bool,
}

/// The private copy of a root-owned file, removed however the edit ends,
/// a panic included.
struct Removed<'edit> {
    copies: &'edit EditFolder,
    copy: EditCopy,
}

impl Drop for Removed<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.copies.remove(&self.copy) {
            tracing::warn!(
                "the private copy {} could not be removed: {error}",
                self.copy.as_str()
            );
        }
    }
}

pub fn command(command: ConfigsCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let Ok(catalog) = ConfigFileCatalog::load(&catalog_dir(|key| std::env::var_os(key))) else {
        return unavailable("the config files catalog could not be read", out);
    };

    let source = SystemConfigFiles::system(Arc::new(super::root::helper(out.settings().color)));
    let listing = source.listing(&specs(&catalog));

    match command {
        ConfigsCommand::List(read) => list(&read, &catalog, &listing, out),
        ConfigsCommand::Show(args) => show(&args.key, &catalog, &listing, &source, out),
        ConfigsCommand::Edit(args) => edit(&args, &catalog, &listing, &source, runtime, out),
    }
}

pub fn files(catalog: &ConfigFileCatalog, listing: &Listing) -> Vec<FileRow> {
    let expanded = catalog
        .sections()
        .iter()
        .flat_map(ConfigSection::entries)
        .map(|entry| entry.id().to_owned())
        .collect();

    rows(catalog, listing, &expanded, "")
        .into_iter()
        .filter(|row| {
            !row.path.is_empty() && row.kind != RowKind::Folder && row.kind != RowKind::Section
        })
        .map(|row| FileRow {
            key: row.key,
            path: row.path,
            name: row.name,
            admin: row.admin,
            read_only: row.read_only,
        })
        .collect()
}

/// The write-back plan of an edit that changed something, `None` when
/// the copy is as the file was.
pub fn write_back(
    file: &EditedFile,
    original: &ConfigText,
    edited: ConfigText,
) -> Option<toolbox_plan::Plan> {
    (original != &edited).then(|| write_plan(file.write(edited, original.fingerprint())))
}

fn list(
    list: &ReadList,
    catalog: &ConfigFileCatalog,
    listing: &Listing,
    out: &mut Out<'_>,
) -> Exit {
    super::filter::print_all(list, &files(catalog, listing), out, file_listing)
}

/// The files as a list with a header, since a key and a path look alike.
fn file_listing(rows: &[FileRow]) -> Shown<'_> {
    let note = |row: &FileRow| match (row.read_only, row.admin) {
        (true, _) => "read only",
        (false, true) => "root",
        (false, false) => "",
    };

    let root = rows.iter().filter(|row| note(row) == "root").count();
    let read_only = rows.iter().filter(|row| row.read_only).count();
    let mut summary = vec![rows.len().to_string()];
    match root {
        0 => {}
        1 => summary.push("1 needs root".to_owned()),
        _ => summary.push(format!("{root} need root")),
    }

    if read_only > 0 {
        summary.push(format!("{read_only} read only"));
    }

    Shown {
        title: "Config files",
        summary: Some(summary.join(", ")),
        head: &["FILE", "PATH", "NOTE"],
        rows: rows
            .iter()
            .map(|row| {
                Row::plain(vec![
                    row.key.clone(),
                    row.path.clone(),
                    note(row).to_owned(),
                ])
            })
            .collect(),
        hint: Some("garuda-toolbox configs edit <key>".to_owned()),
    }
}

fn show(
    key: &str,
    catalog: &ConfigFileCatalog,
    listing: &Listing,
    source: &dyn ConfigFileSource,
    out: &mut Out<'_>,
) -> Exit {
    let found = view_target(catalog, listing, key)
        .map(Target::Root)
        .or_else(|| target(catalog, listing, key));

    let text = match found {
        Some(Target::Root(path)) => source
            .read(&path, &Arc::new(AtomicBool::new(false)))
            .map(|read| read.content.as_str().to_owned())
            .map_err(|problem| read_problem(&problem)),
        Some(Target::Home(path)) => source
            .home_file(&path)
            .ok_or_else(|| "there is no home folder".to_owned())
            .and_then(|file| std::fs::read_to_string(file).map_err(|error| error.to_string()))
            .map_err(|problem| (format!("the file was not read: {problem}"), String::new())),
        None => return unknown(key, out),
    };

    match text {
        Ok(text) => {
            write!(out, "{text}").ok();
            Exit::Success
        }
        Err((what, raw)) => {
            out::failure(out, &what, &raw);
            Exit::Failure
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "one command's parts, used once")]
fn edit(
    args: &ConfigEdit,
    catalog: &ConfigFileCatalog,
    listing: &Listing,
    source: &dyn ConfigFileSource,
    runtime: &Runtime,
    out: &mut Out<'_>,
) -> Exit {
    if args.confirm.json {
        // The editor takes the terminal, which JSON Lines leave to nobody.
        crate::say_error(out, "the editor needs the terminal, leave out --json");
        return Exit::NeedsYes;
    }

    if view_target(catalog, listing, &args.key).is_some() {
        crate::say_error(out, "this file is shown, never edited here");
        return Exit::Failure;
    }

    let Some(found) = target(catalog, listing, &args.key) else {
        return unknown(&args.key, out);
    };

    let Some(editor) = editor(source) else {
        let text = "no editor found: set EDITOR to micro, nano, vim or another known one";
        crate::say_error(out, text);
        return Exit::Unavailable;
    };

    let (file, content) = match read_for_edit(found, source) {
        Ok(read) => read,
        Err((what, raw)) => {
            out::failure(out, &what, &raw);
            return Exit::Refused;
        }
    };

    let copies = EditFolder::from_env(
        |key| std::env::var_os(key),
        &toolbox_config::Paths::from_process().cache_dir(),
    );

    let name = match &file {
        EditedFile::Root(path) => Target::Root(path.clone()),
        EditedFile::Home(path) => Target::Home(path.clone()),
    };

    let copy = match copies.create(name.file_name(), &content) {
        Ok(copy) => Removed {
            copies: &copies,
            copy,
        },
        Err(error) => {
            crate::say_error(out, &format!("no private copy could be made: {error}"));
            return Exit::Failure;
        }
    };

    review(
        editor, &copy.copy, &copies, &file, &content, args, runtime, out,
    )
}

/// The text of `found` and the file the write goes to: for a root-owned
/// file the real path its links lead to.
fn read_for_edit(
    found: Target,
    source: &dyn ConfigFileSource,
) -> Result<(EditedFile, ConfigText), (String, String)> {
    match found {
        Target::Root(path) => source
            .read(&path, &Arc::new(AtomicBool::new(false)))
            .map(|read| (EditedFile::Root(read.path), read.content))
            .map_err(|problem| read_problem(&problem)),
        Target::Home(path) => {
            let file = source
                .home_file(&path)
                .ok_or_else(|| ("there is no home folder".to_owned(), String::new()))?;

            let text = std::fs::read_to_string(file)
                .map_err(|error| ("the file was not read".to_owned(), error.to_string()))?;

            let content = ConfigText::new(text)
                .map_err(|error| ("the file is no usable text".to_owned(), error.to_string()))?;

            Ok((EditedFile::Home(path), content))
        }
    }
}

/// `VISUAL`, then `EDITOR`, then the first known editor installed.
fn editor(source: &dyn ConfigFileSource) -> Option<Editor> {
    let variable = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());

    choose_editor(
        variable("VISUAL").as_deref(),
        variable("EDITOR").as_deref(),
        |editor| source.installed(editor),
    )
}

#[expect(clippy::too_many_arguments, reason = "one edit's parts, used once")]
fn review(
    editor: Editor,
    copy: &EditCopy,
    copies: &EditFolder,
    file: &EditedFile,
    original: &ConfigText,
    args: &ConfigEdit,
    runtime: &Runtime,
    out: &mut Out<'_>,
) -> Exit {
    let ended = edit_in_place(editor, Path::new(copy.as_str()), out);
    if ended != Exit::Success {
        return ended;
    }

    let edited = match copies.read(copy) {
        Ok(Ok(text)) => text,
        Ok(Err(invalid)) => {
            crate::say_error(
                out,
                &format!("the edited copy is no usable text: {invalid}"),
            );

            return Exit::Failure;
        }
        Err(error) => {
            crate::say_error(out, &format!("the edited copy was not read: {error}"));
            return Exit::Failure;
        }
    };

    let shown = Diff::new(original.as_str(), edited.as_str()).plain();
    let Some(plan) = write_back(file, original, edited) else {
        writeln!(out, "Nothing to do, the file is unchanged").ok();
        return Exit::Success;
    };

    write!(out, "{shown}").ok();
    run(Ok(plan), args.confirm, None, runtime, out)
}

/// The editor on `file`, in this terminal, as the user. The path is
/// absolute, so no editor reads it as an option.
fn edit_in_place(editor: Editor, file: &Path, out: &mut Out<'_>) -> Exit {
    let program = toolbox_system::locate_editor(editor.program())
        .unwrap_or_else(|| editor.program().path().into());

    let mut command = std::process::Command::new(program);
    if editor == Editor::Emacs {
        command.arg("-nw");
    }

    let status = command.arg(file).status();

    match status {
        Ok(status) if status.success() => Exit::Success,
        Ok(status) => {
            let text = format!("the editor ended with {status}, nothing is saved");
            crate::say_error(out, &text);
            Exit::Failure
        }
        Err(error) => {
            crate::say_error(out, &format!("the editor did not start: {error}"));
            Exit::Failure
        }
    }
}

fn unknown(key: &str, out: &mut Out<'_>) -> Exit {
    crate::say_error(
        out,
        &format!("no config file is called `{key}`, see `configs list`"),
    );

    let known = super::names::config_keys();
    if let Some(hint) = did_you_mean(key, known.iter().map(String::as_str)) {
        writeln!(out, "{}", out.paint().hint(&hint)).ok();
    }

    Exit::Unavailable
}

/// Why a root-owned file was not read, in the page's codes, and the
/// program's own words for `--verbose`.
fn read_problem(problem: &ReadProblem) -> (String, String) {
    let code = match problem {
        ReadProblem::Refused(reason) => reason.as_str(),
        ReadProblem::NotRun(refusal) => refusal.code(),
        ReadProblem::Cancelled => "cancelled",
        ReadProblem::Failed(raw) => return ("the file was not read".to_owned(), raw.clone()),
    };

    (format!("the file was not read ({code})"), String::new())
}
