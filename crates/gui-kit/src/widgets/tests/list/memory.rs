//! A row keeps what it measured until it leaves the list.

use egui::{IdSalt, Ui};

use super::super::super::list::memory;
use super::super::super::{ActionRow, ListItem, RowHeights, RowList, Tone, split_row_in};
use crate::icons;
use crate::testing::harness;

struct Row(&'static str);

impl ListItem for Row {
    fn key(&self) -> &str {
        self.0
    }

    fn name(&self) -> &str {
        self.0
    }
}

fn kept(ui: &Ui, key: &str) -> bool {
    let row = ui.make_persistent_id(IdSalt::new("rows")).with(key);
    memory::get(ui, row).height.is_some()
}

#[test]
fn a_row_that_leaves_the_list_leaves_no_memory_behind() {
    let mut harness = harness(300.0, 300.0).build_ui_state(
        |ui, (rows, last): &mut (Vec<Row>, bool)| {
            let heights = RowHeights::Measured { estimate: 30.0 };
            RowList::new("rows", "Rows", heights).show(ui, rows, |ui, row, _| {
                drop(ui.label(row.0));
            });

            *last = kept(ui, "c");
        },
        (vec![Row("a"), Row("b"), Row("c")], false),
    );

    harness.run();
    assert!(harness.state().1, "a drawn row keeps its height");

    harness.state_mut().0.pop();
    harness.run();
    assert!(!harness.state().1, "the row that left keeps nothing");
}

#[test]
fn a_dropped_row_takes_its_action_and_split_sizes_with_it() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |ui, (rows, kept_c): &mut (Vec<Row>, (bool, bool))| {
            let heights = RowHeights::Measured { estimate: 30.0 };
            RowList::new("rows", "Rows", heights).show(ui, rows, |ui, row, state| {
                ActionRow::new(icons::TOOLS, Tone::Neutral, row.0)
                    .row(state.row)
                    .show(ui, |ui| drop(ui.button("Act")));
                split_row_in(
                    ui,
                    state.row,
                    120.0,
                    |ui| drop(ui.label(row.0)),
                    |ui| drop(ui.button("Go")),
                );
            });

            let row = ui.make_persistent_id(IdSalt::new("rows")).with("c");
            let sizes = memory::get(ui, row);
            *kept_c = (sizes.actions_width.is_some(), sizes.split.is_some());
        },
        (vec![Row("a"), Row("b"), Row("c")], (false, false)),
    );

    harness.run();
    assert_eq!(harness.state().1, (true, true), "the row keeps its sizes");

    harness.state_mut().0.pop();
    harness.run();
    assert_eq!(
        harness.state().1,
        (false, false),
        "the dropped row keeps nothing"
    );
}
