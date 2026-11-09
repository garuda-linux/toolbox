use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use toolbox_catalog::{Check, ToggleCatalog};
use toolbox_plan::PackageName;
use toolbox_system::{Libalpm, SyncSource};

use super::{CATEGORIES, Category, Lists, TARGET_DIR, moves, wiki};
use crate::{Failure, root};

/// Where `wiki-sync` saves the fetched pages by default, beside the build
/// output so they are never committed.
const SOURCES_DIR: &str = "target/wiki-sources";

/// Whether the run writes, and whether it may move an app to another first
/// package.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Writing {
    #[default]
    DryRun,
    Write,
    WriteAcceptingMoves,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct WikiOptions {
    pub local: Option<PathBuf>,
    pub sources: Option<PathBuf>,
    pub out: Option<PathBuf>,
    pub include_aur: bool,
    pub unfiltered: bool,
    pub writing: Writing,
}

pub fn wiki_sync(args: &[String]) -> Result<String, Failure> {
    let options = parse_options(args).ok_or(Failure::Usage)?;
    let mut report = String::new();

    let offered = offered_packages(&Libalpm::system(), &mut report, options.unfiltered)
        .map_err(|error| Failure::Problems(vec![error]))?;

    let mut found = Vec::with_capacity(CATEGORIES.len());
    let mut problems = Vec::new();

    for category in CATEGORIES {
        match page_text(category, &options) {
            Ok(text) => {
                let (apps, not_offered) = wiki::keep(
                    wiki::parse_page(&text),
                    offered.as_ref(),
                    options.include_aur,
                );

                // Writing to a String cannot fail.
                writeln!(
                    report,
                    "{}: {} apps found, {not_offered} not offered",
                    category.id,
                    apps.len(),
                )
                .ok();
                found.push((category, apps));
            }
            Err(error) => problems.push(error),
        }
    }

    if !problems.is_empty() {
        return Err(Failure::Problems(problems));
    }

    let lists = Lists::new(found).without_commands_of();
    let out = options.out.unwrap_or_else(|| root().join(TARGET_DIR));

    let catalogs =
        shipped(&root().join(TARGET_DIR)).map_err(|error| Failure::Problems(vec![error]))?;

    let shipped = shipped_packages(&root().join(TARGET_DIR))
        .map_err(|error| Failure::Problems(vec![error]))?;

    for line in package_diff(&shipped, &lists.packages()) {
        writeln!(report, "{line}").ok();
    }

    let moved = moves::moves(&moves::firsts(&catalogs), &lists);

    let writes = options.writing != Writing::DryRun;
    let accepted = options.writing == Writing::WriteAcceptingMoves;

    moves::gate(&moved, writes, accepted).map_err(Failure::Problems)?;

    for line in &moved {
        writeln!(report, "{line}").ok();
    }

    let summary = finish(&lists, &out, writes).map_err(|error| Failure::Problems(vec![error]))?;

    report.push_str(&summary);

    for line in lists.dropped() {
        write!(report, "\nleft out {line}").ok();
    }

    Ok(report)
}

/// Writes the catalogs into `out` when `write` is set. A wiki page that
/// broke would empty its catalog, so without it the run only reports.
///
/// # Errors
///
/// When a catalog cannot be written.
pub fn finish(lists: &Lists, out: &Path, write: bool) -> Result<String, String> {
    if write {
        return lists.write(out);
    }

    Ok(format!(
        "dry run: nothing written to {}. Check the lines above, then pass --write.",
        out.display()
    ))
}

pub fn parse_options(args: &[String]) -> Option<WikiOptions> {
    let mut options = WikiOptions::default();
    let mut write = false;
    let mut accept_moves = false;
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--include-aur" => options.include_aur = true,
            "--unfiltered" => options.unfiltered = true,
            "--write" => write = true,
            "--accept-moves" => accept_moves = true,
            "--local" => options.local = Some(PathBuf::from(rest.next()?)),
            "--sources" => options.sources = Some(PathBuf::from(rest.next()?)),
            "--out" => options.out = Some(PathBuf::from(rest.next()?)),
            _ => return None,
        }
    }

    // Accepting moves means nothing for a run that writes nothing.
    options.writing = match (write, accept_moves) {
        (true, true) => Writing::WriteAcceptingMoves,
        (true, false) => Writing::Write,
        (false, false) => Writing::DryRun,
        (false, true) => return None,
    };

    Some(options)
}

/// The packages of the sync databases. Without them the sync fails,
/// unless `--unfiltered` keeps every repository app with a warning.
pub fn offered_packages(
    source: &dyn SyncSource,
    report: &mut String,
    unfiltered: bool,
) -> Result<Option<BTreeSet<PackageName>>, String> {
    match source.offered() {
        Ok(offered) => Ok(Some(offered.0)),
        Err(error) if unfiltered => {
            writeln!(
                report,
                "warning: no sync databases ({error}), every repository app is kept"
            )
            .ok();
            Ok(None)
        }
        Err(error) => Err(format!(
            "no sync databases ({error}): pass --unfiltered to keep every repository app"
        )),
    }
}

pub fn package_diff(before: &BTreeSet<String>, after: &BTreeSet<String>) -> Vec<String> {
    let mut lines = Vec::new();

    for name in after.difference(before) {
        lines.push(format!("package added: {name}"));
    }

    for name in before.difference(after) {
        lines.push(format!("package removed: {name}"));
    }

    lines
}

/// Every package the shipped catalogs check or install beside it. A
/// missing file counts as empty, a new category has none yet.
///
/// # Errors
///
/// When an existing catalog cannot be read or parsed: counted as empty it
/// would show every package of its category as added.
pub fn shipped_packages(dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut packages = BTreeSet::new();

    for (_, catalog) in shipped(dir)? {
        for section in catalog.sections() {
            for entry in section.entries() {
                if let Check::Package(name) = entry.check() {
                    packages.insert(name.as_str().to_owned());
                }

                for name in entry.also() {
                    packages.insert(name.as_str().to_owned());
                }
            }
        }
    }

    Ok(packages)
}

/// The shipped catalog of each category that has one.
///
/// # Errors
///
/// When an existing catalog cannot be read or parsed.
pub fn shipped(dir: &Path) -> Result<Vec<(Category, ToggleCatalog)>, String> {
    let mut catalogs = Vec::new();

    for category in CATEGORIES {
        let path = dir.join(category.file_name());

        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
        };

        let catalog = ToggleCatalog::parse(&text, &category.file_name())
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;

        catalogs.push((category, catalog));
    }

    Ok(catalogs)
}

fn page_text(category: Category, options: &WikiOptions) -> Result<String, String> {
    if let Some(dir) = &options.local {
        return wiki::read_source(category, dir);
    }

    let sources = options
        .sources
        .clone()
        .unwrap_or_else(|| root().join(SOURCES_DIR));

    fs::create_dir_all(&sources)
        .map_err(|error| format!("cannot create {}: {error}", sources.display()))?;

    wiki::fetch(category, &sources)
}
