//! Menus, lists and notes take the frosted look over the live blur: a
//! see-through tint, never the solid raised fill.

use egui::Theme;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{Select, menu_button, menu_item};
use crate::testing::{blurred_rects, filled_rects, harness, textured_rects};
use crate::toasts::{ToastKind, ToastTexts, Toasts};
use crate::tokens;

fn frosted(harness: &Harness<'_, impl Sized>) -> bool {
    let tokens = tokens::current(&harness.ctx);
    let tinted = filled_rects(harness, crate::frost::raised(&harness.ctx));
    let textured = textured_rects(harness);
    let solid = filled_rects(harness, tokens.raised);

    let blurred = blurred_rects(harness);

    // The blur itself is a white textured rect, the solid fill is not
    // textured.
    !tinted.is_empty()
        && tinted.iter().all(|rect| textured.contains(rect))
        && tinted.iter().all(|rect| blurred.contains(rect))
        && solid
            .iter()
            .filter(|rect| !textured.contains(rect))
            .all(|rect| !tinted.contains(rect))
}

#[test]
fn an_open_menu_is_frosted_in_both_themes() {
    for theme in [Theme::Dark, Theme::Light] {
        let mut harness = harness(300.0, 240.0).build_ui(|ui| {
            menu_button(ui, "More actions", |ui| {
                menu_item(ui, "Copy");
                menu_item(ui, "Remove");
            });
        });

        harness.ctx.set_theme(theme);
        harness.run();
        harness.get_by_label("More actions").click();
        harness.run();

        assert!(
            harness.query_by_label("Copy").is_some(),
            "{theme:?}: menu shut"
        );
        assert!(frosted(&harness), "{theme:?}: the menu is solid");
    }
}

#[test]
fn an_open_list_is_frosted() {
    let mut harness = harness(300.0, 240.0).build_ui(|ui| {
        _ = Select::new("Sort by", &["Name", "Size"])
            .selected(Some(0))
            .show(ui);
    });

    harness.run();
    harness.get_by_label("Sort by").click();
    harness.run();

    assert!(
        harness.query_by_label("Size").is_some(),
        "the list did not open"
    );
    assert!(frosted(&harness), "the list is solid");
}

#[test]
fn a_note_is_frosted() {
    let texts = ToastTexts {
        name: "Notification",
        close: "Close",
    };

    let mut toasts = Toasts::default();
    toasts.add(ToastKind::Info, "Copied the link.");
    let mut harness = harness(800.0, 300.0).build_ui_state(
        move |ui, toasts: &mut Toasts| {
            let page = ui.max_rect();
            toasts.show(ui.ctx(), page, texts);
        },
        toasts,
    );

    harness.run_steps(2);

    assert!(frosted(&harness), "the note is solid");
}
