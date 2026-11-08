//! The Config Files page: which rows the catalog and the system give,
//! what a row opens, which editor runs, and what an edit changed.
//!
//! Every edit is of a private copy in the user's editor in the Console.
//! The review shows the changes before the write: a root-owned file is
//! written by garuda-toolbox-helper and never opened as root, a file in
//! the home by the app as the user. What the catalog marks read-only hands out
//! root or the accounts: its text is only shown.

mod diff;
pub mod session;
#[cfg(test)]
mod tests;
pub mod viewing;

use std::collections::BTreeSet;

pub use diff::{Diff, hunk_lines};
use toolbox_catalog::{ConfigEntry, ConfigFileCatalog};
use toolbox_plan::{
    ConfigFilePath, EditCopy, Editor, FetchRefusal, HomePath, Plan, RootAction, UserAction,
};
pub use toolbox_state::configs::{ConfigEntryLocation, ConfigEntrySpec, Found, Link, Listing};

use self::session::EditWrite;
use super::search::{Needle, Searchable};

/// Between an entry's id and the name of a file in its folder. No id and
/// no file name holds one.
const KEY_SEPARATOR: char = '/';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Home(HomePath),
    Root(ConfigFilePath),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum RowKind {
    Section,
    #[default]
    File,
    Folder,
    Child,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent state of the row"
)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Row {
    pub kind: RowKind,
    /// What the page's commands take: the entry id, or id and file name.
    pub key: String,
    /// English from the catalog, except for a `child`: its file name.
    pub name: String,
    pub path: String,
    pub description: String,
    pub url: String,
    pub icon: String,
    /// `open` (default app), `edit` (Console), `view` (the text, read
    /// only) or empty for none.
    pub action: &'static str,
    /// The file belongs to root: saving an edit needs an administrator.
    pub admin: bool,
    /// Shown but never edited here: the file hands out root or the
    /// accounts, so the helper does not write it.
    pub read_only: bool,
    pub link: bool,
    /// Where a link leads, as far as it could be followed.
    pub target: String,
    /// The code of why a link is not edited here, or empty.
    pub refusal: &'static str,
    pub expanded: bool,
    pub count: usize,
}

impl Target {
    /// The path as the user knows it: `~/.bashrc` or `/etc/fstab`.
    pub fn shown(&self) -> String {
        match self {
            Self::Home(path) => path.to_string(),
            Self::Root(path) => path.to_string(),
        }
    }

    /// The last component, which names the private copy.
    pub fn file_name(&self) -> &str {
        let path = match self {
            Self::Home(path) => path.as_str(),
            Self::Root(path) => path.as_str(),
        };

        path.rsplit_once('/').map_or(path, |(_, name)| name)
    }
}

impl Searchable for Row {
    fn texts(&self) -> impl Iterator<Item = &str> {
        [
            self.name.as_str(),
            self.path.as_str(),
            self.description.as_str(),
        ]
        .into_iter()
    }
}

/// The rows of the page.
///
/// Every section with something to show, its files and folders, and the
/// files of the folders in `expanded`. With a `search`, only what matches
/// it (name, path, description or a file in a folder, whatever the case),
/// and matching folders show their files.
pub fn rows(
    catalog: &ConfigFileCatalog,
    listing: &Listing,
    expanded: &BTreeSet<String>,
    search: &str,
) -> Vec<Row> {
    let needle = Needle::new(search);
    let mut rows = Vec::new();

    for section in catalog.sections() {
        let entries: Vec<Row> = section
            .entries()
            .iter()
            .filter_map(|entry| Some((entry, listing.get(entry.id())?)))
            .flat_map(|(entry, found)| entry_rows(entry, found, expanded, &needle))
            .collect();

        if entries.is_empty() {
            continue;
        }

        rows.push(Row {
            kind: RowKind::Section,
            key: section.id().to_owned(),
            name: section.title().to_owned(),
            icon: section.icon().to_owned(),
            ..Row::default()
        });
        rows.extend(entries);
    }

    rows
}

/// The file behind a row's `key`, when the catalog lists it and the
/// system has it. A file of a folder must be one the listing found, so a
/// key can never name a file the page did not show.
pub fn target(catalog: &ConfigFileCatalog, listing: &Listing, key: &str) -> Option<Target> {
    let (id, file) = match key.split_once(KEY_SEPARATOR) {
        Some((id, file)) => (id, Some(file)),
        None => (key, None),
    };

    let entry = catalog.entry(id)?;

    match (listing.get(id)?, file) {
        (Found::File { link: _ }, None) => Some(location(entry)),
        (Found::Folder(files), Some(file)) if files.iter().any(|name| name == file) => {
            match entry.location() {
                ConfigEntryLocation::Home(folder) => {
                    HomePath::new(format!("{}/{file}", folder.as_str()))
                        .ok()
                        .map(Target::Home)
                }
                ConfigEntryLocation::Root(folder) => folder.join(file).ok().map(Target::Root),
            }
        }
        (Found::File { link: _ } | Found::Folder(_), Some(_) | None) => None,
    }
}

/// The root-owned file behind a row's `key` whose text the page shows
/// without an edit: a read-only entry of the catalog, or a file of its
/// folder, that the page shows and whose link the helper follows.
pub fn view_target(
    catalog: &ConfigFileCatalog,
    listing: &Listing,
    key: &str,
) -> Option<ConfigFilePath> {
    let id = key.split_once(KEY_SEPARATOR).map_or(key, |(id, _file)| id);

    if !catalog.entry(id)?.is_read_only() {
        return None;
    }

    if let Some(Found::File { link: Some(link) }) = listing.get(id)
        && link.refusal.is_some()
    {
        return None;
    }

    match target(catalog, listing, key)? {
        Target::Root(path) => Some(path),
        Target::Home(_) => None,
    }
}

/// The editor for the copy: what `$VISUAL` names, else what `$EDITOR`
/// names, else the first fallback, each only when it is `installed`. The
/// variables only pick from the closed list, their text is never run.
pub fn choose_editor<F>(visual: Option<&str>, editor: Option<&str>, installed: F) -> Option<Editor>
where
    F: Fn(Editor) -> bool,
{
    [visual, editor]
        .into_iter()
        .flatten()
        .filter_map(Editor::from_variable)
        .chain(Editor::FALLBACKS)
        .find(|candidate| installed(*candidate))
}

/// The editor on the copy, as one user step in the Console.
#[must_use = "a plan does nothing until it is run"]
pub fn edit_plan(editor: Editor, copy: EditCopy) -> Plan {
    Plan::user(UserAction::EditFile { editor, copy })
}

/// The write-back: one root step for a root-owned file, one step of the
/// user for a file of the user.
#[must_use = "a plan does nothing until it is run"]
pub fn write_plan(write: EditWrite) -> Plan {
    match write {
        EditWrite::Root(write) => Plan::root(RootAction::WriteConfigFile { write }),
        EditWrite::Home(write) => Plan::user(UserAction::WriteHomeFile { write }),
    }
}

fn location(entry: &ConfigEntry) -> Target {
    match entry.location() {
        ConfigEntryLocation::Home(path) => Target::Home(path.clone()),
        ConfigEntryLocation::Root(path) => Target::Root(path.clone()),
    }
}

fn entry_rows(
    entry: &ConfigEntry,
    found: &Found,
    expanded: &BTreeSet<String>,
    needle: &Needle,
) -> Vec<Row> {
    let target = location(entry);
    let admin = matches!(target, Target::Root(_));
    let base = Row {
        key: entry.id().to_owned(),
        name: entry.name().to_owned(),
        path: target.shown(),
        description: entry.description().unwrap_or_default().to_owned(),
        url: entry.url().unwrap_or_default().to_owned(),
        admin,
        read_only: entry.is_read_only(),
        ..Row::default()
    };

    let own_match = needle.finds(&base);

    match found {
        Found::File { link } if own_match => vec![file_row(base, link.as_ref())],
        Found::File { link: _ } => Vec::new(),
        Found::Folder(files) => {
            let shown: Vec<&String> = files
                .iter()
                .filter(|file| own_match || needle.is_in(file))
                .collect();

            if shown.is_empty() {
                return Vec::new();
            }

            let open = expanded.contains(entry.id()) || !needle.is_empty();
            let children: Vec<Row> = shown
                .into_iter()
                .filter(|_| open)
                .map(|file| Row {
                    kind: RowKind::Child,
                    key: format!("{}{KEY_SEPARATOR}{file}", entry.id()),
                    name: file.clone(),
                    path: format!("{}{KEY_SEPARATOR}{file}", target.shown()),
                    description: base.description.clone(),
                    url: base.url.clone(),
                    action: action(admin, entry.is_read_only(), false),
                    admin,
                    read_only: entry.is_read_only(),
                    ..Row::default()
                })
                .collect();

            let folder = Row {
                kind: RowKind::Folder,
                expanded: open,
                count: files.len(),
                ..base
            };

            std::iter::once(folder).chain(children).collect()
        }
    }
}

fn file_row(base: Row, link: Option<&Link>) -> Row {
    let refusal = link.and_then(|link| link.refusal);

    Row {
        kind: RowKind::File,
        action: action(base.admin, base.read_only, refusal.is_some()),
        link: link.is_some(),
        target: link.map(|link| link.target.clone()).unwrap_or_default(),
        refusal: refusal.map(FetchRefusal::as_str).unwrap_or_default(),
        ..base
    }
}

/// A root-owned file is edited, or viewed when it is read-only, unless
/// it is a link the helper refuses. A file of the user is always edited:
/// its write follows links as the user, so no link is refused.
const fn action(admin: bool, read_only: bool, refused: bool) -> &'static str {
    match (admin, read_only, refused) {
        (true, _, true) => "",
        (true, true, false) => "view",
        (true, false, false) | (false, _, _) => "edit",
    }
}
