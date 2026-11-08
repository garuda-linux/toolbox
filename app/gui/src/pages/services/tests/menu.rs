//! Every entry of a unit's menu shows an icon, so the texts line up and
//! no entry looks unfinished beside one that has one.

use std::collections::BTreeSet;
use std::sync::Arc;

use egui::text::LayoutJob;
use toolbox_core::features::services::ActiveState;
use toolbox_gui_kit::widgets::Tone;
use toolbox_plan::{UnitName, UnitScope};

use super::super::unit_row::{UnitRow, UnitRows};
use crate::pages::rows::actions::RowKind as _;

fn row(masked: bool) -> UnitRow {
    UnitRow {
        unit: UnitName::new("bluetooth.service").expect("a valid unit name"),
        scope: UnitScope::System,
        name: "bluetooth.service".to_owned(),
        title: Arc::new(LayoutJob::default()),
        description: Arc::new(LayoutJob::default()),
        state: ("running".into(), Tone::Success),
        file: None,
        queued: Vec::new(),
        enabled: true,
        masked,
        active: ActiveState::Active,
        switchable: !masked,
        menu: "Actions".to_owned(),
    }
}

fn icons(item: &UnitRow) -> Vec<(&'static str, Option<&'static str>)> {
    UnitRows::SECTIONS
        .iter()
        .flat_map(|section| section.actions)
        .filter(|action| action.offered(item))
        .map(|action| {
            let icon = action.icon(item).map(|icon| icon.name());
            (action.label(item), icon)
        })
        .collect()
}

#[test]
fn every_entry_of_a_unit_menu_has_an_icon() {
    for masked in [false, true] {
        let item = row(masked);
        let entries = icons(&item);
        let bare: Vec<_> = entries.iter().filter(|(_, icon)| icon.is_none()).collect();
        assert!(bare.is_empty(), "entries without an icon: {bare:?}");

        let distinct: BTreeSet<_> = entries.iter().map(|(_, icon)| icon).collect();
        assert_eq!(distinct.len(), entries.len(), "two entries share a glyph");
    }
}

fn mask_icon(masked: bool) -> Option<&'static str> {
    let (_, icon) = icons(&row(masked))
        .into_iter()
        .find(|(label, _)| label.ends_with("mask"))?;

    icon
}

#[test]
fn mask_and_unmask_show_a_closed_and_an_open_lock() {
    assert_eq!(mask_icon(false), Some("object-locked"));
    assert_eq!(mask_icon(true), Some("object-unlocked"));
}
