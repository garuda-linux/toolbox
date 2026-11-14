//! A list of many rows touches only those it draws, also with one of them
//! focused. The AUR list has about a hundred thousand.

use std::cell::Cell;

use egui::ScrollArea;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::super::{ListItem, RowHeights, RowList, Rows};
use crate::testing::harness;

const COUNT: usize = 100_000;

struct Row(String);

impl ListItem for Row {
    fn key(&self) -> &str {
        &self.0
    }

    fn name(&self) -> &str {
        &self.0
    }
}

/// Builds each row when asked, and counts the asking.
#[derive(Default)]
struct Many {
    asked: Cell<usize>,
}

impl Rows for Many {
    type Item<'a> = Row;

    fn len(&self) -> usize {
        COUNT
    }

    fn get(&self, index: usize) -> Option<Row> {
        self.asked.set(self.asked.get() + 1);
        (index < COUNT).then(|| Row(format!("row {index}")))
    }
}

/// Measured rows, as many as `count` says.
struct Measured {
    count: usize,
    asked: Cell<usize>,
}

impl Rows for Measured {
    type Item<'a> = Row;

    fn len(&self) -> usize {
        self.count
    }

    fn get(&self, index: usize) -> Option<Row> {
        self.asked.set(self.asked.get() + 1);
        (index < self.count).then(|| Row(format!("row {index}")))
    }
}

#[test]
fn a_frame_asks_only_for_the_rows_it_draws_also_with_one_focused() {
    let mut harness = harness(300.0, 400.0).build_ui_state(
        |ui, many: &mut Many| {
            ScrollArea::vertical().show(ui, |ui| {
                RowList::new("many", "Many", RowHeights::Fixed(40.0)).show(
                    ui,
                    many,
                    |ui, row, _| drop(ui.label(&row.0)),
                );
            });
        },
        Many::default(),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::ListItem, "row 3")
        .click();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(Role::ListItem, "row 3")
            .is_focused()
    );

    harness.state().asked.set(0);
    harness.step();

    let asked = harness.state().asked.get();
    assert!(
        asked < 100,
        "{asked} of {COUNT} rows asked for in one frame"
    );
}

#[test]
fn a_frame_with_the_same_measured_rows_builds_no_offsets() {
    let rows = Measured {
        count: COUNT,
        asked: Cell::new(0),
    };

    let mut harness = harness(300.0, 400.0).build_ui_state(
        |ui, rows: &mut Measured| {
            ScrollArea::vertical().show(ui, |ui| {
                let heights = RowHeights::Measured { estimate: 30.0 };
                RowList::new("measured", "Measured", heights).show(ui, rows, |ui, row, _| {
                    drop(ui.label(&row.0));
                });
            });
        },
        rows,
    );

    harness.run();
    harness.state().asked.set(0);
    harness.step();
    harness.step();

    let asked = harness.state().asked.get();
    assert!(
        asked < 200,
        "{asked} of {COUNT} rows asked for in two frames"
    );

    harness.state_mut().count = COUNT - 1;
    harness.state().asked.set(0);
    harness.step();
    assert!(
        harness.state().asked.get() >= COUNT - 1,
        "the offsets were not built again"
    );
}

#[test]
fn measured_rows_that_swap_places_leave_no_gap() {
    let rows = |tall_first: bool| {
        let first = Row("first".to_owned());
        let (tall, short) = (Row("tall\nrow\nhere".to_owned()), Row("short".to_owned()));
        if tall_first {
            vec![first, tall, short]
        } else {
            vec![first, short, tall]
        }
    };

    let mut harness = harness(300.0, 400.0).build_ui_state(
        |ui, shown: &mut Vec<Row>| {
            let heights = RowHeights::Measured { estimate: 30.0 };
            RowList::new("swapped", "Swapped", heights).show(ui, shown, |ui, row, _| {
                drop(ui.label(&row.0));
            });
        },
        rows(true),
    );

    harness.run();
    *harness.state_mut() = rows(false);
    harness.run();

    let short = harness
        .get_by_role_and_label(Role::ListItem, "short")
        .rect();

    let tall = harness
        .get_by_role_and_label(Role::ListItem, "tall\nrow\nhere")
        .rect();

    assert!(
        (tall.top() - short.bottom()).abs() < 2.0,
        "short ends at {}, tall starts at {}",
        short.bottom(),
        tall.top()
    );
}

/// Typing in the search above the list keeps the focus outside it.
#[test]
fn a_frame_with_the_focus_outside_the_list_asks_only_for_the_rows_it_draws() {
    let mut harness = harness(300.0, 400.0).build_ui_state(
        |ui, (many, search): &mut (Many, String)| {
            ui.add(egui::TextEdit::singleline(search).hint_text("Search"));
            ScrollArea::vertical().show(ui, |ui| {
                RowList::new("many", "Many", RowHeights::Fixed(40.0)).show(
                    ui,
                    many,
                    |ui, row, _| drop(ui.label(&row.0)),
                );
            });
        },
        (Many::default(), String::new()),
    );

    harness.run();
    harness.get_by_role(Role::TextInput).click();
    harness.run();

    harness.state().0.asked.set(0);
    harness.step();
    harness.step();

    let asked = harness.state().0.asked.get();
    assert!(
        asked < 200,
        "{asked} of {COUNT} rows asked for in two frames"
    );
}
