//! A page that shows one toggle catalog under its header: Components,
//! Gaming and System Tweaks differ only in what [`CatalogKind`] names.

use std::marker::PhantomData;

use egui::Ui;
use toolbox_core::Snapshot;
use toolbox_gui_kit::row_cache::{RowCache, Same};

use super::toggles::{LeadSections, TogglePage};
use super::{PageContext, PageView};
use crate::catalogs::CatalogId;

pub trait CatalogKind {
    const CATALOG: CatalogId;
    const TITLE: &'static str;
    const SUBTITLE: &'static str;
    type Lead: LeadSections;
    const LEAD: Self::Lead;

    /// A literal per page, because `tracing` names a span at compile time
    /// and the logs keep each page's own name.
    fn span() -> tracing::Span;
}

/// The lead sections' data, keyed by the system and the queue.
pub type LeadData<K> =
    RowCache<(Same<Snapshot>, u64), <<K as CatalogKind>::Lead as LeadSections>::Data>;

#[derive(Debug)]
pub struct CatalogPage<K>
where
    K: CatalogKind,
{
    toggles: TogglePage,
    lead: LeadData<K>,
    kind: PhantomData<K>,
}

impl<K: CatalogKind> Default for CatalogPage<K> {
    fn default() -> Self {
        Self {
            toggles: TogglePage::new(K::CATALOG, K::TITLE, K::SUBTITLE).waiting_for_plans(),
            lead: RowCache::new(),
            kind: PhantomData,
        }
    }
}

#[cfg(test)]
impl<K: CatalogKind> CatalogPage<K> {
    /// How often the lead sections' data was built, for the page's tests.
    pub const fn lead_builds(&self) -> u64 {
        self.lead.builds()
    }
}

impl<K: CatalogKind> PageView for CatalogPage<K> {
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let listed = context.catalogs.get(K::CATALOG).catalog.is_ok();
        self.toggles.header(ui, listed);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = K::span().entered();
        let rest = (Same::of(context.snapshot), context.revision);
        let data = self.lead.get(&[], rest, || {
            let _build = tracing::trace_span!("build lead").entered();
            K::LEAD.data(context)
        });

        self.toggles.ui(ui, context, (&K::LEAD, &*data));
    }
}
