//! `wiki-sync`: the Arch wiki's application lists into the Discover
//! catalogs. Each list line `{{App|name|description|url|{{Pkg|package}} …}}`
//! becomes one app.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use regex::Regex;
use toolbox_plan::PackageName;

use super::{App, Category, https_url, wiki_text};
use crate::regex;

/// The raw wikitext of a page.
const RAW_PAGE: &str = "https://wiki.archlinux.org/index.php?action=raw&title=";

/// The longest a page may take to fetch, in seconds.
const FETCH_SECONDS: &str = "120";

static APP: LazyLock<Regex> = LazyLock::new(|| regex(r"(?i)\{\{App\|(.*)\}\}"));
static PKG: LazyLock<Regex> = LazyLock::new(|| regex(r"(?i)\{\{Pkg\|(.*?)\}\}"));
static AUR: LazyLock<Regex> = LazyLock::new(|| regex(r"(?i)\{\{AUR\|(.*?)\}\}"));

/// One `{{App|…}}` line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiApp {
    pub app: App,
    /// Only `{{AUR|…}}` packages: no repository builds it.
    pub aur_only: bool,
}

pub fn parse_page(text: &str) -> Vec<WikiApp> {
    text.lines().filter_map(parse_app).collect()
}

/// One app line: name, description and website are the first three
/// fields, every `{{Pkg|…}}` and `{{AUR|…}}` after them a package. All of
/// them are kept: the sync checks every one against the sync databases.
/// `None` for a line without an app or without a package.
pub fn parse_app(line: &str) -> Option<WikiApp> {
    let content = APP.captures(line)?.get(1)?.as_str();
    let parts = split_fields(content);
    let (name, rest) = parts.split_first()?;
    let (description, rest) = rest.split_first()?;
    let (url, fields) = rest.split_first()?;

    if fields.is_empty() {
        return None;
    }

    let package_text = fields.join("|");
    let repository = template_names(&PKG, &package_text);
    let aur = template_names(&AUR, &package_text);
    let aur_only = repository.is_empty();
    let packages: Vec<String> = repository.into_iter().chain(aur).collect();

    if packages.is_empty() {
        return None;
    }

    Some(WikiApp {
        app: App {
            title: wiki_text(name),
            description: wiki_text(description),
            url: https_url(&wiki_text(url)),
            packages,
        },
        aur_only,
    })
}

/// The fields of a template body, split at `|` outside nested templates
/// and links.
pub fn split_fields(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0_usize;

    for letter in content.chars() {
        match letter {
            '{' | '[' => depth = depth.saturating_add(1),
            // A closer without its opener stays at the top level, so a
            // stray bracket hides no later field behind a negative depth.
            '}' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }

        if letter == '|' && depth == 0 {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(letter);
        }
    }

    parts.push(current);
    parts
}

/// The apps worth listing, sorted by title, and how many were not offered.
/// With the sync databases (`offered`) an app stays when they offer every
/// one of its packages, or `include_aur` is set. Without them only AUR-only
/// apps go, unless `include_aur`.
pub fn keep(
    apps: Vec<WikiApp>,
    offered: Option<&BTreeSet<PackageName>>,
    include_aur: bool,
) -> (Vec<App>, usize) {
    let mut not_offered = 0;
    let mut kept: Vec<App> = apps
        .into_iter()
        .filter(|wiki| {
            let listed = offered.map_or(include_aur || !wiki.aur_only, |offered| {
                include_aur || every_package_offered(&wiki.app.packages, offered)
            });

            if !listed {
                not_offered += 1;
            }

            listed
        })
        .map(|wiki| wiki.app)
        .collect();

    kept.sort_by_cached_key(|app| sort_key(&app.title));

    (kept, not_offered)
}

/// Whether the sync databases offer every package of an entry. A name
/// that is no package name fails the entry like a missing one.
fn every_package_offered(packages: &[String], offered: &BTreeSet<PackageName>) -> bool {
    packages
        .iter()
        .all(|name| PackageName::new(name).is_ok_and(|name| offered.contains(&name)))
}

/// Fetches the category's wiki page into `sources/<category>.txt` and
/// returns its text. Runs `curl` (https only, fails on HTTP errors) so the
/// development tools need no HTTP library.
///
/// # Errors
///
/// curl's message when it cannot fetch the page.
pub fn fetch(category: Category, sources: &Path) -> Result<String, String> {
    let file = source_file(category, sources);
    let page = format!("{RAW_PAGE}{}", category.wiki_page());
    let output = Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--proto", "=https", "--max-time", FETCH_SECONDS, "--output"])
        .arg(&file)
        .arg(&page)
        .output()
        .map_err(|error| format!("cannot run curl: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "{page}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    read_source(category, sources)
}

/// The saved page of the category in `sources`.
///
/// # Errors
///
/// The file cannot be read.
pub fn read_source(category: Category, sources: &Path) -> Result<String, String> {
    let file = source_file(category, sources);

    fs::read_to_string(&file).map_err(|error| format!("cannot read {}: {error}", file.display()))
}

fn source_file(category: Category, sources: &Path) -> PathBuf {
    sources.join(format!("{}.txt", category.id))
}

fn template_names(template: &Regex, text: &str) -> Vec<String> {
    template
        .captures_iter(text)
        .filter_map(|found| found.get(1))
        .filter_map(|body| body.as_str().split('|').next())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Accents and case count only between otherwise equal titles, lowercase
/// first.
fn sort_key(title: &str) -> (String, String) {
    let base = title
        .to_lowercase()
        .chars()
        .map(|letter| match letter {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            other => other,
        })
        .collect();

    let lower_first = title
        .chars()
        .map(|letter| {
            if letter.is_uppercase() {
                letter.to_lowercase().next().unwrap_or(letter)
            } else {
                letter.to_uppercase().next().unwrap_or(letter)
            }
        })
        .collect();

    (base, lower_first)
}
