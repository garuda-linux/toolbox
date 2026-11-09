//! The Discover catalogs `data/catalog/apps/<category>.toml`: the eight
//! categories, the app records `wiki-sync` produces and the writer it uses
//! to write the files.

mod clean;
pub mod tasks;
#[cfg(test)]
mod tests;
pub mod wiki;
mod writer;

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub use clean::{https_url, wiki_text};
use toolbox_plan::{PackageName, protection};

/// Where the catalogs go, relative to the repository root.
pub const TARGET_DIR: &str = "data/catalog/apps";

/// The categories in page order: catalog id, English title and Breeze
/// category icon.
pub const CATEGORIES: [Category; 8] = [
    Category::new("documents", "Documents", "applications-office"),
    Category::new("games", "Games", "applications-games"),
    Category::new("internet", "Internet", "applications-internet"),
    Category::new("multimedia", "Multimedia", "applications-multimedia"),
    Category::new("other", "Other", "applications-other"),
    Category::new("science", "Science", "applications-science"),
    Category::new("security", "Security", "preferences-security"),
    Category::new("utilities", "Utilities", "applications-utilities"),
];

/// The apps `wiki-sync` found, per category in [`CATEGORIES`] order.
pub type Found = Vec<(Category, Vec<App>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Category {
    pub id: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
}

/// One app as `wiki-sync` found it, before ids and duplicates are sorted
/// out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub title: String,
    pub description: String,
    pub url: Option<String>,
    /// The first package is the entry's check, the rest come along.
    pub packages: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Lists {
    lists: Vec<(Category, Vec<(String, App)>)>,
    dropped: Vec<String>,
}

impl Category {
    const fn new(id: &'static str, title: &'static str, icon: &'static str) -> Self {
        Self { id, title, icon }
    }

    /// The Arch wiki page that lists the category's apps.
    pub fn wiki_page(self) -> String {
        if self.id == "games" {
            "List_of_games".to_owned()
        } else {
            format!("List_of_applications/{}", self.title)
        }
    }

    /// The catalog file name in [`TARGET_DIR`].
    pub fn file_name(self) -> String {
        format!("{}.toml", self.id)
    }
}

impl Lists {
    /// Takes the apps per category in [`CATEGORIES`] order. An app is dropped
    /// when an earlier app already has its first package or entry id, so a
    /// click queues each package once, and when it has no package, title or
    /// description.
    pub fn new(found: Found) -> Self {
        let mut seen = BTreeSet::new();
        let mut dropped = Vec::new();
        let mut lists = Vec::with_capacity(found.len());

        for (category, apps) in found {
            let mut kept = Vec::with_capacity(apps.len());

            for app in apps {
                match writer::entry_id(&app) {
                    Some(id) if seen.insert(id.clone()) => kept.push((id, app)),
                    Some(id) => dropped.push(format!("{}: {id} repeats", category.id)),
                    None => dropped.push(format!("{}: {:?} is incomplete", category.id, app.title)),
                }
            }

            lists.push((category, kept));
        }

        Self { lists, dropped }
    }

    /// Drops apps that only name a command of a protected package, such as
    /// su in util-linux: their switch would remove the whole package.
    pub fn without_commands_of(mut self) -> Self {
        for (category, apps) in &mut self.lists {
            apps.retain(|(_, app)| {
                let Some(package) = app.packages.first() else {
                    return true;
                };

                let command = PackageName::new(package)
                    .is_ok_and(|name| protection(&name).is_some())
                    && app.title.to_lowercase() != *package;

                if command {
                    self.dropped.push(format!(
                        "{}: {:?} is a command of the protected {package}",
                        category.id, app.title
                    ));
                }

                !command
            });
        }

        self
    }

    /// Why apps were left out, one line each.
    pub fn dropped(&self) -> &[String] {
        &self.dropped
    }

    /// Every package of every kept app, for the diff before the write.
    pub fn packages(&self) -> BTreeSet<String> {
        self.lists
            .iter()
            .flat_map(|(_, apps)| apps)
            .flat_map(|(_, app)| &app.packages)
            .cloned()
            .collect()
    }

    /// Writes one catalog per category into `dir` and returns a summary.
    ///
    /// # Errors
    ///
    /// The first file that cannot be written.
    pub fn write(&self, dir: &Path) -> Result<String, String> {
        fs::create_dir_all(dir)
            .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;

        let mut summary = String::new();

        for (category, apps) in &self.lists {
            let path = dir.join(category.file_name());

            fs::write(&path, writer::render(*category, apps))
                .map_err(|error| format!("cannot write {}: {error}", path.display()))?;

            // Writing to a String cannot fail.
            writeln!(summary, "{}: {} apps", category.id, apps.len()).ok();
        }

        let total: usize = self.lists.iter().map(|(_, apps)| apps.len()).sum();

        write!(
            summary,
            "{total} apps written to {}, {} left out",
            dir.display(),
            self.dropped.len()
        )
        .ok();

        Ok(summary)
    }
}
