//! `toggle list` and `toggle show`: the switches of Components, System
//! Tweaks, Gaming and the Setup Assistant.

#[cfg(test)]
mod tests;

use std::io::Write as _;

use serde::Serialize;
use toolbox_catalog::{ToggleCatalog, ToggleEntry};
use toolbox_cli::Exit;
use toolbox_cli::args::{EntryShow, ToggleList, TogglePage};
use toolbox_cli::output::{Paint, json};
use toolbox_services::catalogs::Catalogs;
use toolbox_services::desktop;

use super::change::Refused;
use super::lists::{self, ToggleRow, listing};
use super::out::{Out, say_error};
use super::{catalogs, filter, snapshot};

/// One switch as `toggle show` prints it: its row of `toggle list`, then
/// what the catalog says of it.
#[derive(Debug, Serialize)]
pub struct Shown<'a> {
    #[serde(flatten)]
    row: &'a ToggleRow,
    description: &'a str,
    /// What tells whether it is on, such as `package docker`.
    check: String,
    /// The packages and services it brings along.
    also: Vec<String>,
}

pub fn list(list: &ToggleList, out: &mut Out<'_>) -> Exit {
    let catalogs = catalogs();
    let rows = rows(&catalogs, list.page);

    filter::print(&list.read, &rows, &filter::toggles(list), out, |rows| {
        listing::toggles(rows, "Switches", "garuda-toolbox toggle on <id>")
    })
}

pub fn show(show: &EntryShow, out: &mut Out<'_>) -> Exit {
    let catalogs = catalogs();
    let rows = rows(&catalogs, None);
    let pages = pages(&catalogs, None);
    let found = rows.iter().find(|row| row.id == show.id).and_then(|row| {
        pages
            .iter()
            .find_map(|(_, catalog)| catalog.entry(&show.id))
            .map(|entry| shown(row, entry))
    });

    let Some(shown) = found else {
        let refused = Refused::Unknown(format!(
            "no switch is called `{}`, see `toggle list`",
            show.id
        ))
        .hinted(&show.id, rows.iter().map(|row| row.id.as_str()));

        say_error(out, refused.message());
        return refused.exit();
    };

    let text = if show.read.json {
        json(&shown)
    } else {
        text(out.paint(), &shown)
    };

    write!(out, "{text}").ok();
    Exit::Success
}

fn shown<'a>(row: &'a ToggleRow, entry: &'a ToggleEntry) -> Shown<'a> {
    let also = entry
        .also()
        .iter()
        .map(|package| package.as_str().to_owned())
        .chain(
            entry
                .also_units()
                .iter()
                .map(|unit| unit.as_str().to_owned()),
        )
        .chain(
            entry
                .also_user_units()
                .iter()
                .map(|unit| format!("{unit} (yours)")),
        )
        .collect();

    Shown {
        row,
        description: entry.description(),
        check: entry.check().to_string(),
        also,
    }
}

fn text(paint: Paint, shown: &Shown<'_>) -> String {
    let also = if shown.also.is_empty() {
        "none".to_owned()
    } else {
        shown.also.join(", ")
    };

    let available = if shown.row.available { "yes" } else { "no" };
    paint.facts(
        &shown.row.name,
        &[
            ("description", shown.description),
            ("id", &shown.row.id),
            ("page", shown.row.page),
            ("section", &shown.row.section),
            ("state", shown.row.state),
            ("available", available),
            ("checks", &shown.check),
            ("brings", &also),
        ],
    )
}

fn rows(catalogs: &Catalogs, page: Option<TogglePage>) -> Vec<ToggleRow> {
    let current = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    lists::toggles(
        &pages(catalogs, page),
        &snapshot(catalogs),
        &desktop::desktops(&current),
    )
}

/// The page catalogs in page order, only `page` when one is asked for.
fn pages(catalogs: &Catalogs, page: Option<TogglePage>) -> Vec<(TogglePage, &ToggleCatalog)> {
    [
        (TogglePage::Components, &catalogs.components),
        (TogglePage::Tweaks, &catalogs.settings),
        (TogglePage::Gaming, &catalogs.gaming),
        (TogglePage::Setup, &catalogs.setup),
    ]
    .into_iter()
    .filter(|(listed, _)| page.is_none_or(|wanted| wanted == *listed))
    .filter_map(|(listed, toggles)| toggles.catalog.as_ref().ok().map(|found| (listed, found)))
    .collect()
}
