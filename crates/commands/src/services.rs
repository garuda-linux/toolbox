//! `services list` and `services show`: the system's units, or the user's.

#[cfg(test)]
mod tests;

use std::io::Write as _;

use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{ServicesList, UnitShow};
use toolbox_cli::output::{Paint, json};
use toolbox_plan::UnitScope;
use toolbox_services::units::{Systemd, UnitLister as _};

use super::change::Refused;
use super::lists::{self, ServiceRow, listing};
use super::out::{Out, say_error, unavailable};
use super::{filter, spinner};

pub fn list(list: &ServicesList, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let Some(rows) = read(list.user, list.read.json, runtime, out) else {
        return unavailable("the services could not be read", out);
    };

    filter::print(&list.read, &rows, &filter::services(list), out, |rows| {
        listing::services(rows, list.user)
    })
}

pub fn show(show: &UnitShow, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let Some(rows) = read(show.user, show.read.json, runtime, out) else {
        return unavailable("the services could not be read", out);
    };

    let unit = show.unit.as_str();
    let Some(row) = rows.iter().find(|row| row.name == unit) else {
        let refused = Refused::Unknown(format!("there is no unit {unit}, see `services list`"))
            .hinted(unit, rows.iter().map(|row| row.name.as_str()));

        say_error(out, refused.message());
        return refused.exit();
    };

    let text = if show.read.json {
        json(row)
    } else {
        text(out.paint(), row, show.user)
    };

    write!(out, "{text}").ok();
    Exit::Success
}

fn read(user: bool, json: bool, runtime: &Runtime, out: &Out<'_>) -> Option<Vec<ServiceRow>> {
    let scope = UnitScope::from_user(user);
    let listed = spinner::while_reading(json || out.quiet(), || {
        runtime.block_on(Systemd.list(scope))
    });

    match listed {
        Ok(units) => Some(lists::services(&units)),
        Err(error) => {
            tracing::warn!("the units could not be listed: {error}");
            None
        }
    }
}

fn text(paint: Paint, row: &ServiceRow, user: bool) -> String {
    let state = format!("{} ({})", row.active, row.state);
    let whose = if user { "yours" } else { "the system's" };

    paint.facts(
        &row.name,
        &[
            ("description", &row.description),
            ("state", &state),
            ("at boot", if row.at_boot { "yes" } else { "no" }),
            ("service of", whose),
        ],
    )
}
