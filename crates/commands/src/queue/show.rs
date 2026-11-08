#[cfg(test)]
mod tests;

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::output::Row;
use toolbox_core::{ChangeView, Summary};
use toolbox_services::queue::edit::Removal;

use super::said::changes;
use crate::out::Listing;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChangeRow {
    /// Counts from 1 in plan order, for `queue remove`.
    pub number: usize,
    pub domain: &'static str,
    /// The change's name within its kind, for `queue remove` too.
    #[serde(rename = "id")]
    pub key: String,
    /// The change in words, as the Changes list says it.
    pub change: String,
}

#[derive(Debug, Serialize)]
pub struct Shown<'a> {
    pub changes: &'a [ChangeRow],
    pub steps: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unpicked {
    /// No change has that number or name. A hint may follow on its own
    /// line.
    Unknown(String),
    Ambiguous(String),
}

impl From<&ChangeRow> for Removal {
    fn from(row: &ChangeRow) -> Self {
        Self {
            domain: row.domain.to_owned(),
            key: row.key.clone(),
        }
    }
}

impl Unpicked {
    pub const fn exit(&self) -> Exit {
        match self {
            Self::Unknown(_) => Exit::Unavailable,
            Self::Ambiguous(_) => Exit::Usage,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Unknown(message) | Self::Ambiguous(message) => message,
        }
    }
}

pub fn rows(views: &[ChangeView], words: impl Fn(&Summary) -> String) -> Vec<ChangeRow> {
    views
        .iter()
        .enumerate()
        .map(|(index, view)| ChangeRow {
            number: index + 1,
            domain: view.domain,
            key: view.key.clone(),
            change: words(&view.summary),
        })
        .collect()
}

/// The changes, numbered. `in_window` points at the window for the
/// apply.
pub fn listing(rows: &[ChangeRow], in_window: bool) -> Listing<'_> {
    let summary = changes(rows.len());

    let hint = if in_window {
        "the window holds this queue, apply it there"
    } else if rows.is_empty() {
        "garuda-toolbox queue add <change>"
    } else {
        "garuda-toolbox queue apply"
    };

    Listing {
        title: "Queued",
        summary: Some(summary),
        head: &[],
        rows: rows
            .iter()
            .map(|row| Row::plain(vec![row.number.to_string(), row.change.clone()]))
            .collect(),
        hint: Some(hint.to_owned()),
    }
}

/// # Errors
///
/// [`Unpicked`] for no such change, or a name several changes share.
pub fn pick(rows: &[ChangeRow], selector: &str) -> Result<Vec<Removal>, Unpicked> {
    if let Ok(number) = selector.parse::<usize>() {
        return rows
            .iter()
            .find(|row| row.number == number)
            .map(|row| vec![Removal::from(row)])
            .ok_or_else(|| {
                Unpicked::Unknown(format!(
                    "there is no queued change {number}, see `queue show`"
                ))
            });
    }

    let named: Vec<&ChangeRow> = rows.iter().filter(|row| row.key == selector).collect();
    match named.as_slice() {
        [] => {
            let text = format!("no queued change is called `{selector}`, see `queue show`");
            let hint = toolbox_cli::suggest::did_you_mean(
                selector,
                rows.iter().map(|row| row.key.as_str()),
            );

            Err(Unpicked::Unknown(match hint {
                Some(hint) => format!("{text}\n{hint}"),
                None => text,
            }))
        }
        [row] => Ok(vec![Removal::from(*row)]),
        several => Err(Unpicked::Ambiguous(format!(
            "`{selector}` names {} queued changes, take one back by its number, see `queue show`",
            several.len()
        ))),
    }
}
