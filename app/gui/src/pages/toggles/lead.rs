//! Sections a toggle page shows before its catalog's, under the same
//! search and chips, as System Tweaks' system-wide choices.

use std::borrow::Cow;

use egui::{Context, Ui};
use toolbox_core::features::toggles::shown::Tally;

use crate::pages::PageContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeadChip {
    pub id: &'static str,
    pub title: Cow<'static, str>,
    /// What is on of the rows the search matched, for the chip and the
    /// All chip's sum.
    pub tally: Tally,
    /// Such as "2/4", beside the title.
    pub count: String,
}

impl LeadChip {
    pub fn new(ctx: &Context, id: &'static str, title: Cow<'static, str>, tally: Tally) -> Self {
        Self {
            id,
            title,
            tally,
            count: super::tally_text(ctx, tally),
        }
    }
}

pub trait LeadSections {
    /// Each section's chip, in the order they show, its count over the
    /// rows `query` (trimmed, lower case, empty for all) matches.
    fn chips(&self, ctx: &Context, context: &PageContext<'_>, query: &str) -> Vec<LeadChip>;

    /// Shows section `id` with its rows that match `query` (trimmed, lower
    /// case, empty for all). Returns whether it showed a row.
    fn show(&self, ui: &mut Ui, context: &PageContext<'_>, id: &str, query: &str) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct NoLead;

impl LeadSections for NoLead {
    fn chips(&self, _ctx: &Context, _context: &PageContext<'_>, _query: &str) -> Vec<LeadChip> {
        Vec::new()
    }

    fn show(&self, _ui: &mut Ui, _context: &PageContext<'_>, _id: &str, _query: &str) -> bool {
        false
    }
}
