#[cfg(test)]
mod tests;

use super::catalog::{CatalogKind, CatalogPage};
use super::toggles::NoLead;
use crate::catalogs::CatalogId;

pub type ComponentsPage = CatalogPage<Components>;

#[derive(Debug)]
pub struct Components;

impl CatalogKind for Components {
    const CATALOG: CatalogId = CatalogId::Components;
    const TITLE: &'static str = "page-components";
    const SUBTITLE: &'static str = "page-components-subtitle";
    type Lead = NoLead;
    const LEAD: NoLead = NoLead;

    fn span() -> tracing::Span {
        tracing::trace_span!("components ui")
    }
}
