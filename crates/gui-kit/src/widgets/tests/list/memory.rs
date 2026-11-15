//! A row keeps what it measured until it leaves the list.

use egui::{IdSalt, Ui};

use super::super::super::list::memory;
use super::super::super::{ListItem, RowHeights, RowList};
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
