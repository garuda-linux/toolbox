//! The chips above the list: the repositories' counts, the AUR chip and the
//! chips of an index that is still read or cannot be.

use egui::{Ui, vec2};
use toolbox_core::features::packages::list::{Counts, Filter};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::chip;

use super::super::PackagesPage;
use crate::i18n;

pub struct AurChip {
    /// `None` until the AUR's copy is read.
    pub count: Option<usize>,
}

/// What the chips count. A count that is not known shows none.
pub struct ChipCounts {
    /// `None` without pacman's databases.
    pub repository: Option<Counts>,
    pub updates_known: bool,
    /// `None` while the AUR is off.
    pub aur: Option<AurChip>,
}

/// The chips while the index is read or cannot be: the repository's are off
/// and count nothing, the AUR's stays as far as it is on.
pub(super) fn idle_filters(page: &mut PackagesPage, ui: &mut Ui, aur_on: bool) {
    let aur = aur_on.then_some(AurChip { count: None });
    filters(
        page,
        ui,
        ChipCounts {
            repository: None,
            updates_known: false,
            aur,
        },
    );
}

/// The repository chips go off without pacman's databases, as their
/// counts would only be zeros. The AUR's chip needs none of them.
pub(super) fn filters(page: &mut PackagesPage, ui: &mut Ui, counts: ChipCounts) {
    let ctx = ui.ctx().clone();
    let repository = counts.repository;
    let count = |pick: fn(Counts) -> usize| repository.map(pick);
    let mut chips = vec![
        (
            Filter::Installed,
            "packages-filter-installed",
            count(|known| known.installed),
        ),
        (
            Filter::Updates,
            "packages-filter-updates",
            count(|known| known.updates).filter(|_| counts.updates_known),
        ),
        (
            Filter::Orphans,
            "packages-filter-orphans",
            count(|known| known.orphans),
        ),
        (
            Filter::Optional,
            "packages-filter-optional",
            count(|known| known.optional),
        ),
        (Filter::All, "packages-filter-all", count(|known| known.all)),
    ];

    if let Some(aur_chip) = counts.aur {
        chips.push((Filter::Aur, "packages-filter-aur", aur_chip.count));
    }

    let gap = ui.tokens().sizes.control_gap;
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        for (filter, id, shown) in chips {
            let title = i18n::text(&ctx, id);
            let number = shown.map(|number| number.to_string());
            let enabled = repository.is_some() || filter == Filter::Aur;
            let selected = filter == page.filter;
            let response = ui.add_enabled(enabled, |ui: &mut Ui| {
                chip(ui, &title, number.as_deref(), selected)
            });

            if response.clicked() {
                picked = Some(filter);
            }
        }
    });

    if let Some(filter) = picked {
        page.filter = filter;
    }
}
