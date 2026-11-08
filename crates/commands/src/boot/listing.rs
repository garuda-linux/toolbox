use std::fmt::Write as _;

use toolbox_cli::output::{Mark, Row};
use toolbox_core::features::boot::view::EntryRow;

use super::ModuleRow;
use crate::out::Listing;

pub fn module_listing(rows: &[ModuleRow]) -> Listing<'_> {
    let mark = |state: &str| match state {
        "built" | "installed" => Mark::On,
        "broken" => Mark::Problem,
        _ => Mark::Off,
    };

    let built = rows
        .iter()
        .filter(|row| mark(row.state) == Mark::On)
        .count();

    let broken = rows.iter().filter(|row| row.state == "broken").count();
    let mut summary = format!("{}, {built} built", rows.len());
    if broken > 0 {
        write!(summary, ", {broken} broken").ok();
    }

    Listing {
        title: "DKMS modules",
        summary: Some(summary),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let cells = [row.module.as_str(), &row.version, &row.kernel, row.state];
                Row::cells(mark(row.state), &cells)
            })
            .collect(),
        hint: (built < rows.len()).then(|| "garuda-toolbox dkms build".to_owned()),
    }
}

pub fn entry_listing(rows: &[EntryRow]) -> Listing<'_> {
    Listing {
        title: "Boot entries",
        summary: Some(rows.len().to_string()),
        head: &["ENTRY", "TITLE"],
        rows: rows
            .iter()
            .map(|row| Row::plain(vec![row.entry.clone(), row.title.clone()]))
            .collect(),
        hint: (!rows.is_empty()).then(|| "garuda-toolbox boot set default <entry>".to_owned()),
    }
}
