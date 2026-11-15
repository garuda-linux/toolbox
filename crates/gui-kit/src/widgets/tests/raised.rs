use egui::accesskit::Role;
use egui::{Id, Theme};
use egui_kittest::kittest::Queryable as _;

use super::super::list::ListItem;
use super::super::{ButtonKind, ModalWrapper, RowHeights, RowList, card, card_row, pill_button};
use crate::testing::{fill_under, filled_rect, harness, rim_colours_around, rims_around};
use crate::tokens;

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

#[test]
fn a_focused_row_leaves_its_soft_button_without_an_outline() {
    let rows = [Item("ada"), Item("grace")];
    let mut harness = harness(800.0, 300.0).build_ui(move |ui| {
        RowList::new("rows", "Accounts", RowHeights::Fixed(56.0)).show(ui, &rows, |ui, item, _| {
            _ = pill_button(ui, &format!("Edit {}", item.0), ButtonKind::Soft);
        });
    });

    harness.run();
    harness.get_by_role_and_label(Role::ListItem, "ada").focus();
    harness.run();
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::ListItem, "ada")
            .is_focused(),
        "the row holds the focus"
    );

    // The row's own focus ring goes round the button too.
    let ring = tokens::current(&harness.ctx).accent_ring;
    let button = harness.get_by_label("Edit ada").rect();
    let rims: Vec<_> = rim_colours_around(&harness, button.center())
        .into_iter()
        .filter(|colour| *colour != ring)
        .collect();

    assert!(rims.is_empty(), "only a hovered row lifts it: {rims:?}");
}

fn dialog_with_keep(theme: Theme) -> egui_kittest::Harness<'static> {
    let mut harness = harness(600.0, 400.0).build_ui(|ui| {
        ModalWrapper::new(Id::unique("keep"), "Discard edits")
            .instant()
            .show(ui.ctx(), |ui| _ = pill_button(ui, "Keep", ButtonKind::Soft));
    });

    harness.ctx.set_theme(theme);
    harness.run();
    harness
}

#[test]
fn a_soft_button_on_a_dialog_takes_the_raised_field_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = dialog_with_keep(theme);
        let tokens = tokens::current(&harness.ctx);
        let centre = harness.get_by_label("Keep").rect().center();

        assert_eq!(
            fill_under(&harness, centre),
            Some(tokens.field_on_raised),
            "{theme:?}"
        );
    }
}

#[test]
fn a_dialog_dims_the_window_with_the_scrim_of_its_theme() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = dialog_with_keep(theme);
        let scrim = tokens::current(&harness.ctx).scrim;

        assert!(filled_rect(&harness, scrim).is_some(), "{theme:?}");
    }
}
