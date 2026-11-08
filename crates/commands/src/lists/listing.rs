//! How each list reads as text: the mark of every row, the summary in the
//! title's brackets and the next step under it.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;

use toolbox_cli::output::{Mark, Row};

use super::{GroupRow, KernelRow, LocaleRow, ServiceRow, ToggleRow, UserRow};
use crate::out::Listing;

pub fn services(rows: &[ServiceRow], user: bool) -> Listing<'_> {
    let running = count(rows, |row| row.active == "active");
    let failed = count(rows, |row| row.active == "failed");
    let mut summary = format!("{}, {running} running", rows.len());
    if failed > 0 {
        write!(summary, ", {failed} failed").ok();
    }

    let user = if user { " --user" } else { "" };

    Listing {
        title: "Services",
        summary: Some(summary),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let (mark, state) = match row.active {
                    "active" => (Mark::On, "running"),
                    "inactive" => (Mark::Off, "stopped"),
                    "failed" => (Mark::Problem, "failed"),
                    other => (Mark::Blank, other),
                };

                let boot = if row.at_boot { "at boot" } else { "" };
                Row::cells(mark, &[&row.name, state, boot])
            })
            .collect(),
        hint: Some(format!("garuda-toolbox services start{user} <name>")),
    }
}

/// Switches and apps alike: the id to type first, then the title.
pub fn toggles<'a>(rows: &'a [ToggleRow], title: &'a str, hint: &str) -> Listing<'a> {
    let on = count(rows, |row| row.state == "on");
    let unknown = count(rows, |row| row.state == "unknown");
    let mut summary = format!("{}, {on} on", rows.len());
    if unknown > 0 {
        write!(summary, ", {unknown} unknown").ok();
    }

    Listing {
        title,
        summary: Some(summary),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let mark = match row.state {
                    "on" => Mark::On,
                    "off" => Mark::Off,
                    _ => Mark::Unknown,
                };

                Row::cells(mark, &[&row.id, &row.name])
            })
            .collect(),
        hint: Some(hint.to_owned()),
    }
}

pub fn kernels(rows: &[KernelRow]) -> Listing<'_> {
    let installed = count(rows, |row| row.installed);
    let running = count(rows, |row| row.running);

    Listing {
        title: "Kernels",
        summary: Some(format!(
            "{}, {installed} installed, {running} running",
            rows.len()
        )),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let (mark, state) = if row.running {
                    (Mark::On, "running")
                } else if row.installed {
                    (Mark::On, "installed")
                } else {
                    (Mark::Off, "")
                };

                Row::cells(mark, &[&row.name, &row.version, state])
            })
            .collect(),
        hint: Some("garuda-toolbox kernels install <name>".to_owned()),
    }
}

pub fn locales(rows: &[LocaleRow]) -> Listing<'_> {
    let enabled = count(rows, |row| row.enabled);

    Listing {
        title: "Locales",
        summary: Some(format!("{}, {enabled} enabled", rows.len())),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let (mark, state) = match (row.enabled, row.system) {
                    (_, true) => (Mark::On, "system"),
                    (true, false) => (Mark::On, "enabled"),
                    (false, false) => (Mark::Off, ""),
                };

                Row::cells(mark, &[&row.locale, &row.charset, state])
            })
            .collect(),
        hint: Some("garuda-toolbox locales enable <locale>".to_owned()),
    }
}

/// `hidden` is how many system accounts the filter left out.
pub fn users(rows: &[UserRow], hidden: usize) -> Listing<'_> {
    let (summary, hint) = if hidden == 0 {
        (rows.len().to_string(), None)
    } else {
        (
            format!("{} shown, {hidden} system accounts hidden", rows.len()),
            Some("add --all to see them".to_owned()),
        )
    };

    Listing {
        title: "Users",
        summary: Some(summary),
        head: &["USER", "NAME", "SHELL"],
        rows: rows
            .iter()
            .map(|row| Row::cells(Mark::Blank, &[&row.name, &row.full_name, &row.shell]))
            .collect(),
        hint,
    }
}

pub fn groups(rows: &[GroupRow]) -> Listing<'_> {
    Listing {
        title: "Groups",
        summary: Some(rows.len().to_string()),
        head: &["GROUP", "MEMBERS"],
        rows: rows
            .iter()
            .map(|row| Row::plain(vec![row.name.clone(), row.members.join(", ")]))
            .collect(),
        hint: None,
    }
}

fn count<T>(rows: &[T], wanted: impl Fn(&T) -> bool) -> usize {
    rows.iter().filter(|row| wanted(row)).count()
}
