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
    /// Whether every state the tally counts was read.
    pub read: bool,
    /// Such as "2/4" or "?/4" beside the title, none while the search
    /// shows nothing to count.
    pub count: Option<String>,
}

impl LeadChip {
    pub fn new(
        ctx: &Context,
        id: &'static str,
        title: Cow<'static, str>,
        (tally, read): (Tally, bool),
    ) -> Self {
        Self {
            id,
            title,
            tally,
            read,
            count: super::shown_count(ctx, tally, read),
        }
    }
}

pub trait LeadSections {
    /// What the sections show, read from the system and the queue alone,
    /// so the page keeps it until either changes.
    type Data: std::fmt::Debug;

    fn data(&self, context: &PageContext<'_>) -> Self::Data;

    /// Each section's chip, in the order they show, its count over the
    /// rows `query` (trimmed, lower case, empty for all) matches.
    fn chips(&self, ctx: &Context, data: &Self::Data, query: &str) -> Vec<LeadChip>;

    /// How many of the rows could not be read, for the page's banner.
    fn unread(&self, _data: &Self::Data) -> usize {
        0
    }

    /// Shows section `id` with its rows that match `query` (trimmed, lower
    /// case, empty for all). Returns whether it showed a row.
    fn show(&self, ui: &mut Ui, on: (&PageContext<'_>, &Self::Data), id: &str, query: &str)
    -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct NoLead;

impl LeadSections for NoLead {
    type Data = ();

    fn data(&self, _context: &PageContext<'_>) {}

    fn chips(&self, _ctx: &Context, (): &(), _query: &str) -> Vec<LeadChip> {
        Vec::new()
    }

    fn show(&self, _ui: &mut Ui, _: (&PageContext<'_>, &()), _id: &str, _query: &str) -> bool {
        false
    }
}
