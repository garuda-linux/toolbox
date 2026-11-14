//! The page's entries in the command palette: every operation, tool and
//! repair, and the settings reset, each with the tab that holds it.

use toolbox_core::ChangeSet;
use toolbox_core::features::maintenance::page::{operations, repairs, tools};

use super::Tab;

/// The text id of each entry's name and its tab, in the page's order.
pub fn entries() -> Vec<(String, Tab)> {
    let operations = operations(&ChangeSet::default())
        .into_iter()
        .map(|row| (row.operation.text_id(), Tab::Common));

    let tools = tools(|_| false)
        .into_iter()
        .map(|row| (row.tool.text_id(), Tab::Common));

    let repairs = repairs(|_| false)
        .into_iter()
        .map(|row| (row.repair.text_id(), Tab::Repairs));

    let reset = ("maintenance-tab-reset".to_owned(), Tab::Reset);

    operations
        .chain(tools)
        .chain(std::iter::once(reset))
        .chain(repairs)
        .collect()
}
