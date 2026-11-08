//! The Gaming page: launchers, tools and emulators to install, from
//! `gaming.toml`.

use super::catalog::{CatalogKind, CatalogPage};
use super::toggles::NoLead;
use crate::catalogs::CatalogId;

pub type GamingPage = CatalogPage<Gaming>;

#[derive(Debug)]
pub struct Gaming;

impl CatalogKind for Gaming {
    const CATALOG: CatalogId = CatalogId::Gaming;
    const TITLE: &'static str = "page-gaming";
    const SUBTITLE: &'static str = "page-gaming-subtitle";
    type Lead = NoLead;
    const LEAD: NoLead = NoLead;

    fn span() -> tracing::Span {
        tracing::trace_span!("gaming ui")
    }
}
