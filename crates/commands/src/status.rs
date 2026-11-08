//! `garuda-toolbox status`: Home's facts in the terminal.

mod command;
mod facts;
#[cfg(test)]
mod tests;
mod watch;

use std::fmt::Write as _;
use std::time::SystemTime;

use serde::Serialize;
use toolbox_cli::output::Paint;
use toolbox_core::features::status::days_since;
use toolbox_system::{Facts, HealthLevel, PendingUpdate, ProbeError, Status};

pub use self::command::status;
pub use self::facts::{Disk, FactsView, Packages, facts_view};
use super::out::failure_text;

/// One fact and whether it could be read. Scripts tell them apart by
/// `state`, the value only comes with `ok`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum Part<T> {
    Ok { value: T },
    Unavailable,
    Failed { cause: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Package {
    #[serde(rename = "id")]
    pub name: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub level: &'static str,
    pub message: String,
    pub fix_available: bool,
}

/// What `status` prints, as `--json` gives it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StatusView {
    pub updates: Part<Vec<Package>>,
    pub aur_updates: Part<Vec<Package>>,
    pub files_to_merge: Part<Vec<String>>,
    /// Whole days since the last update, `null` when none is logged.
    pub last_update_days: Part<Option<u64>>,
    pub health: Part<Vec<Issue>>,
    pub restart_needed: Part<bool>,
    /// `None` when the facts were not read, as in a view of the update
    /// check alone.
    #[serde(flatten)]
    pub facts: Option<FactsView>,
}

enum Shown {
    Value(String),
    Unavailable,
    /// The program's own words, without the probe's prefix.
    Failed(String),
}

impl StatusView {
    /// The view with the facts read besides the update check.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn with_facts(mut self, facts: &Facts, now: SystemTime) -> Self {
        self.facts = Some(facts_view(facts, now));
        self
    }
}

pub fn view(status: &Status, restart: &Result<bool, ProbeError>, now: SystemTime) -> StatusView {
    StatusView {
        updates: part(&status.updates, |updates| packages(updates)),
        aur_updates: part(&status.aur_updates, |updates| packages(updates)),
        files_to_merge: part(&status.merges, |paths| {
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        }),
        last_update_days: part(&status.last_update, |last| {
            last.map(|time| days_since(time, now))
        }),
        health: part(&status.health, |issues| {
            issues
                .iter()
                .map(|issue| Issue {
                    level: level(issue.level),
                    message: issue.message.clone(),
                    fix_available: issue.fix_available,
                })
                .collect()
        }),
        restart_needed: part(restart, |needed| *needed),
        facts: None,
    }
}

/// The view as a `Status` section.
///
/// The readable facts come first, then the ones not available, a line per
/// health finding and a short `error:` line per part that failed. The
/// programs' own words follow only with `verbose`.
pub fn text(view: &StatusView, paint: Paint, verbose: bool) -> String {
    let mut rows: Vec<(&str, Shown)> = rows(view).into_iter().collect();
    if let Some(read) = &view.facts {
        rows.extend(facts::rows(read));
    }

    let mut facts: Vec<(&str, &str)> = Vec::new();
    let mut failed: Vec<(&str, &str)> = Vec::new();
    for (label, shown) in &rows {
        if let Shown::Value(value) = shown {
            facts.push((label, value));
        }
    }

    for (label, shown) in &rows {
        match shown {
            Shown::Unavailable => facts.push((label, "not available")),
            Shown::Failed(raw) => failed.push((label, raw)),
            Shown::Value(_) => {}
        }
    }

    let mut text = paint.facts("Status", &facts);
    text.push_str(&findings(view, paint));
    text.push_str(&failures(&failed, paint, verbose));
    text
}

/// `--brief`: what needs attention in one line, or `all good`. A part that
/// could not be read is left out, as in [`needs_attention`].
pub fn brief(view: &StatusView) -> String {
    let todo = attention(view);
    if todo.is_empty() {
        "all good".to_owned()
    } else {
        todo.join(", ")
    }
}

/// Whether `status` exits with [`toolbox_cli::Exit::Attention`].
pub fn needs_attention(view: &StatusView) -> bool {
    !attention(view).is_empty()
}

/// The things to do, each counted. Info findings ask for nothing.
fn attention(view: &StatusView) -> Vec<String> {
    let at = |wanted: &str| {
        counted_part(&view.health, |issues| {
            issues.iter().filter(|issue| issue.level == wanted).count()
        })
    };

    let restart = matches!(view.restart_needed, Part::Ok { value: true });

    [
        counted(counted_part(&view.updates, Vec::len), "update", "updates"),
        counted(
            counted_part(&view.aur_updates, Vec::len),
            "AUR update",
            "AUR updates",
        ),
        counted(
            counted_part(&view.files_to_merge, Vec::len),
            "file to merge",
            "files to merge",
        ),
        counted(at("critical"), "critical issue", "critical issues"),
        counted(at("high"), "high issue", "high issues"),
        counted(at("low"), "low issue", "low issues"),
        restart.then(|| "restart needed".to_owned()),
    ]
    .into_iter()
    .flatten()
    .chain(view.facts.iter().flat_map(facts::attention))
    .collect()
}

fn counted_part<T>(part: &Part<T>, count: impl FnOnce(&T) -> usize) -> usize {
    match part {
        Part::Ok { value } => count(value),
        Part::Unavailable | Part::Failed { cause: _ } => 0,
    }
}

fn counted(count: usize, one: &str, many: &str) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {one}")),
        _ => Some(format!("{count} {many}")),
    }
}

fn rows(view: &StatusView) -> [(&'static str, Shown); 6] {
    [
        row("Updates", &view.updates, |list| count(list.len())),
        row("AUR updates", &view.aur_updates, |list| count(list.len())),
        row("Files to merge", &view.files_to_merge, |files| {
            joined_or_none(files)
        }),
        row("Last update", &view.last_update_days, |days| match days {
            None => "never".to_owned(),
            Some(0) => "today".to_owned(),
            Some(1) => "1 day ago".to_owned(),
            Some(days) => format!("{days} days ago"),
        }),
        row("Health", &view.health, |issues| match issues.len() {
            0 => "no findings".to_owned(),
            1 => "1 finding".to_owned(),
            found => format!("{found} findings"),
        }),
        row("Restart", &view.restart_needed, |needed| {
            if *needed { "needed" } else { "not needed" }.to_owned()
        }),
    ]
}

/// A line per health finding, after a blank one.
fn findings(view: &StatusView, paint: Paint) -> String {
    let Part::Ok { value: issues } = &view.health else {
        return String::new();
    };

    if issues.is_empty() {
        return String::new();
    }

    issues.iter().fold(String::from("\n"), |mut text, issue| {
        let line = match issue.level {
            "error" => paint.error(&issue.message),
            "warning" => paint.warning(&issue.message),
            other => format!("{other}: {}", issue.message),
        };

        writeln!(text, "{}", paint.item(&line)).ok();
        text
    })
}

/// A short line per part that failed, after a blank one.
fn failures(failed: &[(&str, &str)], paint: Paint, verbose: bool) -> String {
    if failed.is_empty() {
        return String::new();
    }

    // Without `verbose` only the last failure with words of its own passes
    // them on, so the hint that says how to see them comes once.
    let last = failed.iter().rposition(|(_, raw)| !raw.trim().is_empty());
    let mut text = String::from("\n");
    for (place, (label, raw)) in failed.iter().enumerate() {
        let what = format!("{} could not be read", label.to_lowercase());
        let raw = if verbose || Some(place) == last {
            *raw
        } else {
            ""
        };

        text.push_str(&failure_text(paint, &what, raw, verbose));
    }

    text
}

fn part<T, V>(result: &Result<T, ProbeError>, value: impl FnOnce(&T) -> V) -> Part<V> {
    match result {
        Ok(found) => Part::Ok {
            value: value(found),
        },
        Err(ProbeError::Unavailable) => Part::Unavailable,
        Err(error) => Part::Failed {
            cause: error.to_string(),
        },
    }
}

fn row<'a, T>(label: &'a str, part: &Part<T>, show: impl FnOnce(&T) -> String) -> (&'a str, Shown) {
    let shown = match part {
        Part::Ok { value } => Shown::Value(show(value)),
        Part::Unavailable => Shown::Unavailable,
        Part::Failed { cause } => {
            let raw = cause.strip_prefix("probe failed: ").unwrap_or(cause);
            Shown::Failed(raw.to_owned())
        }
    };

    (label, shown)
}

fn packages(updates: &[PendingUpdate]) -> Vec<Package> {
    updates
        .iter()
        .map(|update| Package {
            name: update.name.as_str().to_owned(),
            from: update.from.clone(),
            to: update.to.clone(),
        })
        .collect()
}

fn joined_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}

fn count(packages: usize) -> String {
    match packages {
        0 => "none".to_owned(),
        1 => "1 package".to_owned(),
        _ => format!("{packages} packages"),
    }
}

const fn level(level: HealthLevel) -> &'static str {
    match level {
        HealthLevel::Critical => "critical",
        HealthLevel::High => "high",
        HealthLevel::Low => "low",
        HealthLevel::Info => "info",
    }
}
