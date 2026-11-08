use egui_kittest::kittest::Queryable as _;

use super::super::list::ListItem;
use super::super::{ButtonKind, RowHeights, RowList, card, card_row, pill_button};
use crate::testing::{harness, rims_around};

struct Item(&'static str);

impl ListItem for Item {
    fn key(&self) -> &str {
        self.0
    }

    fn name(&self) -> &str {
        self.0
    }
}

fn rimmed(harness: &egui_kittest::Harness<'_>, label: &str) -> bool {
    let button = harness.get_by_label(label).rect();
    rims_around(harness, button.center())
        .iter()
        .any(|width| *width >= 1.0)
}

#[test]
fn a_soft_button_on_a_hovered_row_keeps_an_outline() {
    // The hover fill and a Soft button at rest share one colour, the same
    // as on a neutral banner.
    let rows = [Item("ada"), Item("grace")];
    let mut harness = harness(800.0, 300.0).build_ui(move |ui| {
        RowList::new("rows", "Accounts", RowHeights::Fixed(56.0)).show(ui, &rows, |ui, item, _| {
            _ = pill_button(ui, &format!("Edit {}", item.0), ButtonKind::Soft);
        });
    });

    harness.run();
    assert!(!rimmed(&harness, "Edit ada"), "a row at rest needs no rim");

    harness.get_by_label("Edit ada").hover();
    harness.run();
    harness.run();

    assert!(rimmed(&harness, "Edit ada"));
    assert!(!rimmed(&harness, "Edit grace"), "only the highlighted row");
}

#[test]
fn a_soft_button_on_a_hovered_card_row_keeps_an_outline() {
    let mut harness = harness(800.0, 300.0).build_ui(|ui| {
        card(ui, |ui| {
            for (index, name) in ["ada", "grace"].into_iter().enumerate() {
                card_row(ui, index == 0, |ui| {
                    _ = pill_button(ui, &format!("Edit {name}"), ButtonKind::Soft);
                });
            }
        });
    });

    harness.run();
    assert!(!rimmed(&harness, "Edit ada"), "a row at rest needs no rim");

    harness.get_by_label("Edit ada").hover();
    harness.run();
    harness.run();

    assert!(rimmed(&harness, "Edit ada"));
    assert!(!rimmed(&harness, "Edit grace"), "only the highlighted row");
}
