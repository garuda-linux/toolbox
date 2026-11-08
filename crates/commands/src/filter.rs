//! Which rows a `list` shows: the ones its own filters and `--search`
//! keep. JSON holds the same rows, the text's summary still counts the
//! whole list and says how many are shown.

mod named;
#[cfg(test)]
mod tests;

use std::io::Write;

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::args::{
    AppsList, KernelsList, LocalesList, ReadList, Search, ServicesList, ToggleList,
};
use toolbox_cli::output::json;

use super::lists::{KernelRow, LocaleRow, ServiceRow, ToggleRow};
use super::out::{self, Listing};

type Wanted<'a, T> = Box<dyn Fn(&T) -> bool + 'a>;

/// The texts `--search` looks in: the name or id, and the shown name.
pub trait Named {
    fn names(&self) -> Vec<&str>;
}

/// What a list's own flags keep. A row stays when every flag given keeps it.
pub struct Filter<'a, T> {
    wanted: Vec<Wanted<'a, T>>,
}

impl<'a, T> Filter<'a, T> {
    pub fn all() -> Self {
        Self { wanted: Vec::new() }
    }

    /// `wanted` decides too, when its flag `given` is set.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn only(mut self, given: bool, wanted: impl Fn(&T) -> bool + 'a) -> Self {
        if given {
            self.wanted.push(Box::new(wanted));
        }

        self
    }
}

/// The rows that stay, `None` when no flag and no search narrow the list.
pub fn kept<'r, T>(
    rows: &'r [T],
    search: Option<&Search>,
    filter: &Filter<'_, T>,
) -> Option<Vec<&'r T>>
where
    T: Named,
{
    if search.is_none() && filter.wanted.is_empty() {
        return None;
    }

    Some(
        rows.iter()
            .filter(|row| filter.wanted.iter().all(|wanted| wanted(row)))
            .filter(|row| search.is_none_or(|search| search.matches(row.names())))
            .collect(),
    )
}

/// The whole list's summary with the count of the rows shown.
pub fn summary(whole: Option<String>, shown: usize) -> String {
    whole.map_or_else(
        || format!("{shown} shown"),
        |whole| format!("{whole}, {shown} shown"),
    )
}

pub fn narrow<'a>(whole: Listing<'a>, part: Listing<'_>, shown: usize) -> Listing<'a> {
    Listing {
        title: whole.title,
        summary: Some(summary(whole.summary, shown)),
        head: whole.head,
        rows: part.rows,
        hint: part.hint,
    }
}

pub fn print<T>(
    list: &ReadList,
    rows: &[T],
    filter: &Filter<'_, T>,
    out: &mut dyn Write,
    show: impl Fn(&[T]) -> Listing<'_>,
) -> Exit
where
    T: Serialize + Named + Clone,
{
    let Some(found) = kept(rows, list.search.as_ref(), filter) else {
        return out::print(list.read(), rows, out, show);
    };

    if list.json {
        write!(out, "{}", json(&found)).ok();
        return Exit::Success;
    }

    let part: Vec<T> = found.into_iter().cloned().collect();
    out::write_listing(narrow(show(rows), show(&part), part.len()), out)
}

/// [`print`] of a list with no flags of its own, only `--search`.
pub fn print_all<T>(
    list: &ReadList,
    rows: &[T],
    out: &mut dyn Write,
    show: impl Fn(&[T]) -> Listing<'_>,
) -> Exit
where
    T: Serialize + Named + Clone,
{
    print(list, rows, &Filter::all(), out, show)
}

pub fn services(list: &ServicesList) -> Filter<'static, ServiceRow> {
    Filter::all()
        .only(list.running, |row: &ServiceRow| row.active == "active")
        .only(list.failed, |row: &ServiceRow| row.active == "failed")
        .only(list.enabled, |row: &ServiceRow| row.at_boot)
}

/// An unknown state is neither on nor off.
pub fn toggles(list: &ToggleList) -> Filter<'static, ToggleRow> {
    Filter::all()
        .only(list.on, |row: &ToggleRow| row.state == "on")
        .only(list.off, |row: &ToggleRow| row.state == "off")
}

pub fn apps(list: &AppsList) -> Filter<'static, ToggleRow> {
    Filter::all().only(list.installed, |row: &ToggleRow| row.state == "on")
}

pub fn kernels(list: &KernelsList) -> Filter<'static, KernelRow> {
    Filter::all().only(list.installed, |row: &KernelRow| row.installed)
}

/// The system's locale counts as enabled, as its mark shows.
pub fn locales(list: &LocalesList) -> Filter<'static, LocaleRow> {
    Filter::all().only(list.enabled, |row: &LocaleRow| row.enabled || row.system)
}
