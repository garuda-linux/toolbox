//! One selection language: the accent tint of the sidebar, and in light
//! themes rims where a white surface sat on pale grey.

use egui::{Rect, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{chip, segmented, switch};
use crate::testing::{
    fill_under, filled_rect, harness, painted_text, rims_around, rims_in, text_colour,
};
use crate::tokens;

fn shown(theme: Theme, draw: impl FnMut(&mut egui::Ui) + 'static) -> Harness<'static> {
    let mut harness = harness(400.0, 120.0).build_ui(draw);
    harness.ctx.set_theme(theme);
    harness.run();
    harness
}

fn chosen_pill(harness: &Harness<'_>) -> Rect {
    let tokens = tokens::current(&harness.ctx);
    filled_rect(harness, tokens.input).expect("the chosen pill is the input fill")
}

/// A checked chip is Kirigami's translucent accent fill, not the solid
/// selection tint of a list row.
#[test]
fn a_selected_chip_takes_the_accent_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = shown(theme, |ui| _ = chip(ui, "Audio", Some("3"), true));
        let tokens = tokens::current(&harness.ctx);
        let label = painted_text(&harness, "Audio").expect("the chip's name is painted");
        let inside = label.pos + egui::vec2(-6.0, label.galley.rect.center().y);
        let fill = fill_under(&harness, inside);

        assert!(
            fill.is_some_and(|fill| fill != tokens.window && fill != tokens.card),
            "{theme:?}: no accent fill under the chip: {fill:?}"
        );
        assert_eq!(text_colour(&harness, "Audio"), Some(tokens.text));
        assert_eq!(text_colour(&harness, "3"), Some(tokens.text));
    }
}

#[test]
fn an_off_switch_has_a_rim_in_light() {
    let harness = shown(Theme::Light, |ui| _ = switch(ui, false, "Bluetooth"));

    let centre = harness.get_by_label("Bluetooth").rect().center();
    let rims = rims_around(&harness, centre);

    assert!(rims.iter().any(|width| *width >= 1.0), "{rims:?}");
}

/// The chosen tab is a card-coloured pill inset in its rect, with a 1 px
/// card outline and no strip (darklystyle.cpp:7354-7355).
#[test]
fn the_chosen_tab_is_an_inset_pill_with_the_card_outline() {
    let harness = shown(Theme::Light, |ui| {
        _ = segmented(ui, "Scope", "Scope", &[(0_u8, "System"), (1, "User")], 0);
    });

    let tokens = tokens::current(&harness.ctx);
    assert!(
        crate::testing::lines_in(&harness, tokens::mix(tokens.window, tokens.text, 0.2)).is_empty(),
        "no base line under the tabs"
    );
    assert!(
        filled_rect(&harness, tokens.accent_tint).is_none(),
        "no accent strip along its top"
    );
    let tab = harness.get_by_label("System").rect();
    let pill = chosen_pill(&harness);
    let rims = rims_in(&harness, tokens.card_outline);

    assert!(
        (pill.left() - tab.left() - segmented::PILL_INSET).abs() < 0.5,
        "tab {tab:?}, pill {pill:?}"
    );
    assert!(
        rims.iter()
            .any(|(rim, width)| rim.contains(pill.center()) && (*width - 1.0).abs() < 0.01),
        "the pill has a 1 px card outline: {rims:?}"
    );
}

/// The chosen tab's label keeps 8 points of margin on each side inside the pill.
#[test]
fn the_chosen_pill_keeps_the_text_margin_around_its_label() {
    let harness = shown(Theme::Light, |ui| {
        _ = segmented(
            ui,
            "Theme",
            "Theme",
            &[(0_u8, "As the desktop"), (1, "Dark")],
            0,
        );
    });

    let pill = chosen_pill(&harness);
    let text = painted_text(&harness, "As the desktop").expect("the label is painted");
    let label = text.pos.x..=text.pos.x + text.galley.size().x;

    assert!(
        label.start() - pill.left() >= 8.0 - 0.5,
        "the label starts {} px inside the pill",
        label.start() - pill.left()
    );
    assert!(
        pill.right() - label.end() >= 8.0 - 0.5,
        "the label ends {} px short of the pill's edge",
        pill.right() - label.end()
    );
}

/// A tab under the pointer fills the same inset pill the chosen tab draws,
/// so the hover is never larger than the choice beside it.
#[test]
fn a_tab_under_the_pointer_fills_a_pill_as_large_as_the_chosen_one() {
    let mut harness = shown(Theme::Light, |ui| {
        _ = segmented(ui, "Scope", "Scope", &[(0_u8, "System"), (1, "User")], 0);
    });

    harness.get_by_label("User").hover();
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let tab = harness.get_by_label("User").rect();
    let chosen = chosen_pill(&harness);
    let hover = filled_rect(&harness, tokens.item_hover).expect("the tab under the pointer fills");

    assert!(
        (hover.left() - tab.left() - segmented::PILL_INSET).abs() < 0.5,
        "tab {tab:?}, hover {hover:?}"
    );
    assert!(
        (hover.height() - chosen.height()).abs() < 0.5,
        "hover {hover:?}, chosen {chosen:?}"
    );
}

/// The chosen pill is as tall as the face of a field or button beside it:
/// those paint 2 px inside their rect, so the pill does too, top and bottom.
#[test]
fn the_chosen_pill_is_as_tall_as_a_field_beside_it() {
    let harness = shown(Theme::Dark, |ui| {
        _ = segmented(ui, "Scope", "Scope", &[(0_u8, "System"), (1, "User")], 0);
    });

    let tab = harness.get_by_label("System").rect();
    let pill = chosen_pill(&harness);

    assert!(
        (pill.top() - tab.top() - 2.0).abs() < 0.5
            && (tab.bottom() - pill.bottom() - 2.0).abs() < 0.5,
        "tab {tab:?}, pill {pill:?}"
    );
}
