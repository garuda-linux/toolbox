use egui_kittest::kittest::Queryable as _;

use super::super::empty_state;
use crate::testing::{harness, text_colour, textured_rects};
use crate::{icons, tokens};

#[test]
fn an_empty_state_draws_one_large_icon_whatever_its_state() {
    for icon in [icons::ERROR, icons::SYSTEM_SEARCH] {
        let mut harness = harness(600.0, 300.0)
            .build_ui(move |ui| _ = empty_state(ui, icon, "Nothing here", "Try again later."));

        harness.run();

        assert_eq!(textured_rects(&harness).len(), 1, "{icon:?}");
    }
}

#[test]
fn an_empty_state_icon_is_the_placeholder_size_of_kirigami() {
    let mut harness = harness(600.0, 300.0)
        .build_ui(|ui| _ = empty_state(ui, icons::SYSTEM_SEARCH, "Nothing here", ""));

    harness.run();

    let icon = textured_rects(&harness)
        .first()
        .copied()
        .expect("the icon is painted");

    assert!((icon.width() - 96.0).abs() < 0.5, "{icon:?}");
}

#[test]
fn an_empty_state_title_is_in_the_secondary_text_colour() {
    let mut harness = harness(600.0, 300.0)
        .build_ui(|ui| _ = empty_state(ui, icons::SYSTEM_SEARCH, "Nothing here", "Try again."));

    harness.run();
    let muted = tokens::current(&harness.ctx).secondary;

    assert_eq!(text_colour(&harness, "Nothing here"), Some(muted));
    assert!(harness.get_by_label("Nothing here").rect().height() > 0.0);
}
