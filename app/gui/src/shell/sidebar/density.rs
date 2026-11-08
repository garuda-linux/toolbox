use egui::Ui;
use toolbox_gui_kit::tokens::Sizes;
use toolbox_gui_kit::widgets::{NavLook, group_label_height, nav_divider_height};

use crate::page::{Group, PageId};
use crate::page_ui;

#[derive(Clone, Copy, Debug)]
pub(super) struct Density {
    pub(super) row: f32,
    pub(super) gap: f32,
    pub(super) group_gap: f32,
    /// Above and below a collapsed group's divider, beside the gap between
    /// entries.
    pub(super) divider: f32,
    /// Above and below the search entry.
    pub(super) top_gap: f32,
    pub(super) compact: bool,
    /// Above a packed group heading: the room the packed list leaves,
    /// shared out, so the heading does not cling to the entry above it.
    pub(super) label_room: f32,
    pub(super) expanded: bool,
}

impl Density {
    const fn normal(sizes: &Sizes, expanded: bool) -> Self {
        Self {
            row: sizes.nav_row_height,
            gap: sizes.nav_gap,
            group_gap: sizes.nav_group_gap,
            divider: divider_room(sizes, sizes.nav_group_gap, sizes.nav_gap),
            top_gap: sizes.control_gap,
            compact: false,
            label_room: 0.0,
            expanded,
        }
    }

    const fn compact(sizes: &Sizes, expanded: bool) -> Self {
        Self {
            row: sizes.nav_row_height,
            gap: sizes.nav_gap_compact,
            group_gap: sizes.nav_group_gap_compact,
            divider: divider_room(sizes, sizes.nav_group_gap_compact, sizes.nav_gap_compact),
            top_gap: sizes.nav_group_gap_compact,
            compact: true,
            label_room: 0.0,
            expanded,
        }
    }

    /// Normal size while the search entry and every page fit `available`,
    /// the height below the title row. The sum comes from the sizes, not
    /// from what the last frame drew, so the choice holds still.
    pub(super) fn fitting(ui: &Ui, sizes: &Sizes, expanded: bool, available: f32) -> Self {
        let normal = Self::normal(sizes, expanded);
        if normal.needed(ui) <= available {
            return normal;
        }

        let mut packed = Self::compact(sizes, expanded);
        let headings = Self::headings(ui.ctx());
        if expanded && headings > 0.0 {
            let spare = available - packed.needed(ui);

            // A heading keeps a little room above even when the list
            // scrolls, or it reads as part of the entry above.
            packed.label_room = (spare / headings).clamp(sizes.tight_gap, sizes.nav_group_gap);
        }

        packed
    }

    fn needed(self, ui: &Ui) -> f32 {
        let label = group_label_height(ui, self.compact);
        let search = self.top_gap.mul_add(2.0, self.row) + ui.spacing().item_spacing.y;
        search
            + self.pages_height(ui.ctx(), label, false)
            + self.pages_height(ui.ctx(), label, true)
    }

    fn headings(ctx: &egui::Context) -> f32 {
        let mut groups: Vec<Group> = PageId::ALL
            .into_iter()
            .filter(|page| page_ui::listed(ctx, *page) && page.group().heading_id().is_some())
            .map(PageId::group)
            .collect();

        groups.dedup();
        groups.into_iter().map(|_| 1.0).sum()
    }

    /// How tall `pages` draws the app's own pages (`foot`) or every
    /// other group, each heading `label` tall.
    pub(super) fn pages_height(self, ctx: &egui::Context, label: f32, foot: bool) -> f32 {
        let mut height = 0.0;
        let mut group = None;
        for page in PageId::ALL {
            if (page.group() == Group::App) != foot || !page_ui::listed(ctx, page) {
                continue;
            }

            if group != Some(page.group()) {
                group = Some(page.group());
                height += match page.group().heading_id() {
                    Some(_) if self.expanded => self.label_room + label + self.gap,
                    Some(_) => nav_divider_height(self.divider) + self.gap,
                    None => self.group_gap,
                };
            }

            height += self.row + self.gap;
        }

        // No gap follows the last entry.
        (height - self.gap).max(0.0)
    }

    pub(super) const fn look(self) -> NavLook {
        NavLook {
            expanded: self.expanded,
            height: self.row,
        }
    }
}

/// The group gap, or more where the gap between entries leaves the
/// divider less than [`Sizes::nav_divider_room`].
const fn divider_room(sizes: &Sizes, group_gap: f32, gap: f32) -> f32 {
    group_gap.max(sizes.nav_divider_room - gap)
}
