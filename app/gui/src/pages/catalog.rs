//! A page that shows one toggle catalog under its header: Components,
//! Gaming and System Tweaks differ only in what [`CatalogKind`] names.

use std::marker::PhantomData;

use egui::Ui;

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

#[derive(Debug)]
pub struct CatalogPage<K> {
    toggles: TogglePage,
    kind: PhantomData<K>,
}

impl<K: CatalogKind> Default for CatalogPage<K> {
    fn default() -> Self {
        Self {
            toggles: TogglePage::new(K::CATALOG, K::TITLE, K::SUBTITLE).waiting_for_plans(),
            kind: PhantomData,
        }
    }
}

impl<K: CatalogKind> PageView for CatalogPage<K> {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        self.toggles.header(ui);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = K::span().entered();
        self.toggles.ui(ui, context, &K::LEAD);
    }
}
