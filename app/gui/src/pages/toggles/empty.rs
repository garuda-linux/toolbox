//! What a toggle page shows when its search keeps no row. While a picked
//! chip hides the matches of other sections, it keeps the pick, names the
//! section and offers to show every match.

use egui::Ui;
use toolbox_gui_kit::widgets::{ButtonKind, empty_state, pill_button};
use toolbox_gui_kit::{UiExt as _, icons};

use super::LeadChip;
use super::listed::SectionChip;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::search_empty::search_empty_state;

/// The section a chip picked, while it hides `matches` entries of other
/// sections that match the search.
#[derive(Debug)]
pub struct Picked {
    title: String,
    matches: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clear {
    Search,
    Section,
}

/// The picked section and how many entries of the whole page match the
/// search, when the pick hides all of them. The chips count the matches.
pub fn elsewhere(
    picked: Option<&str>,
    leads: &[LeadChip],
    sections: &[SectionChip],
) -> Option<Picked> {
    let picked = picked?;
    let lead_title = leads
        .iter()
        .find(|lead| lead.id == picked)
        .map(|lead| lead.title.clone().into_owned());

    let title = lead_title.or_else(|| {
        sections
            .iter()
            .find(|chip| chip.id == picked)
            .map(|chip| chip.title.clone())
    })?;

    let matches: usize = leads
        .iter()
        .map(|lead| lead.tally.total)
        .chain(sections.iter().map(|chip| chip.tally.total))
        .sum();

    (matches > 0).then_some(Picked { title, matches })
}

/// Shows the empty state of `search` (trimmed). Returns what its button
/// asks to clear.
pub fn show(ui: &mut Ui, search: &str, hidden: Option<&Picked>) -> Option<Clear> {
    if search.is_empty() {
        return None;
    }

    let ctx = ui.ctx().clone();
    let Some(picked) = hidden else {
        let detail = i18n::text_with(
            &ctx,
            "toggles-nothing-matches-detail",
            &[("search", Arg::from(search))],
        );

        return search_empty_state(ui, search, &detail).then_some(Clear::Search);
    };

    let count = picked.matches;
    let named = [
        ("section", Arg::from(picked.title.as_str())),
        ("search", Arg::from(search)),
    ];

    let title = i18n::text_with(&ctx, "toggles-nothing-in-section", &named);

    let detail = i18n::text_with(&ctx, "toggles-nothing-in-section-detail", &named);
    let button = i18n::text_with(
        &ctx,
        "toggles-show-all-matches",
        &[("count", Arg::from(count))],
    );

    empty_state(ui, icons::SYSTEM_SEARCH, &title, &detail);
    ui.add_space(ui.tokens().sizes.card_gap);
    let clicked = ui
        .vertical_centered(|ui| pill_button(ui, &button, ButtonKind::Soft).clicked())
        .inner;

    clicked.then_some(Clear::Section)
}
