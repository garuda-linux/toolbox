use egui::accesskit::Role;
use egui::{Id, Theme};
use egui_kittest::kittest::Queryable as _;

use super::super::list::ListItem;
use super::super::{ButtonKind, ModalWrapper, RowHeights, RowList, card, card_row, pill_button};
use crate::testing::{fill_under, filled_rect, harness, rim_colours_around};
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

/// Breeze frames every push button, at rest in a mix of its fill and text
/// and under the pointer in the accent, so the accent frame marks a hover.
fn accent_framed(harness: &egui_kittest::Harness<'_>, label: &str) -> bool {
    let button = harness.get_by_label(label).rect();
    let accent = tokens::current(&harness.ctx).accent;
    rim_colours_around(harness, button.center()).contains(&accent)
}

/// Darkly's push button has no outline: the pointer lightens its fill, and
/// the accent frame stays off it.
fn hover_lightens_only_the_pointed_button(
    harness: &egui_kittest::Harness<'_>,
    hovered: &str,
    other: &str,
) {
    let tokens = tokens::current(&harness.ctx);
    let centre = |label: &str| harness.get_by_label(label).rect().center();

    assert!(!accent_framed(harness, hovered), "no outline on {hovered}");
    assert_eq!(
        fill_under(harness, centre(hovered)),
        Some(tokens.button_hover),
        "{hovered} under the pointer"
    );
    assert_eq!(
        fill_under(harness, centre(other)),
        Some(tokens.input),
        "{other} at rest"
    );
}

#[test]
fn a_soft_button_on_a_hovered_row_takes_the_hover_fill_and_no_outline() {
    let rows = [Item("ada"), Item("grace")];
    let mut harness = harness(800.0, 300.0).build_ui(move |ui| {
        RowList::new("rows", "Accounts", RowHeights::Fixed(56.0)).show(ui, &rows, |ui, item, _| {
            _ = pill_button(ui, &format!("Edit {}", item.0), ButtonKind::Soft);
        });
    });

    harness.run();
    harness.get_by_label("Edit ada").hover();
    harness.run();
    harness.run();

    hover_lightens_only_the_pointed_button(&harness, "Edit ada", "Edit grace");
}

#[test]
fn a_soft_button_on_a_hovered_card_row_takes_the_hover_fill_and_no_outline() {
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
    harness.get_by_label("Edit ada").hover();
    harness.run();
    harness.run();

    hover_lightens_only_the_pointed_button(&harness, "Edit ada", "Edit grace");
}

/// A focused list row does not ring the button in it: the row shows its
/// own line, and the button keeps its frame at rest.
#[test]
fn a_focused_row_does_not_ring_its_soft_button() {
    let rows = [Item("ada"), Item("grace")];
    let mut harness = harness(800.0, 300.0).build_ui(move |ui| {
        RowList::new("rows", "Accounts", RowHeights::Fixed(56.0)).show(ui, &rows, |ui, item, _| {
            _ = pill_button(ui, &format!("Edit {}", item.0), ButtonKind::Soft);
        });
    });

    harness.run();
    harness.get_by_role_and_label(Role::ListItem, "ada").focus();
    harness.key_press(egui::Key::F13);
    harness.run();
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::ListItem, "ada")
            .is_focused(),
        "the row holds the focus"
    );

    let ring = tokens::current(&harness.ctx).accent_ring;
    let button = harness.get_by_label("Edit ada").rect();
    let rims = rim_colours_around(&harness, button.center());

    assert!(
        !rims.contains(&ring),
        "a focus ring rings the button: {rims:?}"
    );
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
            Some(tokens.input),
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
